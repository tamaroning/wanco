# wanco 🐶

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler which supports cross-platform (CPU and OS) Checkpoint/Restore functionalities. wanco is forked from [Wasker](https://github.com/mewz-project/wasker).


See [examples](./examples) for quick start.

## Build

Prerequisites:
- CMake and C++ compiler
- Cargo
- LLVM 17 (recommend to use llvm.sh if you are using apt)
- POSIX compliant OS (Linux, TODO: support macOS)
- clang or clang++ (version 17 or later)
- protocol buffer (Run `apt install libprotobuf-dev`)
    - For development, run `apt install libprotobuf-dev protobuf-compiler`
<!-- - libunwind-dev (Run `apt install libunwind-dev`) -->

First you need to clone the project:

```sh
$ git clone git@github.com:tamaroning/wanco.git
$ cd wanco
```

This project includes C++ projects and Rust projects.
To build the entire project, run the following commands.

```sh
$ mkdir build
$ cmake .. -DCMAKE_BUILD_TYPE=Release
```

To install runtime libraries, run the following commands:

```sh
$ sudo make install
```

On Linux, the runtime libraries are installed in `/usr/local/lib/` by default.

## Run

After building the project, you will find the `wanco` binary in the top of `build` directory.

Before compiling wasm modules, make sure to add clang to the PATH environment variable or to specify the path to clang or clang++ by using the `--clang-path` option. (clang/clang++ version 17 or later is required.)

If `--clang-path` is not set, `clang` is used by default.


To compile the hello-world example, run:

```sh
$ wanco examples/hello.wat -o hello
$ ./hello
Hello World!
```

To show the help, run:

```sh
$ wanco --help
```

### Enable Checkpoint/Restore functionalities

Compile a WebAssembly file with C/R enabled and run it:

```sh
$ wanco --enable-cr --optimize-cr demo/fib.wat
$ a.out
```

While the process is running, you can trigger checkpoint by sending `SIGUSR1` signal from another teminal:

(The running process is automatically terminated and the snapshot file is created.)

```sh
$ pkill -10 a.out
```

To restore the execution, run:

```sh
$ ./a.out --restore checkpoint.json
```

Note: Snapshot files are named `checkpoint.json` or `checkpoint.pb` (binary format generated with protobuf).

### Compile and assemble only

If you do not want a generated object file to be linked with runtime libraries, specify the `-c` option when running the compiler:
LLVM assembly file (`.ll`) will be generated.

```sh
$ wanco examples/hello.wat -c -o hello.ll
```

After that, you can link it with the runtime library together by using clang

```
$ clang -flto -no-pie hello.ll /usr/local/lib/libwanco_rt.a /usr/local/lib/libwanco_wasi.a -o hello
```

## Test

To test the compiler, run:

```sh
$ cargo test
```

## LICENSE

- benchmark/: See LICENSE files in each directory
- others: MIT (See [LICENSE](./LICENSE))
