#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=10
CHECKPOINT_FILE="checkpoint.pb"

LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
SQLITE_DIR=${SCRIPT_DIR}/../sqlite_example
BENCH_DIR=${SCRIPT_DIR}/..

measure_execution_time() {
    local exe_name=$(basename "$1")
    echo "--- $exe_name ---"
    echo "command: $@"

    # check command run without error
    "$@" > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "Error: failed to run command"
        exit 1
    fi

    exec_times=()
    for i in $(seq 1 $NUM_RUNS); do
        rm -f $CHECKPOINT_FILE
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_FILE"
            exit 1
        fi

        local exec_time=$(get_elapsed_time "$@")
        #echo "$i: Exec time: $exec_time"
        exec_times+=($exec_time)
    done

    # avg and mean
    print_avg_and_mean ${exec_times[@]}
}


cd $LLAMA2_DIR
measure_execution_time "./../llama2" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
measure_execution_time "./../llama2-c-cr" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_execution_time "./nbody" "--" 10000000
measure_execution_time "./nbody-cr" "--" 10000000

measure_execution_time "./binary-trees" "--" 18
measure_execution_time "./binary-trees-cr" "--" 18

# dbファイルや関連するファイルを削除
#rm -f test.db test.db.journal
#rm -f -rf test.db.lock
#measure_execution_time "$SQLITE_DIR/target/local/sqlite_example" "test.db"
 