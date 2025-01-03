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

class ElfFile {
public:
  ElfFile(const std::string &path);
  ~ElfFile();

  std::span<uint8_t> get_section_data(const std::string &section_name);

  void print_dwarf_line_table();

private:
  int fd;
  // Elf object from libelf
  Elf *elf;
  // Dwarf object from libdwarf
  Dwarf_Debug dbg;

  bool initialize_elf();

  bool initialize_dwarf();
};

std::span<const uint8_t> get_stackmap_section();

std::optional<std::vector<uint8_t>> get_section_data(const char *section_name);

} // namespace wanco
