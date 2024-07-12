#include "exec_env.h"
#include <cstdint>
#include <cstdlib>
#include <iostream>

/* Print a string from memory */
extern "C" void print(ExecEnv *exec_env, int64_t offset, int32_t len) {
  for (int i = 0; i < len; i++) {
    putchar(exec_env->memory_base[offset + i]);
  }
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

extern "C" WasiError fd_write(ExecEnv *exec_env, int fd, int buf_iovec_addr,
                              int vec_len, int size_addr) {
  char *iovec_ptr = (char *)&exec_env->memory_base[buf_iovec_addr];
  IoVec *iovec = (IoVec *)iovec_ptr;

  int len = 0;
  for (int i = 0; i < vec_len; i++) {
    char *buf_ptr = (char *)(exec_env->memory_base + iovec[i].iov_base);
    size_t buf_len = iovec[i].iov_len;
    for (size_t j = 0; j < buf_len; j++) {
      printf("%c", buf_ptr[j]);
    }
    len += buf_len;
  }
  int *size_ptr = (int *)(exec_env->memory_base + size_addr);
  *size_ptr = len;
  return SUCCESS;
}

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
