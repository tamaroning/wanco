This directory is for C/R demos.
See [../examples](../examples) instead for general usage.

## C/R demo

First, compile the fibonacci program (fib.wat) with C/R enabled and run it:

```sh
$ wanco fib.wat --enable-cr --optimize-cr
$ ./a.out
```

Then send a signal SIGUSR1(=10) from another terminal:

```sh
$ pkill a.out -10
```

checkpoint.json or checkpoint.pb is created.
(.pb means binary format generated with protobuf.)


Now you can restore the execution from the checkpoint file:

```sh
./a.out --restore <checkpoint file>
```
