#include <stdint.h>
#include <stdlib.h>
#include "exec_env.h"

extern const int32_t INIT_MEMORY_SIZE;
const int32_t PAGE_SIZE = 65536;

extern void aot_main(ExecEnv*);

int32_t memory_grow(ExecEnv* exec_env, int32_t inc_pages) {
    int32_t old_size = exec_env->memory_size;
    int32_t new_size = old_size + inc_pages;

    void* res = realloc(exec_env->memory_base, new_size * PAGE_SIZE);
    if (res == NULL)
        return -1;

    exec_env->memory_base = res;
    exec_env->memory_size = new_size;
    return old_size;
}

int main() {
    ExecEnv exec_env = {
        .memory_base = (int8_t*) malloc(INIT_MEMORY_SIZE * PAGE_SIZE),
        .memory_size = INIT_MEMORY_SIZE,
    };
    aot_main(&exec_env);
    return 0;
}
