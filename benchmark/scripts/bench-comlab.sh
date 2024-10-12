#!/bin/bash
echo "Run this script in the root of the project"

SCRIPT_DIR=$(dirname $(realpath $0))
LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark

num_runs=1
should_build=0

if [ $should_build -eq 1 ]; then
    orig_branch=$(git symbolic-ref --short HEAD)
    mkdir build

    git checkout master
    (cd build && sudo make install)
    cargo build --release

    clang -O1 ${LABBENCH_DIR}/nbody.c -o "nbody-x86-64" -Wl,-lm
    clang -O1 ${LABBENCH_DIR}/binary-trees.c -o "binary-trees-x86-64"
    # llama2 is already built.

    # nbody
    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${LABBENCH_DIR}/nbody.c.wasm \
        -o "nbody-wasm"
    # binary-trees
    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${LABBENCH_DIR}/binary-trees.c.wasm \
        -o "binary-trees-wasm"

    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${LABBENCH_DIR}/../llama2-c/llama2-c.wasm \
        -o "llama2-c-wasm"

    git checkout $orig_branch
fi

function measure_wasm_checkpoint_size {
    local exe=$1
    local sleep_time=$2
    local arg1=$3

    total_file_size=0
    for i in $(seq 1 $num_runs); do
        echo "Run $i"
        rm checkpoint.json
        "./$exe" -- $arg1 \
            & sleep $sleep_time \
            & pkill -10 -f "$exe"
        sleep 0.5
        file_size=$(stat -c%s checkpoint.json)
        echo "File size: $file_size"
        total_file_size=$(echo "$total_file_size + $file_size" | bc)
    done

    average_file_size=$(echo "$total_file_size / $num_runs" | bc)
    echo "Average file size of $exe: $average_file_size"
}

function measure_llama_wasm_checkpoint_size {
    cd benchmark/llama2-c
    ./../../llama2-c-wasm -- "model.bin" "-n" 0 "-i" 'Once upon a time' \
        & sleep 0.22 \
        & pkill -10 -f "llama2-c-wasm"
    sleep 0.5
    echo "File size: $(stat -c%s checkpoint.json)"
    cd ../../
}

#measure_wasm_checkpoint_size "nbody-wasm" 0.25 10000000
#measure_wasm_checkpoint_size "binary-trees-wasm" 0.6 18
#measure_llama_wasm_checkpoint_size

# wasm
# - nbody: 7967
# - binary-trees: 195245
# - llama2: 1348258
sleep 0.5

rm -rf checkpoint
mkdir checkpoint
./nbody-x86-64 10000000 & sleep 0.37 & criu dump --shell-job -t $(pgrep nbody-x86-64) -D checkpoint
du -sb checkpoint
sleep 0.5

rm -rf checkpoint
mkdir checkpoint
./binary-trees-x86-64 18 & sleep 0.6 & criu dump --shell-job -t $(pgrep binary-trees-x86-64) -D checkpoint
du -sb checkpoint
sleep 0.5

cd benchmark/llama2-c
rm -rf checkpoint
mkdir checkpoint
./llama2-c-x86-64 "model.bin" "-n" 0 "-i" 'Once upon a time' & sleep 0.03 & criu dump --shell-job -t $(pgrep llama2-c-x86-64) -D checkpoint
du -sb checkpoint
cd ../../
sleep 0.5

# criu
# - nbody: 131364
# - binary-trees: 4096
# - llama2: 275192