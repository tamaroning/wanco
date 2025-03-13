#pragma once
#include <cstdint>
#include <libelf.h>
#include <optional>
#include <span>
#include <string>

namespace wanco {

using address_t = uint64_t;

// RAII class for ELF file. We extract section data and DWARF from this.
class ElfFile {
public:
  ElfFile(const std::string &path);
  ~ElfFile();

  // Get section data by section name.
  std::optional<std::span<uint8_t>> get_section_data(const std::string &section_name);

private:
  int fd;
  // The Elf object from libelf
  Elf *elf;

  bool initialize_elf();
};

std::span<const uint8_t> get_stackmap_section();

} // namespace wanco
