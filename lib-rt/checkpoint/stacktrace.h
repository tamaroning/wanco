#pragma once
#include "stackmap/elf.h"
#include <vector>

namespace wanco {

std::vector<WasmCallStackEntry> get_stack_trace(ElfFile &elf);

}
