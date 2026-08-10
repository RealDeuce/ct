cmake_minimum_required(VERSION 3.20)
project(cepheus_trader_client VERSION 0.7.4 LANGUAGES C CXX)

if(WIN32)
  enable_language(RC)
endif()

# The client core is shared on every platform and privately embeds static
# fallback dependencies. Ensure in-tree Botan and Cap'n Proto builds are
# suitable for that link even when this is an ordinary non-portable build.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

include(CTest)
include(GNUInstallDirs)
include(ProcessorCount)

ProcessorCount(CT_DETECTED_BUILD_JOBS)
if(NOT CT_DETECTED_BUILD_JOBS OR CT_DETECTED_BUILD_JOBS LESS 1)
  set(CT_DETECTED_BUILD_JOBS 1)
endif()
set(
  CT_BOTAN_BUILD_JOBS
  "${CT_DETECTED_BUILD_JOBS}"
  CACHE STRING
  "Parallel jobs for the pinned Botan build"
)
if(NOT CT_BOTAN_BUILD_JOBS MATCHES "^[1-9][0-9]*$")
  message(FATAL_ERROR "CT_BOTAN_BUILD_JOBS must be a positive integer")
endif()
unset(CT_DETECTED_BUILD_JOBS)

find_package(Threads REQUIRED)

find_package(PkgConfig REQUIRED)
option(
  CT_PORTABLE_CLIENT
  "Build a self-contained client package from pinned static dependencies"
  OFF
)
option(
  CT_USE_SYSTEM_BOTAN
  "Prefer an installed Botan 3 package before building the pinned source"
  ON
)
if(CT_PORTABLE_CLIENT)
  set(CT_USE_SYSTEM_BOTAN OFF CACHE BOOL "Use installed Botan" FORCE)
endif()
include(cmake/BotanPinned.cmake)

set(
  CT_BOTAN_ROOT
  ""
  CACHE PATH
  "Prebuilt pinned Botan prefix (bypasses system and source builds)"
)
if(CT_BOTAN_ROOT)
  find_path(
    CT_PREBUILT_BOTAN_INCLUDE_DIR
    botan/build.h
    PATHS "${CT_BOTAN_ROOT}/include/botan-3"
    NO_DEFAULT_PATH
    REQUIRED
  )
  find_library(
    CT_PREBUILT_BOTAN_LIBRARY
    NAMES botan-3
    PATHS "${CT_BOTAN_ROOT}/lib" "${CT_BOTAN_ROOT}/lib64"
    NO_DEFAULT_PATH
    REQUIRED
  )
  add_library(ct-botan UNKNOWN IMPORTED GLOBAL)
  set_target_properties(
    ct-botan
    PROPERTIES
      IMPORTED_LOCATION "${CT_PREBUILT_BOTAN_LIBRARY}"
      INTERFACE_INCLUDE_DIRECTORIES "${CT_PREBUILT_BOTAN_INCLUDE_DIR}"
  )
  target_link_libraries(ct-botan INTERFACE Threads::Threads ${CMAKE_DL_LIBS})
  set(CT_BOTAN_TARGET ct-botan)
  message(STATUS "Using cached pinned Botan ${CT_BOTAN_VERSION}")
elseif(CT_USE_SYSTEM_BOTAN)
  pkg_check_modules(BOTAN QUIET IMPORTED_TARGET botan-3>=3.2)
endif()

if(CT_BOTAN_ROOT)
  # The imported target above is complete.
elseif(BOTAN_FOUND)
  set(CT_BOTAN_TARGET PkgConfig::BOTAN)
  message(STATUS "Using installed Botan ${BOTAN_VERSION}")
