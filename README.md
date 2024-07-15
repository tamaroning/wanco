# wanco

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler.

## Build

Prerequisites:
- C++ compiler
- Makefile
- Cargo

First you need to clone the project:
```
git clone git@github.com:tamaroning/wanco.git
cd wanco
```

To build and install the runtime library (libwanco.a), run the following commands.
Libraries (libwanco.a) will be installed in /usr/local/lib/.

```
lib/cpp
make
sudo make install
```

Build the wanco compiler:
```
cargo build --release
cp target/release/wanco .
```

## Run

Before running the compiler, make sure that a C++ compiler can be invoked via the `c++` command and the runtime library (libwanco.a) is installed in /usr/local/lib/.
Specify an input file which is a WebAssembly text or binary format.

```
$ wanco examples/hello.wat -o hello
$ ./hello
Hello World!
```

### Compile and assemble only

If you do not want to link the object files, specify the `-c` option.
LLVM assembly file (`.ll`) will also be generated.

```
wanco examples/hello.wat -c -o hello.o
```

After that, you can link it with the runtime library together by using C++ compiler.

```
c++ -no-pie hello.o /usr/local/lib/libwanco.a -o hello
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
