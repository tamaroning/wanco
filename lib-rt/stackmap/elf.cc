#include "stackmap/elf.h"
#include <cstdint>
#include <elf.h>
#include <fcntl.h>
#include <fstream>
#include <gelf.h>
#include <iostream>
#include <libdwarf/dwarf.h>
#include <link.h>
#include <optional>
#include <span>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <vector>
// libdwarf
#include <libdwarf/libdwarf.h>
// libelf
#include <libelf.h>
// libunwind
#define UNW_LOCAL_ONLY
#include <libunwind.h>
namespace wanco {

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

  while ((scn = elf_nextscn(elf, scn)) != nullptr) {
    if (gelf_getshdr(scn, &shdr) == nullptr) {
      continue;
    }

    const char *name = elf_strptr(elf, shdr.sh_link, shdr.sh_name);
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

void ElfFile::print_dwarf_line_table() {
  Dwarf_Error error;
  std::cout << "DWARF Line Table:" << std::endl;

  Dwarf_Sig8 sig8;
  Dwarf_Unsigned typeoff;
  int res;
  // Since we are only searching for line tables iterate over CU with is_info
  // set.
  while ((res = dwarf_next_cu_header_c(dbg, true, NULL, NULL, NULL, NULL, NULL,
                                       NULL, &sig8, &typeoff, NULL, &error)) ==
         DW_DLV_OK) {
    std::cout << "CU:" << std::endl;
    Dwarf_Die cu_die;
    if (dwarf_siblingof(dbg, NULL, &cu_die, &error) != DW_DLV_OK) {
      std::cerr << "Failed to get CU DIE: " << dwarf_errmsg(error) << std::endl;
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

    for (Dwarf_Unsigned i = 0; i < line_count; ++i) {
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
      std::cout << "Line: " << line_addr << "(" << lineno << ":" << colno << ")"
                << std::endl;
      // TODO: we should free DIE here.
    }
  }
  if (res == DW_DLV_ERROR) {
    printf("Error in dwarf_next_cu_header\n");
    exit(1);
  }
  if (res == DW_DLV_NO_ENTRY) {
    printf("DONE\n");
    /* Done. */
    return;
  }
}

void do_stacktrace() {
  unw_cursor_t cursor;
  unw_context_t context;
  unw_getcontext(&context);
  unw_init_local(&cursor, &context);
  auto count = 0;
  do {
    unw_word_t offset, pc;
    char fname[64];
    unw_get_reg(&cursor, UNW_REG_IP, &pc);
    fname[0] = '\0';
    (void)unw_get_proc_name(&cursor, fname, sizeof(fname), &offset);
    Dl_info info;
    dladdr((void *)pc, &info);
    fprintf(stderr, "%s : backtrace [%d] %s(%s+0x%lx) [%p]\n", __func__, count,
            info.dli_fname, fname, offset, (void *)pc);
    count++;
  } while (unw_step(&cursor) > 0);
}

struct SectionInfo {
  const char *name;
  const uint8_t *address;
  size_t size;

  std::span<const uint8_t> get_span() const {
    return std::span<const uint8_t>{address, size};
  }
};

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

  // continue to search
  return 0;
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

} // namespace wanco