else()
  include(ExternalProject)
  find_package(Python3 REQUIRED COMPONENTS Interpreter)
  find_program(
    CT_BOTAN_MAKE_EXECUTABLE
    NAMES gmake make mingw32-make
    REQUIRED
  )
  if(CMAKE_GENERATOR MATCHES "Makefiles")
    set(CT_BOTAN_BUILD_COMMAND $(MAKE) -C <BINARY_DIR>)
    set(CT_BOTAN_INSTALL_COMMAND $(MAKE) -C <BINARY_DIR> install)
  else()
    set(
      CT_BOTAN_BUILD_COMMAND
      "${CT_BOTAN_MAKE_EXECUTABLE}"
      -C <BINARY_DIR>
      "-j${CT_BOTAN_BUILD_JOBS}"
    )
    set(
      CT_BOTAN_INSTALL_COMMAND
      "${CT_BOTAN_MAKE_EXECUTABLE}"
      -C <BINARY_DIR>
      install
    )
  endif()
  set(CT_BOTAN_INSTALL_DIR "${CMAKE_CURRENT_BINARY_DIR}/botan-install")
  set(CT_BOTAN_CROSS_ARGUMENTS)
  if(NOT WIN32)
    list(APPEND CT_BOTAN_CROSS_ARGUMENTS "--cc-abi-flags=-fPIC")
  endif()
  if(CMAKE_CROSSCOMPILING AND WIN32)
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86|i[3-6]86)$")
      set(CT_BOTAN_CPU x86_32)
    else()
      set(CT_BOTAN_CPU x86_64)
    endif()
    list(
      APPEND CT_BOTAN_CROSS_ARGUMENTS
      --os=mingw
      "--cpu=${CT_BOTAN_CPU}"
      "--ar-command=${CMAKE_AR}"
    )
  endif()
  file(MAKE_DIRECTORY "${CT_BOTAN_INSTALL_DIR}/include/botan-3")
  add_library(ct-botan STATIC IMPORTED GLOBAL)
  set_target_properties(
    ct-botan
    PROPERTIES
      IMPORTED_LOCATION "${CT_BOTAN_INSTALL_DIR}/lib/libbotan-3.a"
      INTERFACE_INCLUDE_DIRECTORIES "${CT_BOTAN_INSTALL_DIR}/include/botan-3"
  )
  ExternalProject_Add(
    ct-botan-project
    URL "${CT_BOTAN_URL}"
    URL_HASH "SHA256=${CT_BOTAN_SHA256}"
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    CONFIGURE_COMMAND
      "${Python3_EXECUTABLE}" <SOURCE_DIR>/configure.py
      "--prefix=${CT_BOTAN_INSTALL_DIR}"
      "--with-build-dir=<BINARY_DIR>"
      "--cc-bin=${CMAKE_CXX_COMPILER}"
      ${CT_BOTAN_CROSS_ARGUMENTS}
      --build-targets=static
      --minimized-build
      --enable-modules=tls13,auto_rng,system_rng,sha2_32,hex,x25519
      --without-documentation
      --disable-modules=bzip2,lzma,zlib,sqlite3,tpm,pkcs11
    BUILD_COMMAND ${CT_BOTAN_BUILD_COMMAND}
    INSTALL_COMMAND ${CT_BOTAN_INSTALL_COMMAND}
    BUILD_BYPRODUCTS "${CT_BOTAN_INSTALL_DIR}/lib/libbotan-3.a"
  )
  add_dependencies(ct-botan ct-botan-project)
  target_link_libraries(ct-botan INTERFACE Threads::Threads ${CMAKE_DL_LIBS})
  set(CT_BOTAN_TARGET ct-botan)
  message(STATUS "Building pinned Botan ${CT_BOTAN_VERSION}")
  unset(CT_BOTAN_BUILD_COMMAND)
  unset(CT_BOTAN_INSTALL_COMMAND)
endif()

if(WIN32 AND BOTAN_FOUND)
  get_target_property(
    CT_BOTAN_LINK_OPTIONS
    PkgConfig::BOTAN
    INTERFACE_LINK_OPTIONS
  )
  if(CT_BOTAN_LINK_OPTIONS)
    list(REMOVE_ITEM CT_BOTAN_LINK_OPTIONS -pthread)
    set_property(
      TARGET PkgConfig::BOTAN
      PROPERTY INTERFACE_LINK_OPTIONS "${CT_BOTAN_LINK_OPTIONS}"
    )
  endif()
