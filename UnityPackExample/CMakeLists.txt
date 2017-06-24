cmake_minimum_required (VERSION 2.8.11)
project (UnityPackRustExample)

include_directories(${CMAKE_SOURCE_DIR}/../UnityPack/include)
link_directories(${CMAKE_SOURCE_DIR}/../UnityPack/target/release/)

add_executable(UnityPackRustExample src/main.c)
TARGET_LINK_LIBRARIES(UnityPackRustExample unitypack)

