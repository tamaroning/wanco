#include "checkpoint.h"
#include "aot.h"
#include "api.h"
#include "snapshot/snapshot.h"
#include "stackmap/elf.h"
#include "stackmap/metadata.h"
#include "stackmap/stackmap.h"
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <map>
#include <string>
#include <utility>
#include <vector>

namespace wanco {

static auto load_from_address(const uint8_t *address, const std::string &type)
    -> Value {
  if (type == "i32") {
    return Value{*reinterpret_cast<const int32_t *>(address)};
  }
  if (type == "i64") {
    return Value{*reinterpret_cast<const int64_t *>(address)};
  } else if (type == "f32") {
    return Value{*reinterpret_cast<const float *>(address)};
  } else if (type == "f64") {
    return Value{*reinterpret_cast<const double *>(address)};
  } else {
    Fatal() << "Unsupported type: " << type << std::endl;
    exit(EXIT_FAILURE);
  }
}

static auto get_wasm_value(const std::string &type, uint8_t *bp,
                           stackmap::Location &stackmap_loc) -> Value {

  switch (stackmap_loc.kind) {
  case stackmap::LocationKind::DIRECT: {
    // value = [Reg + Offset]
    int32_t const offset = stackmap_loc.offset;
    const uint8_t *addr = bp + offset;
    return load_from_address(addr, type);
  } break;
  case stackmap::LocationKind::INDIRECT: {
    // value = [[Reg + Offset]]
    int32_t const offset = stackmap_loc.offset;
    const auto **indirect_addr =
        reinterpret_cast<const uint8_t **>(bp + offset);
    const uint8_t *addr = *indirect_addr;
    return load_from_address(addr, type);
  } break;
  case stackmap::LocationKind::CONSTANT: {
    // Offset is the value we want.
    int32_t const value = stackmap_loc.offset;
    if (type == "i32") {
      return Value{value};
    }
    if (type == "i64") {
      return Value{static_cast<int64_t>(value)};
    } else if (type == "f32") {
      return Value{static_cast<float>(value)};
    } else if (type == "f64") {
      return Value{static_cast<double>(value)};
    } else {
      Fatal() << "Unsupported type: " << type << std::endl;
      exit(EXIT_FAILURE);
    }
  } break;
  default:
    Fatal() << "Unsupported location kind: "
            << stackmap::location_kind_to_string(stackmap_loc.kind) << '\n';
    exit(EXIT_FAILURE);
    break;
  }
}

void checkpoint_callstack(ElfFile & /*elf*/,
                          std::vector<WasmCallStackEntry> &trace,
                          std::vector<MetadataEntry> &metadata,
                          const stackmap::Stackmap &llvm_stackmap) {

  // Populate LLVM stackmap records
  // (func, insn) -> record
  // TODO(tamaron): use index instead of record
  std::map<std::pair<uint32_t, uint32_t>, stackmap::StkMapRecord> loc_to_record;
  for (const stackmap::StkMapRecord &record : llvm_stackmap.stkmap_records) {
    uint32_t const func = (record.patchpoint_id >> 32) & 0xFFFFFFFF;
    uint32_t const insn = record.patchpoint_id & 0xFFFFFFFF;
    loc_to_record[std::make_pair(func, insn)] = record;
  }

  // Populate patchpoint metadata
  std::map<std::pair<uint32_t, uint32_t>, MetadataEntry> loc_to_metadata;
  for (const MetadataEntry &entry : metadata) {
    loc_to_metadata[std::make_pair(entry.func, entry.insn)] = entry;
  }

  for (const WasmCallStackEntry &frame : trace) {
    Debug() << "Wasm Frame: func=\"" << frame.function_name
            << "\", insn=" << frame.location.insn_offset << '\n';

    std::pair<uint32_t, uint32_t> const wasm_loc =
        std::make_pair(frame.location.function, frame.location.insn_offset);

    // Find patchpoint entry
    auto it = loc_to_metadata.find(wasm_loc);
    if (it == loc_to_metadata.end()) {
      Fatal() << "Failed to find metadata entry" << '\n';
      exit(EXIT_FAILURE);
    }
    MetadataEntry const &entry = it->second;

    // Find LLVM stackmap record
    auto it2 = loc_to_record.find(wasm_loc);
    if (it2 == loc_to_record.end()) {
      Fatal() << "Failed to find stackmap record" << '\n';
      exit(EXIT_FAILURE);
    }
    stackmap::StkMapRecord &record = it2->second;

    push_frame(&exec_env);
    set_pc_to_frame(&exec_env, frame.location.function,
                    frame.location.insn_offset);

    // Save locals
    size_t loc_count = 0;
    Debug() << "Locals: " << '\n';
    for (const std::string &local_ty : entry.locals) {
      stackmap::Location stackmap_loc = record.locations.at(loc_count);
      Value const value = get_wasm_value(local_ty, frame.bp, stackmap_loc);

      Debug() << "  Value: " << value.to_string() << '\n';
      chkpt.frames.back().locals.push_back(value);
      loc_count++;
    }

    // Save a value stack
    Debug() << "  Stack: " << '\n';
    for (const std::string &stack_value_ty : entry.stack) {
      stackmap::Location stackmap_loc = record.locations.at(loc_count);
      Value const value =
          get_wasm_value(stack_value_ty, frame.bp, stackmap_loc);
      Debug() << "  Value: " << value.to_string() << '\n';
      chkpt.frames.back().stack.push_back(value);
      loc_count++;
    }
  }
}
} // namespace wanco
