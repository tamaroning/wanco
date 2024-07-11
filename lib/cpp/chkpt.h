#pragma once
#include "exec_env.h"
#include <fstream>

void encode_checkpoint_json(std::ofstream &ofs, Checkpoint *chkpt);

Checkpoint decode_checkpoint_json(std::ifstream &f);
