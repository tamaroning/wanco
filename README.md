# wanco

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler which supports Checkpoint/Restore functionalities.


See [examples](./examples) for quick start.

## Build

Prerequisites:
- CMake and C++ compiler
- Cargo
- LLVM 17
- POSIX compliant OS (Linux, macOS, etc. NOTE: Windows is not tested)
- clang or clang++ (version 17 or later)

First you need to clone the project:
```
$ git clone git@github.com:tamaroning/wanco.git
$ cd wanco
```

To build and install the libraries (libwanco_rt.a and libwanco_wasi.a), run the following commands.
Wanco libraries (libwanco_rt.a and libwanco_wasi.a) will be installed in /usr/local/lib/.

```
$ mkdir build
$ cd build
$ cmake -DCMAKE_BUILD_TYPE=Release ../lib
$ make && sudo make install
$ cd ..
```

Build the wanco compiler:
```
$ cargo build --release
$ cp target/release/wanco .
```

## Run

To show the help, run:
```
$ wanco --help
```

Before running the compiler, add clang to the PATH environment variable or specify the path to clang or clang++ by using the `--clang-path` option.


Compile the hello-world example, run:

```
$ wanco examples/hello.wat -o hello
$ ./hello
Hello World!
```

### Using C/R functionalities

Compile a WebAssembly file with C/R enabled and run it:

```
$ ./wanco --checkpoint --restore demo/fib.wat
$ a.out
```

While tje process is running, you can trigger checkpoint by sending `SIGUSR1` signal from another teminal:

(The running process is automatically terminated and the snapshot file is created.)

```
$ pkill -10 a.out
```

To restore the execution, run:

```
$ ./a.out --restore checkpoint.json
```

Note: The C/R feature is still experimental and may not work correctly.

### Compile and assemble only

If you do not want to link the object files, specify the `-c` option.
LLVM assembly file (`.ll`) will be generated.

```
$ wanco examples/hello.wat -c -o hello.ll
```

After that, you can link it with the runtime library together by using clang

```
$ clang -flto -no-pie hello.ll /usr/local/lib/libwanco_rt.a /usr/local/lib/libwanco_wasi.a -o hello
```

## Test

Run

```
$ cargo test
```

## TODO

- [x] WASI preview 0
- [ ] WASI preview 1
- [ ] WASI preview 2
- [ ] WASI NN
- [x] Checkpoint
    - tables are not supported
- [x] Restore
    - tables are not supported
- [ ] cross-compilation
- [ ] dynamic-memory

## LICENSE

MIT
