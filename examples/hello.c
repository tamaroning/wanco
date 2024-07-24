// compile with:
//   <wasi-sdk>/bin/clang -target wasm32-wasi hello.c -o hello.wasm
//   wanco ./hello.wasm
// Run with: ./a.out -- arg1 arg2

#include <stdio.h>

int main(int argc, char **argv) {
  printf("Hello, world!\n");
  printf("argc: %d\n", argc);
  for (int i = 0; i < argc; i++) {
    printf("argv[%d]: %s\n", i, argv[i]);
  }
  return 0;
}
