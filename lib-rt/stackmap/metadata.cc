#include "metadata.h"
#include "nlohmann/json.hpp"

namespace wanco {

std::vector<MetadataEntry> parse_wanco_metadata(std::span<const uint8_t> data) {
  std::vector<MetadataEntry> metadata;

  nlohmann::json j = nlohmann::json::parse(data.begin(), data.end());
  for (const auto &entry : j) {
    uint32_t wasm_func = entry["func"];
    uint32_t wasm_insn = entry["insn"];
    std::vector<std::string> locals = entry["locals"];
    std::vector<std::string> params = entry["stack"];
    metadata.push_back(MetadataEntry{.func = wasm_func,
                                     .insn = wasm_insn,
                                     .locals = locals,
                                     .stack = params});
  }
  return metadata;
}

} // namespace wanco
