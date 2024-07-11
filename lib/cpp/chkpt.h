#pragma once
#include "exec_env.h"
#include <fstream>

extern ExecEnv exec_env;
extern const int32_t PAGE_SIZE;

void encode_checkpoint_json(std::ofstream &ofs, Checkpoint *chkpt);

Checkpoint decode_checkpoint_json(std::ifstream &f);
