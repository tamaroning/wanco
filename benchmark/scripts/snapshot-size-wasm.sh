#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=3
CHECKPOINT_FILE="checkpoint.pb"
SKIP_BUILD=1

LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
BENCH_DIR=${SCRIPT_DIR}/..

measure_wasm_checkpoint_size() {
    local exe_name=$(echo $1 | sed 's/[^a-zA-Z0-9_-]//g')
    echo "--- $exe_name ---"
    echo "ommand: $@"

    # check command run without error
    "$@" > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "Error: failed to run command"
        exit 1
    fi

    local half_elapsed_time=$(get_half_elapsed_time "$@")
    echo "half elapsed time: $half_elapsed_time"

    file_sizes=()
    for i in $(seq 1 $NUM_RUNS); do
        rm -f $CHECKPOINT_FILE
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_FILE"
            exit 1
        fi
        sleep 0.5

        "$@" > /dev/null 2>&1 \
            & sleep $half_elapsed_time \
            & pkill -10 -f "$exe_name"
        sleep 0.5
        local file_size=$(stat -c%s $CHECKPOINT_FILE)
        echo "$i: File size: $file_size"
        file_sizes+=($file_size)
    done

    # avg and mean
    local total_file_size=0
    for file_size in ${file_sizes[@]}; do
        total_file_size=$(echo "$total_file_size + $file_size" | bc)
    done
    local average_file_size=$(echo "$total_file_size / $NUM_RUNS" | bc)
    echo "Average: $average_file_size"
    # mean
    local sorted_file_sizes=($(echo ${file_sizes[@]} | tr ' ' '\n' | sort -n))
    local num_file_sizes=${#sorted_file_sizes[@]}
    local median_file_size=${sorted_file_sizes[$((num_file_sizes / 2))]}
    echo "Mean: $median_file_size"
}

if [ $SKIP_BUILD -eq 0 ]; then
    echo "Compiling wasm files with wanco"
    cd $BENCH_DIR
    wanco --enable-cr ${LLAMA2_DIR}/llama2-c.wasm -o "llama2-c-cr"
    wanco --enable-cr ${LABBENCH_DIR}/nbody.c.wasm -o "nbody-cr"
    wanco --enable-cr ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees-cr"
fi

cd $LLAMA2_DIR
measure_wasm_checkpoint_size "./../llama2-c-cr" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_wasm_checkpoint_size "./nbody-cr" "--" 10000000
measure_wasm_checkpoint_size "./binary-trees-cr" "--" 18

