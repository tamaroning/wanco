SUBDIRS = ../computer-lab-benchmark ../llama2-c ../gapbs
WASM_FILES = $(foreach dir,$(SUBDIRS),$(wildcard $(dir)/*.wasm))

WASMC = wanco
WASMFLAGS = -O1
WASMFLAGS_CR = -O1 --enable-cr

all: aot cr-aot

clean:
	rm -f *.aot

aot:
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.aot; \
		echo "$(WASMC) $(WASMFLAGS) -o $$output $$wasm"; \
        $(WASMC) $(WASMFLAGS) -o $$output $$wasm; \
	done

cr-aot:
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.cr.aot; \
		echo "$(WASMC) $(WASMFLAGS_CR) -o $$output $$wasm"; \
		$(WASMC) $(WASMFLAGS_CR) -o $$output $$wasm; \
	done

.PHONY: all clean aot cr-aot
