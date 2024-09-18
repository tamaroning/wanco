#pragma once
#include <span>
#include <cstdint>
#include <optional>
#include <vector>

std::span<const uint8_t>
get_stackmap_section ();

std::optional<std::vector<uint8_t>>
get_section_data (const char *section_name);
