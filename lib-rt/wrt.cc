#include "aot.h"
#include "chkpt/chkpt.h"
#include "wanco.h"
#include <chrono>
#include <csignal>
#include <cstdio>
#include <execinfo.h>
#include <string>
#include <string_view>
#include <sys/mman.h>
#include <ucontext.h>

// global instancce of execution environment
ExecEnv exec_env;

namespace wanco {

uint64_t CHKPT_START_TIME = 0;
uint64_t RESTORE_START_TIME = 0;

// global instance of checkpoint
Checkpoint chkpt;
// global instance of stackmap info
stackmap::Stackmap g_stackmap;
// global instance of linear memory
std::string linear_memory;

static std::string_view USAGE = R"(WebAssembly AOT executable
USAGE: <this file> [options] -- [arguments]

OPTIONS:
  no options: Run the WebAssembly AOT module from the beginning
  --help: Display this message and exit
  --restore <FILE>: Restore an execution from a checkpoint file
)";

// signal handler for debugging
static void signal_segv_handler(int signum) {
  void *array[10];
  size_t size = 0;
  ASSERT(signum == SIGSEGV && "Unexpected signal");

  // get void*'s for all entries on the stack
  size = backtrace(array, 10);

  // print out all the frames to stderr
  fprintf(stderr, "Error: segmentation fault\n");
  backtrace_symbols_fd(array, size, STDERR_FILENO);
  exit(1);
}

static void signal_chkpt_handler(int signum) {
  ASSERT(signum == SIGCHKPT && "Unexpected signal");
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT_START;
}

struct Config {
  std::string restore_file;
} __attribute__((aligned(32)));

std::string allocate_memory(int32_t num_pages) {
  uint64_t const num_bytes = num_pages * PAGE_SIZE;
  std::string new_memory(num_bytes, 0);
  return new_memory;
}

auto extend_memory(ExecEnv *exec_env, int32_t inc_pages) -> int32_t {
  ASSERT(inc_pages >= 0);
  int32_t old_size = exec_env->memory_size;
  int32_t new_size = old_size + inc_pages;

  if (inc_pages == 0) {
    return old_size;
  }

  linear_memory.resize(new_size * PAGE_SIZE, 0);

  exec_env->memory_base = reinterpret_cast<int8_t *>(linear_memory.data());
  exec_env->memory_size = new_size;
  return old_size;
}

static auto parse_from_args(int argc, char **argv) -> Config {
  Config config;
  for (int i = 1; i < argc; i++) {
    if (std::string(argv[i]) == "--restore") {
      if (i + 1 >= argc) {
        Fatal() << "Error: Missing argument for --restore" << '\n';
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
      Fatal() << "Unknown argument: " << argv[i] << "." << '\n'
              << "If you want to pass arguments to the WebAssembly "
                 "module, pass them after '--'."
              << '\n';
      exit(1);
    }
  }
  return config;
}

static auto prepare_checkpoint() -> void {
  ElfFile elf_file{"/proc/self/exe"};
  auto stackmap_section = elf_file.get_section_data(".llvm_stackmaps");
  if (stackmap_section.has_value()) {
    g_stackmap = wanco::stackmap::parse_stackmap(stackmap_section.value());
  }
}

static auto wanco_main(int argc, char **argv) -> int {
  signal(SIGSEGV, signal_segv_handler);

  // Parse CLI arguments
  Config const config = parse_from_args(argc, argv);

  prepare_checkpoint();

  if (config.restore_file.empty()) {
    // Allocate memory
    int const memory_size = INIT_MEMORY_SIZE;
    linear_memory = allocate_memory(memory_size);
    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = reinterpret_cast<int8_t *>(linear_memory.data()),
        .memory_size = memory_size,
        .migration_state = MigrationState::STATE_NONE,
        .argc = argc,
        .argv = reinterpret_cast<uint8_t **>(argv),
    };
  } else {
    RESTORE_START_TIME =
        std::chrono::duration_cast<std::chrono::microseconds>(
            std::chrono::system_clock::now().time_since_epoch())
            .count();

    // Restore from checkpoint
    std::ifstream ifs(config.restore_file);
    if (!ifs.is_open()) {
      Fatal() << "Failed to open checkpoint file: " << config.restore_file
              << '\n';
      return 1;
    }

    if (!config.restore_file.ends_with(".pb")) {
      Warn() << "The file does not have a .pb extension. "
                "Attempting to parse as proto."
             << '\n';
    }
    chkpt = decode_checkpoint_proto(ifs);
    chkpt.prepare_restore();
    Info() << "Checkpoint has been loaded" << '\n';
    Info() << "- call stack: " << chkpt.frames.size() << " frames" << '\n';
    Info() << "- value stack: " << chkpt.restore_stack.size() << " values"
           << '\n';

    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = reinterpret_cast<int8_t *>(linear_memory.data()),
        .memory_size = chkpt.memory_size,
        .migration_state = MigrationState::STATE_RESTORE,
        .argc = argc,
        .argv = reinterpret_cast<uint8_t **>(argv),
    };
  }
  // Register signal handler
  signal(SIGCHKPT, signal_chkpt_handler);

  aot_main(&exec_env);

  CHKPT_START_TIME = std::chrono::duration_cast<std::chrono::microseconds>(
                         std::chrono::system_clock::now().time_since_epoch())
                         .count();

  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT_CONTINUE) {
    chkpt.memory_size = exec_env.memory_size;

    // write snapshot
    std::ofstream ofs("checkpoint.pb");
    encode_checkpoint_proto(ofs, chkpt, exec_env.memory_base);
    Info() << "Snapshot has been saved to checkpoint.pb" << '\n';

    auto time = std::chrono::duration_cast<std::chrono::microseconds>(
                    std::chrono::system_clock::now().time_since_epoch())
                    .count();
    time = time - wanco::CHKPT_START_TIME;
    // TODO(tamaron): remove this (research purpose)
    std::ofstream chktime("chkpt-time.txt");
    chktime << time << '\n';
    chktime.close();
    Info() << "Checkpoint time has been saved to chkpt-time.txt" << '\n';
  }

  // cleanup
  munmap(exec_env.memory_base, exec_env.memory_size * PAGE_SIZE);
  return 0;
}

} // namespace wanco

auto main(int argc, char **argv) -> int {
  return wanco::wanco_main(argc, argv);
}
