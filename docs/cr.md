# How it works?

## Checkpoint

We have a two different checkpoint mechanisms:

- new checkpoint (v2)
- legacy checkpoint (v1)

v1 and v2 checkpoint are enabled by passing `--enable-cr` or `--legacy-cr` to the AOT compiler, respectively.
The biggest difference of them is that v1 checkpoint imposes a little overhead while v2 does not impose any overhead.


### New checkpoint (v2)

In the v2 checkpoint mechanism, the compiler only inserts migration points, where the program checks if checkpoint request is sent. In the current implementation, migration points are inserted at the beggining of functions and loops.

Each migration point is implemented as a memory load instruction which cause a segmentation fault if a checkpoint signal has been sent to the process. This technique is originally used in the Java HotSpot VM.

Also, the compiler emits stackmaps, a set of mappings from WebAssembly variables to certain memory locations (or CPU registers). Stackmaps are used to extract the WebAssembly runtime states from the process after the main thread is suspended.


### Legacy checkpoint (v1)

In legacy checkpoint, we use a similar techinque with [Binaryen's Asyncify](https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html).
If the v1 checkpoint is enabled, the compiler instruments code that unwinds the call stack and store all value stack and local variables on each stack frame.
Binaryen's Asyncify is implemented as a [Binaryen pass](https://github.com/WebAssembly/binaryen) while ours is implemented as instrumentation of LLVM IR.

## Restore

TBA
