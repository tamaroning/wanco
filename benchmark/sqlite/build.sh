#!/bin/bash
# https://github.com/JamesMenetrey/unine-twine/blob/main/benchmarks/sqlite/build-wasm.sh

SCRIPT_DIR=$(dirname $(realpath $0))
OUT_DIR=${SCRIPT_DIR}/build/wasm
OUT_FILE_WASM=${OUT_DIR}/sqlite.wasm
OUT_FILE_AOT=${OUT_DIR}/benchmark-wasm.aot

WASI_SDK_DIR=$WASI_SDK
WASI_SYSSCRIPT_DIR=${WASI_SDK_DIR}/share/wasi-sysroot
WASI_DEFINED_SYMBOLS_FILE=${WASI_SDK_DIR}/share/wasi-sysroot/share/wasm32-wasi/defined-symbols.txt

INSTRUMENTATION_SRC=timing_callbacks

# Check WASI SDK is present
if [ ! -d $WASI_SDK_DIR ] 
then
    echo "Error: WASI SDK not located into $WASI_SDK_DIR."
    echo "Stopping building benchmark-wasm."
    exit 1
fi

mkdir -p ${OUT_DIR}
rm -f ${OUT_FILE_WASM} ${OUT_FILE_AOT}

# Use WASI SDK to build out the .wasm binary
${WASI_SDK_DIR}/bin/clang \
        --target=wasm32-wasi \
        -O3 \
        --sysroot=${WASI_SYSSCRIPT_DIR} \
        -Wl,--export=malloc \
        -Wl,--export=free \
        -Wl,--allow-undefined-file=${WASI_DEFINED_SYMBOLS_FILE} \
        -Wl,--strip-all \
        -DSQLITE_OS_OTHER \
        -DSQLITE_ENABLE_MEMSYS3 \
        -I${SCRIPT_DIR}/src \
        -o ${OUT_FILE_WASM} \
        -D_WASI_EMULATED_PROCESS_CLOCKS \
        -Wl,-lwasi-emulated-signal
        ${SCRIPT_DIR}/src/*.c

if [ -f ${OUT_FILE_WASM} ]; then
        echo "build ${OUT_FILE_WASM} success"
else
        echo "build ${OUT_FILE_WASM} fail"
fi
