#include "stackmap.h"
#include <cstdint>
#include <elf.h>
#include <fstream>
#include <iostream>
#include <link.h>
#include <optional>
#include <span>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <vector>

static int find_elf_section(struct dl_phdr_info *info, size_t size, void *data);

struct SectionInfo {
  const char *name;
  const uint8_t *address;
  size_t size;
};

std::span<const uint8_t> get_stackmap_section() {
  // get section whose name == "llvm.stackmaps"
  // using /proc/self/maps

  SectionInfo section_info = {".llvm_stackmaps", nullptr, 0};

  // `dl_iterate_phdr` を使用してセクションを検索
  dl_iterate_phdr(find_elf_section, &section_info);

  if (section_info.address == nullptr) {
    std::cerr << "Section " << section_info.name << " not found.\n";
    exit(1);
  }

  std::cerr << "Found section " << section_info.name << " at address "
            << static_cast<const void *>(section_info.address)
            << ", size: " << section_info.size << " bytes\n";

  return std::span<const uint8_t>{section_info.address, section_info.size};
}

// コールバック関数: 各プログラムヘッダーに対して呼び出される
static int find_elf_section(struct dl_phdr_info *info, size_t size,
                            void *data) {
  SectionInfo *targetSection = reinterpret_cast<SectionInfo *>(data);

  // ELFファイルのベースアドレス
  std::cerr << "name: " << info->dlpi_name << std::endl;
  std::cerr << std::hex << "Info: " << info << std::endl;
  // Base address of the ELF file
  std::cerr << "Base address: " << info->dlpi_addr << std::endl;
  const uint8_t *base = reinterpret_cast<const uint8_t *>(info->dlpi_addr);
  if (base == nullptr) {
  }

  for (int i = 0; i < info->dlpi_phnum; ++i) {
    const ElfW(Phdr) &phdr = info->dlpi_phdr[i];

    // PT_LOADセグメントを探す
    if (phdr.p_type == PT_LOAD) {
      const uint8_t *segment_start = base + phdr.p_vaddr;
      const uint8_t *segment_end = segment_start + phdr.p_memsz;

      // セクションヘッダーテーブルを探す
      if (phdr.p_flags & PF_X) { // 実行可能なセグメントを探索
        const ElfW(Ehdr) *ehdr =
            reinterpret_cast<const ElfW(Ehdr) *>(segment_start);
        const ElfW(Shdr) *shdr =
            reinterpret_cast<const ElfW(Shdr) *>(segment_start + ehdr->e_shoff);

        for (int j = 0; j < ehdr->e_shnum; ++j) {
          const char *sectionName = reinterpret_cast<const char *>(
              segment_start + shdr[ehdr->e_shstrndx].sh_offset +
              shdr[j].sh_name);
          std::cerr << "Section name: " << sectionName << std::endl;

          if (strcmp(sectionName, targetSection->name) == 0) {
            targetSection->address = segment_start + shdr[j].sh_offset;
            targetSection->size = shdr[j].sh_size;
            return 1; // セクションが見つかった場合は探索終了
          }
        }
      }
    }
  }

  return 0; // 継続して探索
}

std::optional<std::vector<uint8_t>> get_section_data(const char *section_name) {
  // Get full path of the executable
  char exe_path[512];
  ssize_t len = readlink("/proc/self/exe", exe_path, sizeof(exe_path) - 1);
  if (len == -1) {
    // Failed to get executable path
    std::cerr << "Error: Failed to get executable path\n";
    return std::nullopt;
  }
  exe_path[len] = '\0';

  std::ifstream elf_file(exe_path, std::ios::binary);
  if (!elf_file) {
    // Failed to open executable file
    std::cerr << "Error: Failed to open executable file\n";
    return std::nullopt;
  }

  Elf64_Ehdr ehdr;
  elf_file.read(reinterpret_cast<char *>(&ehdr), sizeof(ehdr));

  // Check if the file is an ELF file
  if (memcmp(ehdr.e_ident, ELFMAG, SELFMAG) != 0) {
    // Invalid ELF header
    std::cerr << "Error: Invalid ELF header\n";
    return std::nullopt;
  }

  elf_file.seekg(ehdr.e_shoff, std::ios::beg);

  std::vector<Elf64_Shdr> shdrs(ehdr.e_shnum);
  elf_file.read(reinterpret_cast<char *>(shdrs.data()),
                ehdr.e_shnum * sizeof(Elf64_Shdr));

  Elf64_Shdr shstrtab = shdrs[ehdr.e_shstrndx];
  std::vector<char> shstrtab_data(shstrtab.sh_size);
  elf_file.seekg(shstrtab.sh_offset, std::ios::beg);
  elf_file.read(shstrtab_data.data(), shstrtab.sh_size);

  for (const auto &shdr : shdrs) {
    const char *name = shstrtab_data.data() + shdr.sh_name;
    if (strcmp(name, section_name) == 0) {
      std::vector<uint8_t> section_data(shdr.sh_size);
      elf_file.seekg(shdr.sh_offset, std::ios::beg);
      elf_file.read(reinterpret_cast<char *>(section_data.data()),
                    shdr.sh_size);
      return section_data;
    }
  }

  return std::nullopt;
}
