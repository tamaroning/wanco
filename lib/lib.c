#include <stdint.h>
#include <stdio.h>

extern const int8_t memory_base[];
extern const int global_mem_size;

/* Print a string from memory */
void print(int64_t offset, int32_t len) {
    for (int i = 0; i < len; i++) {
        putchar(memory_base[offset + i]);
    }
}
