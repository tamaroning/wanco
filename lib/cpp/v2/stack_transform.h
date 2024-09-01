#pragma once
#include <string>
#include <cstdint>
#include <vector>

class FrameV2
{
public:
  // function name
  std::string name;
  // offset from the start of the function
  uint64_t offset;
  // stack pointer
  uint64_t sp;
};

std::vector<FrameV2>
get_stack_trace ();
