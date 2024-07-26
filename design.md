# デザイン

## TODO

Checkpoint
- [x] before call
- [ ] loop start
- [x] C: memory
- [x] C: stack
- [x] C: locals
- [x] C: globals
- [ ] C: sockets
- [ ] C: files

Restore
- [x] before call
- [ ] loop start
- [x] R: memory
- [x] R: stack
- [x] R: locals
- [ ] R: globals
- [ ] R: sockets
- [ ] R: files


## コンパイルとリンク

1. lib.o, wrt.oをコンパイルしておく
2. wancoでmod.wasmをwasm.oにコンパイル
3. gccでwasm.o, lib.o, wrt.oをリンク

## オブジェクトファイル

wasm.o: AOTコンパイルされたWebAssemblyモジュール
- aot_main (func): グローバル変数、データセグメントの初期化後にwasmのスタート関数を呼び出す
- func_xxx (func): コンパイルされたwasm関数
- INIT_MEMORY_SIZE (global): 初期化時のlinear memoryのページ数

lib.o: Wanco + WASI APIライブラリ
- print (func): デバッグ用のprint関数
- WASI API関連の関数: TBA

wrt.o: WebAssembly Nativeランタイム
- main (func):
    1. INIT_MEMORY_SIZEの値を取得し、malloc等でlinear memoryを確保する
    2. exec_envを作成し、初期化する
    2. aot_main(ExecEnv*)を呼び出す
- memory_grow (func): linear memoryの拡張
- memory_size (func): linear memoryのページ数取得

## wasm関数のコンパイル

関数名は、`func_<定義された順の番号>`にコンパイルされる。

第一引数にExecEnvへのポインタを挿入する。
e.g.
```wat
(func $foo (param $len i32) ... )
;; compiles to
define i32 @func_1(ExecEnv* %exec_env_ptr, i32 %0)
```

## PIE

PIEを作成する場合は、Inkwell側にRelocationMode::PICを指定する必要がある。
(gccのデフォルトはPIE)

gcc -no-pie でリンクする場合はInkwellはRelocationMode::DefaultでOK

## マイグレーション
AOTランタイム(ExecEnv)にマイグレーションの状態を持たせておく

ExecEnv::migration_status
- NONE
- CHECKPOINT
- RESTORE

## ストア

保存の開始: 関数直前とループ内の先頭に以下のコード列を置く
- migration_status = CHECKPOINT
- スタック保存用のAPI呼び出し
- ローカル変数保存用のAPI呼び出し
- 適当な値で関数からリターン
保存の継続: 関数呼び出し直後に同じコード列を置いてaot_mainまでアンワインドを行う
aot_mainではグローバル変数保存用のAPIを呼び出す

### ストアのトリガー

とりあえずPOSIX signalを使う。
10番のシグナルハンドラでmigration_stateにCHECKPOINTをセットする(10と12はユーザー定義)

```sh
wanco module.wasm --checkpoint
# link...
./a.out --emit-checkpoint checkpoint.json
# from other shell
pkill -10 a.out
# checkpoint.json is created
```

## リストア

```sh
wanco module.wasm --restore
# link...
./a.out --restore-from checkpoint.json
```

func
%entry:
if state = RESTORE
    int32_t restore_op_index = pop_op_index(exec_env);
    if restore_op_index = 0
        @1 = ...
        br %restore_op_0
    else if restore_op_index = 6
        @1 = ...
        @2 = ...
        br %restore_op_6
    else
        unreachable
else
    br %main
%main

ExecEnv {
    ...
    RestoreState* restore_state;
}

RestoreState {
    ...
    i32 restore_state;
    i32 restore_state_0;
    i32 restore_state_1;
}

## cross-compile

### build LLVM17

1. git clone llvm
2. checkout to version 17
3. mkdir ../build && cd ../build

```
cmake -DLLVM_TARGETS_TO_BUILD="X86" -DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD="AArch64" -DCMAKE_INSTALL_PREFIX=<path> ../llvm-project/llvm
```
available targets:
/llvm-project/llvm/lib/Target

### rustup

```
rustup target add aarch64-unknown-linux-gnu
```

wasmtime libaray requires aarch64-linux-gnu-gcc

sudo apt install gcc-aarch64-linux-gnu

```
sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
#maybe?
sudo apt install libstdc++-11-dev-arm64-cross
#maybe?
sudo apt install g++-multilib
```

### build lib

```
cmake ../lib -DTARGET="aarch64" -DCMAKE_INSTALL_PREFIX="/usr/local/wanco-aarch64"
sudo make install
```

### QEMU

sudo apt install qemu-user qemu-user-static 

QEMU_LD_PREFIX=/usr/aarch64-linux-gnu/ qemu-aarch64 a.out

### Wanco


RUST_LOG="debug" cargo run -- -l/usr/local/wanco-aarch64/lib demo/fib.wat --target aarch64-linux-gnu

## WASI
porting 
https://github.com/bytecodealliance/wasm-micro-runtime/tree/main/core/iwasm/libraries


/wanco/lib/rust/wasi$ cbindgen --output my_header.h

  (type (;2;) (func (param i32) (result i32)))
  (type (;5;) (func (param i32 i32) (result i32)))
  (type (;7;) (func (param i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32 i32) (result i32)))

  (import "wasi_ephemeral_nn" "set_input" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn9set_input17habc1a5a257d08256E (type 7)))
  (import "wasi_ephemeral_nn" "fini_single" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn11fini_single17h38620d609cfbfec6E (type 2)))
  (import "wasi_ephemeral_nn" "compute_single" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn14compute_single17hdf087585f88c7b30E (type 2)))
  (import "wasi_ephemeral_nn" "load_by_name_with_config" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn24load_by_name_with_config17hce74e8b424b5269eE (type 10)))
  (import "wasi_ephemeral_nn" "init_execution_context" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn22init_execution_context17h92c8f2798d40884dE (type 5)))
  (import "wasi_ephemeral_nn" "compute" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn7compute17hf8fc249f3c722447E (type 2)))
  (import "wasi_ephemeral_nn" "get_output" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn10get_output17h79c3d69a05b9703dE (type 10)))
  (import "wasi_ephemeral_nn" "get_output_single" (func $_ZN16wasmedge_wasi_nn9generated17wasi_ephemeral_nn17get_output_single17h0df47ece3c176f76E (type 10)))
 