# wanco

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler.

## Build

Prerequisites:
- C++ compiler
- Makefile
- Cargo
- LLVM 17

First you need to clone the project:
```
$ git clone git@github.com:tamaroning/wanco.git
$ cd wanco
```

To build and install the runtime library (libwanco.a), run the following commands.
Libraries (libwanco.a) will be installed in /usr/local/lib/.

```
$cd lib/cpp
$ make
$sudo make install
```

Build the wanco compiler:
```
$ cargo build --release
$ cp target/release/wanco .
```

## Run

```
$ wanco --help
Usage: wanco [OPTIONS] <INPUT_FILE>

Arguments:
    <INPUT_FILE>  

Options:
    -o, --output-file <OUTPUT_FILE>  Place the output file
    -c                               Compile and assemble, but do not link
    --checkpoint                     Enable the checkpoint feature
    --restore                        Enable the restore feature
    -O <OPTIMIZATION>                [default: 2] [possible values: 0, 1, 2, 3]
    -h, --help                       Print help
```

Before running the compiler, make sure that a C++ compiler can be invoked via the `c++` command and the runtime library (libwanco.a) is installed in /usr/local/lib/.
Specify an input file which is a WebAssembly text or binary format.

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

Trigger checkpoint by sending `SIGUSR1` signal from another teminal:

(The running process is automatically terminated and the snapshot file is created.)

```
$ pkill -10 a.out
```

Restore the execution:

```
$ ./a.out --restore checkpoint.json
```

### Compile and assemble only

If you do not want to link the object files, specify the `-c` option.
LLVM assembly file (`.ll`) will also be generated.

```
$ wanco examples/hello.wat -c -o hello.o
```

After that, you can link it with the runtime library together by using C++ compiler.

```
$ c++ -no-pie hello.o /usr/local/lib/libwanco.a -o hello
```

## Test

Run

```
$ cargo test
```

## TODO

- [x] WASI preview 0
    - some are missing
- [ ] WASI preview 1
- [ ] WASI preview 2
- [ ] WASI NN
- [x] Checkpoint
    - tables are not supported
- [x] Restore
    - tables and global variables are not supported

## LICENSE

MIT

<!--
- tests/spec/: Apache-2.0
- others: MIT
-->
