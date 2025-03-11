#pragma once
#include "wanco.h"
#include <vector>

namespace wanco {

struct NativeStackFrame {
    std::string function_name;
    uint64_t pc;
    uint8_t* sp;
    uint8_t* bp;
};

std::vector<NativeStackFrame> get_stack_trace();

}
