#include <stdint.h>
#include <stdio.h>

extern int8_t* memory_base;

static void dump_memory(int32_t offset, int32_t len) {
    printf("### Memory dump:\n");
    for (int i = 0; i < 0x10; i++) {
        printf("%04x | ", i * 0x10);

        for (int j = 0; j < 0x10; j++) {
            printf("%02x ", (uint8_t) memory_base[i * 0x10 + j]);
            printf(" ");
        }

        printf("\n");
    }
    printf("### Memory dump end\n");
}
