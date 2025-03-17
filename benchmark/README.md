# benchmark

```bash
python3 -m venv venv
. venv/bin/activate
python3 -m pip install matplotlib

# measure execution time
python3 ./new-scripts/exec-time.py
# generate whisker plots
python3 ./new_scripts/whisker.py result.json -o result.jpg
```

TODO: measure checkpoint and restore time

TODO: measure snapshot size
