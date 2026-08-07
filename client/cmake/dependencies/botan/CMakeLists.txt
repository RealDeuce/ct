cmake_minimum_required(VERSION 3.20)
project(ct_botan_dependency LANGUAGES CXX)

include(ExternalProject)
include(ProcessorCount)
include("${CMAKE_CURRENT_LIST_DIR}/../../BotanPinned.cmake")

set(CT_INSTALL_PREFIX "" CACHE PATH "Pinned Botan installation directory")
if(NOT CT_INSTALL_PREFIX)
  message(FATAL_ERROR "CT_INSTALL_PREFIX is required")
endif()

ProcessorCount(CT_BUILD_JOBS)
if(NOT CT_BUILD_JOBS OR CT_BUILD_JOBS LESS 1)
  set(CT_BUILD_JOBS 1)
endif()
find_package(Python3 REQUIRED COMPONENTS Interpreter)
find_program(CT_MAKE_EXECUTABLE NAMES gmake make REQUIRED)

ExternalProject_Add(
  botan
  URL "${CT_BOTAN_URL}"
  URL_HASH "SHA256=${CT_BOTAN_SHA256}"
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  CONFIGURE_COMMAND
    "${Python3_EXECUTABLE}" <SOURCE_DIR>/configure.py
    "--prefix=${CT_INSTALL_PREFIX}"
    "--with-build-dir=<BINARY_DIR>"
    "--cc-bin=${CMAKE_CXX_COMPILER}"
    --build-targets=static
    --minimized-build
    --enable-modules=tls13,auto_rng,system_rng,sha2_32,hex
    --without-documentation
    --disable-modules=bzip2,lzma,zlib,sqlite3,tpm,pkcs11
  BUILD_COMMAND
    "${CT_MAKE_EXECUTABLE}" -C <BINARY_DIR> "-j${CT_BUILD_JOBS}"
  INSTALL_COMMAND "${CT_MAKE_EXECUTABLE}" -C <BINARY_DIR> install
  BUILD_BYPRODUCTS "${CT_INSTALL_PREFIX}/lib/libbotan-3.a"
)
