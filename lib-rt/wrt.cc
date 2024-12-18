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

// linear memory: 4GiB
static constexpr uint64_t LINEAR_MEMORY_BEGIN = 0x100000000000;
static constexpr uint64_t MAX_LINEAR_MEMORY_SIZE = 0x400000; // 4GiB
// guard page: 2GiB
static constexpr uint64_t GUARD_PAGE_SIZE = 0x200000;

static std::string_view USAGE = R"(WebAssembly AOT executable
USAGE: <this file> [options] -- [arguments]

OPTIONS:
  no options: Run the WebAssembly AOT module from the beginning
  --help: Display this message and exit
  --restore <FILE>: Restore an execution from a checkpoint file
)";

// forward decl
static void dump_exec_env(ExecEnv &exec_env);
static void dump_checkpoint(Checkpoint &chkpt);

extern "C" auto memory_grow(ExecEnv *exec_env, int32_t inc_pages) -> int32_t;

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

auto allocate_memory(int32_t num_pages) -> int8_t * {
  uint64_t const num_bytes = num_pages * PAGE_SIZE;

  // Memory layout
  // 0x100000000000 - 0x100000000000 + 0x400000: linear memory
  // Guard pages are placed at the beginning and the end of the linear memory
  // (Unused linear memory is allocated as guard pages before memory.grow is
  // called)

  // Add guard pages
  Info() << "Allocating guard pages" << '\n';
  if (mmap((void *)(LINEAR_MEMORY_BEGIN - GUARD_PAGE_SIZE),
           (GUARD_PAGE_SIZE * 2) + MAX_LINEAR_MEMORY_SIZE, PROT_NONE,
           MAP_ANONYMOUS | MAP_PRIVATE | MAP_FIXED, -1, 0) == nullptr) {
    Fatal() << "Failed to allocate guard pages" << '\n';
  }

  // Allocate linear memory
  if (munmap((void *)LINEAR_MEMORY_BEGIN, num_bytes) < 0) {
    Fatal() << "Failed to unmap part of guard pages" << '\n';
    exit(1);
  };
  auto *res = static_cast<int8_t *>(mmap((void *)LINEAR_MEMORY_BEGIN, num_bytes,
                                         PROT_READ | PROT_WRITE,
                                         MAP_ANONYMOUS | MAP_PRIVATE, -1, 0));
  if (res == nullptr) {
    Fatal() << "Failed to allocate " << num_pages * PAGE_SIZE
            << " bytes to linear memory" << '\n';
    exit(1);
  }
  Info() << "Allocating linear memory: " << num_pages
         << " pages, starting at 0x" << std::hex << (uint64_t)res << '\n';
// Zero out memory
#ifdef __FreeBSD__
  std::memset(res, 0, num_bytes);
#endif

  return res;
}

static auto extend_memory(ExecEnv *exec_env, int32_t inc_pages) -> int32_t {
  ASSERT(inc_pages >= 0);
  int32_t const old_size = exec_env->memory_size;
  int32_t const new_size = old_size + inc_pages;

  if (inc_pages == 0) {
    return old_size;
  }

  // Unmap requested pages
  if (munmap(exec_env->memory_base + (old_size * PAGE_SIZE),
             inc_pages * PAGE_SIZE) < 0) {
    Fatal() << "Failed to unmap guard pages: inc_pages=" << std::dec
            << inc_pages << '\n';
    exit(1);
  }
  auto *res =
      static_cast<int8_t *>(mremap(exec_env->memory_base, old_size * PAGE_SIZE,
                                   new_size * PAGE_SIZE, MREMAP_MAYMOVE));
  if (res == nullptr) {
    Fatal() << "Failed to grow memory (" << inc_pages << ")" << '\n';
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

static auto wanco_main(int argc, char **argv) -> int {
  signal(SIGSEGV, signal_segv_handler);

  // Parse CLI arguments
  Config const config = parse_from_args(argc, argv);

  if (config.restore_file.empty()) {
    // Allocate memory
    int const memory_size = INIT_MEMORY_SIZE;
    int8_t *memory = allocate_memory(memory_size);
    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = memory,
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

    int8_t *memory = nullptr;
    if (!config.restore_file.ends_with(".pb")) {
      Warn() << "The file does not have a .pb extension. "
                "Attempting to parse as proto."
             << '\n';
    }
    auto p = decode_checkpoint_proto(ifs);
    chkpt = p.first;
    memory = p.second;
    chkpt.prepare_restore();
    Info() << "Checkpoint has been loaded" << '\n';
    Info() << "- call stack: " << chkpt.frames.size() << " frames" << '\n';
    Info() << "- value stack: " << chkpt.restore_stack.size() << " values"
           << '\n';

    // Initialize exec_env
    exec_env = ExecEnv{
        .memory_base = memory,
        .memory_size = chkpt.memory_size,
        .migration_state = MigrationState::STATE_RESTORE,
        .argc = argc,
        .argv = reinterpret_cast<uint8_t **>(argv),
    };
  }
  // Register signal handler
  signal(SIGCHKPT, signal_chkpt_handler);

  aot_main(&exec_env);

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
  }

  // cleanup
  munmap(exec_env.memory_base, exec_env.memory_size * PAGE_SIZE);
  return 0;
}

} // namespace wanco

auto main(int argc, char **argv) -> int {
  return wanco::wanco_main(argc, argv);
}
