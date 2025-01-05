#pragma once
#include "wanco.h"
#include <span>
#include <vector>

namespace wanco {

struct MetadataEntry {
  uint32_t func;
  uint32_t insn;
  std::vector<std::string> locals;
  std::vector<std::string> stack;
};

std::vector<MetadataEntry> parse_wanco_metadata(std::span<const uint8_t> data);

} // namespace wanco
