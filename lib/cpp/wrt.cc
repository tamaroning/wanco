#include "chkpt.h"
#include "exec_env.h"
#include <cassert>
#include <csignal>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <sys/mman.h>
#include <ucontext.h>
#include <unistd.h>

const int32_t PAGE_SIZE = 65536;
// 10 and 12 are reserved for SIGUSR1 and SIGUSR2
const int SIGCHKPT = 10;
// execution environment
ExecEnv exec_env;
Checkpoint chkpt;

std::string_view USAGE = R"(This file is a WebAssembly AOT executable.
USAGE: <this file> [options]

OPTIONS:
  no options: Run the WebAssembly AOT module from the beginning
  --help: Display this message and exit
  --restore <FILE>: Restore an execution from a checkpoint JSON file
  --llvm-layout: Use LLVM layout for memory (Use 4GB linear memory)
)";

// from wasm AOT module
extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

// forward decl
void dump_exec_env(ExecEnv &exec_env);
void dump_checkpoint(Checkpoint &chkpt);

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages);

void signal_chkpt_handler(int signum) {
  assert(signum == SIGCHKPT && "Unexpected signal");
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT_START;
}

struct Config {
  std::string restore_file;
  bool use_llvm_layout = false;
};

int8_t *allocate_memory(const Config &config, int32_t num_pages) {
  uint64_t num_bytes = num_pages * PAGE_SIZE;
  // FIXME: should use mmap
  int8_t *res = (int8_t *)malloc(num_bytes);
  if (res == NULL) {
    std::cerr << "Error: Failed to allocate " << num_pages * PAGE_SIZE
              << " bytes to linear memory" << std::endl;
    exit(1);
  }
  std::cerr << "[info] Allocating liear memory: " << num_pages
            << " pages, starting at 0x" << std::hex << (uint64_t)res
            << std::endl;
  // Zero out memory
  std::memset(res, 0, num_bytes);
  return res;
}

void segv_handler(int signum, siginfo_t *info, void *context) {
  ucontext_t *uc = (ucontext_t *)context;
  void *fault_address = info->si_addr;

  if (!(exec_env.memory_base <= fault_address &&
        fault_address < exec_env.memory_base + 64 * 1024 * 1024)) {
    printf("Segmentation fault at address: %p\n", fault_address);
    exit(EXIT_FAILURE);
  }

  printf("[debug] Try mmap : %p\n", fault_address);

  void *page_start = (void *)((uintptr_t)fault_address & ~(PAGE_SIZE - 1));
  if (mmap(page_start, PAGE_SIZE, PROT_READ | PROT_WRITE | PROT_EXEC,
           MAP_ANONYMOUS | MAP_PRIVATE | MAP_FIXED, -1, 0) == MAP_FAILED) {
    perror("mmap");
    exit(EXIT_FAILURE);
  }

  printf("[debug] mmap succeeded at address: %p\n", page_start);
}

Config parse_from_args(int argc, char **argv) {
  Config config;
  for (int i = 1; i < argc; i++) {
    if (std::string(argv[i]) == "--restore") {
      if (i + 1 >= argc) {
        std::cerr << "Error: Missing argument for --restore" << std::endl;
        exit(1);
      }
      config.restore_file = argv[i + 1];
      i++;
    } else if (std::string(argv[i]) == "--llvm-layout") {
      config.use_llvm_layout = true;
    } else if (std::string(argv[i]) == "--help") {
      std::cerr << USAGE;
      exit(0);
    } else if (std::string(argv[i]) == "--") {
      return config;
    } else {
      std::cerr << "WARNING: Ignored unknown argument: " << argv[i]
                << std::endl;
    }
  }
  return config;
}

