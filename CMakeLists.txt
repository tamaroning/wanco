cmake_minimum_required(VERSION 3.15)
project(wanco-project)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 20)

# if CXX is set override CXX
if (DEFINED CXX)
    set(CMAKE_CXX_COMPILER "${CXX}")
endif()

include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5 # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)

# Rust projects
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

# C++ projects
add_subdirectory(lib-rt)

install(TARGETS wanco_rt DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
install(FILES ${CMAKE_BINARY_DIR}/libwanco_wasi.a DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
install(FILES ${CMAKE_BINARY_DIR}/wanco DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)
