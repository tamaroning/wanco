#include "stackmap.h"
#include "wanco.h"
#include "x86_64.h"
#include <cstdint>
#include <elf.h>
#include <iostream>
#include <link.h>
#include <span>
#include <sstream>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <vector>

namespace wanco {
namespace stackmap {

static uint16_t parse_u16(const uint8_t *&ptr) {
  uint16_t value = *reinterpret_cast<const uint16_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static uint32_t parse_u32(const uint8_t *&ptr) {
  uint32_t value = *reinterpret_cast<const uint32_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static uint64_t parse_u64(const uint8_t *&ptr) {
  uint64_t value = *reinterpret_cast<const uint64_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static Header parse_header(const uint8_t *&ptr) {
  Header header = *reinterpret_cast<const Header *>(ptr);
  ptr += sizeof(header);
  return header;
}

static StkSizeRecord parse_stk_size_record(const uint8_t *&ptr) {
  StkSizeRecord record = *reinterpret_cast<const StkSizeRecord *>(ptr);
  ptr += sizeof(record);
  return record;
}

static Constant parse_constant(const uint8_t *&ptr) {
  Constant constant = *reinterpret_cast<const Constant *>(ptr);
  ptr += sizeof(constant);
  return constant;
}

static Location parse_location(const uint8_t *&ptr) {
  Location location = *reinterpret_cast<const Location *>(ptr);
  ptr += sizeof(location);
  return location;
}

static LiveOut parse_live_out(const uint8_t *&ptr) {
  LiveOut live_out = *reinterpret_cast<const LiveOut *>(ptr);
  ptr += sizeof(live_out);
  return live_out;
}

static StkMapRecord parse_stk_map_record(const uint8_t *&ptr) {
  uint64_t patchpoint_id = parse_u64(ptr);
  uint32_t instruction_offset = parse_u32(ptr);
  uint16_t record_flags = parse_u16(ptr);
  uint16_t num_locations = parse_u16(ptr);

  std::vector<Location> locations;
  for (uint16_t i = 0; i < num_locations; i++)
    locations.push_back(parse_location(ptr));

  uint32_t padding1 = 0;
  if ((uint64_t)ptr % 8 != 0) {
    ASSERT((uint64_t)ptr % 8 == 4 && "Invalid data alignment");
    padding1 = parse_u32(ptr);
  }
  uint16_t padding2 = parse_u16(ptr);
  uint16_t num_live_outs = parse_u16(ptr);
  std::vector<LiveOut> live_outs;
  for (uint16_t i = 0; i < num_live_outs; i++)
    live_outs.push_back(parse_live_out(ptr));

  uint32_t padding3 = 0;
  if ((uint64_t)ptr % 8 != 0) {
    ASSERT((uint64_t)ptr % 8 == 4 && "Invalid data alignment");
    padding3 = parse_u32(ptr);
  }

  return StkMapRecord{.patchpoint_id = patchpoint_id,
                      .instruction_offset = instruction_offset,
                      .record_flags = record_flags,
                      .num_locations = num_locations,
                      .locations = locations,
                      .padding1 = padding1,
                      .padding2 = padding2,
                      .num_live_outs = num_live_outs,
                      .live_outs = live_outs,
                      .padding3 = padding3};
}

Stackmap parse_stackmap(std::span<const uint8_t> data) {
  const uint8_t *ptr = data.data();
  ASSERT(ptr != nullptr && "Invalid data");
  ASSERT((uint64_t)ptr % 8 == 0 && "Invalid data alignment");
  Header header = parse_header(ptr);

  uint32_t num_functions = parse_u32(ptr);
  uint32_t num_constants = parse_u32(ptr);
  uint32_t num_records = parse_u32(ptr);

  std::vector<StkSizeRecord> stksize_records;
  for (uint32_t i = 0; i < num_functions; i++)
    stksize_records.push_back(parse_stk_size_record(ptr));

  std::vector<Constant> constants;
  for (uint32_t i = 0; i < num_constants; i++)
    constants.push_back(parse_constant(ptr));

  std::vector<StkMapRecord> stkmap_records;
  for (uint32_t i = 0; i < num_records; i++)
    stkmap_records.push_back(parse_stk_map_record(ptr));

  return Stackmap{.header = header,
                  .num_functions = num_functions,
                  .num_constants = num_constants,
                  .num_records = num_records,
                  .stksize_records = stksize_records,
                  .constants = constants,
                  .stkmap_records = stkmap_records};
}

static std::string location_kind_to_string(LocationKind kind) {
  switch (kind) {
  case LocationKind::REGISTER:
    return "Register";
  case LocationKind::DIRECT:
    return "Direct";
  case LocationKind::INDIRECT:
    return "Indirect";
  case LocationKind::CONSTANT:
    return "Constant";
  case LocationKind::CONSTANT_INDEX:
    return "Constant index";
  default:
    return "Unknown";
  }
}

std::string stackmap_to_string(const Stackmap &stackmap) {
  std::stringstream ss;

  ss << "Version: " << stackmap.header.version << std::endl;
  ss << "Num functions: " << stackmap.num_functions << std::endl;
  ss << "Num constants: " << stackmap.num_constants << std::endl;
  ss << "Num records: " << stackmap.num_records << std::endl;

  for (size_t i = 0; i < stackmap.stksize_records.size(); i++) {
    const StkSizeRecord &record = stackmap.stksize_records[i];
    ss << "StkSizeRecord[" << i << "]" << std::endl;
    ss << "  Address: 0x" << std::hex << record.function_address << std::endl;
    ss << "  Stack size: " << std::dec << record.stack_size << std::endl;
    ss << "  Record count: " << record.record_count << std::endl;
  }

  /*
  for (size_t i = 0; i < stackmap.constants.size (); i++)
    {
      const Constant &constant = stackmap.constants[i];
      ss << "Constant[" << i << "]" << std::endl;
      ss << "  Value: " << constant.large_constant << std::endl;
    }
    */

  for (size_t i = 0; i < stackmap.stkmap_records.size(); i++) {
    const StkMapRecord &record = stackmap.stkmap_records[i];
    ss << "StkMapRecord[" << i << "]" << std::endl;
    ss << "  Patchpoint ID: " << record.patchpoint_id << std::endl;
    ss << "  Instruction offset: " << record.instruction_offset << std::endl;
    ss << "  Record flags: " << record.record_flags << std::endl;
    ss << "  Num locations: " << record.num_locations << std::endl;

    for (size_t j = 0; j < record.locations.size(); j++) {
      const Location &location = record.locations[j];
      ss << "  Location[" << j << "] = ";
      Register reg{location.dwarf_regnum};
      switch (location.kind) {
      case LocationKind::REGISTER: {
        ss << reg_to_string(reg) << std::endl;
      } break;
      case LocationKind::DIRECT: {
        ss << reg_to_string(reg) << " + " << location.offset << std::endl;
      } break;
      case LocationKind::INDIRECT: {
        ss << "[" << reg_to_string(reg) << " + " << location.offset << "]"
           << std::endl;
      } break;
      case LocationKind::CONSTANT: {
        ss << location.offset << std::endl;
      } break;
      case LocationKind::CONSTANT_INDEX: {
        ASSERT(location.offset < stackmap.constants.size() &&
               "Invalid constant index");
        ss << "Constants[" << location.offset
           << "] = " << stackmap.constants[location.offset].large_constant
           << std::endl;
      } break;
      default:
        ss << "Unknown" << std::endl;
        break;
      }
    }
  }

  return ss.str();
}

} // namespace stackmap
} // namespace wanco
