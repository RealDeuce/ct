include(CheckSymbolExists)

function(ct_add_opendoors target source_root)
  check_symbol_exists(vsnprintf "stdio.h" CT_OPENDOORS_HAVE_VSNPRINTF)

  set(
    opendoors_sources
    ODAuto.c
    ODBlock.c
    ODCFile.c
    ODCmdLn.c
    ODCom.c
    ODCore.c
    ODDrBox.c
    ODEdit.c
    ODEdStr.c
    ODEmu.c
    ODFormat.c
    ODFmtFB.c
    ODGetIn.c
    ODGraph.c
    ODInEx1.c
    ODInEx2.c
    ODInQue.c
    ODKrnl.c
    ODList.c
    ODLog.c
    ODMulti.c
    ODPlat.c
    ODPCB.c
    ODPopup.c
    ODPrntf.c
    ODRA.c
    ODSafe.c
    ODScrn.c
    ODSpawn.c
    ODStand.c
    ODStat.c
    ODStr.c
    ODSync.c
    ODUtil.c
    ODWCat.c
    ODWin.c
  )
  list(TRANSFORM opendoors_sources PREPEND "${source_root}/")
  if(WIN32)
    list(
      APPEND opendoors_sources
      "${source_root}/ODConsole.c"
      "${source_root}/ODFrame.c"
    )
  endif()

  add_library(${target} STATIC ${opendoors_sources})
  set_target_properties(
    ${target}
    PROPERTIES
      C_STANDARD 99
      C_STANDARD_REQUIRED YES
      OUTPUT_NAME ODoors
  )
  target_compile_definitions(
    ${target}
    PRIVATE
      HAS_INTTYPES_H
      $<$<BOOL:${CT_OPENDOORS_HAVE_VSNPRINTF}>:OPENDOORS_HAVE_VSNPRINTF=1>
    PUBLIC
      $<$<PLATFORM_ID:Windows>:OD_WIN32_STATIC>
      $<$<PLATFORM_ID:Windows>:OD_WINDOWS_CONSOLE>
  )
  if(APPLE)
    target_compile_definitions(${target} PRIVATE __unix__)
  endif()
  target_include_directories(${target} PUBLIC "${source_root}")
  if(WIN32)
    target_link_libraries(
      ${target}
      PRIVATE ws2_32 user32 gdi32 advapi32 shell32 uuid comctl32
    )
  else()
    target_link_libraries(${target} PUBLIC Threads::Threads)
  endif()
endfunction()