int main(int argc, char **argv) {
  // Parse CLI arguments
  Config config = parse_from_args(argc, argv);

  // Since we cannot allocate entire 64bit address space, we trap SIGSEGV and mmap the page on demand
  if (config.use_llvm_layout) {
    struct sigaction sa;
    sa.sa_sigaction = segv_handler;
    sa.sa_flags = SA_SIGINFO;
    sigemptyset(&sa.sa_mask);
    if (sigaction(SIGSEGV, &sa, NULL) == -1) {
      std::cerr << "Error: Failed to set signal handler" << std::endl;
      exit(EXIT_FAILURE);
    }
  }

  if (config.restore_file.empty()) {
    // Allocate memory
    int memory_size = INIT_MEMORY_SIZE;
    if (config.use_llvm_layout) {
      memory_size = 64; // 64KiB
    }
    int8_t *memory = allocate_memory(config, memory_size);
    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = memory,
        .memory_size = memory_size,
        .migration_state = MigrationState::STATE_NONE,
        .argc = argc,
        .argv = (uint8_t **)argv,
    };
  } else {
    // Restore from checkpoint
    if (config.use_llvm_layout) {
      // TODO: support
      std::cerr << "Error: --llvm-layout is not supported for restore"
                << std::endl;
      return 1;
    }

    std::ifstream ifs(config.restore_file);
    if (!ifs.is_open()) {
      std::cerr << "Error: Failed to open checkpoint" << config.restore_file
                << std::endl;
      return 1;
    }

    std::cerr << "[info] Loading checkpoint from " << config.restore_file
              << std::endl;
    chkpt = decode_checkpoint_json(ifs);

    // ceil(memory.size / PAGE_SIZE)
    int32_t memory_size = (chkpt.memory.size() + PAGE_SIZE - 1) / PAGE_SIZE;
    // Allocate memory
    int8_t *memory = allocate_memory(config, memory_size);

    std::cerr << "[info] Restoring memory: 0x" << std::hex
              << chkpt.memory.size() << " bytes" << std::endl;
    std::memcpy(memory, chkpt.memory.data(), chkpt.memory.size());
    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = memory,
        .memory_size = memory_size,
        .migration_state = MigrationState::STATE_RESTORE,
        .argc = argc,
        .argv = (uint8_t **)argv,
    };
    std::cerr << "[info] Restore start" << std::endl;
  }
  // Register signal handler
  signal(SIGCHKPT, signal_chkpt_handler);

  aot_main(&exec_env);

  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT_CONTINUE) {
    chkpt.memory = std::vector<int8_t>(exec_env.memory_base,
                                       exec_env.memory_base +
                                           exec_env.memory_size * PAGE_SIZE);
    std::ofstream ofs("checkpoint.json");
    encode_checkpoint_json(ofs, chkpt);
  }

  // cleanup
  free(exec_env.memory_base);
  return 0;
}

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages) {
  assert(inc_pages >= 0);
  int32_t old_size = exec_env->memory_size;
  int32_t new_size = old_size + inc_pages;

  // FIXME: Should use mremap
  int8_t *res = (int8_t *)realloc(exec_env->memory_base, new_size * PAGE_SIZE);
  if (res == NULL) {
    std::cerr << "Error: Failed to grow memory (" << inc_pages << ")"
              << std::endl;
    return -1;
  }
  // Zero out new memory
  std::memset(res + old_size * PAGE_SIZE, 0, inc_pages * PAGE_SIZE);

  exec_env->memory_base = res;
  exec_env->memory_size = new_size;
  return old_size;
}

/*
** checkpoint related functions
*/

// locals
extern "C" void push_frame(ExecEnv *exec_env) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.push_back(Frame());
}

extern "C" void set_pc_to_frame(ExecEnv *exec_env, int32_t fn_index,
                                int32_t pc) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().fn_index = fn_index;
  chkpt.frames.back().pc = pc;
}

extern "C" void push_local_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(i32));
}

extern "C" void push_local_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(i64));
}

extern "C" void push_local_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(f32));
}

extern "C" void push_local_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(f64));
}

// stack
extern "C" void push_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(i32));
}

extern "C" void push_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(i64));
}

extern "C" void push_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(f32));
}

extern "C" void push_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(f64));
}

// globals
extern "C" void push_global_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(i32));
}

extern "C" void push_global_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(i64));
}

extern "C" void push_global_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(f32));
}

extern "C" void push_global_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(f64));
}

void dump_exec_env(ExecEnv &exec_env) {
  std::cout << "Migration state: " << (int)exec_env.migration_state
            << std::endl;
  std::cout << "Memory base: 0x" << std::hex << (void *)exec_env.memory_base
            << std::endl;
  std::cout << "Memory size: " << exec_env.memory_size << std::endl;
}

void dump_checkpoint(Checkpoint &chkpt) {
  std::cout << "Checkpoint" << std::endl;
  std::cout << "Frames: " << (chkpt.frames.empty() ? "(empty)" : "")
            << std::endl;
  for (size_t i = 0; i < chkpt.frames.size(); i++) {
    const Frame &frame = chkpt.frames[i];
    std::cout << "  Frame[" << i << "]" << std::endl;
    std::cout << "    Location: Op[" << frame.pc << "] at Func["
              << frame.fn_index << "]" << std::endl;
    std::cout << "    Locals:" << (frame.locals.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "      " << local.to_string() << std::endl;
    }
  }

  std::cout << "Stack:" << (chkpt.stack.empty() ? "(empty)" : "") << std::endl;
  for (auto &value : chkpt.stack) {
    std::cout << "  " << value.to_string() << std::endl;
  }

  std::cout << "Globals" << (chkpt.globals.empty() ? "(empty)" : "")
            << std::endl;
  for (auto &value : chkpt.globals) {
    std::cout << "  " << value.to_string() << std::endl;
  }
}

// Restore
extern "C" void pop_front_frame(ExecEnv *exec_env) {
  assert(exec_env->migration_state == MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  assert(!chkpt.frames.empty() && "No frame to restore");
  Frame &frame = chkpt.frames.front();
  std::cerr << "[debug] call to pop_front_frame -> Fn[" << frame.fn_index << "]"
            << std::endl;

  chkpt.frames.pop_front();
  // Restore is completed if there are no more frames to restore
  if (chkpt.frames.empty()) {
    std::cerr << "[debug] Restore completed" << std::endl;
    exec_env->migration_state = MigrationState::STATE_NONE;
    // chkpt = Checkpoint();
  }
}

extern "C" bool frame_is_empty(ExecEnv *exec_env) {
  return chkpt.frames.empty();
}

extern "C" int32_t get_pc_from_frame(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(exec_env->migration_state == MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  return chkpt.frames.front().pc;
}

extern "C" int32_t pop_front_local_i32(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_local_i64(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_local_f32(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_local_f64(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_i32(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_i64(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_f32(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_f64(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_front_global_i32(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_global_i64(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_global_f32(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_global_f64(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}
