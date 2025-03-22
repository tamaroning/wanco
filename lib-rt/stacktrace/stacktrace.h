#pragma once
#include "arch/arch.h"
#include "wanco.h"
#include <deque>
#include <sys/ucontext.h>

namespace wanco {

struct NativeStackFrame {
  // Function name.
  std::string function_name;
  // Address offset from the beginning of the function.
  uint64_t pc_offset;
  // program counter
  uint64_t pc;
  // stack pointer
  uint8_t *sp;
  // base pointer
  uint8_t *bp;
};

int save_context(ucontext_t *);

std::pair<std::deque<NativeStackFrame>, CallerSavedRegisters> get_stack_trace();

} // namespace wanco
