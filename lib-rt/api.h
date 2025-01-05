#pragma once
#include "aot.h"

extern "C" void push_frame(ExecEnv *exec_env);

extern "C" void set_pc_to_frame(ExecEnv *exec_env, int32_t fn_index,
                                int32_t pc);

extern "C" void push_local_i32(ExecEnv *exec_env, int32_t i32);

extern "C" void push_local_i64(ExecEnv *exec_env, int64_t i64);

extern "C" void push_local_f32(ExecEnv *exec_env, float f32);

extern "C" void push_local_f64(ExecEnv *exec_env, double f64);

extern "C" void push_i32(ExecEnv *exec_env, int32_t i32);

extern "C" void push_i64(ExecEnv *exec_env, int64_t i64);

extern "C" void push_f32(ExecEnv *exec_env, float f32);

extern "C" void push_f64(ExecEnv *exec_env, double f64);
