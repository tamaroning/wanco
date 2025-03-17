CC = ${WASI_SDK_PATH}/bin/clang
CFLAGS = -D_WASI_EMULATED_MMAN -lwasi-emulated-mman -lm

all: run model.bin tokenizer.bin

clean:
	rm -f model.bin tokenizer.bin

run:
	$(CC) $(CFLAGS) -O3 -o run.wasm run.c
	$(CC) $(CFLAGS) -O3 -o runq.wasm runq.c

model.bin:
	wget -O model.bin --no-clobber https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/stories260K.bin

tokenizer.bin:
	wget -O tokenizer.bin --no-clobber https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/tok512.bin

.PHONY: all clean
