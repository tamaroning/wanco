#include "exec_env.h"
#include "nlohmann/json.h"
#include <fstream>

using nlohmann::json;

Checkpoint decode_checkpoint_json(std::ifstream &f) {
  json j = json::parse(f);

  j["stack"] = nlohmann::json::array();
  j["frames"] = nlohmann::json::array();
  j["globals"] = nlohmann::json::array();

  Checkpoint chkpt;
  return chkpt;
}
