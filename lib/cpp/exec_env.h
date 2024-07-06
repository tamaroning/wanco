#pragma once
#include <cstdint>
#include <string>
#include <vector>

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
  std::vector<Value> locals;
};

class Checkpoint {
public:
  std::vector<Value> stack;
  std::vector<Frame> frames;
};

enum class MigrationState : int32_t {
  STATE_NONE = 0,
  STATE_CHECKPOINT = 1,
  STATE_RESTORE = 2,
};

extern "C" struct ExecEnv {
  int8_t *memory_base;
  int32_t memory_size;
  MigrationState migration_state;
  Checkpoint *chkpt;
};
