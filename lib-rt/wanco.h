#pragma once
#include <iostream>

#define ASSERT(condition)                                                      \
  do {                                                                         \
    if (!(condition)) {                                                        \
      std::cerr << "Assertion failed: (" #condition ") in file " << __FILE__   \
                << ", line " << std::dec << __LINE__ << std::endl;             \
      std::abort();                                                            \
    }                                                                          \
  } while (false)

namespace wanco {
constexpr bool USE_PROTOBUF = true;
constexpr bool DEBUG_ENABLED = false;
} // namespace wanco

#define DEBUG_LOG if constexpr(wanco::DEBUG_ENABLED) Debug()

class Debug {
public:
  Debug() {}

  template <typename T> std::ostream &operator<<(T &&val) {
    out << "[debug] " << std::forward<T>(val);
    return out;
  }

private:
  std::ostream &out = std::cerr;
};

class Info {
public:
  Info() {}

  template <typename T> std::ostream &operator<<(T &&val) {
    out << "[info] " << std::forward<T>(val);
    return out;
  }

private:
  std::ostream &out = std::cerr;
};

class Fatal {
public:
  Fatal() {}

  template <typename T> std::ostream &operator<<(T &&val) {
    out << "Fatal Error: " << std::forward<T>(val);
    return out;
  }

private:
  std::ostream &out = std::cerr;
};

class Warn {
public:
  Warn() {}

  template <typename T> std::ostream &operator<<(T &&val) {
    out << "Warning: " << std::forward<T>(val);
    return out;
  }

private:
  std::ostream &out = std::cerr;
};
