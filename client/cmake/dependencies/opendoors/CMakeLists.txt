cmake_minimum_required(VERSION 3.20)
project(ct_opendoors_dependency LANGUAGES C)

include(GNUInstallDirs)
find_package(Threads REQUIRED)
include("${CMAKE_CURRENT_LIST_DIR}/../../OpenDoors.cmake")

set(CT_OPENDOORS_SOURCE_DIR "" CACHE PATH "OpenDoors source directory")
if(NOT CT_OPENDOORS_SOURCE_DIR)
  message(FATAL_ERROR "CT_OPENDOORS_SOURCE_DIR is required")
endif()

ct_add_opendoors(ct-opendoors "${CT_OPENDOORS_SOURCE_DIR}")
install(
  TARGETS ct-opendoors
  ARCHIVE DESTINATION "${CMAKE_INSTALL_LIBDIR}"
)
install(
  DIRECTORY "${CT_OPENDOORS_SOURCE_DIR}/"
  DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/opendoors"
  FILES_MATCHING PATTERN "*.h"
)
