#include "wanco.h"
#include <cstdio>
#include <cstdlib>
#include <fcntl.h>
#include <gelf.h>
#include <link.h>
#include <sys/mman.h>
#include <unistd.h>
#include <optional>
#include "elf.h"

namespace wanco {

ElfFile::ElfFile(const std::string &path)
    : fd(open(path.c_str(), O_RDONLY)), elf(nullptr) {

  if (fd < 0) {
    perror("Failed to open ELF file");
    exit(EXIT_FAILURE);
  }

  if (!initialize_elf()) {
    close(fd);
    exit(EXIT_FAILURE);
  }
}

ElfFile::~ElfFile() {
  if (elf != nullptr) {
    elf_end(elf);
  }
  if (fd >= 0) {
    close(fd);
  }
}

auto ElfFile::initialize_elf() -> bool {
  if (elf_version(EV_CURRENT) == EV_NONE) {
    Fatal() << "libelf init failed" << '\n';
  }

  elf = elf_begin(fd, ELF_C_READ, nullptr);
  if (elf == nullptr) {
    Fatal() << "elf_begin failed: " << elf_errmsg(0) << '\n';
    return false;
  }
  return true;
}

auto ElfFile::get_section_data(const std::string &section_name)
    -> std::optional<std::span<uint8_t>> {
  Elf_Scn *scn = nullptr;
  GElf_Shdr shdr;

  // get index of strtab section
  size_t shstrndx = 0;
  if (elf_getshdrstrndx(elf, &shstrndx) != 0) {
    Fatal() << "elf_getshdrstrndx failed: " << elf_errmsg(0) << '\n';
    exit(EXIT_FAILURE);
  }

  while ((scn = elf_nextscn(elf, scn)) != nullptr) {
    if (gelf_getshdr(scn, &shdr) == nullptr) {
      continue;
    }

    const char *name = elf_strptr(elf, shstrndx, shdr.sh_name);
    if ((name != nullptr) && section_name == name) {
      Elf_Data *data = elf_getdata(scn, nullptr);
      if (data == nullptr) {
        Fatal() << "elf_getdata failed: " << elf_errmsg(0) << '\n';
        exit(EXIT_FAILURE);
      }
      const std::span<uint8_t> sp {reinterpret_cast<uint8_t *>(data->d_buf), data->d_size};
    return sp;
    }
  }
  return std::nullopt;
}

} // namespace wanco
