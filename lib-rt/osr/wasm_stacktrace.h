#pragma once
#include "chkpt/chkpt.h"
#include "stackmap/stackmap.h"
#include "stacktrace/stacktrace.h"
#include <deque>
#include <vector>

namespace wanco {

// A class representing a location in a WebAssembly program.
class WasmLocation {
public:
  int32_t get_func() const { return func; }

  // Returns the instruction offset from the beginning of the function.
  int32_t get_insn() const { return insn; }

  static WasmLocation from_stackmap_id(uint64_t id) {
    int32_t func = (id & 0xFFFFFFFF00000000) >> 32;
    int32_t insn = (int32_t)(id & 0xFFFFFFFF);
    return WasmLocation(func, insn);
  }

private:
  WasmLocation(int32_t func, int32_t insn) : func(func), insn(insn) {}

  // Function index.
  int32_t func;
  // Instruction offset from the beginning of the function.
  // -1 if the location represents a function entry.
  int32_t insn;
};

// A struct representing a single WebAssembly stack frame.
class WasmStackFrame {
public:
  WasmLocation loc;
  std::deque<Value> locals;
  std::vector<Value> stack;

  std::string to_string() const {
    std::string s = "WasmStackFrame[";
    s += "func=" + std::to_string(loc.get_func());
    s += ", insn=" + std::to_string(loc.get_insn());
    s += ", locals=[";
    for (const auto &v : locals)
      s += v.to_string() + ", ";
    s += "], stack=[";
    for (const auto &v : stack)
      s += v.to_string() + ", ";
    s += "]]";
    return s;
  }
};

std::vector<WasmStackFrame>
asr_exit(const stackmap::CallerSavedRegisters &regs,
         const std::deque<NativeStackFrame> &callstack,
         const stackmap::Stackmap &stackmap);

} // namespace wanco
