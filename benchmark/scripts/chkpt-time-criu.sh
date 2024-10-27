#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=3
CHECKPOINT_DIR="checkpoint"
SKIP_BUILD=1

LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
SQLITE_DIR=${SCRIPT_DIR}/../sqlite_example
BENCH_DIR=${SCRIPT_DIR}/..

measure_criu_checkpoint_time() {
    local exe_name=$(basename "$1")
    echo "--- $exe_name ---"
    echo "command: $@"
    # grep with first five characters of the command
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
        rm -rf -f $CHECKPOINT_DIR
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_DIR"
            exit 1
        fi
        mkdir $CHECKPOINT_DIR
        sleep 0.3

        # sqliteではdbのlockを取るので、--file-locksで無理やりダンプする
        "$@" > /dev/null 2>&1 & \
            local time=$(
                sleep $half_elapsed_time
                local time=$(get_elapsed_time criu dump --shell-job -t $(pgrep $exe_name) --file-locks -D $CHECKPOINT_DIR)
                echo $time
            )
        
        sleep 0.3
        
        echo "$i: chkpt time: $time"
        times+=($time)
    done

    print_avg_and_mean ${times[@]}
}

if [ $SKIP_BUILD -eq 0 ]; then
    echo "Compiling c files with clang"
    cd $BENCH_DIR
    clang -O1 ${LABBENCH_DIR}/nbody.c -o "nbody-native" -O1 -Wl,-lm
    clang -O1 ${LABBENCH_DIR}/binary-trees.c -o "binary-trees-native" -O1
fi

cd $LLAMA2_DIR
measure_criu_checkpoint_time "./llama2-c-x86-64" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_criu_checkpoint_time "./nbody-native" 10000000
measure_criu_checkpoint_time "./binary-trees-native" 18

# dbファイルや関連するファイルを削除
#rm -f test.db test.db.journal
#rm -f -rf test.db.lock
#measure_criu_checkpoint_time $SQLITE_DIR/target/local/sqlite_example test.db

