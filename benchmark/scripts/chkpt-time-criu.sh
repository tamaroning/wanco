#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=10
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

    local half_elapsed_time=$(echo "$(get_half_elapsed_time "$@") - 0.1"| bc)
    echo "half elapsed time: $half_elapsed_time"

    chkpt_times=()
    restore_times=()
    file_sizes=()
    for i in $(seq 1 $NUM_RUNS); do
        rm -rf -f $CHECKPOINT_DIR
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_DIR"
            exit 1
        fi
        rm -rf -f $CHECKPOINT_DIR
        mkdir $CHECKPOINT_DIR
        sleep 0.1

        # sqliteではdbのlockを取るので、--file-locksで無理やりダンプする
        "$@" > /dev/null 2>&1 & \
            local time=$(
                sleep $half_elapsed_time
                local time=$(get_elapsed_time criu dump --shell-job -t $(pgrep $exe_name) --file-locks -D $CHECKPOINT_DIR)
                echo $time
            )
        echo "$i: Checkpoint time: $time"
        chkpt_times+=($time)
        sleep 0.1

        local file_size=$(du -sb "$CHECKPOINT_DIR" | cut -f1)
        echo "$i: File size: $file_size"
        file_sizes+=($file_size)
        sleep 0.1


        # restore time
        local restore_time=$(
            local time=$(get_elapsed_time criu restore --shell-job -D $CHECKPOINT_DIR)
            echo $time
        )
        echo "$i: Restore time: $restore_time"
        restore_times+=($restore_time)
    done

    echo "--- Checkpoint time ---"
    print_avg_and_mean ${chkpt_times[@]}
    echo "--- Restore time ---"
    print_avg_and_mean ${restore_times[@]}
    echo "--- File size ---"
    print_avg_and_mean ${file_sizes[@]}
}

if [ $SKIP_BUILD -eq 0 ]; then
    echo "Compiling c files with clang"
    cd $BENCH_DIR
    clang -O1 ${LABBENCH_DIR}/nbody.c -o "nbody-native" -O1 -Wl,-lm
    clang -O1 ${LABBENCH_DIR}/binary-trees.c -o "binary-trees-native" -O1
fi

cd $LLAMA2_DIR
measure_criu_checkpoint_time "../llama2" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_criu_checkpoint_time "./nbody" "--" 10000000
measure_criu_checkpoint_time "./binary-trees" "--" 18

# dbファイルや関連するファイルを削除
#rm -f test.db test.db.journal
#rm -f -rf test.db.lock
#measure_criu_checkpoint_time $SQLITE_DIR/target/local/sqlite_example test.db

