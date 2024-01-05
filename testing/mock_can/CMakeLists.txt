######################################################################
# @author      : karl (karlsasssie@gmail.com)
# @file        : CMakeLists
# @created     : Friday Apr 07, 2023 15:39:05 CEST
######################################################################

cmake_minimum_required(VERSION 3.5)
project(mock_can C)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(mock_can src/can_adapter.c)
target_include_directories(mock_can PUBLIC src/)

