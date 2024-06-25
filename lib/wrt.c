#include <stdint.h>
#include <stdlib.h>

extern int8_t* memory_base;
extern int32_t global_mem_size;
const int32_t PAGE_SIZE = 65536;

extern void wanco_main();

int32_t memory_size() {
    return global_mem_size;
}

int32_t memory_glow(int32_t inc_pages) {
    int32_t old_size = global_mem_size;
    int32_t new_size = old_size + inc_pages;
    
    void* res = realloc(memory_base, new_size * PAGE_SIZE);
    if (res == NULL)
        return -1;

    memory_base = res;
    global_mem_size = new_size;
    return old_size;
}

int main() {
    memory_base = malloc(global_mem_size * PAGE_SIZE);
    wanco_main();
    return 0;
}
