#pragma once
#include "stackmap/stackmap.h"
#include <cstdint>
#include <libdwarf/libdwarf.h>
#include <libelf.h>
#include <map>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace wanco {

void do_stacktrace();

struct WasmLocation {
  // function index
  uint32_t function;
  // instruction offset
  uint32_t insn_offset;
};

// Corresponding to a frame in the native stack trace.
struct WasmCallStackEntry {
  std::string function_name;
  WasmLocation location;
  uint8_t *sp;
  uint8_t *bp;
};

using address_t = uint64_t;

// RAII class for ELF file. We extract section data and DWARF from this.
class ElfFile {
public:
  ElfFile(const std::string &path);
  ~ElfFile();

  // Get section data by section name.
  std::span<uint8_t> get_section_data(const std::string &section_name);

  // Get wasm location from pc address (low pc).
  std::optional<std::pair<address_t, WasmLocation>>
  get_wasm_location(address_t address);

private:
  int fd;
  // The Elf object from libelf
  Elf *elf;
  // The Dwarf object from libdwarf
  Dwarf_Debug dbg;

  std::vector<std::pair<address_t, WasmLocation>> locations;

  bool initialize_elf();

  bool initialize_dwarf();

  void initialize_wasm_location();
};

std::vector<WasmCallStackEntry> get_stack_trace(ElfFile &elf);

std::span<const uint8_t> get_stackmap_section();

struct MetadataEntry {
  uint32_t func;
  uint32_t insn;
  std::vector<std::string> locals;
  std::vector<std::string> stack;
};

std::vector<MetadataEntry> parse_wanco_metadata(std::span<const uint8_t> data);

/*
// Translate native stack trace to wasm state.
class CheckpointContext {
public:
private:
  // Mapping from wasm location to LLVM stackmap record
  std::map<WasmLocation, stackmap::StkMapRecord> loc_to_stackmap;
};
*/

void callstack_to_interpreter(ElfFile &elf,
                              std::vector<WasmCallStackEntry> &trace,
                              std::vector<MetadataEntry> &metadata,
                              stackmap::Stackmap llvm_stackmap);

} // namespace wanco
