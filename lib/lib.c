#include <stdint.h>
#include <stdio.h>

extern const int8_t memory_base[];

/* Print a string from memory */
void print(int64_t offset, int32_t len) {
    for (int i = 0; i < len; i++) {
        putchar(memory_base[offset + i]);
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
    SUCCESS,
    // Add other error types here
} WasiError;

WasiError fd_write(int fd, int buf_iovec_addr, int vec_len, int size_addr) {
    char* iovec_ptr = (char*) &memory_base[buf_iovec_addr];
    IoVec* iovec = (IoVec*)iovec_ptr;

    printf("iov_base: 0x%x, iov_len: %d\n", iovec->iov_base, iovec->iov_len);

    int len = 0;
    for (int i = 0; i < vec_len; i++){
        char* buf_ptr = (char *)(memory_base + iovec[i].iov_base);
        size_t buf_len = iovec[i].iov_len;
        for (int j = 0; j < buf_len; j++){
            printf("%c", buf_ptr[j]);
        }
        len += buf_len;
    }
    int* size_ptr = (int *)(memory_base + size_addr);
    *size_ptr = len;
    return SUCCESS;
}
