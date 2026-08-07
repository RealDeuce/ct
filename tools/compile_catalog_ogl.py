#!/usr/bin/env python3
"""Validate catalog OGL references and compile their Section 15 notices."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]


class CatalogLicenseError(ValueError):
    pass


@dataclass(frozen=True)
class Notice:
    notice_id: str
    texts: tuple[str, ...]


@dataclass(frozen=True)
class Source:
    source_id: str
    notice_ids: tuple[str, ...]


def string_list(value: object, label: str, *, allow_empty: bool = True) -> list[str]:
    if not isinstance(value, list) or any(
        not isinstance(item, str) or not item.strip() for item in value
    ):
        raise CatalogLicenseError(f"{label} must be a list of non-empty strings")
    if not allow_empty and not value:
        raise CatalogLicenseError(f"{label} must not be empty")
    if len(value) != len(set(value)):
        raise CatalogLicenseError(f"{label} contains a duplicate")
    return value


def load_registry(
    path: Path,
) -> tuple[list[Notice], list[Source], list[str], list[str]]:
    with path.open("rb") as source_file:
        data = tomllib.load(source_file)
    if data.get("schema_version") != 1:
        raise CatalogLicenseError(f"{path}: unsupported schema version")

    catalog_ids = string_list(
        data.get("catalog_source_ids"), "catalog_source_ids", allow_empty=False
    )
    required_ids = string_list(
        data.get("required_entry_source_ids"),
        "required_entry_source_ids",
        allow_empty=False,
    )

    notices: list[Notice] = []
    seen_notice_ids: set[str] = set()
    text_owner: dict[str, str] = {}
    for index, record in enumerate(data.get("notice", []), start=1):
        label = f"notice record {index}"
        if not isinstance(record, dict):
            raise CatalogLicenseError(f"{label} must be a table")
        notice_id = record.get("notice_id")
        if not isinstance(notice_id, str) or not notice_id:
            raise CatalogLicenseError(f"{label} has no notice_id")
        if notice_id in seen_notice_ids:
            raise CatalogLicenseError(f"duplicate notice_id {notice_id!r}")
        seen_notice_ids.add(notice_id)

        title = record.get("title")
        if not isinstance(title, str) or not title.strip():
            raise CatalogLicenseError(f"{notice_id}: title must be non-empty")
        for field in (
            "source_descriptions",
            "open_game_content_descriptions",
            "excluded_product_identity_descriptions",
        ):
            string_list(record.get(field), f"{notice_id}.{field}")
        texts = tuple(
            string_list(
                record.get("texts"),
                f"{notice_id}.texts",
                allow_empty=False,
            )
        )
        for text in texts:
            previous = text_owner.get(text)
            if previous is not None:
                raise CatalogLicenseError(
                    f"{notice_id} repeats text owned by {previous}"
                )
            text_owner[text] = notice_id
        notices.append(Notice(notice_id, texts))

    sources: list[Source] = []
    seen_source_ids: set[str] = set()
    for index, record in enumerate(data.get("source", []), start=1):
        label = f"source record {index}"
        if not isinstance(record, dict):
            raise CatalogLicenseError(f"{label} must be a table")
        source_id = record.get("source_id")
        if not isinstance(source_id, str) or not source_id:
            raise CatalogLicenseError(f"{label} has no source_id")
        if source_id in seen_source_ids:
            raise CatalogLicenseError(f"duplicate source_id {source_id!r}")
        seen_source_ids.add(source_id)
        title = record.get("title")
        if not isinstance(title, str) or not title.strip():
            raise CatalogLicenseError(f"{source_id}: title must be non-empty")
        for field in (
            "source_descriptions",
            "open_game_content_descriptions",
            "excluded_product_identity_descriptions",
        ):
            string_list(record.get(field), f"{source_id}.{field}")
        notice_ids = tuple(
            string_list(
                record.get("notice_ids"),
                f"{source_id}.notice_ids",
                allow_empty=False,
            )
        )
        unknown_notices = sorted(set(notice_ids) - seen_notice_ids)
        if unknown_notices:
            raise CatalogLicenseError(
                f"{source_id} references unknown notice IDs: "
                + ", ".join(unknown_notices)
            )
        sources.append(Source(source_id, notice_ids))

    for source_id in catalog_ids + required_ids:
        if source_id not in seen_source_ids:
            raise CatalogLicenseError(f"unknown registry source ID {source_id!r}")
    return notices, sources, catalog_ids, required_ids


def catalog_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(path.glob("*.toml"))


def entry_source_ids(path: Path, *, include_drafts: bool) -> list[str]:
    with path.open("rb") as entry_file:
        entry = tomllib.load(entry_file)
    if "design_id" not in entry:
        return []
    metadata = entry.get("catalog")
    if not isinstance(metadata, dict):
        raise CatalogLicenseError(f"{path}: missing [catalog]")
    if not include_drafts and metadata.get("status") != "active":
        return []
    string_list(
        metadata.get("open_game_content_designations"),
        f"{path}.catalog.open_game_content_designations",
        allow_empty=False,
    )
    source_ids = string_list(
        entry.get("source_ids"),
        f"{path}.source_ids",
        allow_empty=False,
    )
    return source_ids


def compile_notices(
    registry_path: Path, catalog_path: Path, *, include_drafts: bool
) -> str:
    notices, sources, catalog_ids, required_ids = load_registry(registry_path)
    selected_sources = set(catalog_ids)
    for path in catalog_files(catalog_path):
        source_ids = entry_source_ids(path, include_drafts=include_drafts)
        for required_id in required_ids:
            if source_ids and required_id not in source_ids:
                raise CatalogLicenseError(
                    f"{path}: source_ids omits required {required_id!r}"
                )
        selected_sources.update(source_ids)

    known_ids = {source.source_id for source in sources}
    unknown = sorted(selected_sources - known_ids)
    if unknown:
        raise CatalogLicenseError(
            "catalog references unknown source IDs: " + ", ".join(unknown)
        )

    selected_notice_ids: set[str] = set()
    for source in sources:
        if source.source_id in selected_sources:
            selected_notice_ids.update(source.notice_ids)

    output_texts: list[str] = []
    for notice in notices:
        if notice.notice_id in selected_notice_ids:
            output_texts.extend(notice.texts)

    return "15. COPYRIGHT NOTICE\n\n" + "\n\n".join(output_texts) + "\n"


def replace_section15(license_text: str, section15: str) -> str:
    marker = "15. COPYRIGHT NOTICE"
    ending = "END OF LICENSE"
    start = license_text.find(marker)
    if start < 0:
        raise CatalogLicenseError("license has no Section 15 marker")
    end = license_text.find(ending, start)
    if end < 0:
        raise CatalogLicenseError("license has no END OF LICENSE marker")
    if license_text.find(marker, start + len(marker), end) >= 0:
        raise CatalogLicenseError("license has more than one Section 15 marker")
    return license_text[:start] + section15.rstrip() + "\n\n" + license_text[end:]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--registry", type=Path, default=ROOT / "catalog" / "ogl-sources.toml"
    )
    parser.add_argument(
        "--catalog", type=Path, default=ROOT / "catalog" / "ships"
    )
    parser.add_argument(
        "--include-drafts",
        action="store_true",
        help="validate and include draft entries as well as active entries",
    )
    destination = parser.add_mutually_exclusive_group()
    destination.add_argument("--output", type=Path)
    destination.add_argument(
        "--update-license",
        type=Path,
        help="replace Section 15 in a complete OGL file in place",
    )
    destination.add_argument(
        "--check",
        action="store_true",
        help="verify that OPEN_GAME_LICENSE.md has the compiled Section 15",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        output = compile_notices(
            args.registry, args.catalog, include_drafts=args.include_drafts
        )
    except (CatalogLicenseError, OSError, tomllib.TOMLDecodeError) as error:
        print(f"catalog OGL error: {error}", file=sys.stderr)
        return 1
    if args.check:
        license_path = ROOT / "OPEN_GAME_LICENSE.md"
        try:
            original = license_path.read_text(encoding="utf-8")
            expected = replace_section15(original, output)
        except (CatalogLicenseError, OSError) as error:
            print(f"catalog OGL error: {error}", file=sys.stderr)
            return 1
        if original != expected:
            print(
                "catalog OGL error: OPEN_GAME_LICENSE.md Section 15 is stale",
                file=sys.stderr,
            )
            return 1
    elif args.update_license:
        try:
            original = args.update_license.read_text(encoding="utf-8")
            updated = replace_section15(original, output)
            args.update_license.write_text(updated, encoding="utf-8")
        except (CatalogLicenseError, OSError) as error:
            print(f"catalog OGL error: {error}", file=sys.stderr)
            return 1
    elif args.output:
        args.output.write_text(output, encoding="utf-8")
    else:
        print(output, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
