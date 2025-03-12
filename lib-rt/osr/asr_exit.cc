#include "chkpt/chkpt.h"
#include "osr/wasm_stacktrace.h"
#include "stackmap/stackmap.h"
#include "stacktrace/stacktrace.h"
#include "wanco.h"
#include <deque>
#include <map>
#include <memory>
#include <vector>

namespace wanco {

static WasmStackFrame osr_exit(const NativeStackFrame &native_frame,
                               const stackmap::CallerSavedRegisters &regs,
                               const stackmap::Stackmap &stackmap,
                               std::shared_ptr<stackmap::StkMapRecord> record);

static std::map<std::pair<int32_t, int32_t>,
                std::shared_ptr<stackmap::StkMapRecord>>
populate_stackmap(const stackmap::Stackmap &stackmap) {
  std::map<std::pair<int32_t, int32_t>, std::shared_ptr<stackmap::StkMapRecord>>
      map;

  for (auto &record : stackmap.stkmap_records) {
    auto id = record->patchpoint_id;
    auto loc = WasmLocation::from_stackmap_id(id);
    auto pc_offset = record->instruction_offset;

    Info() << "Record: " << id << " Func: " << loc.get_func() << " pc_offset=0x"
           << std::hex << pc_offset << '\n';

    map[std::make_pair(loc.get_func(), pc_offset)] = record;
  }

  return map;
}

// Perform All-stack replacement exit.
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

    auto it = stackmap_table.find({func_index, native_frame.pc_offset});
    if (it == stackmap_table.end()) {
      Fatal() << "Failed to find stackmap for " << func_name << ", pc_offset=0x"
              << std::hex << native_frame.pc_offset << '\n';
      exit(1);
    }
    auto &stackmap_record = it->second;

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
                            const stackmap::Location loc,
                            const NativeStackFrame &native_frame,
                            const stackmap::CallerSavedRegisters &regs,
                            Value::Type ty) {
  Debug() << stackmap::location_to_string(stackmap, loc) << '\n';

  switch (loc.kind) {
  case stackmap::LocationKind::REGISTER: {
    stackmap::Register reg{loc.dwarf_regnum};
    uint64_t value = regs.get_value(reg);
    return value_from_memory(reinterpret_cast<const uint8_t *>(&value), ty);
  } break;
  case stackmap::LocationKind::DIRECT: {
    stackmap::Register reg{loc.dwarf_regnum};
    if (reg == stackmap::Register::RBP) {
      ASSERT(false && "RBP not supported");
      exit(1);
    } else {
      uint64_t value = regs.get_value(reg) + loc.offset;
      return value_from_memory(reinterpret_cast<const uint8_t *>(&value), ty);
    }
  } break;
  case stackmap::LocationKind::INDIRECT: {
    stackmap::Register reg{loc.dwarf_regnum};
    if (reg == stackmap::Register::RBP) {
      uint8_t *address = native_frame.bp + loc.offset;
      return value_from_memory(address, ty);
    } else {
      uint64_t address = regs.get_value(reg) + loc.offset;
      return value_from_memory(reinterpret_cast<const uint8_t *>(address), ty);
    }
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

  Debug() << "Num locals: " << num_locals << ", Num stack: " << num_stack
          << '\n';

  std::vector<Value> locals;
  std::vector<Value> stack;

  size_t i = 1;
  while (num_locals--) {
    // decode value type
    uint64_t value_ty = retrieve_constant_location(record->locations.at(i++));
    Value::Type ty = decode_value_type(value_ty);
    // decode value
    Value value = retrieve_value(stackmap, record->locations.at(i++),
                                 native_frame, regs, ty);
    locals.push_back(value);
  }

  while (num_stack--) {
    // decode value type
    uint64_t value_ty = retrieve_constant_location(record->locations.at(i++));
    Value::Type ty = decode_value_type(value_ty);
    // decode value
    Value value = retrieve_value(stackmap, record->locations.at(i++),
                                 native_frame, regs, ty);
    stack.push_back(value);
  }

  return WasmStackFrame{
      .loc = WasmLocation::from_stackmap_id(record->patchpoint_id),
      .locals = locals,
      .stack = stack};
}

} // namespace wanco