#include "aot.h"
#include "chkpt.h"
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

  int8_t *res = (int8_t *)mmap(NULL, num_bytes, PROT_READ | PROT_WRITE,
                               MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
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

  if (config.restore_file.empty()) {
    // Allocate memory
    int memory_size = INIT_MEMORY_SIZE;
    if (config.use_llvm_layout) {
      // Override memory size to 4GB
      memory_size = 64;
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
    std::ofstream ofs("checkpoint.json");
    encode_checkpoint_json(ofs, chkpt);
    std::cerr << "[info] Snapshot saved to checkpoint.json" << std::endl;
  }

  // cleanup
  munmap(exec_env.memory_base, exec_env.memory_size * PAGE_SIZE);
  return 0;
}
