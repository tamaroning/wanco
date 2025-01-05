#include "checkpoint.h"
#include "snapshot/snapshot.h"
#include <iostream>
#include <map>

namespace wanco {

static Value load_from_address(const uint8_t *address,
                               const std::string &type) {
  if (type == "i32") {
    return Value{*reinterpret_cast<const int32_t *>(address)};
  } else if (type == "i64") {
    return Value{*reinterpret_cast<const int64_t *>(address)};
  } else if (type == "f32") {
    return Value{*reinterpret_cast<const float *>(address)};
  } else if (type == "f64") {
    return Value{*reinterpret_cast<const double *>(address)};
  } else {
    std::cerr << "Unsupported type: " << type << std::endl;
    exit(EXIT_FAILURE);
  }
}

static Value get_wasm_value(const uint8_t *address, const std::string &type,
                            uint8_t *bp, stackmap::Location &stackmap_loc) {

  switch (stackmap_loc.kind) {
  case stackmap::LocationKind::DIRECT: {
    // value = [Reg + Offset]
    int32_t offset = stackmap_loc.offset;
    const uint8_t *addr = bp + offset;
    return load_from_address(addr, type);
  } break;
  case stackmap::LocationKind::INDIRECT: {
    // value = [[Reg + Offset]]
    int32_t offset = stackmap_loc.offset;
    const uint8_t **indirect_addr =
        reinterpret_cast<const uint8_t **>(bp + offset);
    const uint8_t *addr = *indirect_addr;
    return load_from_address(addr, type);
  } break;
  case stackmap::LocationKind::CONSTANT: {
    // Offset is the value we want.
    int32_t value = stackmap_loc.offset;
    if (type == "i32") {
      return Value{value};
    } else if (type == "i64") {
      return Value{static_cast<int64_t>(value)};
    } else if (type == "f32") {
      return Value{static_cast<float>(value)};
    } else if (type == "f64") {
      return Value{static_cast<double>(value)};
    } else {
      std::cerr << "Unsupported type: " << type << std::endl;
      exit(EXIT_FAILURE);
    }
  } break;
  default:
    std::cerr << "Unsupported location kind: "
              << stackmap::location_kind_to_string(stackmap_loc.kind)
              << std::endl;
    exit(EXIT_FAILURE);
    break;
  }
}

void checkpoint_callstack(ElfFile &elf, std::vector<WasmCallStackEntry> &trace,
                          std::vector<MetadataEntry> &metadata,
                          stackmap::Stackmap llvm_stackmap) {

  // Populate LLVM stackmap records
  // (func, insn) -> record
  // TODO: use index instead of record
  std::map<std::pair<uint32_t, uint32_t>, stackmap::StkMapRecord> loc_to_record;
  for (const stackmap::StkMapRecord &record : llvm_stackmap.stkmap_records) {
    uint32_t func = (record.patchpoint_id >> 32) & 0xFFFFFFFF;
    uint32_t insn = record.patchpoint_id & 0xFFFFFFFF;
    loc_to_record[std::make_pair(func, insn)] = record;
  }

  // Populate patchpoint metadata
  std::map<std::pair<uint32_t, uint32_t>, MetadataEntry> loc_to_metadata;
  for (const MetadataEntry &entry : metadata) {
    loc_to_metadata[std::make_pair(entry.func, entry.insn)] = entry;
  }

  for (const WasmCallStackEntry &frame : trace) {
    std::cout << "Frame: wasm-func=" << frame.function_name
              << ", wasm-insn=" << frame.location.insn_offset << std::endl;

    std::pair<uint32_t, uint32_t> wasm_loc =
        std::make_pair(frame.location.function, frame.location.insn_offset);

    // Find patchpoint entry
    auto it = loc_to_metadata.find(wasm_loc);
    if (it == loc_to_metadata.end()) {
      std::cerr << "Failed to find metadata entry" << std::endl;
      exit(EXIT_FAILURE);
    }
    MetadataEntry &entry = it->second;

    // Find LLVM stackmap record
    auto it2 = loc_to_record.find(wasm_loc);
    if (it2 == loc_to_record.end()) {
      std::cerr << "Failed to find stackmap record" << std::endl;
      exit(EXIT_FAILURE);
    }
    stackmap::StkMapRecord &record = it2->second;

    // Save locals
    size_t loc_count = 0;
    for (const std::string &local_ty : entry.locals) {
      std::cout << "  Local: " << local_ty << std::endl;

      stackmap::Location stackmap_loc = record.locations.at(loc_count);
      Value value = get_wasm_value(frame.bp, local_ty, frame.bp, stackmap_loc);
      std::cout << "    Value: " << value.to_string() << std::endl;
      loc_count++;
    }

    // Save a value stack
    for (const std::string &stack_value_ty : entry.stack) {
      std::cout << "  Stack: " << stack_value_ty << std::endl;
      stackmap::Location stackmap_loc = record.locations.at(loc_count);
      Value value =
          get_wasm_value(frame.bp, stack_value_ty, frame.bp, stackmap_loc);
      std::cout << "    Value: " << value.to_string() << std::endl;

      loc_count++;
    }
  }
}
} // namespace wanco
