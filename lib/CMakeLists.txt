cmake_minimum_required(VERSION 3.10)

project(wanco)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(COMMON_RUST "${CMAKE_BINARY_DIR}/common-rust")
include(ExternalProject)

add_library(wanco_rt)
target_compile_options(wanco_rt PRIVATE -Wall -g)
target_sources(wanco_rt PRIVATE
    cpp/lib.cc
    cpp/wrt.cc
    cpp/chkpt_decode.cc
    cpp/chkpt_encode.cc 
    )


file(GLOB_RECURSE wanco_wasi_src "${CMAKE_SOURCE_DIR}/rust/**/*")
set_source_files_properties(wanco_wasi_src PROPERTIES GENERATED TRUE)

# https://stackoverflow.com/questions/31162438/how-can-i-build-rust-code-with-a-c-qt-cmake-project
ExternalProject_Add(wanco_wasi
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BINARY_DIR "${CMAKE_SOURCE_DIR}/rust"
    BUILD_COMMAND cargo build --release --workspace --target-dir "${COMMON_RUST}"
    INSTALL_COMMAND ""
    LOG_BUILD ON
    )

install(TARGETS wanco_rt DESTINATION /usr/local/lib)
install(FILES "${COMMON_RUST}/release/libwanco_wasi.a" DESTINATION /usr/local/lib)