endif()

option(
  CT_USE_SYSTEM_CAPNPROTO
  "Prefer an installed Cap'n Proto package before downloading it"
  ON
)
if(CT_PORTABLE_CLIENT)
  set(CT_USE_SYSTEM_CAPNPROTO OFF CACHE BOOL "Use installed Cap'n Proto" FORCE)
  set(BUILD_SHARED_LIBS OFF CACHE BOOL "Build static dependencies" FORCE)
endif()
include(cmake/CapnProtoPinned.cmake)

set(
  CT_CAPNPROTO_ROOT
  ""
  CACHE PATH
  "Prebuilt pinned Cap'n Proto prefix (bypasses system and source builds)"
)
if(CT_CAPNPROTO_ROOT)
  find_package(
    CapnProto ${CT_CAPNPROTO_VERSION} EXACT REQUIRED CONFIG
    PATHS
      "${CT_CAPNPROTO_ROOT}/lib/cmake/CapnProto"
      "${CT_CAPNPROTO_ROOT}/lib64/cmake/CapnProto"
    NO_DEFAULT_PATH
  )
  set(CT_USING_CACHED_CAPNPROTO TRUE)
elseif(CT_USE_SYSTEM_CAPNPROTO)
  find_package(CapnProto 1.0 QUIET CONFIG)

  # Some package layouts do not put their CMake configuration on CMake's
  # default prefix path, even though pkg-config knows the installation prefix.
  if(NOT CapnProto_FOUND)
    pkg_get_variable(CAPNPROTO_PREFIX capnp prefix)
    if(CAPNPROTO_PREFIX)
      find_package(
        CapnProto 1.0 QUIET CONFIG
        HINTS
          "${CAPNPROTO_PREFIX}/lib/cmake/CapnProto"
          "${CAPNPROTO_PREFIX}/lib64/cmake/CapnProto"
      )
    endif()
  endif()
endif()

if(CapnProto_FOUND)
  if(CT_USING_CACHED_CAPNPROTO)
    message(STATUS "Using cached pinned Cap'n Proto ${CapnProto_VERSION}")
  else()
    message(STATUS "Using installed Cap'n Proto ${CapnProto_VERSION}")
  endif()
else()
  include(FetchContent)
  if(CT_USE_SYSTEM_CAPNPROTO)
    message(
      STATUS
      "Cap'n Proto was not found; fetching pinned release ${CT_CAPNPROTO_VERSION}"
    )
  else()
    message(
      STATUS
      "System Cap'n Proto disabled; fetching pinned release ${CT_CAPNPROTO_VERSION}"
    )
  endif()

  FetchContent_Declare(
    capnproto
    URL "${CT_CAPNPROTO_URL}"
    URL_HASH "SHA256=${CT_CAPNPROTO_SHA256}"
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  )

  # Cap'n Proto shares these parent-scope cache option names. Its CMake 3.10
  # policy scope discards ordinary-variable overrides, so temporarily force
  # the cache entries and restore their prior state after adding the targets.
  set(_ct_saved_build_testing "${BUILD_TESTING}")
  foreach(_ct_option IN ITEMS WITH_FIBERS WITH_OPENSSL WITH_ZLIB)
    get_property(
      _ct_${_ct_option}_was_cached
      CACHE "${_ct_option}"
      PROPERTY TYPE
      SET
    )
    if(_ct_${_ct_option}_was_cached)
      get_property(
        _ct_saved_${_ct_option}
        CACHE "${_ct_option}"
        PROPERTY VALUE
      )
    endif()
  endforeach()

  set(BUILD_TESTING OFF CACHE BOOL "Build tests" FORCE)
  set(WITH_FIBERS OFF CACHE STRING "Cap'n Proto fiber support" FORCE)
  set(WITH_OPENSSL OFF CACHE STRING "Cap'n Proto OpenSSL support" FORCE)
  set(WITH_ZLIB OFF CACHE STRING "Cap'n Proto zlib support" FORCE)
  FetchContent_GetProperties(capnproto)
  if(NOT capnproto_POPULATED)
    if(POLICY CMP0169)
      cmake_policy(PUSH)
      cmake_policy(SET CMP0169 OLD)
    endif()
    FetchContent_Populate(capnproto)
    if(POLICY CMP0169)
      cmake_policy(POP)
    endif()
  endif()
  add_subdirectory(
    "${capnproto_SOURCE_DIR}"
    "${capnproto_BINARY_DIR}"
    EXCLUDE_FROM_ALL
  )

  set(BUILD_TESTING "${_ct_saved_build_testing}" CACHE BOOL "Build tests" FORCE)
  foreach(_ct_option IN ITEMS WITH_FIBERS WITH_OPENSSL WITH_ZLIB)
    if(_ct_${_ct_option}_was_cached)
      set(
        "${_ct_option}"
        "${_ct_saved_${_ct_option}}"
        CACHE STRING "Restored parent value" FORCE
      )
    else()
      unset("${_ct_option}" CACHE)
    endif()
    unset(_ct_saved_${_ct_option})
    unset(_ct_${_ct_option}_was_cached)
  endforeach()
  unset(_ct_option)
  unset(_ct_saved_build_testing)

  configure_file(
    "${capnproto_SOURCE_DIR}/LICENSE.txt"
    "${CMAKE_CURRENT_BINARY_DIR}/licenses/CapnProto-MIT.txt"
    COPYONLY
  )
