#pragma once
#include <cstdint>
#include <optional>
#include <span>
#include <vector>

namespace wanco {

void do_stacktrace();

std::span<const uint8_t> get_stackmap_section();

std::optional<std::vector<uint8_t>> get_section_data(const char *section_name);

} // namespace wanco
