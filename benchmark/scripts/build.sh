#!/bin/bash
SCRIPT_DIR=$(dirname $(realpath $0))
source $SCRIPT_DIR/common.sh

cd $BENCH_DIR
echo "Compiling llama2"
wanco -O3 ${LLAMA2_DIR}/llama2-c.wasm -o "llama2"
echo "Compiling llama2 with cr"
wanco -O3 --enable-cr ${LLAMA2_DIR}/llama2-c.wasm -o "llama2-c-cr"
wanco -O3 --enable-cr --disable-loop-cr ${LLAMA2_DIR}/llama2-c.wasm -o "llama2-c-no-loop"

echo "Compiling nbody"
wanco ${LABBENCH_DIR}/nbody.c.wasm -o "nbody"
echo "Compiling nbody with cr"
wanco -O3 --enable-cr ${LABBENCH_DIR}/nbody.c.wasm -o "nbody-cr"
wanco -O3 --enable-cr --disable-loop-cr ${LABBENCH_DIR}/nbody.c.wasm -o "nbody-no-loop"

echo "Compiling binary-trees"
wanco ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees"
echo "Compiling binary-trees with cr"
wanco -O3 --enable-cr ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees-cr"
wanco -O3 --enable-cr --disable-loop-cr ${LABBENCH_DIR}/binary-trees.c.wasm -o "binary-trees-no-loop"

#echo "Compiling sqlite"
#wanco --enable-cr ${SQLITE_DIR}/sqlite_example.wasm -o "sqlite_example-cr"
