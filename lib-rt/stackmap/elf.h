#pragma once
#include <cstdint>
#include <libdwarf/libdwarf.h>
#include <libelf.h>
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
  // whether the location is a begininng of function
  bool is_function;
};

using address_t = uint64_t;

class ElfFile {
public:
  ElfFile(const std::string &path);
  ~ElfFile();

  std::span<uint8_t> get_section_data(const std::string &section_name);

  void initialize_wasm_location();

  void initialize_patchpoint_metadata();

  std::optional<std::pair<address_t, WasmLocation>> get_wasm_location(address_t address);

private:
  int fd;
  // Elf object from libelf
  Elf *elf;
  // Dwarf object from libdwarf
  Dwarf_Debug dbg;

  bool initialize_elf();

  bool initialize_dwarf();

  std::vector<std::pair<address_t, WasmLocation>> locations;
};

std::vector<WasmLocation> get_stack_trace(ElfFile& elf);

std::span<const uint8_t> get_stackmap_section();

std::optional<std::vector<uint8_t>> get_section_data(const char *section_name);

} // namespace wanco
