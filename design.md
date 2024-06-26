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
