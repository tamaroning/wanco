#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

NUM_RUNS=10
CHECKPOINT_FILE="checkpoint.pb"

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

    chkpt_times=()
    restore_times=()
    file_sizes=()
    for i in $(seq 1 $NUM_RUNS); do
        rm -f $CHECKPOINT_FILE
        if [ $? -ne 0 ]; then
            echo "Error: failed to remove $CHECKPOINT_FILE"
            exit 1
        fi

        # measure checkpoint time

        local start_time end_time pid
        #echo "Run $i"
        rm -f chkpt-time.txt
        sleep 0.1
        "$@" > /dev/null 2>&1 & \
            (
                sleep $half_elapsed_time
                pkill -10 -f "$exe_name"
            )
        #echo "$i: Checkpoint time: $chkpt_time"

        sleep 0.1
        # check if $CHECKPOINT_FILE exists
        if [ ! -f $CHECKPOINT_FILE ]; then
            echo "Error: $CHECKPOINT_FILE does not exist"
            exit 1
        fi
        chkpt_time=$(cat chkpt-time.txt)
        chkpt_times+=($chkpt_time)

        local file_size=$(stat -c%s $CHECKPOINT_FILE)
        file_sizes+=($file_size)

        # measure restore time
        sleep 0.1
        $1 "--restore" $CHECKPOINT_FILE > /dev/null 2>&1
        # end time is in restore-finish-time.txt
        sleep 0.1
        restore_time=$(cat restore-time.txt)
        #echo "Restore time: $restore_time"
        
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
measure_wasm_checkpoint_time "./../llama2-c-cr" "--" "model.bin" "-n" 0 "-i" 'Once upon a time'
cd $BENCH_DIR
measure_wasm_checkpoint_time "./nbody-cr" "--" 10000000
measure_wasm_checkpoint_time "./binary-trees-cr" "--" 18
