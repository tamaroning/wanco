#include "aot.h"
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <sys/mman.h>
#include <thread>
#include <unistd.h>

namespace wanco {

int32_t extend_memory(ExecEnv *exec_env, int32_t inc_pages);

}

extern "C" int32_t memory_grow(ExecEnv *exec_env, int32_t inc_pages) {
  return wanco::extend_memory(exec_env, inc_pages);
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

extern "C" void sleep_msec(ExecEnv *exec_env, int32_t ms) {
  std::this_thread::sleep_for(std::chrono::milliseconds(ms));
}

/*
** checkpoint related functions
*/

// locals
extern "C" void push_frame(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.push_back(wanco::Frame());
}

extern "C" void set_pc_to_frame(ExecEnv *exec_env, int32_t fn_index,
                                int32_t pc) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.back().fn_index = fn_index;
  wanco::chkpt.frames.back().pc = pc;
}

extern "C" void push_local_i32(ExecEnv *exec_env, int32_t i32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.back().locals.push_back(wanco::Value(i32));
}

extern "C" void push_local_i64(ExecEnv *exec_env, int64_t i64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.back().locals.push_back(wanco::Value(i64));
}

extern "C" void push_local_f32(ExecEnv *exec_env, float f32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.back().locals.push_back(wanco::Value(f32));
}

extern "C" void push_local_f64(ExecEnv *exec_env, double f64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  wanco::chkpt.frames.back().locals.push_back(wanco::Value(f64));
}

// stack
extern "C" void push_i32(ExecEnv *exec_env, int32_t i32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to push");
  // Debug() << "call to push_i32 -> " << i32 << std::endl;
  wanco::chkpt.frames.back().stack.push_back(wanco::Value(i32));
}

extern "C" void push_i64(ExecEnv *exec_env, int64_t i64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  // Debug() << "call to push_i64 -> " << i64 << std::endl;
  wanco::chkpt.frames.back().stack.push_back(wanco::Value(i64));
}

extern "C" void push_f32(ExecEnv *exec_env, float f32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  // Debug() << "call to push_f32 -> " << f32<< std::endl;
  wanco::chkpt.frames.back().stack.push_back(wanco::Value(f32));
}

extern "C" void push_f64(ExecEnv *exec_env, double f64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  // Debug() << "call to push_f64 -> " << f64<< std::endl;
  wanco::chkpt.frames.back().stack.push_back(wanco::Value(f64));
}

// globals
extern "C" void push_global_i32(ExecEnv *exec_env, int32_t i32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  Debug() << "call to push_global_i32 -> " << i32 << std::endl;
  wanco::chkpt.globals.push_back(wanco::Value(i32));
}

extern "C" void push_global_i64(ExecEnv *exec_env, int64_t i64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  Debug() << "call to push_global_i64 -> " << i64 << std::endl;
  wanco::chkpt.globals.push_back(wanco::Value(i64));
}

extern "C" void push_global_f32(ExecEnv *exec_env, float f32) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  Debug() << "call to push_global_f32 -> " << f32 << std::endl;
  wanco::chkpt.globals.push_back(wanco::Value(f32));
}

extern "C" void push_global_f64(ExecEnv *exec_env, double f64) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  Debug() << "call to push_global_f64 -> " << f64 << std::endl;
  wanco::chkpt.globals.push_back(wanco::Value(f64));
}

// table
extern "C" void push_table_index(ExecEnv *exec_env, int32_t index) {
  ASSERT(exec_env->migration_state ==
             wanco::MigrationState::STATE_CHECKPOINT_CONTINUE &&
         "Invalid migration state");
  Debug() << "call to push_table_index -> " << index << std::endl;
  wanco::chkpt.table.push_back(index);
}

