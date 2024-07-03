#include "exec_env.h"
#include "llvm-libunwind/unwind.h"
#include <cstdlib>
#include <iostream>

extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

const int32_t PAGE_SIZE = 65536;

void dump_checkpoint(Checkpoint *chkpt) {
  std::cout << "Frames:" << std::endl;
  for (auto &frame : chkpt->frames) {
    std::cout << "  Locals:" << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "    " << local.to_string() << std::endl;
    }
  }

  std::cout << "Stack:" << std::endl;
  for (auto &value : chkpt->stack) {
    std::cout << "  " << value.to_string() << std::endl;
  }
}

int main() {
  // Allocate linear memory
  int8_t *memory_base = (int8_t *)malloc(INIT_MEMORY_SIZE * PAGE_SIZE);
  if (memory_base == NULL) {
    std::cerr << "Failed to allocate linear memory ("
              << INIT_MEMORY_SIZE * PAGE_SIZE << " bytes)" << std::endl;
    return -1;
  }

  Checkpoint *chkpt = new Checkpoint();
  ExecEnv exec_env = {
      .memory_base = memory_base,
      .memory_size = INIT_MEMORY_SIZE,
      .chkpt = chkpt,
  };
  try {
    aot_main(&exec_env);
  } catch(int e) {
    std::cout << "Caught exception: " << e << std::endl;
  }

  dump_checkpoint(chkpt);

  delete (chkpt);
  free(memory_base);
  return 0;
}

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

/*
** checkpoint related functions
*/

extern "C" void throw_exception() {
   throw 1;
  //_Unwind_RaiseException(&exception);
}

// locals
extern "C" void new_frame(ExecEnv *exec_env) {
  std::cout << "new_frame" << std::endl;
  //exec_env->chkpt->frames.push_back(Frame());
  std::cout << "new_frame done" << std::endl;
}

extern "C" void add_local_i32(ExecEnv *exec_env, int32_t i32) {
  std::cout << "add_local_i32" << std::endl;
  exec_env->chkpt->frames.back().locals.push_back(Value(i32));
  std::cout << "add_local_i32 done" << std::endl;
}

extern "C" void add_local_i64(ExecEnv *exec_env, int64_t i64) {
  std::cout << "add_local_i64" << std::endl;
  exec_env->chkpt->frames.back().locals.push_back(Value(i64));
  std::cout << "add_local_i64 done" << std::endl;
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
