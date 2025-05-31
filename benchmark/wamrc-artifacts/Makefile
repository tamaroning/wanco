SUBDIRS = ../computer-lab-benchmark ../llama2-c ../gapbs
WASM_FILES = $(foreach dir,$(SUBDIRS),$(wildcard $(dir)/*.wasm))

WASMC = wamrc
WASMFLAGS = --target=x86_64 --opt-level=1

all: aot

aot:
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.aot; \
		echo "$(WASMC) $(WASMFLAGS) -o $$output $$wasm"; \
        $(WASMC) $(WASMFLAGS) -o $$output $$wasm; \
	done

clean:
	rm *.tar.gz wamrc *aot

.PHONY: all clean wamrc
