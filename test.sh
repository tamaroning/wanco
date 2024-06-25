# run all tests/wasker/wat/*.wat
$(cd $(dirname $0) && pwd)
THIS=$(pwd)
WANCO=$THIS/target/debug/wanco

cargo b
for f in $(find . -name "*.wat"); do
  echo "Running $f"
    $WANCO $f; gcc wasm.o lib/lib.o lib/wrt.o; ./a.out
done
