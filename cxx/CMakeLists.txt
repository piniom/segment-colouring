cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 20)

project(proper_interval_online_coloring VERSION 1.0)

include(CTest)

find_package(Boost CONFIG COMPONENTS program_options REQUIRED)
include_directories( ${Boost_INCLUDE_DIR} )

add_executable(strategy_find strategy_find.cpp common.cpp state.cpp game.cpp)
target_link_libraries(strategy_find LINK_PUBLIC ${Boost_LIBRARIES} )

add_executable(strategy_check strategy_check.cpp common.cpp state.cpp game.cpp)
target_link_libraries(strategy_check LINK_PUBLIC ${Boost_LIBRARIES} )

#add_executable(test_code tests/code.cpp common.cpp state.cpp game.cpp)
#target_link_libraries(test_code LINK_PUBLIC ${Boost_LIBRARIES} )
#add_test(NAME Code COMMAND test_code)

