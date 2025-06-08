# benchmark

## Prerequisites

- Linux on x86-64 or AArch64
- [uv](https://github.com/astral-sh/uv): Python package and version manager
- Hyperfine
- (optional) CRIU
- (optional) [My fork of Binaryen](https://github.com/tamaroning/binaryen/tree/checkpoint-restore)
- (optional) WASI SDK (set `WASI_SDK_PATH` environment variable to the path of the WASI SDK)
- (optional) WasmEdge
- (optional) WAMR

Install deps:

```sh
sudo add-apt-repository ppa:criu/ppa
sudo apt install criu hyperfine wasmedge uv
```

Build customized Binaryen:

```
cd <wanco_root_dir>/..
git@github.com:tamaroning/binaryen.git
cd binaryen
git checkout checkpoint-restore
mkdir build
cd build
```

Run all benchmarks:

```bash
cd benchmark
# build the all benchmarks
make all
# Run as root because CRIU requires root privileges
sudo env "PATH=$PATH" ./scripts/run-all-bench.sh
```


Create a symlink to the binary on the `benchmark` (this) directory.

```sh
ln -s ../../binaryen/build/bin/wasm-opt .
```


## Run individual benchmarks manually

```sh
# measure execution time
uv run ./scripts/exec-time.py
# calculate the overhead of the execution time and output to JSON
uv run ./scripts/rewriter.py result.json --output ./results/overhead.json
# generate whisker plots from overhead.json
uv run scripts/plot-exec-time.py results/overhead.json

# measure checkpoint and restore time, snapshot size
# overhead.json is required to get the half elapsed time of the execution time
uv run scripts/chkpt-restore-wasm.py ./results/overhead.json -o ./results/chkpt-restore-wasm.csv

# measure checkpoint and restore time, snapshot size with CRIU (for comparison)
sudo uv run scripts/chkpt-restore-criu.py ./results/overhead.json -o ./results/chkpt-restore-criu.csv

# Plot comparison of CRIU and WASM (checkpoting and restoring time, snapshot size)
uv run scripts/plot-wasm-vs-criu.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
```
