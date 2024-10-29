#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

cd $BENCH_DIR
echo "Compiling llama2"
wanco ${LLAMA2_DIR}/llama2-c.wasm -o "llama2"
echo "Compiling llama2 with cr"
wanco --enable-cr ${LLAMA2_DIR}/llama2-c.wasm -o "llama2-c-cr"

echo "Compiling nbody"
wanco ${LABBENCH_DIR}/nbody.c.wasm -o "nbody"
echo "Compiling nbody with cr"
wanco --enable-cr ${LABBENCH_DIR}/nbody.c.wasm -o "nbody-cr"

echo "Compiling binary-trees"
wanco ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees"
echo "Compiling binary-trees with cr"
wanco --enable-cr ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees-cr"

#echo "Compiling sqlite"
#wanco --enable-cr ${SQLITE_DIR}/sqlite_example.wasm -o "sqlite_example-cr"
