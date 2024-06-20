#include <stdint.h>
#include <stdlib.h>

extern int8_t* memory_base;
extern const int global_mem_size;
const int PAGE_SIZE = 65536;

extern void wanco_main();

int main() {
    memory_base = malloc(global_mem_size * PAGE_SIZE);
    wanco_main();
    return 0;
}
