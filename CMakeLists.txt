# Copyright 2023--present rgconf developers
cmake_minimum_required(VERSION 3.20)
project(rgconf_nanobind LANGUAGES CXX)

find_package(nanobind CONFIG)
if(nanobind_FOUND)
    find_library(RGCONF_C_ABI rgconf_c_abi REQUIRED)
    nanobind_add_module(_core bind/ext.cpp)
    target_include_directories(_core PRIVATE ${CMAKE_SOURCE_DIR}/include)
    target_link_libraries(_core PRIVATE ${RGCONF_C_ABI})
endif()
