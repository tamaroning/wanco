set -e

get_half_elapsed_time() {
    local start_time=$(date +%s.%N)
    "$@" > /dev/null 2>&1
    local code=$?
    local end_time=$(date +%s.%N)
    if [ $code -ne 0 ]; then
        echo "Error: exit with code $code"
        exit 1
    fi

    local elapsed_time=$(echo "$end_time - $start_time" | bc)
    local half_time=$(echo "$elapsed_time / 2" | bc -l)    
    echo "$half_time"
}

get_elapsed_time() {
    local start_time=$(date +%s.%N)
    "$@" > /dev/null 2>&1
    local code=$?
    local end_time=$(date +%s.%N)
    if [ $code -ne 0 ]; then
        echo "Error: exit with code $code"
        exit 1
    fi

    local elapsed_time=$(echo "$end_time - $start_time" | bc)
    echo "$elapsed_time"
}

print_avg_and_mean() {
    local total=0
    for exec_time in "$@"; do
        total=$(echo "$total + $exec_time" | bc)
    done
    local average=$(echo "scale=6; $total / $NUM_RUNS" | bc)
    echo "Average: $average"
    # mean
    local sorted_exec_times=($(echo "$@" | tr ' ' '\n' | sort -n))
    local num_exec_times=${#sorted_exec_times[@]}
    local median_exec_time=${sorted_exec_times[$((num_exec_times / 2))]}
    echo "Mean: $median_exec_time"
}