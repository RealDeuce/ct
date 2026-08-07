#!/usr/bin/env python3
"""Inspect a CPack client archive and optionally run its version probes."""

from __future__ import annotations

import argparse
import os
import subprocess
import tarfile
import tempfile
import zipfile
from pathlib import Path


REQUIRED_BASENAMES = {
    "cepheus-trader-door",
    "cepheus-trader-sysop",
    "cepheus-trader.conf.example",
    "install-xtrn.ini",
    "README.md",
    "PLAYER-GUIDE.md",
    "SYSOP-GUIDE.md",
    "LICENSE.md",
    "OPEN_GAME_LICENSE.md",
    "THIRD_PARTY_LICENSES.md",
    "license.txt",
    "Botan-BSD-2-Clause.txt",
    "CapnProto-MIT.txt",
    "SOURCE-RELEASE.txt",
}
FORBIDDEN_PROGRAMS = {"cepheus-trader-admin", "cepheus-trader-client"}
CLIENT_CORE_NAMES = {
    "cepheus-trader-client-core.dll",
    "libcepheus-trader-client-core.dylib",
    "libcepheus-trader-client-core.so",
}


def normalize_program(name: str) -> str:
    return name[:-4] if name.lower().endswith(".exe") else name


def extract(archive: Path, destination: Path) -> None:
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as package:
            package.extractall(destination)
    else:
        with tarfile.open(archive, "r:*") as package:
            package.extractall(destination, filter="data")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("archive", type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--run", action="store_true", help="run native version probes")
    parser.add_argument("--require-source-url", action="store_true")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory(prefix="ct-package-") as temporary:
        root = Path(temporary)
        extract(args.archive, root)
        paths = [path for path in root.rglob("*") if path.is_file()]
        normalized = {normalize_program(path.name) for path in paths}
        missing = REQUIRED_BASENAMES - normalized
        forbidden = FORBIDDEN_PROGRAMS.intersection(normalized)
        if missing:
            raise SystemExit(f"package is missing: {', '.join(sorted(missing))}")
        if forbidden:
            raise SystemExit(f"package contains forbidden programs: {', '.join(sorted(forbidden))}")
        if not any(path.name.lower() in CLIENT_CORE_NAMES for path in paths):
            raise SystemExit("package is missing the shared client core library")

        source_release = next(path for path in paths if path.name == "SOURCE-RELEASE.txt")
        source_text = source_release.read_text(encoding="utf-8")
        if f"Product version: {args.version}" not in source_text:
            raise SystemExit("SOURCE-RELEASE.txt has the wrong product version")
        if args.require_source_url:
            line = next(
                (line for line in source_text.splitlines() if line.startswith("Source release:")),
                "",
            )
            if not line.removeprefix("Source release:").strip():
                raise SystemExit("SOURCE-RELEASE.txt has no tagged source URL")

        if args.run:
            for program in ("cepheus-trader-door", "cepheus-trader-sysop"):
                executable = next(
                    path for path in paths if normalize_program(path.name) == program
                )
                executable.chmod(executable.stat().st_mode | 0o111)
                result = subprocess.run(
                    [os.fspath(executable), "--version"],
                    check=True,
                    text=True,
                    stdout=subprocess.PIPE,
                )
                expected = f"{program} {args.version}"
                if result.stdout.strip() != expected:
                    raise SystemExit(
                        f"{program} version probe returned {result.stdout.strip()!r}, "
                        f"expected {expected!r}"
                    )
        print(f"client package check: OK ({args.archive})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
