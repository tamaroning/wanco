add_library(wanco STATIC)
target_sources(wanco PRIVATE
    lib.cc
    wrt.cc
    chkpt_decode.cc
    chkpt_encode.cc 
)

target_compile_options(wanco PRIVATE -Wall -g)
target_link_libraries(wanco PRIVATE m)
