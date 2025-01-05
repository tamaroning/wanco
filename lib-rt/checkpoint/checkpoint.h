#pragma once
#include "stackmap/elf.h"
#include "stackmap/metadata.h"
#include "stackmap/stackmap.h"
#include <vector>

namespace wanco {

void checkpoint_callstack(ElfFile &elf, std::vector<WasmCallStackEntry> &trace,
                          std::vector<MetadataEntry> &metadata,
                          const stackmap::Stackmap& llvm_stackmap);

} // namespace wanco
