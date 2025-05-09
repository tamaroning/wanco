SUBDIRS = ../computer-lab-benchmark ../llama2-c ../gapbs
WASM_FILES = $(foreach dir,$(SUBDIRS),$(wildcard $(dir)/*.wasm))

WASMC = ./wamrc
WASMFLAGS = --target=i386 --opt-level=1

all: wamrc aot
	@echo "Downloading WAMR artifacts..."

wamrc:
	wget --no-clobber -nc https://github.com/bytecodealliance/wasm-micro-runtime/releases/download/WAMR-2.2.0/wamrc-2.2.0-x86_64-ubuntu-22.04.tar.gz & \
	tar -xzf wamrc-2.2.0-x86_64-ubuntu-22.04.tar.gz

aot: wamrc
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.aot; \
		echo "$(WASMC) $(WASMFLAGS) -o $$output $$wasm"; \
        $(WASMC) $(WASMFLAGS) -o $$output $$wasm; \
	done

clean:
	rm *.tar.gz wamrc *aot

.PHONY: all clean wamrc