endif()

# A cross-built capnp executable cannot generate sources on the build host.
# Prefer host-native tools outside the target sysroot while continuing to link
# the client against the target Cap'n Proto package or fetched target library.
if(CMAKE_CROSSCOMPILING)
  find_program(
    CT_HOST_CAPNP_EXECUTABLE
    NAMES capnp
    NO_CMAKE_FIND_ROOT_PATH
    REQUIRED
  )
  find_program(
    CT_HOST_CAPNPC_CXX_EXECUTABLE
    NAMES capnpc-c++
    NO_CMAKE_FIND_ROOT_PATH
    REQUIRED
  )
  set(CAPNP_EXECUTABLE "${CT_HOST_CAPNP_EXECUTABLE}")
  set(CAPNPC_CXX_EXECUTABLE "${CT_HOST_CAPNPC_CXX_EXECUTABLE}")
endif()

set(OPENDOORS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/third_party/opendoors")
set(
  CT_OPENDOORS_ROOT
  ""
  CACHE PATH
  "Prebuilt OpenDoors prefix (bypasses the vendored source build)"
)
if(CT_OPENDOORS_ROOT)
  find_path(
    CT_PREBUILT_OPENDOORS_INCLUDE_DIR
    OpenDoor.h
    PATHS "${CT_OPENDOORS_ROOT}/include/opendoors"
    NO_DEFAULT_PATH
    REQUIRED
  )
  find_library(
    CT_PREBUILT_OPENDOORS_LIBRARY
    NAMES ODoors
    PATHS "${CT_OPENDOORS_ROOT}/lib" "${CT_OPENDOORS_ROOT}/lib64"
    NO_DEFAULT_PATH
    REQUIRED
  )
  add_library(ct-opendoors UNKNOWN IMPORTED GLOBAL)
  set_target_properties(
    ct-opendoors
    PROPERTIES
      IMPORTED_LOCATION "${CT_PREBUILT_OPENDOORS_LIBRARY}"
      INTERFACE_INCLUDE_DIRECTORIES "${CT_PREBUILT_OPENDOORS_INCLUDE_DIR}"
  )
  if(WIN32)
    target_compile_definitions(ct-opendoors INTERFACE OD_WIN32_STATIC)
    target_link_libraries(ct-opendoors INTERFACE wsock32 gdi32 comctl32)
  else()
    target_link_libraries(ct-opendoors INTERFACE Threads::Threads)
  endif()
  message(STATUS "Using cached OpenDoors static library")
else()
  include(cmake/OpenDoors.cmake)
  ct_add_opendoors(ct-opendoors "${OPENDOORS_ROOT}")
endif()
add_library(OpenDoors::OpenDoors ALIAS ct-opendoors)
configure_file(
  "${OPENDOORS_ROOT}/license.txt"
  "${CMAKE_CURRENT_BINARY_DIR}/licenses/OpenDoors-LGPL-2.0-or-later.txt"
  COPYONLY
)

file(READ "${CMAKE_CURRENT_SOURCE_DIR}/../OPEN_GAME_LICENSE.md" CT_OGL_TEXT)
configure_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/src/legal_text.hpp.in"
  "${CMAKE_CURRENT_BINARY_DIR}/ct/legal_text.hpp"
  @ONLY
)

