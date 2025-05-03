#pragma once
#include <cstdint>
#include <cstring>
#include <iostream>
#include <thread>
#include <vector>

#define ASSERT(condition)                                                      \
  do {                                                                         \
    if (!(condition)) {                                                        \
      std::cerr << "Assertion failed: (" #condition ") in file " << __FILE__   \
                << ", line " << std::dec << __LINE__ << std::endl;             \
      std::abort();                                                            \
    }                                                                          \
  } while (false)

namespace wanco {
constexpr bool USE_LZ4 = false;
constexpr bool DEBUG_ENABLED = false;
constexpr int NUM_THREADS = 28;

extern uint64_t CHKPT_START_TIME;
extern uint64_t RESTORE_START_TIME;
} // namespace wanco

#define DEBUG_LOG                                                              \
  if constexpr (wanco::DEBUG_ENABLED)                                          \
  Debug()

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

inline void parallel_memcpy(void *dst, const void *src, size_t size,
                            int num_threads) {
  std::vector<std::thread> threads;
  size_t chunk_size = size / num_threads;

  for (int i = 0; i < num_threads; ++i) {
    size_t offset = i * chunk_size;
    size_t current_chunk_size =
        (i == num_threads - 1) ? (size - offset) : chunk_size;

    threads.emplace_back([=]() {
      std::memcpy(static_cast<char *>(dst) + offset,
                  static_cast<const char *>(src) + offset, current_chunk_size);
    });
  }

  for (auto &t : threads) {
    t.join();
  }
}
