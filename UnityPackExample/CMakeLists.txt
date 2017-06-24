cmake_minimum_required (VERSION 2.8.11)
project (UnityPackRustExample)

include_directories(${CMAKE_SOURCE_DIR}/../UnityPackRust/include)
link_directories(${CMAKE_SOURCE_DIR}/../UnityPackRust/target/release/)

add_executable(UnityPackRustExample src/main.c)
TARGET_LINK_LIBRARIES(UnityPackRustExample unitypack)

