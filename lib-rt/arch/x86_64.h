// Do not include this file directly. Include `arch.h` instead.
#pragma once
#include "wanco.h"
#include <cstdint>
#include <string>

#define WANCO_SAVE_REGISTERS()                                                 \
  asm volatile(".intel_syntax noprefix \n\t"                                   \
               "push rbx \n\t"                                                 \
               "push r12 \n\t"                                                 \
               "push r13 \n\t"                                                 \
               "push r14 \n\t"                                                 \
               "push r15 \n\t"                                                 \
               ".att_syntax \n\t");

#define WANCO_RESTORE_REGISTERS(regs)                                          \
  asm volatile(".intel_syntax noprefix \n\t"                                   \
               "pop %0 \n\t"                                                   \
               "pop %1 \n\t"                                                   \
               "pop %2 \n\t"                                                   \
               "pop %3 \n\t"                                                   \
               "pop %4 \n\t"                                                   \
               ".att_syntax \n\t"                                              \
               : "=r"((regs).r15), "=r"((regs).r14), "=r"((regs).r13),         \
                 "=r"((regs).r12), "=r"((regs).rbx));

namespace wanco {

enum class Register {
  // General purpose registers.
  RAX = 0,
  RDX = 1,
  RCX = 2,
  RBX = 3,
  RSI = 4,
  RDI = 5,
  // Frame pointer register.
  RBP = 6,
  // Stack pointer register.
  RSP = 7,
  // Extended integer registers.
  R8 = 8,
  R9 = 9,
  R10 = 10,
  R11 = 11,
  R12 = 12,
  R13 = 13,
  R14 = 14,
  R15 = 15,
  // This isn't actually a register, but is `[rsp + 0]`.
  RET_ADDR = 16,
  // SSE registers.
  XMM0 = 17,
  XMM1 = 18,
  XMM2 = 19,
  XMM3 = 20,
  XMM4 = 21,
  XMM5 = 22,
  XMM6 = 23,
  XMM7 = 24,
  // Extended SSE registers.
  XMM8 = 25,
  XMM9 = 26,
  XMM10 = 27,
  XMM11 = 28,
  XMM12 = 29,
  XMM13 = 30,
  XMM14 = 31,
  XMM15 = 32,
  // Floating point registers.
  ST0 = 33,
  ST1 = 34,
  ST2 = 35,
  ST3 = 36,
  ST4 = 37,
  ST5 = 38,
  ST6 = 39,
  ST7 = 40,
  // MMX registers.
  MM0 = 41,
  MM1 = 42,
  MM2 = 43,
  MM3 = 44,
  MM4 = 45,
  MM5 = 46,
  MM6 = 47,
  MM7 = 48,
};

inline auto reg_to_string(Register &reg) -> std::string {
  switch (reg) {
  case Register::RAX:
    return "RAX";
  case Register::RDX:
    return "RDX";
  case Register::RCX:
    return "RCX";
  case Register::RBX:
    return "RBX";
  case Register::RSI:
    return "RSI";
  case Register::RDI:
    return "RDI";
  case Register::RBP:
    return "RBP";
  case Register::RSP:
    return "RSP";
  case Register::R8:
    return "R8";
  case Register::R9:
    return "R9";
  case Register::R10:
    return "R10";
  case Register::R11:
    return "R11";
  case Register::R12:
    return "R12";
  case Register::R13:
    return "R13";
  case Register::R14:
    return "R14";
  case Register::R15:
    return "R15";
  case Register::RET_ADDR:
    return "RET_ADDR";
  case Register::XMM0:
    return "XMM0";
  case Register::XMM1:
    return "XMM1";
  case Register::XMM2:
    return "XMM2";
  case Register::XMM3:
    return "XMM3";
  case Register::XMM4:
    return "XMM4";
  case Register::XMM5:
    return "XMM5";
  case Register::XMM6:
    return "XMM6";
  case Register::XMM7:
    return "XMM7";
  case Register::XMM8:
    return "XMM8";
  case Register::XMM9:
    return "XMM9";
  case Register::XMM10:
    return "XMM10";
  case Register::XMM11:
    return "XMM11";
  case Register::XMM12:
    return "XMM12";
  case Register::XMM13:
    return "XMM13";
  case Register::XMM14:
    return "XMM14";
  case Register::XMM15:
    return "XMM15";
  case Register::ST0:
    return "ST0";
  case Register::ST1:
    return "ST1";
  case Register::ST2:
    return "ST2";
  case Register::ST3:
    return "ST3";
  case Register::ST4:
    return "ST4";
  case Register::ST5:
    return "ST5";
  case Register::ST6:
    return "ST6";
  case Register::ST7:
    return "ST7";
  case Register::MM0:
    return "MM0";
  case Register::MM1:
    return "MM1";
  case Register::MM2:
    return "MM2";
  case Register::MM3:
    return "MM3";
  case Register::MM4:
    return "MM4";
  case Register::MM5:
    return "MM5";
  case Register::MM6:
    return "MM6";
  case Register::MM7:
    return "MM7";
  default:
    return "Unknown";
  }
}

// For System V ABI.
struct CallerSavedRegisters {
  uint64_t rbx;
  // skip rbp because it is retrieved with libunwind.
  uint64_t r12;
  uint64_t r13;
  uint64_t r14;
  uint64_t r15;

  uint64_t get_value(Register reg) const {
    switch (reg) {
    case Register::RBX:
      return rbx;
    case Register::R12:
      return r12;
    case Register::R13:
      return r13;
    case Register::R14:
      return r14;
    case Register::R15:
      return r15;
    default:
      Fatal() << "Invalid register " << reg_to_string(reg) << '\n';
      exit(1);
    }
  }
};

} // namespace wanco
