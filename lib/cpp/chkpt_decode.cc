#include "exec_env.h"
#include "nlohmann/json.h"
#include <cassert>
#include <fstream>

using nlohmann::json;

static Value decode_value_json(json &j) {
  Value v = {0};
  std::string ty = j["type"];
  if (ty == "i32") {
    v = Value(j["value"].get<int32_t>());
  } else if (ty == "i64") {
    v = Value(j["value"].get<int64_t>());
  } else if (ty == "f32") {
    v = Value(j["value"].get<float>());
  } else if (ty == "f64") {
    v = Value(j["value"].get<double>());
  } else {
    assert(false && "unreachable");
    //__builtin_unreachable();
  }
  return v;
}

Checkpoint decode_checkpoint_json(std::ifstream &f) {
  Checkpoint chkpt;
  json j = json::parse(f);

  for (auto &v : j["stack"]) {
    Value value = decode_value_json(v);
    chkpt.stack.push_back(value);
  }

  for (auto &f : j["frames"]) {
    Frame frame;
    frame.fn_index = f["fn_index"].get<int32_t>();
    frame.pc = f["pc"].get<int32_t>();
    for (auto &v : f["locals"]) {
      Value value = decode_value_json(v);
      frame.locals.push_back(value);
    }
    chkpt.frames.push_front(frame);
  }

  for (auto &g : j["globals"]) {
    Value value = decode_value_json(g);
    chkpt.globals.push_back(value);
  }

  for (auto &m : j["memory"]) {
    chkpt.memory.push_back(m.get<uint8_t>());
  }

  return chkpt;
}
