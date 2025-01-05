#include "stackmap/elf.h"
#include "nlohmann/json.hpp"
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <elf.h>
#include <fcntl.h>
#include <gelf.h>
#include <iostream>
#include <libdwarf/dwarf.h>
#include <libunwind-x86_64.h>
#include <link.h>
#include <map>
#include <optional>
#include <span>
#include <string>
#include <sys/mman.h>
#include <unistd.h>
#include <vector>
// libdwarf
#include <libdwarf/libdwarf.h>
// libelf
#include <libelf.h>

namespace wanco {

constexpr uint32_t FUNCTION_START_INSN_OFFSET = 0xffff;

ElfFile::ElfFile(const std::string &path) : fd(-1), elf(nullptr) {
  fd = open(path.c_str(), O_RDONLY);
  if (fd < 0) {
    perror("Failed to open ELF file");
    exit(EXIT_FAILURE);
  }

  if (!initialize_elf()) {
    close(fd);
    exit(EXIT_FAILURE);
  }

  if (!initialize_dwarf()) {
    elf_end(elf);
    close(fd);
    exit(EXIT_FAILURE);
  }

  initialize_wasm_location();
}

ElfFile::~ElfFile() {
  Dwarf_Error error;
  dwarf_finish(dbg, &error);

  if (elf) {
    elf_end(elf);
  }
  if (fd >= 0) {
    close(fd);
  }
}

bool ElfFile::initialize_elf() {
  if (elf_version(EV_CURRENT) == EV_NONE) {
    std::cerr << "libelf init failed" << std::endl;
  }

  elf = elf_begin(fd, ELF_C_READ, nullptr);
  if (!elf) {
    std::cerr << "elf_begin failed: " << elf_errmsg(0) << std::endl;
    return false;
  }
  return true;
}

bool ElfFile::initialize_dwarf() {
  Dwarf_Error error;
  if (dwarf_init(fd, DW_DLC_READ, nullptr, nullptr, &dbg, &error) !=
      DW_DLV_OK) {
    std::cerr << "Failed to initialize DWARF: " << dwarf_errmsg(error)
              << std::endl;
    return false;
  }

  return true;
}

std::span<uint8_t> ElfFile::get_section_data(const std::string &section_name) {
  Elf_Scn *scn = nullptr;
  GElf_Shdr shdr;

  // get index of strtab section
  size_t shstrndx;
  if (elf_getshdrstrndx(elf, &shstrndx) != 0) {
    std::cerr << "elf_getshdrstrndx failed: " << elf_errmsg(0) << std::endl;
    exit(EXIT_FAILURE);
  }

  while ((scn = elf_nextscn(elf, scn)) != nullptr) {
    if (gelf_getshdr(scn, &shdr) == nullptr) {
      continue;
    }

    const char *name = elf_strptr(elf, shstrndx, shdr.sh_name);
    if (name && section_name == name) {
      Elf_Data *data = elf_getdata(scn, nullptr);
      if (!data) {
        std::cerr << "elf_getdata failed: " << elf_errmsg(0) << std::endl;
        exit(EXIT_FAILURE);
      }
      return {reinterpret_cast<uint8_t *>(data->d_buf), data->d_size};
    }
  }
  std::cerr << "Section '" << section_name << "' not found." << std::endl;
  exit(EXIT_FAILURE);
}

void ElfFile::initialize_wasm_location() {
  std::map<address_t, WasmLocation> location_map;

  Dwarf_Error error;
  std::cout << "DWARF Line Table:" << std::endl;

  Dwarf_Sig8 sig8;
  Dwarf_Unsigned typeoff;
  int res;
  while ((res = dwarf_next_cu_header_c(dbg, true, NULL, NULL, NULL, NULL, NULL,
                                       NULL, &sig8, &typeoff, NULL, &error)) ==
         DW_DLV_OK) {
    Dwarf_Die cu_die;
    if (dwarf_siblingof(dbg, NULL, &cu_die, &error) != DW_DLV_OK) {
      std::cerr << "Failed to get CU DIE: " << dwarf_errmsg(error) << std::endl;
      continue;
    }
    // Get producer
    Dwarf_Attribute attr;
    if (dwarf_attr(cu_die, DW_AT_producer, &attr, &error) != DW_DLV_OK) {
      std::cerr << "Failed to get producer: " << dwarf_errmsg(error)
                << std::endl;
      continue;
    }
    // We are only interested in the line table of the AOT module compiled with
    // wanco
    char *producer;
    if (dwarf_formstring(attr, &producer, &error) != DW_DLV_OK) {
      std::cerr << "Failed to get producer: " << dwarf_errmsg(error)
                << std::endl;
      continue;
    }
    if (std::string(producer) != "wanco") {
      continue;
    }

    // Access line table
    Dwarf_Line *line_buffer;
    Dwarf_Signed line_count;
    if (dwarf_srclines(cu_die, &line_buffer, &line_count, &error) !=
        DW_DLV_OK) {
      std::cerr << "Failed to get line table: " << dwarf_errmsg(error)
                << std::endl;
      continue;
    }

    for (Dwarf_Signed i = 0; i < line_count; ++i) {
      Dwarf_Line line = line_buffer[i];
      Dwarf_Addr line_addr;
      dwarf_lineaddr(line, &line_addr, &error);
      if (error != DW_DLV_OK) {
        std::cerr << "Failed to get line address: " << dwarf_errmsg(error)
                  << std::endl;
        continue;
      }
      // Get line number
      Dwarf_Unsigned lineno;
      dwarf_lineno(line, &lineno, &error);
      if (error != DW_DLV_OK) {
        std::cerr << "Failed to get line number: " << dwarf_errmsg(error)
                  << std::endl;
        continue;
      }

      // Get coloumn number
      Dwarf_Unsigned colno;
      dwarf_lineoff_b(line, &colno, &error);
      if (error != DW_DLV_OK) {
        std::cerr << "Failed to get column number: " << dwarf_errmsg(error)
                  << std::endl;
        continue;
      }

      // Insert a location to the map if we have not seen the same address
      WasmLocation loc;
      loc = WasmLocation{
          .function = static_cast<uint32_t>(lineno),
          .insn_offset = static_cast<uint32_t>(colno),
      };
      address_t addr = static_cast<address_t>(line_addr);
      if (location_map.find(addr) == location_map.end()) {
        location_map[addr] = loc;
      }

      std::cout << "0x" << std::hex << line_addr << std::dec << ": "
                << "Function: " << lineno << ", Offset: " << colno << std::endl;

      //  TODO: we should free DIE here.
    }
  }
  if (res == DW_DLV_ERROR) {
    printf("Error in dwarf_next_cu_header\n");
    exit(1);
  }

  // sort and set the locations
  for (const auto &[addr, loc] : location_map) {
    locations.push_back({addr, loc});
  }

  // dump the locations
  /*
  std::cout << "Location map:" << std::endl;
  for (const auto &[addr, loc] : locations) {
    std::cout << "0x" << std::hex << addr << std::dec << ": "
              << "Function: " << loc.function << ", Offset: " << loc.insn_offset
              << ", IsFunction: " << loc.is_function << std::endl;
  }
  */
}

static std::optional<std::pair<address_t, WasmLocation>>
binary_search(const std::vector<std::pair<address_t, WasmLocation>> &vec,
              address_t addr) {
  auto it = std::lower_bound(
      vec.begin(), vec.end(), std::make_pair(addr, WasmLocation{}),
      [](const std::pair<int, WasmLocation> &a,
         const std::pair<int, WasmLocation> &b) { return a.first < b.first; });

  // `lower_bound` は key 以上の最初の要素を指す
  if (it != vec.end() && it->first == addr) {
    return *it;
  }

  if (it == vec.begin()) {
    return std::nullopt;
  }

  // 一つ前の要素が条件を満たす
  return *(it - 1);
}

std::optional<std::pair<address_t, WasmLocation>>
ElfFile::get_wasm_location(address_t address) {
  return binary_search(locations, address);
}

} // namespace wanco
