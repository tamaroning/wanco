# How it works?

## Checkpoint

We have a two different checkpoint mechanisms:

- new checkpoint (v2)
- legacy checkpoint (v1)

v1 and v2 checkpoint are used by passing `--enable-cr` or `--legacy-cr` to the AOT compiler, respectively.
The biggest difference of them is that v1 checkpoint imposes a little overhead while v2 does not impose any overhead.

### new checkpoint (v2)

TBA

### legacy checkpoint (v1)

In legacy checkpoint, we use a similar techinque with [Binaryen's Asyncify](https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html).
In this techinque, the compiler instruments code that unwinds the call stack and store all value stack and local variables on each stack frame.
Binaryen's Asyncify is implemented as a [Binaryen pass](https://github.com/WebAssembly/binaryen) while ours is implemented as instrumentation of LLVM IR.

## Restore

TBA
