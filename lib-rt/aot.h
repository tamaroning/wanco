#pragma once
#include "chkpt/chkpt.h"
#include "v2/chkpt_v2.h"
#include "wanco.h"
#include <cstdint>

namespace wanco {

// 10 and 12 are reserved for SIGUSR1 and SIGUSR2
const int SIGCHKPT = 10;

enum class MigrationState : int32_t {
  STATE_NONE = 0,
  STATE_CHECKPOINT_START = 1,
  STATE_CHECKPOINT_CONTINUE = 2,
  STATE_RESTORE = 3,
};

extern "C" Checkpoint chkpt;
extern "C" CheckpointV2 chkpt_v2;

} // namespace wanco

extern "C" struct ExecEnv {
  int8_t *memory_base;
  int32_t memory_size;
  wanco::MigrationState migration_state;
  int32_t argc;
  uint8_t **argv;
};

// defined in wasm AOT module
extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

// defined in wrt.c
extern "C" ExecEnv exec_env;
