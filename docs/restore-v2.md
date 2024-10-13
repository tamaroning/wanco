# リストア高速化のメモ (restore-v2)

ライブラリ
libunwind(-dev?)
libdwarf-dev

現状
- マイグレーションポイントに関数内のValue stackの各アドレスに対応するLLVM Valueを復元するコードを挿入する
- リストア時には、各関数のエントリでコールスタックに対応する関数呼び出し場所にジャンプする => リストアが完了してもすべての関数呼び出しでディスパッチが発生するため遅い

基本アイデア
- stack transformationを利用する
- メタデータを利用して、スタックマップ

論文: https://vtechworks.lib.vt.edu/server/api/core/bitstreams/9f1bd95a-cd8d-430a-ad8d-d10924da78a7/content#page=63.23


live variable (llvm register)
- passで取得する？
- params, locals, value stack

aot_mainでスタックを再構築する

push return address
push bp
push locals
bp=sp
