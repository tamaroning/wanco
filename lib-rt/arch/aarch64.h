// Do not include this file directly. Include `arch.h` instead.
#pragma once
#include "wanco.h"
#include <cstdint>
#include <string>

#define WANCO_SAVE_REGISTERS()                                                 \
  asm volatile("stp x19, x20, [sp, #-16]! \n\t"                                \
               "stp x21, x22, [sp, #-16]! \n\t"                                \
               "stp x23, x24, [sp, #-16]! \n\t"                                \
               "stp x25, x26, [sp, #-16]! \n\t");

#define WANCO_RESTORE_REGISTERS(regs)                                          \
  asm volatile("ldp x25, x26, [sp], #16 \n\t"                                  \
               "ldp x23, x24, [sp], #16 \n\t"                                  \
               "ldp x21, x22, [sp], #16 \n\t"                                  \
               "ldp x19, x20, [sp], #16 \n\t"                                  \
               : "=r"((regs).x19), "=r"((regs).x20), "=r"((regs).x21),         \
                 "=r"((regs).x22), "=r"((regs).x23), "=r"((regs).x24),         \
                 "=r"((regs).x25), "=r"((regs).x26));

namespace wanco {

enum class Register {
  // General purpose registers.
  X0 = 0,
  X1 = 1,
  X2 = 2,
  X3 = 3,
  X4 = 4,
  X5 = 5,
  X6 = 6,
  X7 = 7,
  X8 = 8,
  X9 = 9,
  X10 = 10,
  X11 = 11,
  X12 = 12,
  X13 = 13,
  X14 = 14,
  X15 = 15,
  X16 = 16,
  X17 = 17,
  X18 = 18,
  X19 = 19,
  X20 = 20,
  X21 = 21,
  X22 = 22,
  X23 = 23,
  X24 = 24,
  X25 = 25,
  X26 = 26,
  X27 = 27,
  X28 = 28,
  X29 = 29,
  X30 = 30,
  SP = 31,
  PC = 32,
};

constexpr Register BP_REGISTER = Register::X29;

inline auto reg_to_string(Register &reg) -> std::string {
  switch (reg) {
  case Register::X0:
    return "X0";
  case Register::X1:
    return "X1";
  case Register::X2:
    return "X2";
  case Register::X3:
    return "X3";
  case Register::X4:
    return "X4";
  case Register::X5:
    return "X5";
  case Register::X6:
    return "X6";
  case Register::X7:
    return "X7";
  case Register::X8:
    return "X8";
  case Register::X9:
    return "X9";
  case Register::X10:
    return "X10";
  case Register::X11:
    return "X11";
  case Register::X12:
    return "X12";
  case Register::X13:
    return "X13";
  case Register::X14:
    return "X14";
  case Register::X15:
    return "X15";
  case Register::X16:
    return "X16";
  case Register::X17:
    return "X17";
  case Register::X18:
    return "X18";
  case Register::X19:
    return "X19";
  case Register::X20:
    return "X20";
  case Register::X21:
    return "X21";
  case Register::X22:
    return "X22";
  case Register::X23:
    return "X23";
  case Register::X24:
    return "X24";
  case Register::X25:
    return "X25";
  case Register::X26:
    return "X26";
  case Register::X27:
    return "X27";
  case Register::X28:
    return "X28";
  case Register::X29:
    return "X29";
  case Register::X30:
    return "X30";
  case Register::SP:
    return "SP";
  case Register::PC:
    return "PC";
  default:
    Fatal() << "Invalid register " << reg_to_string(reg) << '\n';
    exit(1);
  }
}

// AAPCS64 ABI.
struct CallerSavedRegisters {
  uint64_t x19;
  uint64_t x20;
  uint64_t x21;
  uint64_t x22;
  uint64_t x23;
  uint64_t x24;
  uint64_t x25;
  uint64_t x26;

  uint64_t get_value(Register reg) const {
    switch (reg) {
    case Register::X19:
      return x19;
    case Register::X20:
      return x20;
    case Register::X21:
      return x21;
    case Register::X22:
      return x22;
    case Register::X23:
      return x23;
    case Register::X24:
      return x24;
    case Register::X25:
      return x25;
    case Register::X26:
      return x26;
    default:
      Fatal() << "Invalid register " << reg_to_string(reg) << '\n';
      exit(1);
    }
  }
};

} // namespace wanco
