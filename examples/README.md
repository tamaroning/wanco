# Quick Start

## Trying simple Hello world

```bash
$ wanco ./examples/hello.wat
$ ./a.out
Hello, World
```

## Trying llama2.c

```bash
$ cp benchmark/llama2-c
$ wanco llama2-c.wasm
# a.out can access only the current directory (./model.bin and ./tokenizer.bin)
$ ./a.out -- model.bin
Once upon a time, there was a little boy named Timmy. Timmy loved to play with his toys all day long. One day, Timmy found a truck in his room. He was playing with it and he didn't like his truck. The truck didn't know what to do, but Tim thought it was interesting. He said to his friends, "I like bugs, let's really have fun."
Suddenly, a big wind came and blew the truck out of the truck. The bug said, "Oh no! This is a funny truck!" Tim felt so sad and cried again. The truck shook his head and realized that it was a sluck that needed to help him com
achieved tok/s: 604.265403
```

## Using wasm files compiled from other languages

### C

First, install [WASI-SDK](https://github.com/WebAssembly/wasi-sdk) and use wasi-sdk clang:

```bash
$ <wasi-sdk>/bin/clang -target wasm32-wasi hello.c -o hello.wasm
$ wanco ./hello.wasm
$ ./a.out -- arg1 arg2
Hello, world!
argc: 3
argv[0]: ./a.out
argv[1]: arg1
argv[2]: arg2
```

NOTE: We do not support wasm modules compiled with Emscripten since it uses a
non-standard interfaces.

### Rust

TODO

## Trying Checkpoint/Restore

See [demo/README.md](./demo/README.md)
