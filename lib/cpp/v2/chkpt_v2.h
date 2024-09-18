#pragma once
#include <deque>
#include <vector>
#include "v1/chkpt.h"
#include "stack_transform.h"
#include "stackmap.h"
#include "elf.h"

class CheckpointV2
{
public:
  std::deque<Value> globals;
  std::vector<int8_t> memory;
  int memory_size = 0;
};

// void encode_checkpoint_v2_json(std::ofstream &ofs, CheckpointV2 &chkpt);

// CheckpointV2 decode_checkpoint_v2_json(std::ifstream &f);
