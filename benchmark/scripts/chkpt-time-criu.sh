#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=10
CHECKPOINT_DIR="checkpoint"

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
            (
                sleep $half_elapsed_time
                criu dump --shell-job -t $(pgrep $exe_name) --file-locks -D $CHECKPOINT_DIR
            )
        freezing_time=$(crit decode -i checkpoint/stats-dump | jq '.entries[0].dump.freezing_time')
        memdump_time=$(crit decode -i checkpoint/stats-dump | jq '.entries[0].dump.memdump_time')
        memwrite_time=$(crit decode -i checkpoint/stats-dump | jq '.entries[0].dump.memwrite_time')
        time=$(echo "scale=2; $freezing_time + $memdump_time + $memwrite_time" | bc)
        echo "$i: Checkpoint time: $time"
        chkpt_times+=($time)
        sleep 0.1

        local file_size=$(du -sb "$CHECKPOINT_DIR" | cut -f1)
        echo "$i: File size: $file_size"
        file_sizes+=($file_size)
        sleep 0.1

        # restore time
        criu restore --shell-job -D $CHECKPOINT_DIR
        restore_time=$(crit decode -i checkpoint/stats-restore | jq '.entries[0].restore.restore_time')
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

cd $LLAMA2_DIR
measure_criu_checkpoint_time "../llama2" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_criu_checkpoint_time "./nbody" "--" 10000000
measure_criu_checkpoint_time "./binary-trees" "--" 18

# dbファイルや関連するファイルを削除
#rm -f test.db test.db.journal
#rm -f -rf test.db.lock
#measure_criu_checkpoint_time $SQLITE_DIR/target/local/sqlite_example test.db

