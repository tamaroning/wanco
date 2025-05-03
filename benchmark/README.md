# benchmark

## Prerequisites

- [uv](https://github.com/astral-sh/uv): Python package and version manager
- CRIU

```bash
# Install CRIU
sudo add-apt-repository ppa:criu/ppa
sudo apt install criu

cd benchmark
# build the benchmark programs
make all

# measure execution time
uv run ./scripts/exec-time.py
# calculate the overhead of the execution time and output to JSON
uv run ./scripts/rewriter.py result.json --output overhead.json
# generate whisker plots from overhead.json
uv run ./scripts/whisker-overhead.py overhead.json -o overhead.jpg

# measure checkpoint and restore time, snapshot size
# overhead.json is required to get the half elapsed time of the execution time
uv run scripts/chkpt-restore-wasm.py overhead.json

# measure checkpoint and restore time, snapshot size with CRIU (for comparison)
sudo uv run scripts/chkpt-restore-criu.py overhead.json

# Plot comparison of CRIU and WASM (checkpoting and restoring time, snapshot size)
uv run scripts/plot-wasm-vs-criu.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
```

TODO: measure checkpoint and restore time

TODO: measure snapshot size
