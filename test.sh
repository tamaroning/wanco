# run all tests/wasker/wat/*.wat
for f in $(find . -name "*.wat"); do
  echo "Running $f"
    cargo r $f; gcc wasm.o lib/lib.o lib/wrt.o; ./a.out
done
