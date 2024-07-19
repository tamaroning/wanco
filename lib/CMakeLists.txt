cmake_minimum_required(VERSION 3.10)

project(wanco)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_subdirectory(cpp)

# TODO: rust

install(TARGETS wanco DESTINATION /usr/local/lib)
