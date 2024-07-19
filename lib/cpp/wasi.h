#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct ExecEnv {
  uint8_t *memory;
  int32_t memory_size;
  int32_t migration_state;
};

extern "C" {

int32_t fd_write(const ExecEnv *exec_env, int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3);

} // extern "C"
