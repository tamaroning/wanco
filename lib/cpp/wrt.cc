#include "exec_env.h"
#include <cstdlib>
#include <iostream>

extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

const int32_t PAGE_SIZE = 65536;

void dump_checkpoint(Checkpoint *chkpt);

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages) {
  int32_t old_size = exec_env->memory_size;
  int32_t new_size = old_size + inc_pages;

  int8_t *res = (int8_t *)realloc(exec_env->memory_base, new_size * PAGE_SIZE);
  if (res == NULL)
    return -1;

  exec_env->memory_base = res;
  exec_env->memory_size = new_size;
  return old_size;
}

int main() {
  Checkpoint *chkpt = new Checkpoint();
  ExecEnv exec_env = {
      .memory_base = (int8_t *)malloc(INIT_MEMORY_SIZE * PAGE_SIZE),
      .memory_size = INIT_MEMORY_SIZE,
      .migration_state = MigrationState::STATE_NONE,
      .chkpt = chkpt,
  };
  aot_main(&exec_env);

  // TODO: dump to json
  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT)
    dump_checkpoint(chkpt);

  delete (chkpt);
  return 0;
}

/*
** checkpoint related functions
*/

// locals
extern "C" void new_frame(ExecEnv *exec_env) {
  exec_env->chkpt->frames.push_back(Frame());
}

extern "C" void add_local_i32(ExecEnv *exec_env, int32_t i32) {
  exec_env->chkpt->frames.back().locals.push_back(Value(i32));
}

extern "C" void add_local_i64(ExecEnv *exec_env, int64_t i64) {
  exec_env->chkpt->frames.back().locals.push_back(Value(i64));
}

extern "C" void add_local_f32(ExecEnv *exec_env, float f32) {
  exec_env->chkpt->frames.back().locals.push_back(Value(f32));
}

extern "C" void add_local_f64(ExecEnv *exec_env, double f64) {
  exec_env->chkpt->frames.back().locals.push_back(Value(f64));
}

// stack
extern "C" void push_i32(ExecEnv *exec_env, int32_t i32) {
  exec_env->chkpt->stack.push_back(Value(i32));
}

extern "C" void push_i64(ExecEnv *exec_env, int64_t i64) {
  exec_env->chkpt->stack.push_back(Value(i64));
}

extern "C" void push_f32(ExecEnv *exec_env, float f32) {
  exec_env->chkpt->stack.push_back(Value(f32));
}

extern "C" void push_f64(ExecEnv *exec_env, double f64) {
  exec_env->chkpt->stack.push_back(Value(f64));
}

// globals
extern "C" void add_global_i32(ExecEnv *exec_env, int32_t i32) {
  exec_env->chkpt->globals.push_back(Value(i32));
}

extern "C" void add_global_i64(ExecEnv *exec_env, int64_t i64) {
  exec_env->chkpt->globals.push_back(Value(i64));
}

extern "C" void add_global_f32(ExecEnv *exec_env, float f32) {
  exec_env->chkpt->globals.push_back(Value(f32));
}

extern "C" void add_global_f64(ExecEnv *exec_env, double f64) {
  exec_env->chkpt->globals.push_back(Value(f64));
}

void dump_checkpoint(Checkpoint *chkpt) {
  std::cout << "Frames: " << (chkpt->frames.empty() ? "(empty)" : "")
            << std::endl;
  for (auto &frame : chkpt->frames) {
    std::cout << "  Locals:" << (frame.locals.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "    " << local.to_string() << std::endl;
    }
  }

  std::cout << "Stack:" << (chkpt->stack.empty() ? "(empty)" : "") << std::endl;
  for (auto &value : chkpt->stack) {
    std::cout << "  " << value.to_string() << std::endl;
  }

  std::cout << "Globals" << (chkpt->globals.empty() ? "(empty)" : "")
            << std::endl;
  for (auto &value : chkpt->globals) {
    std::cout << "  " << value.to_string() << std::endl;
  }
}