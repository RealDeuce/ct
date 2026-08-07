# Vendored OpenDoors provenance

Cepheus Trader vendors the OpenDoors sources used by the player door so a
tagged source archive builds without a separate Synchronet checkout.

- Upstream project: Synchronet BBS Software
- Upstream URL: <https://gitlab.synchro.net/main/sbbs>
- Upstream path: `src/odoors`
- Copied at commit: `47feab1e8bf776175b44f40dffebbc9560322e20`
- Earlier handoff audit commit:
  `aab6ab1aca4246a11ae83f3f7d74d6cefbce6fa9`
- Copy date: 2026-08-06
- License: GNU Lesser General Public License, version 2 or later, as stated
  in the source notices (`LGPL-2.0-or-later`)

The live upstream worktree had no local changes under `src/odoors` when
copied. The OpenDoors files at the copy commit are identical to the files at
the handoff audit commit. Cepheus Trader does not copy or compile `xpdev`:
only the excluded upstream `ex_*` examples use those support headers.

## Copied build surface

The copy contains the complete OpenDoors source and header surface compiled by
`client/CMakeLists.txt`, plus its license:

- sources: `ODAuto.c`, `ODBlock.c`, `ODCFile.c`, `ODCmdLn.c`, `ODCom.c`,
  `ODCore.c`, `ODDrBox.c`, `ODEdit.c`, `ODEdStr.c`, `ODEmu.c`, `ODGetIn.c`,
  `ODGraph.c`, `ODInEx1.c`, `ODInEx2.c`, `ODInQue.c`, `ODKrnl.c`, `ODList.c`,
  `ODLog.c`, `ODMulti.c`, `ODPlat.c`, `ODPCB.c`, `ODPopup.c`, `ODPrntf.c`,
  `ODRA.c`, `ODScrn.c`, `ODSpawn.c`, `ODStand.c`, `ODStat.c`, `ODStr.c`,
  `ODUtil.c`, `ODWCat.c`, `ODWin.c`, and Windows-only `ODFrame.c`;
- headers: `ODCom.h`, `ODCore.h`, `ODFrame.h`, `ODGen.h`, `ODInEx.h`,
  `ODInQue.h`, `ODKrnl.h`, `ODPlat.h`, `ODRes.h`, `ODScrn.h`, `ODStat.h`,
  `ODStr.h`, `ODSwap.h`, `ODTypes.h`, `ODUtil.h`, and `OpenDoor.h`; and
- license: `license.txt`.

Prebuilt DLLs, libraries, generated resource intermediates, historical notes,
examples, and obsolete upstream build scripts are not used and were not
copied. Cepheus Trader's CMake file is the build script for this source set.

## Local patches

There are no local patches to the vendored OpenDoors files at initial import.
Record every later source or header modification here with its date, affected
files, and purpose. Build-system integration outside this directory is not an
OpenDoors source patch.
