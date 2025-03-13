#include "stackmap.h"
#include "wanco.h"
#include <elf.h>
#include <link.h>
#include <memory>
#include <sstream>
#include <string>
#include <sys/mman.h>
#include <unistd.h>

namespace wanco::stackmap {

static auto parse_u16(const uint8_t *&ptr) -> uint16_t {
  uint16_t const value = *reinterpret_cast<const uint16_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static auto parse_u32(const uint8_t *&ptr) -> uint32_t {
  uint32_t const value = *reinterpret_cast<const uint32_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static auto parse_u64(const uint8_t *&ptr) -> uint64_t {
  uint64_t const value = *reinterpret_cast<const uint64_t *>(ptr);
  ptr += sizeof(value);
  return value;
}

static auto parse_header(const uint8_t *&ptr) -> Header {
  Header header = *reinterpret_cast<const Header *>(ptr);
  ptr += sizeof(header);
  return header;
}

static auto parse_stk_size_record(const uint8_t *&ptr) -> StkSizeRecord {
  StkSizeRecord record = *reinterpret_cast<const StkSizeRecord *>(ptr);
  ptr += sizeof(record);
  return record;
}

static auto parse_constant(const uint8_t *&ptr) -> Constant {
  Constant constant = *reinterpret_cast<const Constant *>(ptr);
  ptr += sizeof(constant);
  return constant;
}

static auto parse_location(const uint8_t *&ptr) -> Location {
  Location location = *reinterpret_cast<const Location *>(ptr);
  ptr += sizeof(location);
  return location;
}

static auto parse_live_out(const uint8_t *&ptr) -> LiveOut {
  LiveOut live_out = *reinterpret_cast<const LiveOut *>(ptr);
  ptr += sizeof(live_out);
  return live_out;
}

static auto parse_stk_map_record(const uint8_t *&ptr) -> StkMapRecord {
  uint64_t const patchpoint_id = parse_u64(ptr);
  uint32_t const instruction_offset = parse_u32(ptr);
  uint16_t const record_flags = parse_u16(ptr);
  uint16_t const num_locations = parse_u16(ptr);

  std::vector<Location> locations;
  locations.reserve(num_locations);
  for (uint16_t i = 0; i < num_locations; i++) {
    locations.push_back(parse_location(ptr));
  }

  uint32_t padding1 = 0;
  if ((uint64_t)ptr % 8 != 0) {
    ASSERT((uint64_t)ptr % 8 == 4 && "Invalid data alignment");
    padding1 = parse_u32(ptr);
  }
  uint16_t const padding2 = parse_u16(ptr);
  uint16_t const num_live_outs = parse_u16(ptr);
  std::vector<LiveOut> live_outs;
  live_outs.reserve(num_live_outs);
  for (uint16_t i = 0; i < num_live_outs; i++) {
    live_outs.push_back(parse_live_out(ptr));
  }

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

auto parse_stackmap(const std::span<const uint8_t> data) -> Stackmap {
  const uint8_t *ptr = data.data();
  ASSERT(ptr != nullptr && "Invalid data");
  ASSERT((uint64_t)ptr % 8 == 0 && "Invalid data alignment");
  Header const header = parse_header(ptr);

  uint32_t const num_functions = parse_u32(ptr);
  uint32_t const num_constants = parse_u32(ptr);
  uint32_t const num_records = parse_u32(ptr);

  std::vector<StkSizeRecord> stksize_records;
  stksize_records.reserve(num_functions);
  for (uint32_t i = 0; i < num_functions; i++) {
    stksize_records.push_back(parse_stk_size_record(ptr));
  }

  std::vector<Constant> constants;
  constants.reserve(num_constants);
  for (uint32_t i = 0; i < num_constants; i++) {
    constants.push_back(parse_constant(ptr));
  }

  std::vector<std::shared_ptr<StkMapRecord>> stkmap_records;
  stkmap_records.reserve(num_records);
  for (uint32_t i = 0; i < num_records; i++) {
    stkmap_records.push_back(
        std::make_shared<StkMapRecord>(parse_stk_map_record(ptr)));
  }

  return Stackmap{.header = header,
                  .num_functions = num_functions,
                  .num_constants = num_constants,
                  .num_records = num_records,
                  .stksize_records = stksize_records,
                  .constants = constants,
                  .stkmap_records = stkmap_records};
}

auto location_kind_to_string(LocationKind kind) -> std::string {
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

std::string location_to_string(const Stackmap &stackmap,
                               const Location &location) {
  std::stringstream ss;

  Register reg{location.dwarf_regnum};
  switch (location.kind) {
  case LocationKind::REGISTER: {
    ss << reg_to_string(reg);
  } break;
  case LocationKind::DIRECT: {
    ss << reg_to_string(reg) << " + " << location.offset;
  } break;
  case LocationKind::INDIRECT: {
    ss << "[" << reg_to_string(reg) << " + " << location.offset << "]";
  } break;
  case LocationKind::CONSTANT: {
    ss << location.offset;
  } break;
  case LocationKind::CONSTANT_INDEX: {
    ASSERT(location.offset < stackmap.constants.size() &&
           "Invalid constant index");
    ss << "Constants[" << location.offset
       << "] = " << stackmap.constants[location.offset].large_constant << '\n';
  } break;
  }

  return ss.str();
}

auto stackmap_to_string(const Stackmap &stackmap) -> std::string {
  std::stringstream ss;

  ss << "Version: " << static_cast<int>(stackmap.header.version) << '\n';
  ss << "Num functions: " << stackmap.num_functions << '\n';
  ss << "Num constants: " << stackmap.num_constants << '\n';
  ss << "Num records: " << stackmap.num_records << '\n';

  for (size_t i = 0; i < stackmap.stksize_records.size(); i++) {
    const StkSizeRecord &record = stackmap.stksize_records[i];
    ss << "StkSizeRecord[" << i << "]" << '\n';
    ss << "  Address: 0x" << std::hex << record.function_address << '\n';
    ss << "  Stack size: " << std::dec << record.stack_size << '\n';
    ss << "  Record count: " << record.record_count << '\n';
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
    auto &record = stackmap.stkmap_records[i];
    ss << "StkMapRecord[" << i << "]" << '\n';
    ss << "  Patchpoint ID: 0x" << std::hex << record->patchpoint_id << std::dec
       << '\n';
    ss << "  Instruction offset: " << record->instruction_offset << '\n';
    ss << "  Record flags: " << record->record_flags << '\n';
    ss << "  Num locations: " << record->num_locations << '\n';

    for (size_t j = 0; j < record->locations.size(); j++) {
      const Location &location = record->locations[j];
      ss << "  Location[" << j
         << "] = " << location_to_string(stackmap, location) << '\n';
    }
  }

  return ss.str();
}

} // namespace wanco::stackmap
