#!/usr/bin/env python3
"""Reject private/runtime libraries from portable client executables."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


FORBIDDEN = (
    "odoors",
    "capnp",
    "libkj",
    "botan",
    "libstdc++",
    "libwinpthread",
)


def inspect(path: Path) -> str:
    if path.suffix.lower() in {".dll", ".exe"}:
        tool = next(
            (
                candidate
                for candidate in (
                    "x86_64-w64-mingw32-objdump",
                    "i686-w64-mingw32-objdump",
                    "objdump",
                )
                if shutil.which(candidate)
            ),
            None,
        )
        if tool is None:
            raise SystemExit("no PE dependency inspection tool is available")
        command = [tool, "-p", str(path)]
    elif sys.platform == "darwin":
        command = ["otool", "-L", str(path)]
    else:
        command = ["ldd", str(path)]
    result = subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE)
    if path.suffix.lower() in {".dll", ".exe"}:
        return "\n".join(
            line.strip() for line in result.stdout.splitlines() if "DLL Name:" in line
        )
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("executables", nargs="+", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    sections = []
    for executable in args.executables:
        output = inspect(executable)
        lowered = output.lower()
        forbidden = list(FORBIDDEN)
        # FreeBSD's system libc++ uses libgcc_s for the platform unwinder; it
        # is an operating-system library there, not a bundled GNU C++ runtime.
        if not sys.platform.startswith("freebsd"):
            forbidden.append("libgcc_s")
        leaked = [name for name in forbidden if name in lowered]
        if leaked:
            raise SystemExit(f"{executable}: private runtime dependency: {', '.join(leaked)}")
        sections.append(f"[{executable}]\n{output.rstrip()}\n")
    report = "\n".join(sections)
    if args.report:
        args.report.write_text(report, encoding="utf-8", newline="\n")
    else:
        print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
