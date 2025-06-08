#pragma once

#if defined(__x86_64__) || defined(_M_X64)
#include "arch/x86_64.h"
#elif defined(__aarch64__) || defined(_M_ARM64)
#include "arch/aarch64.h"
#endif

