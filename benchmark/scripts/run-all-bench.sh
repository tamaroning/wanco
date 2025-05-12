# Measure the execution time of Wanco
uv run ./scripts/exec-time.py
# Calculate the overhead of Wanco
uv run ./scripts/rewriter.py result.json --output ./results/overhead.json
# Create a graph of the overhead of Wanco
uv run ./scripts/whisker-overhead.py ./results/overhead.json -o ./results/overhead.jpg
# Measure checkpoint and restore time for Wasm and CRIU
uv run scripts/chkpt-restore-wasm.py ./results/overhead.json -o ./results/chkpt-restore-wasm.csv
uv run scripts/chkpt-restore-criu.py ./results/overhead.json -o ./results/chkpt-restore-criu.csv
# Create a comparison of checkpoint time, restore time, and snapshot size
uv run scripts/plot-wasm-vs-criu.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
# Create a comparison of migration time
uv run scripts/plot-migration-time.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
# Create a comparison of code size
uv run scripts/plot-code-size.py -o results/code-size-comparison.png
