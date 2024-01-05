######################################################################
# @author      : karl (karlsasssie@gmail.com)
# @file        : CMakeLists
# @created     : Friday Apr 07, 2023 15:39:05 CEST
######################################################################

cmake_minimum_required(VERSION 3.5)

project(test1 CXX C)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/canzero/
  COMMAND mkdir canzero
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)

add_custom_target(
  run_always
  DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/__header.h
)

add_custom_command(
  OUTPUT 
    ${CMAKE_CURRENT_BINARY_DIR}/__header.h
    ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.h 
    ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.c 
  COMMAND cargo run -- ${CMAKE_CURRENT_SOURCE_DIR}/test.yaml secu ${CMAKE_CURRENT_BINARY_DIR}/canzero
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/testing_cli/
  DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/canzero
)

## mock can
add_subdirectory(mock_can)

add_executable(${PROJECT_NAME} src/main.cpp ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.c)
target_include_directories(${PROJECT_NAME} PRIVATE ${CMAKE_BINARY_DIR}/canzero/)
target_link_libraries(${PROJECT_NAME} PRIVATE mock_can)

