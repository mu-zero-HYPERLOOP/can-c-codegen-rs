

add_library(utility 
    ${CMAKE_CURRENT_SOURCE_DIR}/src/mutex.c 
    ${CMAKE_CURRENT_SOURCE_DIR}/src/thread.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/time_util.c
)

target_include_directories(utility PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)

