CC = cc
CFLAGS = -O3
LDFLAGS = -lm

CC2 = ${WASI_SDK_PATH}/bin/clang
CFLAGS2 = -O3 -D_WASI_EMULATED_MMAN -lwasi-emulated-mman
LDFLAGS2 = -lm

all: native wasm model.bin tokenizer.bin

clean:
	rm -f model.bin tokenizer.bin *.exe *.wasm

native:
	$(CC) $(CFLAGS) -o run.exe run.c $(LDFLAGS)
	$(CC) $(CFLAGS) -o runq.exe runq.c $(LDFLAGS)

wasm:
	$(CC2) $(CFLAGS2) -o run.c.wasm run.c $(LDFLAGS2)
	$(CC2) $(CFLAGS2) -o runq.c.wasm runq.c $(LDFLAGS2)

model.bin:
	wget -nc -O model.bin --no-clobber https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/stories260K.bin

tokenizer.bin:
	wget -nc -O tokenizer.bin --no-clobber https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/tok512.bin

.PHONY: all clean native wasm
