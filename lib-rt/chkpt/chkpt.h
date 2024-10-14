#pragma once
#include "wanco.h"
#include <deque>
#include <fstream>
#include <string>
#include <vector>

namespace wanco {

// 1page = 64KiB
const int32_t PAGE_SIZE = 65536;

class Value {
public:
  enum class Type {
    I32,
    I64,
    F32,
    F64,
  };

  Value(int32_t i32) : i32(i32), type(Type::I32) {}
  Value(int64_t i64) : i64(i64), type(Type::I64) {}
  Value(float f32) : f32(f32), type(Type::F32) {}
  Value(double f64) : f64(f64), type(Type::F64) {}

  Type get_type() const { return type; }

  std::string to_string() const {
    switch (type) {
    case Type::I32:
      return "<type=i32, value=" + std::to_string(i32) + ">";
    case Type::I64:
      return "<type=i64, value=" + std::to_string(i64) + ">";
    case Type::F32:
      return "<type=f32, value=" + std::to_string(f32) + ">";
    case Type::F64:
      return "<type=f64, value=" + std::to_string(f64) + ">";
    }
    __builtin_unreachable();
  }

  union {
    int32_t i32;
    int64_t i64;
    float f32;
    double f64;
  };

private:
  Type type;
};

class Frame {
public:
  // inst
  int32_t fn_index = -1;
  int32_t pc = -1;
  std::deque<Value> locals;
  std::vector<Value> stack;
};

class Checkpoint {
public:
  std::deque<Frame> frames;
  std::deque<Value> globals;
  std::vector<int8_t> memory;
  std::deque<int32_t> table;
  int memory_size = 0;

  // リストア時にはframesではなく、こちらに値スタックを詰む。
  // 値スタックをpopする前に、framesのpop操作が行われるため。
  std::deque<Value> restore_stack;

  void clear() {
    frames.clear();
    globals.clear();
    memory.clear();
    table.clear();
    memory_size = 0;
    restore_stack.clear();
  }

  void prepare_restore() {
    restore_stack.clear();
    for (auto &frame : frames) {
      for (auto &value : frame.stack) {
        restore_stack.push_back(value);
      }
    }
  }
};

void encode_checkpoint_json(std::ofstream &ofs, Checkpoint &chkpt);

Checkpoint decode_checkpoint_json(std::ifstream &f);

wanco::Checkpoint decode_checkpoint_proto(std::ifstream &f);

void encode_checkpoint_proto(std::ofstream &ofs, Checkpoint &chkpt);

} // namespace wanco
