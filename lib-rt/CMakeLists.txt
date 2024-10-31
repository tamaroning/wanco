cmake_minimum_required(VERSION 3.10)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})
find_package(Protobuf REQUIRED)

add_library(wanco_rt)
target_compile_options(wanco_rt PRIVATE -Wall -g)
target_sources(wanco_rt PRIVATE
    api.cc
    wrt.cc
    chkpt/chkpt_protobuf.cc
    lz4/lz4.c
    protobuf/chkpt.pb.cc
    )
include_directories(wanco_rt PUBLIC
    .
    ${Protobuf_INCLUDE_DIRS}
    )
target_link_libraries(wanco_rt PUBLIC
    ${Protobuf_LIBRARIES}
)
