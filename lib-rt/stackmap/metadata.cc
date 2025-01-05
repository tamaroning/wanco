#include "metadata.h"
#include "nlohmann/json.hpp"
#include <cstdint>
#include <span>
#include <string>
#include <vector>

namespace wanco {

auto parse_wanco_metadata(std::span<const uint8_t> data)
    -> std::vector<MetadataEntry> {
  std::vector<MetadataEntry> metadata;

  nlohmann::json const j = nlohmann::json::parse(data.begin(), data.end());
  for (const auto &entry : j) {
    uint32_t const wasm_func = entry["func"];
    uint32_t const wasm_insn = entry["insn"];
    std::vector<std::string> const locals = entry["locals"];
    std::vector<std::string> const params = entry["stack"];
    metadata.push_back(MetadataEntry{.func = wasm_func,
                                     .insn = wasm_insn,
                                     .locals = locals,
                                     .stack = params});
  }
  return metadata;
}

} // namespace wanco
