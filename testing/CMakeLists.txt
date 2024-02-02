######################################################################
# @author      : karl (karlsasssie@gmail.com)
# @file        : CMakeLists
# @created     : Friday Apr 07, 2023 15:39:05 CEST
######################################################################

cmake_minimum_required(VERSION 3.5)

project(test1 C)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(FetchContent)

FetchContent_Declare(
  can-socketcan-platform
  GIT_REPOSITORY https://github.com/mu-zero-HYPERLOOP/can-socketcan-platform
  GIT_TAG origin/main
)

FetchContent_MakeAvailable(can-socketcan-platform)


add_subdirectory(utility)
add_subdirectory(secu)
# add_subdirectory(master)

add_executable(${PROJECT_NAME} src/main.c src/test.c)
add_dependencies(${PROJECT_NAME} secu)

target_include_directories(${PROJECT_NAME} PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/secu/canzero/)

target_link_libraries(${PROJECT_NAME} 
  PRIVATE 
    can-socketcan-platform
    utility
)

