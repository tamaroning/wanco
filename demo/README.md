This directory is for research demos.
See [../examples](../examples) instead for general usage.

## C/R demo

```
wanco fib.wat --checkpoint --restore
./a.out
```

From another terminal:
```
pkill a.out -10
```

restore:
```
./a.out --checkpoint.json
```

## llama2

```
wanco llama2-c.wasm
./a.out -- model.bin -t 0.9
```
