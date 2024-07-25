#pragma once
#include <cstdint>
#include "chkpt.h"

const int32_t PAGE_SIZE = 65536;

// 10 and 12 are reserved for SIGUSR1 and SIGUSR2
const int SIGCHKPT = 10;

enum class MigrationState : int32_t {
  STATE_NONE = 0,
  STATE_CHECKPOINT_START = 1,
  STATE_CHECKPOINT_CONTINUE = 2,
  STATE_RESTORE = 3,
};

extern "C" struct ExecEnv {
  int8_t *memory_base;
  int32_t memory_size;
  MigrationState migration_state;
  int32_t argc;
  uint8_t **argv;
};

// from wasm AOT module
extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

extern "C" ExecEnv exec_env;
extern "C" Checkpoint chkpt;
