# benchmark

```bash
cd benchmark
# build
make all

# setup venv
python3 -m venv venv
. venv/bin/activate
python3 -m pip install matplotlib

# measure execution time
python3 ./new-scripts/exec-time.py
python3 ./new-scripts/rewriter.py result.json --output overhead.json
# generate whisker plots
python3 ./new-scripts/whisker-overhead.py overhead.json -o overhead.jpg
```

TODO: measure checkpoint and restore time

TODO: measure snapshot size
