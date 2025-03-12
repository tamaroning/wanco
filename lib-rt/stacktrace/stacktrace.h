#pragma once
#include "wanco.h"
#include <deque>

namespace wanco {

struct NativeStackFrame {
    // Function name.
    std::string function_name;
    // Address offset from the beginning of the function.
    uint64_t pc_offset;
    // program counter
    uint64_t pc;
    // stack pointer
    uint8_t* sp;
    // base pointer
    uint8_t* bp;
};

std::deque<NativeStackFrame> get_stack_trace();

}