set(CAPNPC_SRC_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}/../protocol")
capnp_generate_cpp(
  CT_PROTOCOL_SOURCES
  CT_PROTOCOL_HEADERS
  "${CMAKE_CURRENT_SOURCE_DIR}/../protocol/ct_admin.capnp"
  "${CMAKE_CURRENT_SOURCE_DIR}/../protocol/ct_rpc.capnp"
  "${CMAKE_CURRENT_SOURCE_DIR}/../protocol/ct_sysop.capnp"
)

add_library(
  ct-display-format
  STATIC
  src/display_format.cpp
)
target_compile_features(ct-display-format PUBLIC cxx_std_20)
target_include_directories(
  ct-display-format
  PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
target_compile_options(
  ct-display-format
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

add_library(
  ct-client-core
  SHARED
  src/admin_protocol.cpp
  src/bbs_config.cpp
  src/bbs_credential.cpp
  src/crypto.cpp
  src/player_identity_registry.cpp
  src/protocol.cpp
  src/sysop_protocol.cpp
  src/tls_connection.cpp
  ${CT_PROTOCOL_SOURCES}
)
target_compile_features(ct-client-core PUBLIC cxx_std_20)
target_include_directories(
  ct-client-core
  PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_BINARY_DIR}"
)
target_link_libraries(
  ct-client-core
  PRIVATE
    CapnProto::capnp
    ct-display-format
    ${CT_BOTAN_TARGET}
)
if(WIN32)
  target_link_directories(ct-client-core PRIVATE ${BOTAN_LIBRARY_DIRS})
  target_link_libraries(ct-client-core PRIVATE advapi32 ws2_32)
endif()
target_compile_options(
  ct-client-core
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)
set_target_properties(
  ct-client-core
  PROPERTIES
    OUTPUT_NAME cepheus-trader-client-core
    WINDOWS_EXPORT_ALL_SYMBOLS ON
)
if(WIN32)
  set_target_properties(ct-client-core PROPERTIES PREFIX "")
endif()
if(MINGW)
  # MinGW auto-exports symbols pulled from static archives unless told not to.
  # Keep Botan, Cap'n Proto/KJ, and the static GNU runtime private to the DLL;
  # otherwise their import symbols collide with the executables' runtimes.
  target_link_options(ct-client-core PRIVATE "-Wl,--exclude-libs,ALL")
endif()

add_library(
  ct-door-presentation
  src/cargo_quantity.cpp
  src/crew_presentation.cpp
  src/door_help.cpp
  src/door_presentation.cpp
)
target_compile_features(ct-door-presentation PUBLIC cxx_std_20)
target_include_directories(
  ct-door-presentation
  PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
target_compile_options(
  ct-door-presentation
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)
target_link_libraries(ct-door-presentation PUBLIC ct-display-format)

add_executable(cepheus-trader-client src/main.cpp)
target_link_libraries(cepheus-trader-client PRIVATE ct-client-core)
target_compile_options(
  cepheus-trader-client
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

add_executable(cepheus-trader-door src/door_main.cpp)
if(WIN32)
  target_sources(
    cepheus-trader-door
    PRIVATE "${OPENDOORS_ROOT}/ODRes.rc"
  )
endif()
target_link_libraries(
  cepheus-trader-door
  PRIVATE
    ct-client-core
    ct-door-presentation
    OpenDoors::OpenDoors
)
target_compile_options(
  cepheus-trader-door
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

if(BUILD_TESTING)
  add_executable(
    ct-bbs-config-test
    tests/bbs_config_test.cpp
  )
  target_link_libraries(
    ct-bbs-config-test
    PRIVATE ct-client-core
  )
  target_compile_options(
    ct-bbs-config-test
    PRIVATE
      -Wall
      -Wextra
      -Wpedantic
      -Werror
  )
  add_test(
    NAME ct-bbs-config
    COMMAND ct-bbs-config-test
  )

  add_executable(
    ct-player-identity-registry-test
    tests/player_identity_registry_test.cpp
  )
  target_link_libraries(
    ct-player-identity-registry-test
    PRIVATE ct-client-core ${CT_BOTAN_TARGET}
  )
  target_compile_options(
    ct-player-identity-registry-test
    PRIVATE
      -Wall
      -Wextra
      -Wpedantic
      -Werror
  )
  add_test(
    NAME ct-player-identity-registry
    COMMAND ct-player-identity-registry-test
  )

  add_executable(
    ct-protocol-test
    tests/protocol_test.cpp
  )
  target_link_libraries(
    ct-protocol-test
    PRIVATE ct-client-core
  )
  target_compile_options(
    ct-protocol-test
    PRIVATE
      -Wall
      -Wextra
      -Wpedantic
      -Werror
  )
  add_test(
    NAME ct-protocol
    COMMAND ct-protocol-test
  )

  add_executable(
    ct-door-presentation-test
    tests/door_presentation_test.cpp
  )
  target_link_libraries(
    ct-door-presentation-test
    PRIVATE ct-door-presentation
  )
  target_compile_options(
    ct-door-presentation-test
    PRIVATE
      -Wall
      -Wextra
      -Wpedantic
      -Werror
  )
  add_test(
    NAME ct-door-presentation
    COMMAND ct-door-presentation-test
  )
endif()

add_executable(cepheus-trader-admin src/admin_main.cpp)
target_link_libraries(cepheus-trader-admin PRIVATE ct-client-core)
target_compile_options(
  cepheus-trader-admin
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

add_executable(cepheus-trader-sysop src/sysop_main.cpp)
target_link_libraries(cepheus-trader-sysop PRIVATE ct-client-core)
target_compile_options(
  cepheus-trader-sysop
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

# An operator-invoked field-alpha load driver. It is intentionally excluded
# from the default build and from CTest: opening fifty live sessions is an
# operations benchmark, not a unit or CI test.
add_executable(
  ct-field-alpha-load
  EXCLUDE_FROM_ALL
  tests/field_alpha_load.cpp
)
target_link_libraries(ct-field-alpha-load PRIVATE ct-client-core)
target_compile_options(
  ct-field-alpha-load
  PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
)

foreach(
  CT_VERSIONED_TARGET
  IN ITEMS
    cepheus-trader-client
    cepheus-trader-door
    cepheus-trader-admin
    cepheus-trader-sysop
    ct-field-alpha-load
)
  target_compile_definitions(
    ${CT_VERSIONED_TARGET}
    PRIVATE CT_PRODUCT_VERSION="${PROJECT_VERSION}"
  )
endforeach()

if(CT_PORTABLE_CLIENT AND CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  foreach(
    CT_PORTABLE_TARGET
    IN ITEMS
      ct-client-core
      cepheus-trader-door
      cepheus-trader-sysop
  )
    target_link_options(
      ${CT_PORTABLE_TARGET}
      PRIVATE -static-libgcc -static-libstdc++
    )
    if(WIN32)
      target_link_options(${CT_PORTABLE_TARGET} PRIVATE -static)
    endif()
  endforeach()
endif()

if(APPLE)
  set(CT_CLIENT_INSTALL_RPATH "@loader_path")
elseif(UNIX)
  set(CT_CLIENT_INSTALL_RPATH "\$ORIGIN")
endif()
set_target_properties(
  cepheus-trader-admin
  cepheus-trader-client
  cepheus-trader-door
  cepheus-trader-sysop
  PROPERTIES BUILD_RPATH_USE_ORIGIN ON
)
if(DEFINED CT_CLIENT_INSTALL_RPATH)
  set_target_properties(
    cepheus-trader-door
    cepheus-trader-sysop
    PROPERTIES INSTALL_RPATH "${CT_CLIENT_INSTALL_RPATH}"
  )
endif()

set(
  CT_SOURCE_RELEASE_URL
  ""
  CACHE STRING
  "Exact source release URL recorded in packaged client kits"
)
configure_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/cmake/source-release.txt.in"
  "${CMAKE_CURRENT_BINARY_DIR}/SOURCE-RELEASE.txt"
  @ONLY
)

install(
  TARGETS
    ct-client-core
    cepheus-trader-door
    cepheus-trader-sysop
  RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}"
  LIBRARY DESTINATION "${CMAKE_INSTALL_BINDIR}"
  COMPONENT Client
)
install(
  FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/cepheus-trader.conf.example"
    "${CMAKE_CURRENT_SOURCE_DIR}/README.md"
    "${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE.md"
    "${CMAKE_CURRENT_SOURCE_DIR}/../OPEN_GAME_LICENSE.md"
    "${CMAKE_CURRENT_SOURCE_DIR}/../THIRD_PARTY_LICENSES.md"
    "${CMAKE_CURRENT_SOURCE_DIR}/third_party/opendoors/license.txt"
    "${CMAKE_CURRENT_SOURCE_DIR}/third_party/licenses/Botan-BSD-2-Clause.txt"
    "${CMAKE_CURRENT_SOURCE_DIR}/third_party/licenses/CapnProto-MIT.txt"
    "${CMAKE_CURRENT_BINARY_DIR}/SOURCE-RELEASE.txt"
  DESTINATION "${CMAKE_INSTALL_DATADIR}/cepheus-trader"
  COMPONENT Client
)
install(
  FILES "${CMAKE_CURRENT_SOURCE_DIR}/../docs/player-guide.md"
  DESTINATION "${CMAKE_INSTALL_DATADIR}/cepheus-trader"
  RENAME "PLAYER-GUIDE.md"
  COMPONENT Client
)
install(
  FILES "${CMAKE_CURRENT_SOURCE_DIR}/../docs/sysop-guide.md"
  DESTINATION "${CMAKE_INSTALL_DATADIR}/cepheus-trader"
  RENAME "SYSOP-GUIDE.md"
  COMPONENT Client
)
install(
  FILES "${CMAKE_CURRENT_SOURCE_DIR}/../install-xtrn.ini"
  DESTINATION "."
  COMPONENT Client
)

string(TOLOWER "${CMAKE_SYSTEM_NAME}" CT_PACKAGE_PLATFORM)
set(
  CT_PACKAGE_ARCHITECTURE
  ""
  CACHE STRING
  "Architecture label used in the client package filename"
)
if(NOT CT_PACKAGE_ARCHITECTURE)
  string(TOLOWER "${CMAKE_SYSTEM_PROCESSOR}" CT_PACKAGE_ARCHITECTURE)
endif()
if(CT_PACKAGE_PLATFORM STREQUAL "windows")
  set(CPACK_GENERATOR ZIP)
else()
  set(CPACK_GENERATOR TXZ)
endif()
set(CPACK_PACKAGE_NAME "cepheus-trader-client")
set(CPACK_PACKAGE_VENDOR "Cepheus Trader contributors")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
if(CT_PORTABLE_CLIENT)
  set(CPACK_STRIP_FILES TRUE)
endif()
set(
  CPACK_PACKAGE_FILE_NAME
  "cepheus-trader-client-${PROJECT_VERSION}-${CT_PACKAGE_PLATFORM}-${CT_PACKAGE_ARCHITECTURE}"
)
set(CPACK_INCLUDE_TOPLEVEL_DIRECTORY ON)
include(CPack)