namespace wanco {
/*
void dump_exec_env(ExecEnv &exec_env) {
  std::cout << "Migration state: " << (int)exec_env.migration_state
            << std::endl;
  std::cout << "Memory base: 0x" << std::hex << (void *)exec_env.memory_base
            << std::endl;
  std::cout << "Memory size: " << exec_env.memory_size << std::endl;
}

void dump_checkpoint(wanco::Checkpoint &chkpt) {
  std::cout << "Checkpoint" << std::endl;
  std::cout << "Frames: " << (chkpt.frames.empty() ? "(empty)" : "")
            << std::endl;
  for (size_t i = 0; i < chkpt.frames.size(); i++) {
    const wanco::Frame &frame = chkpt.frames[i];
    std::cout << "  Frame[" << i << "]" << std::endl;
    std::cout << "    Location: Op[" << frame.pc << "] at Func["
              << frame.fn_index << "]" << std::endl;
    std::cout << "    Locals:" << (frame.locals.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &local : frame.locals) {
      std::cout << "      " << local.to_string() << std::endl;
    }
    std::cout << "Stack:" << (frame.stack.empty() ? "(empty)" : "")
              << std::endl;
    for (auto &value : frame.stack) {
      std::cout << "  " << value.to_string() << std::endl;
    }
  }

  std::cout << "Globals" << (chkpt.globals.empty() ? "(empty)" : "")
            << std::endl;
  for (auto &value : chkpt.globals) {
    std::cout << "  " << value.to_string() << std::endl;
  }
}
*/

void check_restore_finished(ExecEnv *exec_env) {
  // Restore is completed if there are no more frames to restore
  if (wanco::chkpt.frames.empty()) {
    Debug() << " Restore completed" << std::endl;
    exec_env->migration_state = wanco::MigrationState::STATE_NONE;
  }
}

} // namespace wanco

// Restore
extern "C" void pop_front_frame(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  wanco::Frame &frame = wanco::chkpt.frames.front();
  Debug() << "call to pop_front_frame -> Fn[" << frame.fn_index << "]"
          << std::endl;

  if (!frame.locals.empty()) {
    Fatal() << "Locals not empty" << std::endl;
    exit(1);
  }

  wanco::chkpt.frames.pop_front();
  wanco::check_restore_finished(exec_env);
}

extern "C" bool frame_is_empty(ExecEnv *exec_env) {
  return wanco::chkpt.frames.empty();
}

extern "C" int32_t get_pc_from_frame(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  int32_t ret = wanco::chkpt.frames.front().pc;
  Debug() << "call to get_pc_from_frame -> " << ret << std::endl;
  return ret;
}

extern "C" int32_t pop_front_local_i32(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  ASSERT(!wanco::chkpt.frames.front().locals.empty() && "No local to pop");
  wanco::Value v = wanco::chkpt.frames.front().locals.front();
  Debug() << "call to pop_front_local -> " << v.to_string() << std::endl;
  wanco::chkpt.frames.front().locals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_local_i64(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  ASSERT(!wanco::chkpt.frames.front().locals.empty() && "No local to pop");
  wanco::Value v = wanco::chkpt.frames.front().locals.front();
  Debug() << "call to pop_front_local -> " << v.to_string() << std::endl;
  wanco::chkpt.frames.front().locals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_local_f32(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  ASSERT(!wanco::chkpt.frames.front().locals.empty() && "No local to pop");
  wanco::Value v = wanco::chkpt.frames.front().locals.front();
  Debug() << "call to pop_front_local -> " << v.to_string() << std::endl;
  wanco::chkpt.frames.front().locals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_local_f64(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.frames.empty() && "No frame to restore");
  ASSERT(!wanco::chkpt.frames.front().locals.empty() && "No local to pop");
  wanco::Value v = wanco::chkpt.frames.front().locals.front();
  Debug() << "call to pop_front_local -> " << v.to_string() << std::endl;
  wanco::chkpt.frames.front().locals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F64 && "Invalid type");
  return v.f64;
}

extern "C" int32_t pop_i32(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.restore_stack.empty() && "Stack empty");
  wanco::Value v = wanco::chkpt.restore_stack.front();
  Debug() << "call to pop -> " << v.to_string() << std::endl;
  wanco::chkpt.restore_stack.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I32 && "Invalid type");
  // wanco::check_restore_finished (exec_env);
  return v.i32;
}

extern "C" int64_t pop_i64(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.restore_stack.empty() && "Stack empty");
  wanco::Value v = wanco::chkpt.restore_stack.front();
  Debug() << "call to pop -> " << v.to_string() << std::endl;
  wanco::chkpt.restore_stack.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I64 && "Invalid type");
  // wanco::check_restore_finished (exec_env);
  return v.i64;
}

extern "C" float pop_f32(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.restore_stack.empty() && "Stack empty");
  wanco::Value v = wanco::chkpt.restore_stack.front();
  Debug() << "call to pop -> " << v.to_string() << std::endl;
  wanco::chkpt.restore_stack.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F32 && "Invalid type");
  // wanco::check_restore_finished (exec_env);
  return v.f32;
}

