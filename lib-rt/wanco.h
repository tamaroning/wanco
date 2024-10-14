#pragma once

#define ASSERT(condition) \
    do { \
        if (!(condition)) { \
            std::cerr << "Assertion failed: (" #condition ") in file " << __FILE__ \
                      << ", line " << std::dec << __LINE__ << std::endl; \
            std::abort(); \
        } \
    } while (false)
