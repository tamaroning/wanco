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
constexpr bool DEBUG = true;

#ifndef __linux__
#define MREMAP_MAYMOVE -1
#endif

// mremap is only available on Linux
inline void *wanco_mremap(void* old_address, size_t old_size,
                    size_t new_size, int flags) {

#ifdef __linux__
  return mremap(old_address, old_size, new_size, flags);
#else
  // TODO: implement mremap for other platforms
  std::cout << "mremap is not available on this platform" << std::endl;
  return NULL;
#endif
                    }

} // namespace wanco

class Debug {
public:
  Debug() {}

  template <typename T> std::ostream &operator<<(T &&val) {
    if constexpr (wanco::DEBUG) {
      out << "[debug] " << std::forward<T>(val);
    }
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
