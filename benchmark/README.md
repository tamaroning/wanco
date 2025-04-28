# benchmark

## Prerequisites

- [uv](https://github.com/astral-sh/uv): Python package and version manager

```bash
cd benchmark
# build the benchmark programs
make all

# measure execution time
uv run ./scripts/exec-time.py
uv run ./scripts/rewriter.py result.json --output overhead.json
# generate whisker plots
uv run ./scripts/whisker-overhead.py overhead.json -o overhead.jpg

# measure checkpoint and restore time, snapshot size
uv run ./scripts/chkpt-restore-wasm.py
```

TODO: measure checkpoint and restore time

TODO: measure snapshot size
