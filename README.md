# wanco

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler.

## Build

Prerequisites:
- GCC (g++)
- Makefile
- Cargo

First you need to clone the project:
```
git clone git@github.com:tamaroning/wanco.git
cd wanco
```

Build the library and runtime:
```
(cd lib/cpp && make)
```

Build the wanco compiler:
```
cargo build --release
cp target/release/wanco .
```

## Run

Specify an input file which is a WebAssembly text or binary format.
```
wanco examples/hello.wat -o hello.o
```
Then link it with the runtime and library together:
```
g++ -no-pie hello.o lib/cpp/lib.o lib/cpp/wrt.o -o hello
```

> [!NOTE]  
> Use C++ compiler (g++ or clang++) to link the object files because the compiled object may require C++ exception handlers.

Finally, run the compiled module:
```
$ ./hello
Hello World!
```

## Test

Run
```
cargo test
```

## TODO

- WASI preview1 (https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md#modules)
    - use wasi-libc? (https://github.com/WebAssembly/wasi-libc)
- support sqlite-wasm
- compiler driver

## LICENSE

- tests/spec/: Apache-2.0
- others: MIT
