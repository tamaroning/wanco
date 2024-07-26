#include "aot.h"
#include "chkpt.h"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <sys/mman.h>
#include <unistd.h>

int32_t extend_memory(ExecEnv *exec_env, int32_t inc_pages);

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages) {
  return extend_memory(exec_env, inc_pages);
}

/* Print a string from memory */
extern "C" void print(ExecEnv *exec_env, int32_t offset, int32_t len) {
  for (int i = 0; i < len; i++) {
    putchar(exec_env->memory_base[offset + i]);
  }
}

extern "C" void print_i32(ExecEnv *exec_env, int32_t i32) {
  std::cout << std::dec << i32 << std::endl;
}

/*
extern "C" void sleep(ExecEnv *exec_env, int32_t ms) {
  std::this_thread::sleep_for(std::chrono::milliseconds(ms));
}
*/

/*
** checkpoint related functions
*/

// locals
extern "C" void push_frame(ExecEnv *exec_env) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.push_back(Frame());
}

extern "C" void set_pc_to_frame(ExecEnv *exec_env, int32_t fn_index,
                                int32_t pc) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().fn_index = fn_index;
  chkpt.frames.back().pc = pc;
}

extern "C" void push_local_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(i32));
}

extern "C" void push_local_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(i64));
}

extern "C" void push_local_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(f32));
}

extern "C" void push_local_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.frames.back().locals.push_back(Value(f64));
}

// stack
extern "C" void push_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(i32));
}

extern "C" void push_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(i64));
}

extern "C" void push_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(f32));
}

extern "C" void push_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.stack.push_back(Value(f64));
}

// globals
extern "C" void push_global_i32(ExecEnv *exec_env, int32_t i32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(i32));
}

extern "C" void push_global_i64(ExecEnv *exec_env, int64_t i64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(i64));
}

extern "C" void push_global_f32(ExecEnv *exec_env, float f32) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(f32));
}

extern "C" void push_global_f64(ExecEnv *exec_env, double f64) {
  assert(exec_env->migration_state ==
             MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  chkpt.globals.push_back(Value(f64));
}

void dump_exec_env(ExecEnv &exec_env) {
  std::cout << "Migration state: " << (int)exec_env.migration_state
            << std::endl;
  std::cout << "Memory base: 0x" << std::hex << (void *)exec_env.memory_base
            << std::endl;
  std::cout << "Memory size: " << exec_env.memory_size << std::endl;
}

void dump_checkpoint(Checkpoint &chkpt) {
  std::cout << "Checkpoint" << std::endl;
  std::cout << "Frames: " << (chkpt.frames.empty() ? "(empty)" : "")
            << std::endl;
  for (size_t i = 0; i < chkpt.frames.size(); i++) {
    const Frame &frame = chkpt.frames[i];
    std::cout << "  Frame[" << i << "]" << std::endl;
    std::cout << "    Location: Op[" << frame.pc << "] at Func["
              << frame.fn_index << "]" << std::endl;
    std::cout << "    Locals:" << (frame.locals.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "      " << local.to_string() << std::endl;
    }
  }

  std::cout << "Stack:" << (chkpt.stack.empty() ? "(empty)" : "") << std::endl;
  for (auto &value : chkpt.stack) {
    std::cout << "  " << value.to_string() << std::endl;
  }

  std::cout << "Globals" << (chkpt.globals.empty() ? "(empty)" : "")
            << std::endl;
  for (auto &value : chkpt.globals) {
    std::cout << "  " << value.to_string() << std::endl;
  }
}

// Restore
extern "C" void pop_front_frame(ExecEnv *exec_env) {
  assert(exec_env->migration_state == MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  assert(!chkpt.frames.empty() && "No frame to restore");
  Frame &frame = chkpt.frames.front();
  std::cerr << "[debug] call to pop_front_frame -> Fn[" << frame.fn_index << "]"
            << std::endl;

  chkpt.frames.pop_front();
  // Restore is completed if there are no more frames to restore
  if (chkpt.frames.empty()) {
    std::cerr << "[debug] Restore completed" << std::endl;
    exec_env->migration_state = MigrationState::STATE_NONE;
    // chkpt = Checkpoint();
  }
}

extern "C" bool frame_is_empty(ExecEnv *exec_env) {
  return chkpt.frames.empty();
}

extern "C" int32_t get_pc_from_frame(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(exec_env->migration_state == MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  return chkpt.frames.front().pc;
}

extern "C" int32_t pop_front_local_i32(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_local_i64(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_local_f32(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_local_f64(ExecEnv *exec_env) {
  assert(!chkpt.frames.empty() && "No frame to restore");
  assert(!chkpt.frames.front().locals.empty() && "No local to pop");
  Value v = chkpt.frames.front().locals.front();
  std::cerr << "[debug] call to pop_front_local -> " << v.to_string()
            << std::endl;
  chkpt.frames.front().locals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_i32(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_i64(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_f32(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_f64(ExecEnv *exec_env) {
  assert(!chkpt.stack.empty() && "Stack empty");
  Value v = chkpt.stack.back();
  std::cerr << "[debug] call to pop -> " << v.to_string() << std::endl;
  chkpt.stack.pop_back();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_front_global_i32(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  std::cerr << "[debug] call to pop_front_global -> " << v.to_string()
            << std::endl;
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_global_i64(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  std::cerr << "[debug] call to pop_front_global -> " << v.to_string()
            << std::endl;
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_global_f32(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  std::cerr << "[debug] call to pop_front_global -> " << v.to_string()
            << std::endl;
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_global_f64(ExecEnv *exec_env) {
  assert(!chkpt.globals.empty() && "No global to pop");
  Value v = chkpt.globals.front();
  std::cerr << "[debug] call to pop_front_global -> " << v.to_string()
            << std::endl;
  chkpt.globals.pop_front();
  assert(v.get_type() == Value::Type::F64 && "Invalid type");
  return v.f64;
}
