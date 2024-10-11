#!/bin/bash
echo "Run this script in the root of the project"

SCRIPT_DIR=$(dirname $(realpath $0))
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
LLAMA2_WASM=${LLAMA2_DIR}/llama2-c.wasm

num_runs=5
should_build=0


if [ $should_build -eq 1 ]; then
    orig_branch=$(git symbolic-ref --short HEAD)
    mkdir build

    git checkout master
    (cd build && sudo make install)
    cargo build --release

    # no-cr
    cargo run --release -- \
        -O1 \
        ${LLAMA2_WASM} \
        -o llama2-no-cr

    # cr
    cargo run --release -- \
        -O1 \
        --enable-cr \
        ${LLAMA2_WASM} \
        -o llama2-cr

    # cr with optimization
    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${LLAMA2_WASM} \
        -o llama2-cr-opt

    git checkout experiment/wamr-aot-stack-frame
    (cd build && sudo make install)
    cargo build --release

    # cr using AOT_STACK_FRAME
    cargo run --release -- \
        -O1 \
        --enable-cr \
        ${LLAMA2_WASM} \
        -o llama2-cr-wamr

    # cr with optimization using AOT_STACK_FRAME
    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${LLAMA2_WASM} \
        -o llama2-cr-opt-wamr
    
    git checkout $orig_branch
fi

no_cr_total=0
cr_total=0
opt_cr_total=0
wamr_total=0
wamr_opt_total=0

cd ${LLAMA2_DIR}

for i in $(seq 1 $num_runs); do
    echo "Run $i"

    output=$(../../llama2-no-cr -- model.bin -n 0 -i "Once upon a time" 2>&1)
    value=$(echo $output | grep -oP '(?<=achieved tok/s: )\d+\.\d+')
    echo "res = $value tok/s"
    if [ -n "$value" ]; then
        no_cr_total=$(echo "$no_cr_total + $value" | bc)
    else
        echo "Failed to extract value from output: $output"
    fi

    output=$(../../llama2-cr -- model.bin -n 0 -i "Once upon a time" 2>&1)
    value=$(echo $output | grep -oP '(?<=achieved tok/s: )\d+\.\d+')
    echo "res = $value tok/s"
    if [ -n "$value" ]; then
        cr_total=$(echo "$cr_total + $value" | bc)
    else
        echo "Failed to extract value from output: $output"
    fi

    output=$(../../llama2-cr-opt -- model.bin -n 0 -i "Once upon a time" 2>&1)
    value=$(echo $output | grep -oP '(?<=achieved tok/s: )\d+\.\d+')
    echo "res = $value tok/s"
    if [ -n "$value" ]; then
        opt_cr_total=$(echo "$opt_cr_total + $value" | bc)
    else
        echo "Failed to extract value from output: $output"
    fi

    output=$(../../llama2-cr-wamr -- model.bin -n 0 -i "Once upon a time" 2>&1)
    value=$(echo $output | grep -oP '(?<=achieved tok/s: )\d+\.\d+')
    echo "res = $value tok/s"
    if [ -n "$value" ]; then
        wamr_total=$(echo "$wamr_total + $value" | bc)
    else
        echo "Failed to extract value from output: $output"
    fi

    output=$(../../llama2-cr-opt-wamr -- model.bin -n 0 -i "Once upon a time" 2>&1)
    value=$(echo $output | grep -oP '(?<=achieved tok/s: )\d+\.\d+')
    echo "res = $value tok/s"
    if [ -n "$value" ]; then
        wamr_opt_total=$(echo "$wamr_opt_total + $value" | bc)
    else
        echo "Failed to extract value from output: $output"
    fi
done

no_cr_average=$(echo "scale=6; $no_cr_total / $num_runs" | bc)
cr_average=$(echo "scale=6; $cr_total / $num_runs" | bc)
opt_cr_average=$(echo "scale=6; $opt_cr_total / $num_runs" | bc)
wamr_average=$(echo "scale=6; $wamr_total / $num_runs" | bc)
wamr_opt_average=$(echo "scale=6; $wamr_opt_total / $num_runs" | bc)

cr_overhead=$(echo "scale=6; ($no_cr_average - $cr_average) * 100 / $no_cr_average" | bc)
opt_cr_overhead=$(echo "scale=6; ($no_cr_average - $opt_cr_average) * 100 / $no_cr_average" | bc)
wamr_overhead=$(echo "scale=6; ($no_cr_average - $wamr_average) * 100 / $no_cr_average" | bc)
wamr_opt_overhead=$(echo "scale=6; ($no_cr_average - $wamr_opt_average) * 100 / $no_cr_average" | bc)

echo "Results"
echo "no-cr: $no_cr_average tok/s (0% overhead)"
echo "cr: $cr_average tok/s ($cr_overhead% overhead)"
echo "cr-opt: $opt_cr_average tok/s ($opt_cr_overhead% overhead)"
echo "cr-wamr: $wamr_average tok/s ($wamr_overhead% overhead)"
echo "cr-opt-wamr: $wamr_opt_average tok/s ($wamr_opt_overhead% overhead)"
