cmake_minimum_required(VERSION 3.10)

project(wanco)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 20)
set(COMMON_RUST "${CMAKE_BINARY_DIR}/common-rust")
set(COMMON_RUST_RELEASE "${COMMON_RUST}/release")
include(ExternalProject)

# if CXX is set override CXX
if (DEFINED CXX)
    set(CMAKE_CXX_COMPILER "${CXX}")
endif()

set(CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})
#find_package(Libunwind REQUIRED)
#find_package(Libdwarf REQUIRED)

# https://qiita.com/tnishinaga/items/c01dfd5f43e446653cd1#cmake_toolchain_file%E3%82%92%E6%9B%B8%E3%81%8F
if ((DEFINED TARGET) AND ("${TARGET}" STREQUAL "aarch64"))

    # toolchain
    set(CMAKE_AR llvm-ar CACHE FILEPATH "Arhiver")
    set(CMAKE_RANLIB llvm-ranlib CACHE FILEPATH "Ranlib")
    set(CMAKE_AS llvm-as)
    set(CMAKE_NM llvm-nm)
    set(CMAKE_OBJDUMP llvm-objdump)
    set(CMAKE_C_COMPILER clang)
    set(CMAKE_CXX_COMPILER clang++)

    # target
    set(TRIPLE "aarch64-linux-gnu")
    set(RUST_TRIPLE "aarch64-unknown-linux-gnu")
    set(CMAKE_SYSTEM_NAME Linux)
    set(CMAKE_SYSTEM_PROCESSOR aarch64)
    set(CMAKE_C_COMPILER_TARGET "${TRIPLE}")
    set(CMAKE_CXX_COMPILER_TARGET "${TRIPLE}")
    
    # target options
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -target ${TRIPLE}")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -target ${TRIPLE}")
    
    # sysroot
    set(CMAKE_SYSROOT /usr/aarch64-linux-gnu/)
    set(CMAKE_INCLUDE_PATH ${CMAKE_SYSROOT}/usr/include)
    set(CMAKE_LIBRARY_PATH ${CMAKE_SYSROOT}/usr/lib)
    #set(CMAKE_INSTALL_PREFIX ${CMAKE_SYSROOT}/usr)
    set(CMAKE_FIND_ROOT_PATH ${CMAKE_SYSROOT}/usr)
    set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
    set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)

    set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

    # include 
    include_directories(${CMAKE_SYSROOT}/include)
    include_directories(${CMAKE_SYSROOT}/include/c++/11)
    include_directories(${CMAKE_SYSROOT}/include/c++/11/aarch64-linux-gnu/)
    # add -nostdinc++
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -nostdinc++")

    set(RUST_FLAGS "--target=${RUST_TRIPLE}")
    set(COMMON_RUST_RELEASE "${COMMON_RUST}/${RUST_TRIPLE}/release")

    # message
    message(STATUS "Building for target: ${TARGET} (${TRIPLE})")
endif()


add_library(wanco_rt)
target_compile_options(wanco_rt PRIVATE -Wall -g)
target_sources(wanco_rt PRIVATE
    cpp/api.cc
    cpp/wrt.cc
    cpp/chkpt/chkpt_json.cc
    cpp/chkpt/chkpt_protobuf.cc
    cpp/lz4/lz4.c
    #cpp/v2/stack_transform.cc
    #cpp/v2/stackmap.cc
    #cpp/v2/elf.cc
    cpp/protobuf/chkpt.pb.cc
    )
include_directories(wanco_rt PRIVATE
    cpp
    #${LIBUNWIND_INCLUDE_DIRS}
    )
target_link_libraries(wanco_rt PRIVATE lz4 
    # libunwind is dynamically linked
    # ${LIBUNWIND_LIBRARIES}
    )

file(GLOB_RECURSE wanco_wasi_src "${CMAKE_SOURCE_DIR}/rust/**/*")
set_source_files_properties(wanco_wasi_src PROPERTIES GENERATED TRUE)

# https://stackoverflow.com/questions/31162438/how-can-i-build-rust-code-with-a-c-qt-cmake-project
ExternalProject_Add(wanco_wasi
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BINARY_DIR "${CMAKE_SOURCE_DIR}/rust"
    BUILD_COMMAND cargo build --release --workspace --target-dir "${COMMON_RUST}" ${RUST_FLAGS}
    INSTALL_COMMAND ""
    LOG_BUILD ON
    )

install(TARGETS wanco_rt DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
install(FILES "${COMMON_RUST_RELEASE}/libwanco_wasi.a" DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
