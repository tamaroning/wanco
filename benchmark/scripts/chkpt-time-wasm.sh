#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=3
CHECKPOINT_FILE="checkpoint.pb"
SKIP_BUILD=1

LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
SQLITE_DIR=${SCRIPT_DIR}/../sqlite_example
BENCH_DIR=${SCRIPT_DIR}/..



measure_wasm_checkpoint_time() {
    local exe_name=$(basename "$1")
    echo "--- $exe_name ---"
    echo "command: $@"
    # pgrep with first five characters of the command
    local exe_name=$(echo $exe_name | cut -c1-5)

    # check command run without error
    "$@" > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "Error: failed to run command"
        exit 1
    fi

    local half_elapsed_time=$(get_half_elapsed_time "$@")
    echo "half elapsed time: $half_elapsed_time"

    times=()
    for i in $(seq 1 $NUM_RUNS); do
        rm -f $CHECKPOINT_FILE
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_FILE"
            exit 1
        fi

        local start_time end_time pid
        #echo "Run $i"
        "$@" > /dev/null 2>&1 & \
            local time=$(
                sleep $half_elapsed_time
                pid=$(pgrep $exe_name)
                pkill -10 -f "$exe_name"
                start_time=$(date +%s.%N)
                while kill -0 "$pid" 2>/dev/null; do
                    #echo "waiting"
                    sleep 0.0001
                done
                end_time=$(date +%s.%N)
                #echo "start_time: $start_time"
                #echo "end_time: $end_time"
                echo "$end_time - $start_time" | bc
            )
        echo "$i: Time: $time"

        # check if $CHECKPOINT_FILE exists
        if [ ! -f $CHECKPOINT_FILE ]; then
            echo "Error: $CHECKPOINT_FILE does not exist"
            exit 1
        fi

        times+=($time)
    done

    print_avg_and_mean ${times[@]}
}

if [ $SKIP_BUILD -eq 0 ]; then
    echo "Compiling wasm files with wanco"
    cd $BENCH_DIR
    wanco --enable-cr ${LLAMA2_DIR}/llama2-c.wasm -o "llama2-c-cr"
    wanco --enable-cr ${LABBENCH_DIR}/nbody.c.wasm -o "nbody-cr"
    wanco --enable-cr ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees-cr"
    #echo "Compiling sqlite"
    #wanco --enable-cr ${SQLITE_DIR}/sqlite_example.wasm -o "sqlite_example-cr"
fi

cd $LLAMA2_DIR
measure_wasm_checkpoint_time "./../llama2-c-cr" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_wasm_checkpoint_time "./nbody-cr" "--" 10000000
measure_wasm_checkpoint_time "./binary-trees-cr" "--" 18
