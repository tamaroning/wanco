#include "wanco.h"
#include <cstdint>
#include <cstdlib>
#include <libunwind-x86_64.h>
#include <optional>
#include <string>
#include <utility>
#include <vector>
#include "stacktrace.h"
// libunwind
#define UNW_LOCAL_ONLY

namespace wanco {

auto get_stack_trace() -> std::vector<NativeStackFrame> {
  std::vector<NativeStackFrame> trace;
  Debug() << "--- call stack top ---" << '\n';

  // initialize libunwind
  unw_context_t context;
  if (unw_getcontext(&context) != 0) {
    Fatal() << "Failed to get context" << '\n';
    exit(EXIT_FAILURE);
  }
  unw_cursor_t cursor;
  if (unw_init_local(&cursor, &context) != 0) {
    Fatal() << "Failed to initialize cursor" << '\n';
    exit(EXIT_FAILURE);
  }

  do {
    unw_word_t offset = 0;
    char fname[64];
    fname[0] = '\0';
    (void)unw_get_proc_name(&cursor, fname, sizeof(fname), &offset);
    std::string const function_name = {fname};

    // Get pc.
    unw_word_t pc = 0;
    unw_get_reg(&cursor, UNW_REG_IP, &pc);

    // Get sp.
    unw_word_t sp = 0;
    unw_get_reg(&cursor, UNW_REG_SP, &sp);

    // Get frame size
    unw_word_t bp = 0;
    unw_get_reg(&cursor, UNW_TDEP_BP, &bp);


    trace.push_back(NativeStackFrame {
        .function_name = function_name,
        .pc = pc,
        .sp = reinterpret_cast<uint8_t *>(sp),
        .bp = reinterpret_cast<uint8_t *>(bp),
    });

    // Dump the frame
    Debug() << "backtrace[" << trace.size() << "] (" << function_name
            << "): pc=0x" << std::hex << pc << ", sp=0x" << sp
            << ", bp=0x" << bp << '\n';
  } while (unw_step(&cursor) > 0);
  Debug() << "--- call stack bottom ---" << '\n';
  return trace;
}

} // namespace wanco
