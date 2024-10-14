#include "aot.h"
#include "wanco.h"
#include <csignal>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <execinfo.h>
#include <fstream>
#include <iostream>
#include <sys/mman.h>
#include <ucontext.h>
#include <unistd.h>

// global instancce of execution environment
ExecEnv exec_env;

namespace wanco {

// global instance of checkpoint
Checkpoint chkpt;

CheckpointV2 chkpt_v2;

// linear memory: 4GiB
static constexpr uint64_t LINEAR_MEMORY_BEGIN = 0x100000000000;
static constexpr uint64_t MAX_LINEAR_MEMORY_SIZE = 0x400000; // 4GiB
// guard page: 2GiB
static constexpr uint64_t GUARD_PAGE_SIZE = 0x200000;

std::string_view USAGE = R"(WebAssembly AOT executable
USAGE: <this file> [options] -- [arguments]

OPTIONS:
  no options: Run the WebAssembly AOT module from the beginning
  --help: Display this message and exit
  --restore <FILE>: Restore an execution from a checkpoint JSON file
)";

// forward decl
void dump_exec_env(ExecEnv &exec_env);
void dump_checkpoint(Checkpoint &chkpt);

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages);

// signal handler for debugging
void signal_segv_handler(int signum) {
  void *array[10];
  size_t size;
  ASSERT(signum == SIGSEGV && "Unexpected signal");

  // get void*'s for all entries on the stack
  size = backtrace(array, 10);

  // print out all the frames to stderr
  fprintf(stderr, "Error: segmentation fault\n");
  backtrace_symbols_fd(array, size, STDERR_FILENO);
  exit(1);
}

void signal_chkpt_handler(int signum) {
  ASSERT(signum == SIGCHKPT && "Unexpected signal");
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT_START;
}

struct Config {
  std::string restore_file;
};

