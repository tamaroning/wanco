## Directories

- wanco/: AOT compiler and driver
- lib-rt/: runtime library linked to the compiled Wasm module
- lib-wasi/: WASI implementation linked to the compiled Wasm module
- examples/: example Wasm programs for normal AOT compilation
- demo/: example Wasm programs for AOT compilation with C/R
- benchmark/: scripts and programs for benchmark

## Lint

```sh
clang-tidy -p build --fix -checks=* lib-rt/**/*.cc lib-rt/**/*.h
```

## Test

To test the compiler, run:

```sh
$ cargo test
```

<!--
## Devcontainer

```
docker buildx build .
```

-->
