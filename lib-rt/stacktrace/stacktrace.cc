#include "stacktrace.h"
#include "arch/arch.h"
#include "arch/x86_64.h"
#include "wanco.h"
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <deque>
#include <string>

#if defined(__x86_64__) || defined(_M_X64)
#include <libunwind-x86_64.h>
#define UNW_LOCAL_ONLY
#endif

namespace wanco {

unw_context_t uc;

int save_context(ucontext_t *c) {
  uc = convert_ucontext(c);
  return 0;
}

// pre-condition: wanco_save_context() must be called before this function
auto get_stack_trace()
    -> std::pair<std::deque<NativeStackFrame>, CallerSavedRegisters> {
  std::deque<NativeStackFrame> trace;

  // initialize libunwind
  unw_cursor_t cursor;
  if (unw_init_local(&cursor, &uc) != 0) {
    Fatal() << "Failed to initialize cursor" << '\n';
    exit(EXIT_FAILURE);
  }
  CallerSavedRegisters regs = CallerSavedRegisters::from_unw_cursor(&cursor);

  do {
    unw_word_t offset = 0;
    char fname[64];
    fname[0] = '\0';
    (void)unw_get_proc_name(&cursor, fname, sizeof(fname), &offset);
    std::string const function_name = {fname, strlen(fname)};

    // Get pc.
    unw_word_t pc = 0;
    unw_get_reg(&cursor, UNW_REG_IP, &pc);

    // Get sp.
    unw_word_t sp = 0;
    unw_get_reg(&cursor, UNW_REG_SP, &sp);

    // Get frame size
    unw_word_t bp = 0;
    unw_get_reg(&cursor, UNW_TDEP_BP, &bp);

    trace.push_front(NativeStackFrame{
        .function_name = function_name,
        .pc_offset = offset,
        .pc = pc,
        .sp = reinterpret_cast<uint8_t *>(sp),
        .bp = reinterpret_cast<uint8_t *>(bp),
    });
  } while (unw_step(&cursor) > 0);
  return {trace, regs};
}

} // namespace wanco
