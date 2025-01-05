#include "stackmap/elf.h"
#include "wanco.h"
#include <iostream>
// libunwind
#define UNW_LOCAL_ONLY
#include <libunwind.h>

namespace wanco {

std::vector<WasmCallStackEntry> get_stack_trace(ElfFile &elf) {
  std::vector<WasmCallStackEntry> trace;
  Debug() << "--- call stack top ---" << std::endl;

  // initialize libunwind
  unw_context_t context;
  if (unw_getcontext(&context) != 0) {
    Fatal() << "Failed to get context" << std::endl;
    exit(EXIT_FAILURE);
  }
  unw_cursor_t cursor;
  if (unw_init_local(&cursor, &context) != 0) {
    Fatal() << "Failed to initialize cursor" << std::endl;
    exit(EXIT_FAILURE);
  }

  do {
    unw_word_t offset;
    char fname[64];
    fname[0] = '\0';
    (void)unw_get_proc_name(&cursor, fname, sizeof(fname), &offset);
    std::string function_name = {fname};

    // We are only interested in wasm functions
    if (!function_name.starts_with("func_"))
      continue;

    // Get pc.
    unw_word_t pc;
    unw_get_reg(&cursor, UNW_REG_IP, &pc);

    // Get sp.
    unw_word_t sp;
    unw_get_reg(&cursor, UNW_REG_SP, &sp);

    // Get frame size
    unw_word_t bp;
    unw_get_reg(&cursor, UNW_TDEP_BP, &bp);

    // HACK: Since the return address is the address of the next instruction,
    // we need to subtract 1 to get the address of the current instruction.
    std::optional<std::pair<address_t, WasmLocation>> opt =
        elf.get_wasm_location((address_t)(pc - 1));
    if (!opt.has_value()) {
      Fatal() << "Failed to get wasm location" << std::endl;
      exit(EXIT_FAILURE);
    }
    WasmLocation loc = opt.value().second;

    trace.push_back(WasmCallStackEntry{
        .function_name = function_name,
        .location = loc,
        .sp = reinterpret_cast<uint8_t *>(sp),
        .bp = reinterpret_cast<uint8_t *>(bp),
    });

    // Dump the frame
    Debug() << "backtrace[" << trace.size() << "] (" << function_name
            << "): wasm-func=" << loc.function
            << ", wasm-insn=" << loc.insn_offset << std::endl;
    Debug() << "\t pc: " << std::hex << pc << std::dec << ", bp: " << std::hex
            << bp << std::dec << ", sp: " << std::hex << sp << std::dec
            << std::endl;
  } while (unw_step(&cursor) > 0);
  Debug() << "--- call stack bottom ---" << std::endl;
  return trace;
}

} // namespace wanco