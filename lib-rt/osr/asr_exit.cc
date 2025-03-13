#include "chkpt/chkpt.h"
#include "osr/wasm_stacktrace.h"
#include "stackmap/stackmap.h"
#include "stacktrace/stacktrace.h"
#include "wanco.h"
#include <algorithm>
#include <deque>
#include <map>
#include <memory>
#include <optional>
#include <utility>
#include <vector>

namespace wanco {

static WasmStackFrame osr_exit(const NativeStackFrame &native_frame,
                               const stackmap::CallerSavedRegisters &regs,
                               const stackmap::Stackmap &stackmap,
                               std::shared_ptr<stackmap::StkMapRecord> record);

using stackmap_table =
    std::map<int32_t, std::vector<std::shared_ptr<stackmap::StkMapRecord>>>;

// func => stackmap_record[]
static stackmap_table populate_stackmap(const stackmap::Stackmap &stackmap) {
  stackmap_table map;

  for (auto &record : stackmap.stkmap_records) {
    auto id = record->patchpoint_id;
    auto loc = WasmLocation::from_stackmap_id(id);

    auto it = map.find(loc.get_func());
    if (it == map.end()) {
      std::vector<std::shared_ptr<stackmap::StkMapRecord>> ent;
      ent.push_back(record);
      map.insert({loc.get_func(), ent});
    } else {
      auto &ent = it->second;
      ent.push_back(record);
    }
  }

  // sort all entries by the first element
  for (auto &[k, v] : map) {
    std::sort(v.begin(), v.end(), [](auto &left, auto &right) {
      return left->instruction_offset < right->instruction_offset;
    });
  }

  return map;
}

static std::optional<std::shared_ptr<stackmap::StkMapRecord>>
lookup_stackmap(stackmap_table &map, int32_t func_index, int32_t pc_offset) {
  auto it = map.find(func_index);
  if (it == map.end()) {
    Warn() << "Failed to find records for func_" << std::dec << func_index
           << '\n';
    return std::nullopt;
  }

  auto records = it->second;
  auto it2 = std::lower_bound(records.begin(), records.end(), pc_offset,
                              [](const auto &record, const auto &key) {
                                return record->instruction_offset < key;
                              });

  if (it2 == records.end()) {
    Warn() << "Failed to find a record for func_" << std::dec << func_index
           << " pc_offset=" << pc_offset << '\n';
    for (auto &record : records) {
      Debug() << "Instead, found a record for pc_offset=" << std::dec
              << record->instruction_offset << '\n';
    }
    return std::nullopt;
  }

  DEBUG_LOG << "search pc_offset=0x" << std::hex << pc_offset
            << " result pc_offset=0x" << it2->get()->instruction_offset << "\n";

  // FIXME: If the difference between actual pc and an instruction offset in the
  // stackmap record is big, the record is possibly different from the actual
  // stackmap. However, there seems to be no reasonable way to validate the
  // record.
  ASSERT(std::abs(static_cast<int32_t>(pc_offset -
                                       it2->get()->instruction_offset)) <= 3 &&
         "");
  return *it2;
}

// Perform All-stack replacement exit.
// The bottom frame is stored in the return value.
std::vector<WasmStackFrame>
asr_exit(const stackmap::CallerSavedRegisters &regs,
         const std::deque<NativeStackFrame> &callstack,
         const stackmap::Stackmap &stackmap) {
  std::vector<WasmStackFrame> trace;

  auto stackmap_table = populate_stackmap(stackmap);

  for (const auto &native_frame : callstack) {
    auto func_name = native_frame.function_name;
    if (!func_name.starts_with("func_"))
      continue;

    auto func_index_str = func_name.substr(5);
    auto func_index = std::stoi(func_index_str);

    auto record_opt =
        lookup_stackmap(stackmap_table, func_index, native_frame.pc_offset);
    if (!record_opt.has_value()) {
      Fatal() << "Failed to find stackmap for " << func_name << ", pc_offset=0x"
              << std::hex << native_frame.pc_offset << '\n';
      exit(1);
    }
    auto &stackmap_record = record_opt.value();

    auto wasm_frame = osr_exit(native_frame, regs, stackmap, stackmap_record);

    trace.push_back(wasm_frame);

    Debug() << "Found stackmap record for " << func_name << ", pc_offset=0x"
            << std::hex << native_frame.pc_offset << '\n';
  }

  return trace;
}

static Value::Type decode_value_type(int32_t encoded) {
  switch (encoded) {
  case 0:
    return Value::Type::I32;
  case 1:
    return Value::Type::I64;
  case 2:
    return Value::Type::F32;
  case 3:
    return Value::Type::F64;
  default:
    Fatal() << "Invalid value type" << '\n';
    exit(1);
  }
}

static int32_t retrieve_constant_location(const stackmap::Location &loc) {
  if (loc.kind != stackmap::LocationKind::CONSTANT) {
    Fatal() << "Invalid location kind for constant location" << '\n';
    exit(1);
  }

  return loc.offset;
}

static Value value_from_memory(const uint8_t *addr, Value::Type ty) {
  switch (ty) {
  case Value::Type::I32:
    return Value{*reinterpret_cast<const int32_t *>(addr)};
  case Value::Type::I64:
    return Value{*reinterpret_cast<const int64_t *>(addr)};
  case Value::Type::F32:
    return Value{*reinterpret_cast<const float *>(addr)};
  case Value::Type::F64:
    return Value{*reinterpret_cast<const double *>(addr)};
  default:
    Fatal() << "Invalid value type" << '\n';
    exit(1);
  }
}

static Value retrieve_value(const stackmap::Stackmap &stackmap,
                            const stackmap::Location loc, bool loc_is_ptr,
                            const NativeStackFrame &native_frame,
                            const stackmap::CallerSavedRegisters &regs,
                            Value::Type ty) {
  switch (loc.kind) {
  case stackmap::LocationKind::REGISTER: {
    stackmap::Register reg{loc.dwarf_regnum};
    uint64_t value = regs.get_value(reg);
    if (loc_is_ptr)
      return value_from_memory(reinterpret_cast<const uint8_t *>(value), ty);
    else
      return value_from_memory(reinterpret_cast<const uint8_t *>(&value), ty);
  } break;
  case stackmap::LocationKind::DIRECT: {
    stackmap::Register reg{loc.dwarf_regnum};
    uint64_t reg_value;
    if (reg == stackmap::Register::RBP) {
      reg_value = reinterpret_cast<uint64_t>(native_frame.bp);
    } else {
      reg_value = regs.get_value(reg);
    }
    if (loc_is_ptr)
      return value_from_memory(
          *reinterpret_cast<const uint8_t **>(&reg_value) + loc.offset, ty);
    else
      return value_from_memory(
          reinterpret_cast<const uint8_t *>(&reg_value) + loc.offset, ty);
  } break;
  case stackmap::LocationKind::INDIRECT: {
    stackmap::Register reg{loc.dwarf_regnum};
    const uint8_t *address;
    if (reg == stackmap::Register::RBP) {
      address = native_frame.bp + loc.offset;
    } else {
      address = reinterpret_cast<uint8_t *>(regs.get_value(reg)) + loc.offset;
    }

    if (loc_is_ptr)
      return value_from_memory(
          *reinterpret_cast<const uint8_t *const *>(address), ty);
    else
      return value_from_memory(reinterpret_cast<const uint8_t *>(address), ty);
  } break;
  case stackmap::LocationKind::CONSTANT:
    Fatal() << "Constant location kind not supported" << '\n';
    exit(1);
  case stackmap::LocationKind::CONSTANT_INDEX:
    Fatal() << "Constant index location kind not supported" << '\n';
    exit(1);
  }
  // unreachable
}

static WasmStackFrame osr_exit(const NativeStackFrame &native_frame,
                               const stackmap::CallerSavedRegisters &regs,
                               const stackmap::Stackmap &stackmap,
                               std::shared_ptr<stackmap::StkMapRecord> record) {

  // the first location represents the number of wasm locals
  uint64_t num_locals = retrieve_constant_location(record->locations.at(0));
  uint64_t num_stack = (record->locations.size() - 1) / 2 - num_locals;

  std::deque<Value> locals;
  std::vector<Value> stack;

  size_t i = 1;
  while (num_locals--) {
    // decode value type
    uint64_t value_ty = retrieve_constant_location(record->locations.at(i++));
    Value::Type ty = decode_value_type(value_ty);
    // decode value
    Value value = retrieve_value(stackmap, record->locations.at(i++), true,
                                 native_frame, regs, ty);
    locals.push_back(value);
  }

  while (num_stack--) {
    // decode value type
    uint64_t value_ty = retrieve_constant_location(record->locations.at(i++));
    Value::Type ty = decode_value_type(value_ty);
    // decode value
    Value value = retrieve_value(stackmap, record->locations.at(i++), false,
                                 native_frame, regs, ty);
    stack.push_back(value);
  }

  return WasmStackFrame{
      .loc = WasmLocation::from_stackmap_id(record->patchpoint_id),
      .locals = locals,
      .stack = stack};
}

} // namespace wanco