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

if (APPLE)
    # LIBRARY_PATHとINCLUDE_PATHが認識されないので、手動で追加する
    
    # lib-rtで必要
    link_directories(
        /usr/local/opt/protobuf/lib
        )
    # $(brew --prefix (protobuf,abseil))/libのほうが良い
    target_include_directories(wanco_rt PRIVATE
        /usr/local/opt/protobuf/include
        # https://github.com/protocolbuffers/protobuf/issues/16755
        /usr/local/opt/abseil/include
        )
    # wancoビルド時にccに-lzstdが渡されるが、not foundと言われるので、ライブラリパスを追加する
    corrosion_add_target_local_rustflags(wanco
        "-Clink-args=-L/usr/local/opt/zstd/lib -lzstd"
        )
endif()

install(TARGETS wanco_rt DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
install(FILES ${CMAKE_BINARY_DIR}/libwanco_wasi.a DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
install(FILES ${CMAKE_BINARY_DIR}/wanco
    PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE
        GROUP_READ GROUP_EXECUTE
        WORLD_READ WORLD_EXECUTE
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)
