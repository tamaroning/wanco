cmake_minimum_required(VERSION 3.10)

project(wanco)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(libc-wasi/libc_wasi.cmake)

add_library(wanco STATIC
    lib.cc
    wrt.cc
    chkpt_decode.cc
    chkpt_encode.cc
)

target_compile_options(wanco PRIVATE -Wall -g)
target_link_libraries(wanco PRIVATE m)

install(TARGETS wanco DESTINATION /usr/local/lib)
