#include "chkpt.h"
#include "exec_env.h"
#include <cassert>
#include <csignal>
#include <cstdlib>
#include <fstream>
#include <iostream>

const int32_t PAGE_SIZE = 65536;
// 10 and 12 are reserved for SIGUSR1 and SIGUSR2
const int SIGCHKPT = 10;

// execution environment
ExecEnv exec_env;

// from wasm AOT module
extern "C" const int32_t INIT_MEMORY_SIZE;
extern "C" void aot_main(ExecEnv *);

// forward decl
void dump_checkpoint(Checkpoint *chkpt);

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages);

void signal_chkpt_handler(int signum) {
  assert(signum == SIGCHKPT && "Unexpected signal");
  exec_env.migration_state = MigrationState::STATE_CHECKPOINT;
}

int main() {
  // Initialize exec env
  Checkpoint *chkpt = new Checkpoint();
  exec_env = {
      .memory_base = (int8_t *)malloc(INIT_MEMORY_SIZE * PAGE_SIZE),
      .memory_size = INIT_MEMORY_SIZE,
      .migration_state = MigrationState::STATE_NONE,
      .chkpt = chkpt,
  };

  // Register signal handler
  signal(SIGCHKPT, signal_chkpt_handler);

  aot_main(&exec_env);

  // TODO: dump to json
  if (exec_env.migration_state == MigrationState::STATE_CHECKPOINT) {
    dump_checkpoint(chkpt);
    std::ofstream ofs("checkpoint.json");
    encode_checkpoint_json(ofs, chkpt);
  }

  delete (chkpt);
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

// locals
extern "C" void push_frame(ExecEnv *exec_env) {
  exec_env->chkpt->frames.push_back(Frame());
}

extern "C" void set_pc_to_frame(ExecEnv *exec_env, int32_t fn_index,
                                int32_t pc) {
  exec_env->chkpt->frames.back().fn_index = fn_index;
  exec_env->chkpt->frames.back().pc = pc;
}

extern "C" void push_local_i32(ExecEnv *exec_env, int32_t i32) {
  exec_env->chkpt->frames.back().locals.push_back(Value(i32));
}

extern "C" void push_local_i64(ExecEnv *exec_env, int64_t i64) {
  exec_env->chkpt->frames.back().locals.push_back(Value(i64));
}

extern "C" void push_local_f32(ExecEnv *exec_env, float f32) {
  exec_env->chkpt->frames.back().locals.push_back(Value(f32));
}

extern "C" void push_local_f64(ExecEnv *exec_env, double f64) {
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
extern "C" void push_global_i32(ExecEnv *exec_env, int32_t i32) {
  exec_env->chkpt->globals.push_back(Value(i32));
}

extern "C" void push_global_i64(ExecEnv *exec_env, int64_t i64) {
  exec_env->chkpt->globals.push_back(Value(i64));
}

extern "C" void push_global_f32(ExecEnv *exec_env, float f32) {
  exec_env->chkpt->globals.push_back(Value(f32));
}

extern "C" void push_global_f64(ExecEnv *exec_env, double f64) {
  exec_env->chkpt->globals.push_back(Value(f64));
}

void dump_checkpoint(Checkpoint *chkpt) {
  std::cout << "Frames: " << (chkpt->frames.empty() ? "(empty)" : "")
            << std::endl;
  for (size_t i = 0; i < chkpt->frames.size(); i++) {
    const Frame &frame = chkpt->frames[i];
    std::cout << "  Frame[" << i << "]" << std::endl;
    std::cout << "    Location: Op[" << frame.pc << "] at Func["
              << frame.fn_index << "]" << std::endl;
    std::cout << "    Locals:" << (frame.locals.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "      " << local.to_string() << std::endl;
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

// Restore
extern "C" void pop_front_frame(ExecEnv *exec_env) {
  exec_env->chkpt->frames.pop_front();
  if (exec_env->chkpt->frames.empty()) {
    exec_env->migration_state = MigrationState::STATE_NONE;
  }
}

extern "C" int32_t get_pc_from_frame(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->frames.empty() && "No frame to restore");
  assert(exec_env->migration_state == MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  return exec_env->chkpt->frames.front().pc;
}

extern "C" int32_t pop_front_local_i32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->frames.empty() && "No frame to restore");
  assert(!exec_env->chkpt->frames.back().locals.empty() && "No local to pop");
  Value v = exec_env->chkpt->frames.back().locals.front();
  exec_env->chkpt->frames.back().locals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_local_i64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->frames.empty() && "No frame to restore");
  assert(!exec_env->chkpt->frames.back().locals.empty() && "No local to pop");
  Value v = exec_env->chkpt->frames.back().locals.front();
  exec_env->chkpt->frames.back().locals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_local_f32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->frames.empty() && "No frame to restore");
  assert(!exec_env->chkpt->frames.back().locals.empty() && "No local to pop");
  Value v = exec_env->chkpt->frames.back().locals.front();
  exec_env->chkpt->frames.back().locals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_local_f64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->frames.empty() && "No frame to restore");
  assert(!exec_env->chkpt->frames.back().locals.empty() && "No local to pop");
  Value v = exec_env->chkpt->frames.back().locals.front();
  exec_env->chkpt->frames.back().locals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_front_i32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->stack.empty() && "Stack empty");
  Value v = exec_env->chkpt->stack.front();
  exec_env->chkpt->stack.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_i64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->stack.empty() && "Stack empty");
  Value v = exec_env->chkpt->stack.front();
  exec_env->chkpt->stack.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_f32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->stack.empty() && "Stack empty");
  Value v = exec_env->chkpt->stack.front();
  exec_env->chkpt->stack.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_f64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->stack.empty() && "Stack empty");
  Value v = exec_env->chkpt->stack.front();
  exec_env->chkpt->stack.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_front_global_i32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->globals.empty() && "No global to pop");
  Value v = exec_env->chkpt->globals.front();
  exec_env->chkpt->globals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_global_i64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->globals.empty() && "No global to pop");
  Value v = exec_env->chkpt->globals.front();
  exec_env->chkpt->globals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_global_f32(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->globals.empty() && "No global to pop");
  Value v = exec_env->chkpt->globals.front();
  exec_env->chkpt->globals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_global_f64(ExecEnv *exec_env) {
  assert(!exec_env->chkpt->globals.empty() && "No global to pop");
  Value v = exec_env->chkpt->globals.front();
  exec_env->chkpt->globals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}