extern "C" double pop_f64(ExecEnv *exec_env) {
  ASSERT(!wanco::chkpt.restore_stack.empty() && "Stack empty");
  wanco::Value v = wanco::chkpt.restore_stack.front();
  Debug() << "call to pop -> " << v.to_string() << std::endl;
  wanco::chkpt.restore_stack.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F64 && "Invalid type");
  // wanco::check_restore_finished (exec_env);
  return v.f64;
}
extern "C" int32_t pop_front_global_i32(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.globals.empty() && "No global to pop");
  wanco::Value v = wanco::chkpt.globals.front();
  Debug() << "call to pop_front_global -> " << v.to_string() << std::endl;
  wanco::chkpt.globals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I32 && "Invalid type");
  return v.i32;
}

extern "C" int64_t pop_front_global_i64(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.globals.empty() && "No global to pop");
  wanco::Value v = wanco::chkpt.globals.front();
  Debug() << "call to pop_front_global -> " << v.to_string() << std::endl;
  wanco::chkpt.globals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::I64 && "Invalid type");
  return v.i64;
}

extern "C" float pop_front_global_f32(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.globals.empty() && "No global to pop");
  wanco::Value v = wanco::chkpt.globals.front();
  Debug() << "call to pop_front_global -> " << v.to_string() << std::endl;
  wanco::chkpt.globals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F32 && "Invalid type");
  return v.f32;
}

extern "C" double pop_front_global_f64(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.globals.empty() && "No global to pop");
  wanco::Value v = wanco::chkpt.globals.front();
  Debug() << "call to pop_front_global -> " << v.to_string() << std::endl;
  wanco::chkpt.globals.pop_front();
  ASSERT(v.get_type() == wanco::Value::Type::F64 && "Invalid type");
  return v.f64;
}

// table
extern "C" int32_t pop_front_table_index(ExecEnv *exec_env) {
  ASSERT(exec_env->migration_state == wanco::MigrationState::STATE_RESTORE &&
         "Invalid migration state");
  ASSERT(!wanco::chkpt.table.empty() && "Table is empty");
  int32_t idx = wanco::chkpt.table.front();
  Debug() << "call to pop_front_table_index -> " << idx << std::endl;
  wanco::chkpt.table.pop_front();
  return idx;
}

/*
** checkpoint related functions (v2)
*/

/*
extern "C" void
start_checkpoint_v2 (ExecEnv *exec_env)
{
  Info() << " Intercepted" << std::endl;
  ASSERT (exec_env->migration_state
            == wanco::MigrationState::STATE_CHECKPOINT_START
          && "Invalid migration state");
  // exec_env->migration_state = wanco::MigrationState::STATE_CHECKPOINT_START;
  // TODO: wip
  auto frames = get_stack_trace ();
  std::optional<std::vector<uint8_t>> stackmap_section_opt
    = get_section_data (".llvm_stackmaps");
  if (!stackmap_section_opt.has_value ())
    {
      std::cerr << "Error: unable to obtain stackmap section" << std::endl;
      std::exit (1);
    }
  std::vector<uint8_t> stackmap_section = stackmap_section_opt.value ();
  Stackmap::Stackmap stackmap = parse_stackmap (stackmap_section);
  std::cerr << stackmap_to_string (stackmap);

  Info() << " Killed"<< std::endl;
  std::exit (0);
}

extern "C" void
push_global_i32_v2 (ExecEnv *exec_env, int32_t i32)
{
  ASSERT (exec_env->migration_state
            == wanco::MigrationState::STATE_CHECKPOINT_START
          && "Invalid migration state");
  wanco::chkpt_v2.globals.push_back (wanco::Value (i32));
}

extern "C" void
push_global_i64_v2 (ExecEnv *exec_env, int64_t i64)
{
  ASSERT (exec_env->migration_state
            == wanco::MigrationState::STATE_CHECKPOINT_START
          && "Invalid migration state");
  wanco::chkpt_v2.globals.push_back (wanco::Value (i64));
}

extern "C" void
push_global_f32_v2 (ExecEnv *exec_env, float f32)
{
  ASSERT (exec_env->migration_state
            == wanco::MigrationState::STATE_CHECKPOINT_START
          && "Invalid migration state");
  wanco::chkpt_v2.globals.push_back (wanco::Value (f32));
}

extern "C" void
push_global_f64_v2 (ExecEnv *exec_env, double f64)
{
  ASSERT (exec_env->migration_state
            == wanco::MigrationState::STATE_CHECKPOINT_START
          && "Invalid migration state");
  wanco::chkpt_v2.globals.push_back (wanco::Value (f64));
}
*/
