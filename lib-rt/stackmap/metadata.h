#pragma once
#include <cstdint>
#include <span>
#include <string>
#include <vector>

namespace wanco {

struct MetadataEntry {
  uint32_t func;
  uint32_t insn;
  std::vector<std::string> locals;
  std::vector<std::string> stack;
} __attribute__((aligned(64)));

auto parse_wanco_metadata(std::span<const uint8_t> data)
    -> std::vector<MetadataEntry>;

} // namespace wanco
