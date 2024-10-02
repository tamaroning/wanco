# Cross compilation

TODO


compile fib

```
# native (x86-64)
RUST_LOG="debug" cargo run -- demo/fib.wat -o fib-x86 --enable-cr

# aarch64
RUST_LOG="debug" cargo run -- -l/usr/local/wanco-aarch64/lib demo/fib.wat --target aarch64-linux-gnu -o fib-aarch64 --enable-cr
```

## Run

```
./fib-x86
```

From another terminal,
```
pkill -10 fib-x86
```

Restore the state using qemu-aarch64
```
QEMU_LD_PREFIX=/usr/aarch64-linux-gnu/ qemu-aarch64 fib-aarch64 --restore checkpoint.json
```
