cmake_minimum_required(VERSION 3.10)

set(CMAKE_CXX_STANDARD 20)
find_package(Protobuf REQUIRED)
find_package(Libdwarf REQUIRED)
find_package(Libunwind REQUIRED)

protobuf_generate_cpp (PROTO_SRCS PROTO_HDRS protobuf/chkpt.proto)
get_filename_component(PROTO_HDR_DIRS ${PROTO_HDRS} DIRECTORY)

add_library(wanco_rt)
target_compile_options(wanco_rt PRIVATE -Wall -g)
target_sources(wanco_rt PRIVATE
    api.cc
    wrt.cc
    chkpt/chkpt_protobuf.cc
    lz4/lz4.c
    stackmap/elf.cc
    stackmap/stackmap.cc
    ${PROTO_SRCS}
    )
include_directories(wanco_rt PUBLIC
    .
    ${Protobuf_INCLUDE_DIRS}
    ${PROTO_HDR_DIRS}
    )

target_link_libraries(wanco_rt PUBLIC
    ${Protobuf_LIBRARIES}
)
