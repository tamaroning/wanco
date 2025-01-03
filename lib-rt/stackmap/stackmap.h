#pragma once
#include <cstdint>
#include <span>
#include <string>
#include <vector>

namespace wanco {
namespace stackmap {
/*
Header {
  uint8  : Stack Map Version (current version is 3)
  uint8  : Reserved (expected to be 0)
  uint16 : Reserved (expected to be 0)
}
uint32 : NumFunctions
uint32 : NumConstants
uint32 : NumRecords
StkSizeRecord[NumFunctions] {
  uint64 : Function Address
  uint64 : Stack Size (or UINT64_MAX if not statically known)
  uint64 : Record Count
}
Constants[NumConstants] {
  uint64 : LargeConstant
}
StkMapRecord[NumRecords] {
  uint64 : PatchPoint ID
  uint32 : Instruction Offset
  uint16 : Reserved (record flags)
  uint16 : NumLocations
  Location[NumLocations] {
    uint8  : Register | Direct | Indirect | Constant | ConstantIndex
    uint8  : Reserved (expected to be 0)
    uint16 : Location Size
    uint16 : Dwarf RegNum
    uint16 : Reserved (expected to be 0)
    int32  : Offset or SmallConstant
  }
  uint32 : Padding (only if required to align to 8 byte)
  uint16 : Padding
  uint16 : NumLiveOuts
  LiveOuts[NumLiveOuts]
    uint16 : Dwarf RegNum
    uint8  : Reserved
    uint8  : Size in Bytes
  }
  uint32 : Padding (only if required to align to 8 byte)
}
*/

struct __attribute__((packed)) Header {
  uint8_t version;
  uint8_t reserved1;
  uint16_t reserved2;
};

struct __attribute__((packed)) StkSizeRecord {
  uint64_t function_address;
  uint64_t stack_size;
  uint64_t record_count;
};

struct __attribute__((packed)) Constant {
  uint64_t large_constant;
};

enum class LocationKind : uint8_t {
  // Reg
  REGISTER = 0x1,
  // Reg + Offset
  DIRECT = 0x2,
  // [Reg + Offset]
  INDIRECT = 0x3,
  // Offset
  CONSTANT = 0x4,
  // Constants[Offset]
  CONSTANT_INDEX = 0x5
};

struct __attribute__((packed)) Location {
  LocationKind kind;
  uint8_t reserved;
  uint16_t size;
  uint16_t dwarf_regnum;
  uint16_t reserved2;
  int32_t offset;
};

struct __attribute__((packed)) LiveOut {
  uint16_t dwarf_regnum;
  uint8_t reserved;
  uint8_t size;
};

struct StkMapRecord {
  uint64_t patchpoint_id;
  uint32_t instruction_offset;
  uint16_t record_flags;
  uint16_t num_locations;
  std::vector<Location> locations;
  // (only if required to align to 8 byte)
  uint32_t padding1;
  uint16_t padding2;
  uint16_t num_live_outs;
  std::vector<LiveOut> live_outs;
  // (only if required to align to 8 byte)
  uint32_t padding3;
};

struct Stackmap {
  Header header;
  uint32_t num_functions;
  uint32_t num_constants;
  uint32_t num_records;
  std::vector<StkSizeRecord> stksize_records;
  std::vector<Constant> constants;
  std::vector<StkMapRecord> stkmap_records;
};

stackmap::Stackmap parse_stackmap(std::span<const uint8_t> data);

std::string stackmap_to_string(const stackmap::Stackmap &stackmap);

} // namespace stackmap
} // namespace wanco
