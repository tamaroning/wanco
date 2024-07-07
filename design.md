# デザイン

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

- [x] 関数のentryでop_indexでdispatch
- [ ] frameのpop
- [ ] ローカル変数のリストア
- [ ] スタックのリストア
- [ ] グローバル変数のリストア(aot_main)
- [ ] メモリのリストア(main)

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

