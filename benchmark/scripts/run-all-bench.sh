uv run ./scripts/exec-time.py
uv run ./scripts/rewriter.py result.json --output ./results/overhead.json
uv run ./scripts/whisker-overhead.py ./results/overhead.json -o ./results/overhead.jpg
uv run scripts/chkpt-restore-wasm.py ./results/overhead.json -o ./results/chkpt-restore-wasm.csv
uv run scripts/chkpt-restore-criu.py ./results/overhead.json -o ./results/chkpt-restore-criu.csv
# checkpoint time, restore time, snapshot sizeのグラフを表示
uv run scripts/plot-wasm-vs-criu.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
# migration timeのグラフを表示
uv run scripts/plot-migration-time.py results/chkpt-restore-wasm.csv results/chkpt-restore-criu.csv
# code sizeのグラフを表示
uv run scripts/plot-code-size.py -o results/code-size-comparison.png
