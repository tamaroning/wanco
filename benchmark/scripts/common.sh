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