int8_t *allocate_memory(const Config &config, int32_t num_pages) {
  uint64_t num_bytes = num_pages * PAGE_SIZE;

  // Memory layout
  // 0x100000000000 - 0x100000000000 + 0x400000: linear memory
  // Guard pages are placed at the beginning and the end of the linear memory
  // (Unused linear memory is allocated as guard pages before memory.grow is
  // called)

  // Add guard pages
  Debug() << "Allocating guard pages" << std::endl;
  if (mmap((void *)(LINEAR_MEMORY_BEGIN - GUARD_PAGE_SIZE),
           GUARD_PAGE_SIZE * 2 + MAX_LINEAR_MEMORY_SIZE, PROT_NONE,
           MAP_ANONYMOUS | MAP_PRIVATE | MAP_FIXED, -1, 0) == NULL) {
    Fatal() << "Failed to allocate guard pages" << std::endl;
  }

  // Allocate linear memory
  if (munmap((void *)LINEAR_MEMORY_BEGIN, num_bytes) < 0) {
    Fatal() << "Failed to unmap part of guard pages" << std::endl;
    exit(1);
  };
  int8_t *res = (int8_t *)mmap((void *)LINEAR_MEMORY_BEGIN, num_bytes,
                               PROT_READ | PROT_WRITE,
                               MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
  if (res == NULL) {
    Fatal() << "Failed to allocate " << num_pages * PAGE_SIZE
            << " bytes to linear memory" << std::endl;
    exit(1);
  }
  Debug() << "Allocating liear memory: " << num_pages
          << " pages, starting at 0x" << std::hex << (uint64_t)res << std::endl;
// Zero out memory
#ifdef __FreeBSD__
  std::memset(res, 0, num_bytes);
#endif

  return res;
}

int32_t extend_memory(ExecEnv *exec_env, int32_t inc_pages) {
  ASSERT(inc_pages >= 0);
  int32_t old_size = exec_env->memory_size;
  int32_t new_size = old_size + inc_pages;

  if (inc_pages == 0) {
    return old_size;
  }

  // Unmap requested pages
  if (munmap(exec_env->memory_base + old_size * PAGE_SIZE,
             inc_pages * PAGE_SIZE) < 0) {
    Fatal() << "Failed to unmap guard pages: inc_pages=" << std::dec
            << inc_pages << std::endl;
    exit(1);
  }
  int8_t *res = (int8_t *)mremap(exec_env->memory_base, old_size * PAGE_SIZE,
                                 new_size * PAGE_SIZE, MREMAP_MAYMOVE);
  if (res == NULL) {
    Fatal() << "Failed to grow memory (" << inc_pages << ")" << std::endl;
    exit(1);
  }
// Zero out new memory
#ifdef __FreeBSD__
  std::memset(res + old_size * PAGE_SIZE, 0, inc_pages * PAGE_SIZE);
#endif

  exec_env->memory_base = res;
  exec_env->memory_size = new_size;
  return old_size;
}

Config parse_from_args(int argc, char **argv) {
  Config config;
  for (int i = 1; i < argc; i++) {
    if (std::string(argv[i]) == "--restore") {
      if (i + 1 >= argc) {
        Fatal() << "Error: Missing argument for --restore" << std::endl;
        exit(1);
      }
      config.restore_file = argv[i + 1];
      i++;
    } else if (std::string(argv[i]) == "--help") {
      std::cerr << USAGE;
      exit(0);
    } else if (std::string(argv[i]) == "--") {
      return config;
    } else {
      Fatal() << "Unknown argument: " << argv[i] << "." << std::endl
              << "If you want to pass arguments to the WebAssembly "
                 "module, pass them after '--'."
              << std::endl;
      exit(1);
    }
  }
  return config;
}

int wanco_main(int argc, char **argv) {
  signal(SIGSEGV, signal_segv_handler);

  // Parse CLI arguments
  Config config = parse_from_args(argc, argv);

  if (config.restore_file.empty()) {
    // Allocate memory
    int memory_size = INIT_MEMORY_SIZE;
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
    std::ifstream ifs(config.restore_file);
    if (!ifs.is_open()) {
      Fatal() << "Failed to open checkpoint file: " << config.restore_file
              << std::endl;
      return 1;
    }

    if constexpr (USE_PROTOBUF) {
      if (!config.restore_file.ends_with(".pb")) {
        Warn() << "The file does not have a .pb extension. "
                  "Attempting to parse as JSON."
               << std::endl;
      }
      chkpt = decode_checkpoint_proto(ifs);
    } else if (!config.restore_file.ends_with(".json")) {
      Warn() << "The file does not have a .json extension. "
                "Attempting to parse as protobuf."
             << std::endl;
      chkpt = decode_checkpoint_json(ifs);
    }
    chkpt.prepare_restore();

    int32_t memory_size = chkpt.memory_size;
    // Allocate memory and copy contents from checkpoint
    int8_t *memory = allocate_memory(config, memory_size);
    std::memcpy(memory, chkpt.memory.data(), chkpt.memory.size());
    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = memory,
        .memory_size = memory_size,
        .migration_state = MigrationState::STATE_RESTORE,
        .argc = argc,
        .argv = (uint8_t **)argv,
    };
  }
  // Register signal handler
  signal(SIGCHKPT, signal_chkpt_handler);

  aot_main(&exec_env);

  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT_CONTINUE) {
    chkpt.memory = std::vector<int8_t>(exec_env.memory_base,
                                       exec_env.memory_base +
                                           exec_env.memory_size * PAGE_SIZE);
    chkpt.memory_size = exec_env.memory_size;

    // write snapshot
    if constexpr (USE_PROTOBUF) {
      std::ofstream ofs("checkpoint.pb");
      encode_checkpoint_proto(ofs, chkpt);
      Info() << "Snapshot has been saved to checkpoint.pb" << std::endl;
    } else {
      std::ofstream ofs("checkpoint.json");
      encode_checkpoint_json(ofs, chkpt);
      Info() << "Snapshot has been saved to checkpoint.json" << std::endl;
    }
  }

  // cleanup
  munmap(exec_env.memory_base, exec_env.memory_size * PAGE_SIZE);
  return 0;
}

} // namespace wanco

int main(int argc, char **argv) { return wanco::wanco_main(argc, argv); }
