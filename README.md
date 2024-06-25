# wanco

![plot](./animal_dance_dog.png)

wanco is a WebAssembly AOT compiler.

## Build

Prerequisites:
- GCC
- Makefile
- Cargo

## Run

Specify an input file which is a WebAssembly text or binary format.

```
cargo run module.wat -o module.o
```

## Test

```
RUST_LOG="info" cargo t -- --nocapture
```

## TODO

- wasi support
- support sqlite-wasm
- compiler driver

## LICENSE

- tests/spec/: Apache-2.0
- others: MIT
