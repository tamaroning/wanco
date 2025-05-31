SUBDIRS = ../computer-lab-benchmark ../llama2-c ../gapbs
WASM_FILES = $(foreach dir,$(SUBDIRS),$(wildcard $(dir)/*.wasm))

WASMC = wasmedge
WASMFLAGS = compile --optimize 1

all: aot

aot:
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.aot; \
		echo "$(WASMC) $(WASMFLAGS) -- $$wasm $$output"; \
        $(WASMC) $(WASMFLAGS) -- $$wasm $$output; \
	done

clean:
	rm *.aot

.PHONY: all clean aot
