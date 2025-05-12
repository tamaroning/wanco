SUBDIRS = ../computer-lab-benchmark ../llama2-c ../gapbs
WASM_FILES = $(foreach dir,$(SUBDIRS),$(wildcard $(dir)/*.wasm))

WASM_OPT = ../wasm-opt

all: asyncify

asyncify:
	for wasm in $(WASM_FILES); do \
		filename=$$(basename $$wasm .wasm); \
		output=$${filename}.cr.wasm; \
		echo "$(WASM_OPT) --insert-safepoint-call $$wasm -o $$output"; \
		$(WASM_OPT) --insert-safepoint-call $$wasm -o $$output; \
		echo "$(WASM_OPT) --asyncify --pass-arg=asyncify-imports@env.safepoint $$output -o $$output"; \
		$(WASM_OPT) --asyncify --pass-arg=asyncify-imports@env.safepoint $$output -o $$output; \
	done

clean:
	rm *.wasm

.PHONY: all clean
