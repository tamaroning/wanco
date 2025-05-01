# See LICENSE.txt for license details.

ifndef WASI_SDK_PATH
$(error WASI_SDK_PATH is not set)
endif

CXX_WASM = ${WASI_SDK_PATH}/bin/clang++
CXX_FLAGS += -std=c++11 -O3 -Wall
CXX_WASM_FLAGS += -std=c++11 -O3 -Wall -fno-exceptions

KERNELS = bc bfs cc cc_sv pr pr_spmv sssp tc
SUITE_WASM = $(addsuffix .wasm, $(KERNELS))
SUITE_NATIVE = $(addsuffix .exe, $(KERNELS))

.PHONY: all
all: $(SUITE_WASM) $(SUITE_NATIVE)

%.wasm : src/%.cc src/*.h
	$(CXX_WASM) $(CXX_WASM_FLAGS) $< -o $@

%.exe : src/%.cc src/*.h
	$(CXX) $(CXX_FLAGS) $< -o $@


.PHONY: clean
clean:
	rm -f *.exe *.wasm

