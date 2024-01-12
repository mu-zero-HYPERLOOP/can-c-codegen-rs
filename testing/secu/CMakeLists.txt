######################################################################
# @author      : karl (karlsasssie@gmail.com)
# @file        : CMakeLists
# @created     : Friday Apr 07, 2023 15:39:05 CEST
######################################################################

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/canzero/
  COMMAND mkdir canzero
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)

add_custom_target(
  secu_run_always
  DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/__header.h
)

add_custom_command(
  OUTPUT 
    ${CMAKE_CURRENT_BINARY_DIR}/__header.h
    ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.h 
    ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.c 
  COMMAND cargo run -- ${PROJECT_SOURCE_DIR}/test.yaml secu ${CMAKE_CURRENT_BINARY_DIR}/canzero
  WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}/testing_cli/
  DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/canzero
)

add_executable(secu ${CMAKE_CURRENT_SOURCE_DIR}/src/secu.c ${CMAKE_CURRENT_BINARY_DIR}/canzero/canzero.c)

target_link_libraries(secu PRIVATE utility can-socketcan-platform)

target_include_directories(secu 
    PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src 
    PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/canzero/
)
