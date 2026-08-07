#!/usr/bin/env python3
"""Create a debug archive and SHA-256 manifest for release artifacts."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("build", type=Path)
    parser.add_argument("artifacts", type=Path)
    parser.add_argument("--configuration", default="")
    parser.add_argument("--label", default="native")
    args = parser.parse_args()

    binary_root = args.build / args.configuration if args.configuration else args.build
    binaries = []
    for name in ("cepheus-trader-door", "cepheus-trader-sysop"):
        for suffix in ("", ".exe"):
            path = binary_root / f"{name}{suffix}"
            if path.is_file():
                binaries.append(path)
    if len(binaries) < 2:
        raise SystemExit(f"could not find release binaries under {binary_root}")

    args.artifacts.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="ct-debug-") as temporary:
        debug_root = Path(temporary)
        debug_files: list[Path] = []
        for binary in binaries:
            if sys.platform == "darwin" and shutil.which("dsymutil"):
                output = debug_root / f"{binary.name}.dSYM"
                subprocess.run(["dsymutil", str(binary), "-o", str(output)], check=True)
                debug_files.append(output)
                continue
            if binary.suffix.lower() == ".exe":
                tools = (
                    "x86_64-w64-mingw32-objcopy",
                    "i686-w64-mingw32-objcopy",
                    "objcopy",
                )
            else:
                tools = ("objcopy", "llvm-objcopy")
            objcopy = next((tool for tool in tools if shutil.which(tool)), None)
            output = debug_root / f"{binary.name}.debug"
            if objcopy:
                subprocess.run(
                    [objcopy, "--only-keep-debug", str(binary), str(output)], check=True
                )
            else:
                output = debug_root / f"{binary.name}.unstripped"
                shutil.copy2(binary, output)
            debug_files.append(output)
        for pdb in binary_root.glob("*.pdb"):
            copied = debug_root / pdb.name
            shutil.copy2(pdb, copied)
            debug_files.append(copied)

        if any(path.suffix.lower() == ".exe" for path in binaries):
            debug_archive = (
                args.artifacts / f"cepheus-trader-client-{args.label}-debug-symbols.zip"
            )
            with zipfile.ZipFile(
                debug_archive, "w", compression=zipfile.ZIP_DEFLATED
            ) as archive:
                for path in debug_files:
                    if path.is_dir():
                        for nested in path.rglob("*"):
                            if nested.is_file():
                                archive.write(nested, arcname=nested.relative_to(debug_root))
                    else:
                        archive.write(path, arcname=path.relative_to(debug_root))
        else:
            debug_archive = (
                args.artifacts / f"cepheus-trader-client-{args.label}-debug-symbols.tar.xz"
            )
            with tarfile.open(debug_archive, "w:xz") as archive:
                for path in debug_files:
                    archive.add(path, arcname=path.relative_to(debug_root))

    checksum = args.artifacts / f"SHA256SUMS-{args.label}"
    artifact_files = sorted(
        path for path in args.artifacts.iterdir() if path.is_file() and path != checksum
    )
    with checksum.open("w", encoding="utf-8", newline="\n") as output:
        for path in artifact_files:
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            output.write(f"{digest}  {path.name}\n")
    print(f"release artifacts: OK ({len(artifact_files)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
