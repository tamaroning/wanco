#!/bin/bash
echo "Run this script in the root of the project"

SCRIPT_DIR=$(dirname $(realpath $0))
LLAMA2_DIR=${SCRIPT_DIR}/../llama2-c
LLAMA2_WASM=${LLAMA2_DIR}/llama2-c.wasm

should_build=0

CHECKPOINT_FILE="checkpoint.pb"

if [ $should_build -eq 1 ]; then
    wanco $LLAMA2_WASM --enable-cr --optimize-cr -o llama2-c-wasm
fi

cd benchmark/llama2-c

sudo rm checkpoint.pb
sudo rm checkpoint.pb.json
sudo rm -rf checkpoint
mkdir checkpoint
sleep 0.5

../../llama2-c-wasm -- "model.bin" "-n" 0 "-i" 'Once upon a time'  1> /dev/null &
(
sleep 0.2
TARGET_PID=$(pgrep llama2-c-wasm)
start_time=$(date +%s%3N)
pkill -10 llama2-c-wasm
while kill -0 "$TARGET_PID" 2>/dev/null; do
    sleep 0.0001
done
end_time=$(date +%s%3N)
elapsed_time=$((end_time - start_time))

ls checkpoint.pb
echo "pkillコマンド送信からプロセス終了までの時間: ${elapsed_time}ミリ秒"
)

sleep 0.5

./llama2-c-x86-64 "model.bin" "-n" 0 "-i" 'Once upon a time' 1> /dev/null &
(
sleep 0.01
TARGET_PID=$(pgrep llama2-c-x86-64)
start_time=$(date +%s%3N)
criu dump --shell-job -t $(pgrep llama2-c-x86-64) -D checkpoint
#while kill -0 "$TARGET_PID" 2>/dev/null; do
#    sleep 0.0001
#done
end_time=$(date +%s%3N)
elapsed_time=$((end_time - start_time))

ls checkpoit
echo "criu dumpコマンド送信からプロセス終了までの時間: ${elapsed_time}ミリ秒"
)

cd ../../

# llama2.c
# pkill 12 ms
# criu dump 67 ms
