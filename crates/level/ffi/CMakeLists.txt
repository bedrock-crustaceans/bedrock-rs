cmake_minimum_required(VERSION 3.13)
project(leveldb-ffi)

# set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/out)
# set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/out)
# set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/out)

add_subdirectory(leveldb)
add_library(${PROJECT_NAME} STATIC ffi.cpp)
target_include_directories(${PROJECT_NAME} PRIVATE leveldb/include)
target_link_libraries(${PROJECT_NAME} PRIVATE leveldb-mcpe)