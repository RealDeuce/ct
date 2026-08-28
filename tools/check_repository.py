#!/usr/bin/env python3
"""Validate standalone-repository, version, license, and hygiene invariants."""

from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
def tracked_files() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    names = [name for name in result.stdout.split(b"\0") if name]
    if names:
        return [Path(name.decode("utf-8")) for name in names]
    # The validator must also work before the standalone repository's initial
    # commit. `rg --files` observes .gitignore and avoids build output.
    result = subprocess.run(
        ["rg", "--files", "-0"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    return [Path(name.decode("utf-8")) for name in result.stdout.split(b"\0") if name]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def require(pattern: str, relative: str, description: str) -> None:
    if re.search(pattern, read(relative), re.MULTILINE) is None:
        raise ValueError(f"{relative}: missing {description}")


def validate_release_notes(version: str, text: str) -> list[str]:
    """Return missing requirements for one versioned GitHub release body."""
    errors: list[str] = []
    if f"v{version}" not in text:
        errors.append(f"release version v{version}")
    if re.search(r"(?m)^## Compatibility notice\s*$", text) is None:
        errors.append("Compatibility notice section")
    if re.search(r"(?m)^## Highlights\s*$", text) is None:
        errors.append("Highlights section")
    if re.search(r"(?ms)^## Highlights\s*\n\s*-\s+\S", text) is None:
        errors.append("at least one curated highlight")
    if re.search(
        r"(?m)^\*\*Full changelog:\*\* "
        r"https://github\.com/RealDeuce/ct/compare/\S+\.\.\.v"
        + re.escape(version)
        + r"\s*$",
        text,
    ) is None:
        errors.append("version-matched full changelog link")
    return errors


def main() -> int:
    errors: list[str] = []
    files = tracked_files()

    if (ROOT / "REPOSITORY_MIGRATION_HANDOFF.md").exists():
        errors.append("completed repository-migration handoff document is still present")
    for workflow in (
        ".github/workflows/ci.yml",
        ".github/workflows/benchmarks.yml",
    ):
        if not (ROOT / workflow).is_file():
            errors.append(f"missing GitHub Actions workflow: {workflow}")
    if (ROOT / ".gitlab-ci.yml").exists():
        errors.append("GitLab CI configuration remains in this GitHub repository")

    secret_names = re.compile(
        r"(^|/)(admin\.psk|data\.mdb|lock\.mdb)$|"
        r"\.(credential|identity|key|p12|pfx|pem|psk|sig)$",
        re.IGNORECASE,
    )
    generated_parts = {"build", "target", "server-data", "__pycache__", "CMakeFiles"}
    generated_suffixes = {".o", ".obj", ".pyc", ".pyo"}
    for path in files:
        portable = path.as_posix()
        if secret_names.search(portable):
            errors.append(f"tracked secret or signing material: {portable}")
        if generated_parts.intersection(path.parts) or path.suffix.lower() in generated_suffixes:
            errors.append(f"tracked generated/runtime output: {portable}")

    try:
        cargo_version = tomllib.loads(read("server/Cargo.toml"))["package"]["version"]
        cmake_match = re.search(
            r"project\(cepheus_trader_client VERSION ([0-9]+\.[0-9]+\.[0-9]+)",
            read("client/CMakeLists.txt"),
        )
        if cmake_match is None:
            raise ValueError("client/CMakeLists.txt: product version is not declared")
        cmake_version = cmake_match.group(1)
        if cargo_version != cmake_version:
            errors.append(
                f"product version mismatch: Cargo {cargo_version}, CMake {cmake_version}"
            )
        lock = tomllib.loads(read("server/Cargo.lock"))
        locked_package = next(
            (package for package in lock["package"] if package["name"] == "cepheus-trader-server"),
            None,
        )
        if locked_package is None or locked_package["version"] != cargo_version:
            errors.append("server/Cargo.lock does not carry the Cargo package version")
        workflow_versions = read(".github/workflows/ci.yml")
        if f"--version {cargo_version}" not in workflow_versions:
            errors.append("GitHub package checks do not use the common product version")
        release_notes_relative = f"docs/releases/v{cargo_version}.md"
        try:
            release_notes = read(release_notes_relative)
        except OSError:
            errors.append(f"missing curated release notes: {release_notes_relative}")
        else:
            for requirement in validate_release_notes(cargo_version, release_notes):
                errors.append(f"{release_notes_relative}: missing {requirement}")
        if 'docs/releases/${RELEASE_TAG}.md' not in workflow_versions:
            errors.append("GitHub release publishing does not select versioned release notes")
        if '--notes-file "$RELEASE_NOTES"' not in workflow_versions:
            errors.append("GitHub release publishing does not use curated release notes")
    except (KeyError, OSError, tomllib.TOMLDecodeError, ValueError) as error:
        errors.append(str(error))

    compatibility_checks = [
        ("server/src/wire.rs", r"pub const PROTOCOL_VERSION: u16 = 10;", "CT-RPC version 10"),
        ("server/src/admin_wire.rs", r"pub const PROTOCOL_VERSION: u16 = 2;", "admin protocol version 2"),
        ("server/src/sysop_wire.rs", r"pub const PROTOCOL_VERSION: u16 = 2;", "sysop protocol version 2"),
        ("server/src/league_wire.rs", r"pub const PROTOCOL_VERSION: u16 = 1;", "league protocol version 1"),
        ("server/src/store.rs", r"pub const STORAGE_FORMAT_VERSION: u64 = 2;", "storage format version 2"),
        ("server/src/store.rs", r"const SHIP_RECORD_CODEC_VERSION: u8 = 4;", "ship record codec version 4"),
        ("server/src/store.rs", r"const CNS5_COVERAGE_DISTRIBUTION_VERSION: u16 = 1;", "CNS5 coverage distribution version 1"),
        ("server/src/store.rs", r"const CNS5_COVERAGE_SAMPLER_VERSION: u16 = 1;", "CNS5 coverage sampler version 1"),
        ("server/src/store.rs", r"const SETTLEMENT_CAPACITY_SAMPLER_VERSION: u16 = 1;", "settlement capacity sampler version 1"),
        ("server/src/store.rs", r"const FRONTIER_ARRIVAL_SAMPLER_VERSION: u16 = 1;", "frontier arrival sampler version 1"),
        ("server/src/clock.rs", r"pub const CLOCK_FORMAT_VERSION: u64 = 1;", "clock format version 1"),
        ("server/src/universe.rs", r"pub const INITIAL_GENERATION_VERSION: u16 = 1;", "initial generation version 1"),
        ("server/src/celestial.rs", r"pub const CELESTIAL_GENERATION_VERSION: u16 = 1;", "celestial generation version 1"),
        ("server/src/bbs_polity.rs", r"pub const BBS_POLITY_GENERATION_VERSION: u16 = 4;", "BBS polity generation version 4"),
        ("server/src/bbs_polity.rs", r"pub const BBS_COVERAGE_SAMPLER_VERSION: u16 = 1;", "BBS coverage sampler version 1"),
        ("server/src/creation.rs", r"pub const SETUP_REVISION: u64 = 1;", "setup revision 1"),
        ("client/src/protocol.cpp", r"constexpr uint16_t PROTOCOL_VERSION = 10;", "CT-RPC version 10"),
        ("client/src/admin_protocol.cpp", r"constexpr uint16_t PROTOCOL_VERSION = 2;", "admin protocol version 2"),
        ("client/src/sysop_protocol.cpp", r"constexpr uint16_t PROTOCOL_VERSION = 2;", "sysop protocol version 2"),
        ("client/src/league_protocol.cpp", r"constexpr uint16_t PROTOCOL_VERSION = 1;", "league protocol version 1"),
    ]
    for relative, pattern, description in compatibility_checks:
        try:
            require(pattern, relative, description)
        except (OSError, ValueError) as error:
            errors.append(str(error))

    semantic_text_patterns = (
        r"\.role\s*==\s*\"pilot\"",
        r"\.shore_location\s*[!=]=\s*\"Aboard ship\"",
        r"\.source\.find\(",
        r"\.station\.find\(",
        r"\.station\s*==\s*\"captain\"",
    )
    door_source = read("client/src/door_main.cpp")
    for pattern in semantic_text_patterns:
        if re.search(pattern, door_source):
            errors.append(
                "client/src/door_main.cpp: server display text is used as a semantic discriminator"
            )

    cmake_text = read("client/CMakeLists.txt")
    forbidden_build_references = ("../../../odoors", "/synchronet", "src/xpdev")
    for reference in forbidden_build_references:
        if reference in cmake_text:
            errors.append(f"client/CMakeLists.txt: forbidden parent-tree reference {reference!r}")

    vendor = ROOT / "client/third_party/opendoors"
    if not (vendor / "license.txt").is_file() or not (vendor / "README.md").is_file():
        errors.append("vendored OpenDoors license or provenance document is missing")
    for resource in ("ODRes.rc", "ODApp.ico", "ODInfo.ico", "Toolbar.bmp"):
        if not (vendor / resource).is_file():
            errors.append(f"vendored OpenDoors Windows resource is missing: {resource}")
    vendor_names = {path.name for path in vendor.iterdir()} if vendor.is_dir() else set()
    if any(name.lower().startswith("ex_") for name in vendor_names):
        errors.append("vendored OpenDoors contains an excluded ex_* example")
    if any("xpdev" in name.lower() for name in vendor_names):
        errors.append("vendored OpenDoors contains xpdev material used only by examples")
    provenance = read("client/third_party/opendoors/README.md")
    for required in (
        "https://github.com/RealDeuce/OpenDoors",
        "3edf9008a6df2a7d71674f8b43e307d1fc2f721d",
        "https://gitlab.synchro.net/main/sbbs",
        "47feab1e8bf776175b44f40dffebbc9560322e20",
        "aab6ab1aca4246a11ae83f3f7d74d6cefbce6fa9",
        "LGPL-2.0-or-later",
    ):
        if required not in provenance:
            errors.append(f"OpenDoors provenance is missing {required}")

    for relative in (
        "client/third_party/licenses/Botan-BSD-2-Clause.txt",
        "client/third_party/licenses/CapnProto-MIT.txt",
    ):
        try:
            if len(read(relative).splitlines()) < 15:
                errors.append(f"{relative}: license text appears incomplete")
        except OSError as error:
            errors.append(str(error))

    if errors:
        for error in errors:
            print(f"repository check: {error}", file=sys.stderr)
        return 1
    print(f"repository check: OK ({len(files)} source files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
