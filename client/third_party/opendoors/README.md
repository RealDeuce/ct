# Vendored OpenDoors provenance

Cepheus Trader vendors the OpenDoors sources used by the player door so a
tagged source archive builds without a separate OpenDoors checkout.

- Upstream project: OpenDoors
- Upstream URL: <https://github.com/RealDeuce/OpenDoors>
- Copied at commit: `3edf9008a6df2a7d71674f8b43e307d1fc2f721d`
- Copy date: 2026-08-15
- License: GNU Lesser General Public License, version 2 or later, as stated
  in the source notices (`LGPL-2.0-or-later`)

Only files tracked by the recorded commit were copied. The live upstream
worktree had an unrelated modification to the excluded `ex_vote.c` example
and two untracked timing-analysis files; the vendored library surface was
clean at the recorded commit. Cepheus Trader does not copy or compile the
OpenDoors examples or their optional `xpdev` dependency.

The initial 2026-08-06 import came from Synchronet's
[`src/odoors`](https://gitlab.synchro.net/main/sbbs) directory at commit
`47feab1e8bf776175b44f40dffebbc9560322e20` (handoff audit commit
`aab6ab1aca4246a11ae83f3f7d74d6cefbce6fa9`).

## Copied build surface

The copy contains the complete OpenDoors source and header surface compiled by
`client/CMakeLists.txt`, plus its license:

- sources: `ODAuto.c`, `ODBlock.c`, `ODCFile.c`, `ODCmdLn.c`, `ODCom.c`,
  `ODCore.c`, `ODDrBox.c`, `ODEdit.c`, `ODEdStr.c`, `ODEmu.c`, `ODGetIn.c`,
  `ODFormat.c`, `ODFmtFB.c`, `ODGraph.c`, `ODInEx1.c`, `ODInEx2.c`,
  `ODInQue.c`, `ODKrnl.c`, `ODList.c`, `ODLog.c`, `ODMulti.c`, `ODPlat.c`,
  `ODPCB.c`, `ODPopup.c`, `ODPrntf.c`, `ODRA.c`, `ODSafe.c`, `ODScrn.c`,
  `ODSpawn.c`, `ODStand.c`, `ODStat.c`, `ODStr.c`, `ODSync.c`, `ODUtil.c`,
  `ODWCat.c`, `ODWin.c`, and Windows-only `ODConsole.c` and `ODFrame.c`;
- headers: `ODCom.h`, `ODConsole.h`, `ODCore.h`, `ODFormat.h`, `ODFrame.h`,
  `ODGen.h`, `ODInEx.h`, `ODInQue.h`, `ODKrnl.h`, `ODMulti.h`, `ODPlat.h`,
  `ODRes.h`, `ODSafe.h`, `ODScrn.h`, `ODStat.h`, `ODStr.h`, `ODSwap.h`,
  `ODSync.h`, `ODTypes.h`, `ODUtil.h`, `ODVScrn.h`, and `OpenDoor.h`;
- Windows resources: `ODRes.rc`, `ODApp.ico`, `ODInfo.ico`, and `Toolbar.bmp`;
  and
- the Trio sources under `third_party/trio`, used only as the bounded-formatting
  fallback on platforms without `vsnprintf`; and
- license: `license.txt`.

Prebuilt DLLs, libraries, generated resource intermediates, historical notes,
examples, and obsolete upstream build scripts are not used and were not
copied. Cepheus Trader's CMake file is the build script for this source set.

## Local patches

Cepheus Trader carries one local extension to preserve its terminal-profile
contract. Record every later source or header modification here with its date,
affected files, and purpose. Build-system integration outside this directory
is not an OpenDoors source patch.

- 2026-08-07 vendor-history note: added the Windows resource script, icons,
  and toolbar bitmap from the then-pinned Synchronet checkout.
- 2026-08-15: refreshed the complete compiled source surface and resources
  from OpenDoors commit `3edf9008a6df2a7d71674f8b43e307d1fc2f721d`.
- 2026-08-15: retained Cepheus Trader's `user_8bit` extension in `OpenDoor.h`
  and `ODInEx1.c`. It records explicit eight-data-bit framing from
  `DORINFOx.DEF`, `CHAIN.TXT`, and GAP-style `DOOR.SYS` independently of ANSI
  capability so automatic presentation selection remains conservative.
