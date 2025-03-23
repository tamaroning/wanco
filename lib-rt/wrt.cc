#include "aot.h"
#include "chkpt/chkpt.h"
#include "elf/elf.h"
#include "osr/wasm_stacktrace.h"
#include "stackmap/stackmap.h"
#include "stacktrace/stacktrace.h"
#include "wanco.h"
#include <chrono>
#include <csignal>
#include <cstdio>
#include <execinfo.h>
#include <poll.h>
#include <pthread.h>
#include <string>
#include <string_view>
#include <sys/eventfd.h>
#include <sys/mman.h>
#include <ucontext.h>

// global instancce of execution environment
ExecEnv exec_env;

namespace wanco {

uint64_t CHKPT_START_TIME = 0;
uint64_t RESTORE_START_TIME = 0;

// global instance of checkpoint
Checkpoint chkpt;

int efd = 0;

// linear memory: 4GiB
static constexpr uint64_t GUARD_PAGE_BEGIN = 0xA0000000;
static constexpr uint64_t GUARD_PAGE_END = 0xA0010000;
static constexpr uint64_t LINEAR_MEMORY_BEGIN = 0xA0010000;
// static constexpr uint64_t MAX_LINEAR_MEMORY_SIZE = 0x400000;
static constexpr uint64_t GUARD_PAGE2_BEGIN = 0xA0050000;
static constexpr uint64_t GUARD_PAGE2_END = 0xA00600000;

static constexpr uint64_t POLLING_PAGE_BEGIN = 0xA0060000;
static constexpr uint64_t POLLING_PAGE_END = 0xA0061000;

static std::string_view USAGE = R"(WebAssembly AOT executable
USAGE: <this file> [options] -- [arguments]

OPTIONS:
  no options: Run the WebAssembly AOT module from the beginning
  --help: Display this message and exit
  --restore <FILE>: Restore an execution from a checkpoint file
)";

// forward decl
static void start_checkpoint();

static void signal_segv_handler(int signum, siginfo_t *info, void *context) {
  int err = save_context((ucontext_t *)context);
  ASSERT(err == 0 && "Failed to save context");
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT_START;
  uint64_t val = 1;
  write(efd, &val, sizeof(val));
  while (1) {
    sleep(1000);
  }
}

static void signal_chkpt_handler(int signum) {
  uint64_t val = 1;
  write(efd, &val, sizeof(val));
}

void *supervisor_thread(void *arg) {
  ASSERT(efd != 0 && efd != -1 && "efd not initialized");
  struct pollfd pfd = {.fd = efd, .events = POLLIN};
  for (int i = 0; i < 2; i++) {
    if (poll(&pfd, 1, -1) == -1) { // イベント待機
      perror("poll");
      close(efd);
      exit(1);
    }

    if (pfd.revents & POLLIN) {
      uint64_t value;
      // reset the counter
      read(efd, &value, sizeof(value));
      Info() << "Event received! Value: " << std::dec << value << '\n';

      if (i == 0) {
        // mprotect the polling page
        int err = mprotect((void *)POLLING_PAGE_BEGIN,
                           POLLING_PAGE_END - POLLING_PAGE_BEGIN, PROT_NONE);
        ASSERT(err == 0 && "Failed to mprotect polling page");
        // This is necessary to flush the TLB.
        // FIXME: I have no idea why it works if the following line is commented
        // out.

        // asm volatile("invlpg (%0)" ::"r"(exec_env.polling_page) : "memory");
      } else {
        start_checkpoint();
      }
    }
  }

  return NULL;
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
  if (mmap((void *)GUARD_PAGE_BEGIN, GUARD_PAGE_END - GUARD_PAGE_BEGIN,
           PROT_NONE, MAP_ANONYMOUS | MAP_PRIVATE, -1, 0) == nullptr) {
    Fatal() << "Failed to allocate guard pages" << '\n';
  }

  if (mmap((void *)GUARD_PAGE2_BEGIN, GUARD_PAGE2_END - GUARD_PAGE2_BEGIN,
           PROT_NONE, MAP_ANONYMOUS | MAP_PRIVATE, -1, 0) == nullptr) {
    Fatal() << "Failed to allocate guard pages" << '\n';
  }

  // Allocate linear memory
  auto *res = static_cast<int8_t *>(mmap((void *)LINEAR_MEMORY_BEGIN, num_bytes,
                                         PROT_READ | PROT_WRITE,
                                         MAP_ANONYMOUS | MAP_PRIVATE, -1, 0));
  if (res == nullptr) {
    Fatal() << "Failed to allocate " << num_pages * PAGE_SIZE
            << " bytes to linear memory" << '\n';
    exit(1);
  }
  // Zero out memory
#ifdef __FreeBSD__
  std::memset(res, 0, num_bytes);
#endif

  Info() << "Allocating linear memory: " << num_pages
         << " pages, starting at 0x" << std::hex << (uint64_t)res << '\n';

  return res;
}

