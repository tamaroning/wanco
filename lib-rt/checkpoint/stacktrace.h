#pragma once
#include "stackmap/elf.h"

namespace wanco {

std::vector<WasmCallStackEntry> get_stack_trace(ElfFile &elf);

}
