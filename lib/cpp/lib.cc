#include "exec_env.h"
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <thread>

/* Print a string from memory */
extern "C" void print(ExecEnv *exec_env, int32_t offset, int32_t len) {
  for (int i = 0; i < len; i++) {
    putchar(exec_env->memory_base[offset + i]);
  }
}

extern "C" void print_i32(ExecEnv *exec_env, int32_t i32) {
  std::cout << std::dec << i32 << std::endl;
}

extern "C" void sleep(ExecEnv *exec_env, int32_t ms) {
  std::this_thread::sleep_for(std::chrono::milliseconds(ms));
}
