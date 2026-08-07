cmake_minimum_required(VERSION 3.20)
project(ct_capnproto_dependency LANGUAGES CXX)

include(ExternalProject)
include(ProcessorCount)
include("${CMAKE_CURRENT_LIST_DIR}/../../CapnProtoPinned.cmake")

set(CT_INSTALL_PREFIX "" CACHE PATH "Pinned Cap'n Proto installation directory")
if(NOT CT_INSTALL_PREFIX)
  message(FATAL_ERROR "CT_INSTALL_PREFIX is required")
endif()

ProcessorCount(CT_BUILD_JOBS)
if(NOT CT_BUILD_JOBS OR CT_BUILD_JOBS LESS 1)
  set(CT_BUILD_JOBS 1)
endif()

ExternalProject_Add(
  capnproto
  URL "${CT_CAPNPROTO_URL}"
  URL_HASH "SHA256=${CT_CAPNPROTO_SHA256}"
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  CMAKE_ARGS
    "-DCMAKE_BUILD_TYPE=RelWithDebInfo"
    "-DCMAKE_CXX_COMPILER=${CMAKE_CXX_COMPILER}"
    "-DCMAKE_INSTALL_PREFIX=${CT_INSTALL_PREFIX}"
    "-DBUILD_SHARED_LIBS=OFF"
    "-DBUILD_TESTING=OFF"
    "-DWITH_FIBERS=OFF"
    "-DWITH_OPENSSL=OFF"
    "-DWITH_ZLIB=OFF"
  BUILD_COMMAND
    "${CMAKE_COMMAND}" --build <BINARY_DIR> --parallel "${CT_BUILD_JOBS}"
  INSTALL_COMMAND "${CMAKE_COMMAND}" --install <BINARY_DIR>
  BUILD_BYPRODUCTS
    "${CT_INSTALL_PREFIX}/lib/libcapnp.a"
    "${CT_INSTALL_PREFIX}/lib/libkj.a"
)
