#!/bin/bash

echo "Run this script in the root of the project"

SCRIPT_DIR=$(dirname $(realpath $0))
LABBENCH_DIR=${SCRIPT_DIR}/../computer-lab-benchmark

num_runs=5
should_build=1

function compile_no_cr {
    local output_file=$1
    local wasm_file=$2

    cargo run --release -- \
        -O1 \
        ${wasm_file} \
        -o "${output_file}-no-cr"
}

function compile_cr_opt {
    local output_file=$1
    local wasm_file=$2

    cargo run --release -- \
        -O1 \
        --enable-cr \
        --optimize-cr \
        ${wasm_file} \
        -o "${output_file}"-cr-opt
}

function compile_wamr_cr {
    local output_file=$1
    local wasm_file=$2

    cargo run --release -- \
        -O1 \
        --enable-cr \
        ${wasm_file} \
        -o "${output_file}-cr-wamr"
}


if [ $should_build -eq 1 ]; then
    orig_branch=$(git symbolic-ref --short HEAD)
    mkdir build

    git checkout master
    (cd build && sudo make install)
    cargo build --release

    # nbody
    compile_no_cr "nbody" "${LABBENCH_DIR}/nbody.c.wasm"
    compile_cr "nbody" "${LABBENCH_DIR}/nbody.c.wasm"
    compile_cr_opt "nbody" "${LABBENCH_DIR}/nbody.c.wasm"
    # binary-trees
    compile_no_cr "binary-trees" "${LABBENCH_DIR}/binary-trees.c.wasm"
    compile_cr "binary-trees" "${LABBENCH_DIR}/binary-trees.c.wasm"
    compile_cr_opt "binary-trees" "${LABBENCH_DIR}/binary-trees.c.wasm"
    # nop
    compile_no_cr "nop" "${LABBENCH_DIR}/nop.c.wasm"
    compile_cr "nop" "${LABBENCH_DIR}/nop.c.wasm"
    compile_cr_opt "nop" "${LABBENCH_DIR}/nop.c.wasm"


    git checkout experiment/wamr-aot-stack-frame
    (cd build && sudo make install)
    cargo build --release

    compile_wamr_cr "nbody" "${LABBENCH_DIR}/nbody.c.wasm"
    compile_wamr_cr "binary-trees" "${LABBENCH_DIR}/binary-trees.c.wasm"
    compile_wamr_cr "nop" "${LABBENCH_DIR}/nop.c.wasm"
    
    git checkout $orig_branch
fi

function measure_time {
    local exe=$1
    local num_runs=10

    local total_time=0
    # extract user time from format "user    0m0.005s"
    for i in $(seq 1 $num_runs); do
        usr_time=$(time -f "%U" $exe)
    done

    echo $total_time
}

