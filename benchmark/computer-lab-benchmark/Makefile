# build

CC="${WASI_SDK_PATH}/bin/clang"
CFLAGS=""
SRC=binary-trees.c fannkuch-redux.c fasta.c mandelbrot.c mandelbrot-simd.c nbody.c nop.c cat-sync.c
WASM=$(SRC:.c=.c.wasm)

all: binary-trees

all: $(WASM)

%.c.wasm: %.c
	$(CC) $(CFLAGS) -o $@ $<

clean:
	rm -f $(WASM)

.PHONY: all clean
