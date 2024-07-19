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

/*
 * WASI API
 */

typedef struct {
  int iov_base;
  int iov_len;
} IoVec;

typedef enum {
  SUCCESS = 0,
  // Add other error types here
} WasiError;

extern "C" void proc_exit(ExecEnv *exec_env, int code) {
  // exit
  std::cerr << "[debug] proc_exit(" << code << ")" << std::endl;
  std::exit(code);
}

extern "C" WasiError environ_get(ExecEnv *exec_env, int environ,
                                 int environ_buf) {
  // TODO:
  std::cerr << "[debug] environ_get(" << environ << ", " << environ_buf << ")"
            << std::endl;
  return WasiError::SUCCESS;
}

extern "C" WasiError environ_sizes_get(ExecEnv *exec_env, int environ_count,
                                       int environ_buf_size) {
  // TODO:
  std::cerr << "[debug] environ_sizes_get(" << environ_count << ", "
            << environ_buf_size << ")" << std::endl;
  return WasiError::SUCCESS;
}
