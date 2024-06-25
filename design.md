# デザイン

## コンパイル

1. lib.o, wrt.oをコンパイルしておく
2. wancoでmod.wasmをwasm.oにコンパイル
3. gccでwasm.o, lib.o, wrt.oをリンク

## オブジェクトファイル

wasm.o: AOTコンパイルされたWebAssemblyモジュール
- wanco_main (func): グローバル変数、データセグメントの初期化後にスタート関数を呼び出す
- func_xxx (func): コンパイルされたwasm関数
- memory_base (global): linear memoryのベースアドレス
- global_mem_size (global): linear memoryのページ数

lib.o: Wanco + WASI APIライブラリ
- print (func): デバッグ用のprint関数
- WASI API関連の関数: TBA

wrt.o: WebAssembly Nativeランタイム
- _start (func):
    1. global_mem_sizeの値を取得し、malloc等でlinear memoryを確保する
    2. wanco_mainを呼び出す
- memory_grow (func): linear memoryの拡張
- memory_size (func): linear memoryのページ数取得
