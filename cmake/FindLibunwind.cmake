# FindLibunwind.cmake - Find the libunwind library and headers
#
# This module defines the following variables:
#   LIBUNWIND_FOUND       - True if libunwind is found
#   LIBUNWIND_INCLUDE_DIRS - Directories containing libunwind headers
#   LIBUNWIND_LIBRARIES   - Libraries to link against for libunwind
#
# Usage:
#   find_package(Libunwind REQUIRED)
#   target_include_directories(MyTarget PRIVATE ${LIBUNWIND_INCLUDE_DIRS})
#   target_link_libraries(MyTarget PRIVATE ${LIBUNWIND_LIBRARIES})

find_path(LIBUNWIND_INCLUDE_DIRS
  NAMES libunwind.h
  PATHS
    /usr/local/include
    /usr/include
    /opt/local/include
)

find_library(LIBUNWIND_LIBRARIES
  NAMES unwind
  PATHS
    /usr/local/lib
    /usr/lib
    /opt/local/lib
)

# Try to find unwind-x86_64 for 64-bit architectures
find_library(LIBUNWIND_X86_64_LIBRARIES
  NAMES unwind-x86_64
  PATHS
    /usr/local/lib
    /usr/lib
    /opt/local/lib
)

# Combine libraries if both are found
if(LIBUNWIND_X86_64_LIBRARIES)
  list(APPEND LIBUNWIND_LIBRARIES ${LIBUNWIND_X86_64_LIBRARIES})
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(Libunwind DEFAULT_MSG LIBUNWIND_LIBRARIES LIBUNWIND_INCLUDE_DIRS)

mark_as_advanced(LIBUNWIND_INCLUDE_DIRS LIBUNWIND_LIBRARIES LIBUNWIND_X86_64_LIBRARIES)