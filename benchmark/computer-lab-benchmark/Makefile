# build
CC = cc
CFLAGS = -O3

CC2 = "${WASI_SDK_PATH}/bin/clang"
CFLAGS2 = -O3

SRCS=binary-trees.c fannkuch-redux.c fasta.c mandelbrot.c mandelbrot-simd.c nbody.c nop.c cat-sync.c
WASM=$(SRCS:.c=.c.wasm)
EXE=$(SRCS:.c=.c.exe)

all: $(WASM) $(EXE)

%.c.exe: %.c
	$(CC) $(CFLAGS) -o $@ $<

%.c.wasm: %.c
	$(CC2) $(CFLAGS2) -o $@ $<

clean:
	rm -f *.wasm *.exe

.PHONY: all clean
