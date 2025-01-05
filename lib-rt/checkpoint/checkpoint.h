#pragma once
#include "stackmap/elf.h"
#include "stackmap/metadata.h"
#include "stackmap/stackmap.h"

namespace wanco {

void checkpoint_callstack(ElfFile &elf, std::vector<WasmCallStackEntry> &trace,
                          std::vector<MetadataEntry> &metadata,
                          stackmap::Stackmap llvm_stackmap);

}