auto extend_memory(ExecEnv *exec_env, int32_t inc_pages) -> int32_t {
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

static void start_checkpoint() {
  Info() << "Checkpoint started" << std::endl;
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT_CONTINUE;

  ElfFile elf_file{"/proc/self/exe"};
  auto stackmap_section = elf_file.get_section_data(".llvm_stackmaps");
  if (!stackmap_section.has_value()) {
    Fatal() << "Failed to get stackmap section" << std::endl;
    exit(1);
  }

  stackmap::Stackmap stackmap =
      stackmap::parse_stackmap(stackmap_section.value());
  Info() << "Parsed stackmap:" << std::dec << stackmap.stkmap_records.size()
         << " records" << std::endl;

  const auto [native_trace, regs] = get_stack_trace();
  for (const auto &frame : native_trace) {
    Debug() << frame.function_name << " + " << std::dec << frame.pc_offset
            << std::endl;
  }

  const auto wasm_trace = asr_exit(regs, native_trace, stackmap);

  Debug() << "Wasm trace:" << std::endl;
  for (const auto &frame : wasm_trace) {
    Debug() << frame.to_string() << std::endl;
  }

  // store the call stack
  for (const auto &frame : wasm_trace) {
    wanco::chkpt.frames.push_front(wanco::Frame{
        .fn_index = frame.loc.get_func(),
        .pc = frame.loc.get_insn(),
        .locals = frame.locals,
        .stack = frame.stack,
    });
  }

  // store the globals, table, and memory
  store_globals(&exec_env);
  store_table(&exec_env);
  wanco::chkpt.memory_size = exec_env.memory_size;

  // write snapshot
  {
    std::ofstream ofs("checkpoint.pb");
    encode_checkpoint_proto(ofs, wanco::chkpt, exec_env.memory_base);
    Info() << "Snapshot has been saved to checkpoint.pb" << '\n';

    auto time = std::chrono::duration_cast<std::chrono::microseconds>(
                    std::chrono::system_clock::now().time_since_epoch())
                    .count();
    time = time - wanco::CHKPT_START_TIME;
    // TODO(tamaron): remove this (research purpose)
    std::ofstream chktime("chkpt-time.txt");
    chktime << time << '\n';
  }
  exit(0);
}

static void allocate_polling_page() {
  void *polling_page =
      mmap((void *)POLLING_PAGE_BEGIN, POLLING_PAGE_END - POLLING_PAGE_BEGIN,
           PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  if (polling_page == MAP_FAILED) {
    Fatal() << "Failed to mmap a polling page" << '\n';
    exit(1);
  }
}

static void setup_signal_handlers() {
  signal(SIGCHKPT, signal_chkpt_handler);

  struct sigaction sa;
  sa.sa_sigaction = signal_segv_handler;
  sa.sa_flags = SA_SIGINFO;
  sigemptyset(&sa.sa_mask);
  sigaction(SIGSEGV, &sa, NULL);
}

static void setup_supervisor_thread() {
  efd = eventfd(0, 0);
  if (efd == -1) {
    perror("eventfd");
    exit(1);
  }

  // spawn a thread which checks if the main thread is suspended and performs
  // checkpoint
  pthread_t tid;
  int err = pthread_create(&tid, NULL, supervisor_thread, NULL);
  if (err != 0) {
    Fatal() << "Failed to create supervisor thread" << '\n';
    exit(1);
  }
}

static ExecEnv init_env(int argc, char **argv) {
  // Allocate memory
  int const memory_size = INIT_MEMORY_SIZE;
  int8_t *memory = allocate_memory(memory_size);
  // Initialize exec_env
  ExecEnv exec_env = ExecEnv{
      .memory_base = memory,
      .memory_size = memory_size,
      .migration_state = MigrationState::STATE_NONE,
      .argc = argc,
      .argv = reinterpret_cast<uint8_t **>(argv),
  };
  return exec_env;
}

static ExecEnv restore_exec_env(const std::string &restore_file, int argc,
                                char **argv) {
  // Restore from checkpoint
  std::ifstream ifs{restore_file};
  if (!ifs.is_open()) {
    Fatal() << "Failed to open checkpoint file: " << restore_file << '\n';
    exit(1);
  }

  int8_t *memory = nullptr;
  if (!restore_file.ends_with(".pb")) {
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
  ExecEnv exec_env = ExecEnv{
      .memory_base = memory,
      .memory_size = chkpt.memory_size,
      .migration_state = MigrationState::STATE_RESTORE,
      .argc = argc,
      .argv = reinterpret_cast<uint8_t **>(argv),
  };
  return exec_env;
}

static void finalize_legacy_checkpoint() {
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
}

static auto wanco_main(int argc, char **argv) -> int {
  setup_signal_handlers();
  setup_supervisor_thread();
  allocate_polling_page();
  Config const config = parse_from_args(argc, argv);

  if (config.restore_file.empty()) {
    exec_env = init_env(argc, argv);
  } else {
    RESTORE_START_TIME =
        std::chrono::duration_cast<std::chrono::microseconds>(
            std::chrono::system_clock::now().time_since_epoch())
            .count();
    exec_env = restore_exec_env(config.restore_file, argc, argv);
  }

  aot_main(&exec_env);

  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT_CONTINUE) {
    finalize_legacy_checkpoint();
  }

  // cleanup
  munmap(exec_env.memory_base, exec_env.memory_size * PAGE_SIZE);
  return 0;
}

} // namespace wanco

auto main(int argc, char **argv) -> int {
  return wanco::wanco_main(argc, argv);
}
