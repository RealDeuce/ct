#!/usr/bin/env python3
"""Validate the active rule-derived ship catalog."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
import tomllib
from typing import Any

from compile_catalog_ogl import load_registry
from ship_design import DesignError, evaluate, load_toml
from shipbuilding_rules import RuleCompositionError, compose_shipbuilding_rules
from small_craft_design import evaluate_small_craft


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CATALOG = ROOT / "catalog" / "ships"
DEFAULT_RULES = ROOT / "catalog" / "shipbuilding"
DEFAULT_SOURCES = ROOT / "catalog" / "ogl-sources.toml"
TAG = re.compile(r"ship-([1-9][0-9]*)\Z")
FAMILY_TAG = re.compile(r"family-([1-9][0-9]*)\Z")
UPGRADE_PATH_TAG = re.compile(r"upgrade-path-([1-9][0-9]*)\Z")
PLACEHOLDER_NAME = re.compile(r"ship-[1-9][0-9]*\Z", re.IGNORECASE)
PLAYER_DESCRIPTION_META_LANGUAGE = re.compile(
    r"(?:\bCE\b|\bnormaliz(?:e|ed|ing)\b|\bstandard-CE\b|"
    r"\bconstruction-rule\b|\bconstruction engine\b|"
    r"\bsource (?:design|vessel|profile|role)\b|\bsource-specific\b|"
    r"\bpost-conversion\b)",
    re.IGNORECASE,
)


class CatalogValidationError(ValueError):
    pass


def _text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise CatalogValidationError(f"{label} must be non-empty text")
    return value


def _integer(value: object, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise CatalogValidationError(f"{label} must be an integer >= {minimum}")
    return value


def _text_list(
    value: object,
    label: str,
    *,
    minimum: int = 0,
    maximum: int | None = None,
) -> list[str]:
    if (
        not isinstance(value, list)
        or len(value) < minimum
        or (maximum is not None and len(value) > maximum)
        or any(not isinstance(item, str) or not item.strip() for item in value)
        or len(value) != len(set(value))
    ):
        extent = f"{minimum} or more"
        if maximum is not None:
            extent = f"{minimum} to {maximum}"
        raise CatalogValidationError(
            f"{label} must contain {extent} unique non-empty strings"
        )
    return value


def _catalog_metadata(
    path: Path,
    design: dict[str, Any],
    known_source_ids: set[str],
) -> tuple[int, str, int, int, str]:
    metadata = design.get("catalog")
    if not isinstance(metadata, dict):
        raise CatalogValidationError(f"{path}: missing [catalog]")
    allowed = {
        "catalog_id",
        "tag",
        "family_id",
        "upgrade_path_id",
        "progression_stage",
        "status",
        "vessel_kind",
        "display_name",
        "primary_role",
        "secondary_roles",
        "mission_tags",
        "open_game_content_designations",
        "description_paragraphs",
    }
    extra = set(metadata) - allowed
    if extra:
        raise CatalogValidationError(
            f"{path}: unknown catalog field(s): {', '.join(sorted(extra))}"
        )

    catalog_id = _integer(metadata.get("catalog_id"), f"{path}.catalog_id", 1)
    tag = _text(metadata.get("tag"), f"{path}.tag")
    match = TAG.fullmatch(tag)
    if match is None or int(match.group(1)) != catalog_id:
        raise CatalogValidationError(f"{path}: catalog_id and tag disagree")
    if path.name != f"{tag}.toml":
        raise CatalogValidationError(f"{path}: filename and tag disagree")
    family_id = _integer(metadata.get("family_id"), f"{path}.family_id", 1)
    upgrade_path_id = _integer(
        metadata.get("upgrade_path_id"),
        f"{path}.upgrade_path_id",
        1,
    )
    progression_stage = _text(
        metadata.get("progression_stage"),
        f"{path}.progression_stage",
    )
    if progression_stage not in {
        "auxiliary",
        "starter",
        "light",
        "medium",
        "heavy",
        "capital",
    }:
        raise CatalogValidationError(
            f"{path}: invalid progression_stage {progression_stage!r}"
        )
    if metadata.get("status") not in {"active", "draft", "retired"}:
        raise CatalogValidationError(f"{path}: invalid catalog status")
    vessel_kind = _text(metadata.get("vessel_kind"), f"{path}.vessel_kind")
    if vessel_kind not in {"small-craft", "ship", "starship"}:
        raise CatalogValidationError(f"{path}: invalid vessel_kind")
    _text(metadata.get("display_name"), f"{path}.display_name")
    _text(metadata.get("primary_role"), f"{path}.primary_role")
    _text_list(metadata.get("secondary_roles"), f"{path}.secondary_roles")
    _text_list(metadata.get("mission_tags"), f"{path}.mission_tags", minimum=1)
    _text_list(
        metadata.get("open_game_content_designations"),
        f"{path}.open_game_content_designations",
        minimum=1,
    )
    description_paragraphs = _text_list(
        metadata.get("description_paragraphs"),
        f"{path}.description_paragraphs",
        minimum=2,
        maximum=3,
    )
    description = " ".join(description_paragraphs)
    meta_language = PLAYER_DESCRIPTION_META_LANGUAGE.search(description)
    if meta_language is not None:
        raise CatalogValidationError(
            f"{path}.description_paragraphs contains player-facing "
            f"construction commentary {meta_language.group(0)!r}"
        )

    source_ids = set(
        _text_list(design.get("source_ids"), f"{path}.source_ids", minimum=1)
    )
    unknown_sources = source_ids - known_source_ids
    if unknown_sources:
        raise CatalogValidationError(
            f"{path}: unknown source IDs: {', '.join(sorted(unknown_sources))}"
        )
    return (
        catalog_id,
        vessel_kind,
        family_id,
        upgrade_path_id,
        progression_stage,
    )


def _family_membership(
    path: Path,
    known_ship_tags: set[str],
) -> tuple[dict[str, int], int, int]:
    try:
        registry = load_toml(path)
    except DesignError as error:
        raise CatalogValidationError(str(error)) from error
    allowed = {
        "schema_version",
        "catalog_revision",
        "family_count",
        "open_game_content_designations",
        "family",
    }
    extra = set(registry) - allowed
    if extra:
        raise CatalogValidationError(
            f"{path}: unknown field(s): {', '.join(sorted(extra))}"
        )
    if registry.get("schema_version") != 1:
        raise CatalogValidationError(f"{path}: schema_version must be 1")
    _integer(registry.get("catalog_revision"), f"{path}.catalog_revision", 1)
    _text_list(
        registry.get("open_game_content_designations"),
        f"{path}.open_game_content_designations",
        minimum=1,
    )
    records = registry.get("family")
    if not isinstance(records, list):
        raise CatalogValidationError(f"{path}: family must be an array")
    family_count = _integer(
        registry.get("family_count"),
        f"{path}.family_count",
        1,
    )
    if family_count != len(records):
        raise CatalogValidationError(f"{path}: family_count is stale")

    membership: dict[str, int] = {}
    seen_family_ids: set[int] = set()
    seen_family_tags: set[str] = set()
    shared = 0
    singleton = 0
    for position, record in enumerate(records, 1):
        label = f"{path}: family {position}"
        if not isinstance(record, dict):
            raise CatalogValidationError(f"{label} must be a table")
        if set(record) != {
            "family_id",
            "tag",
            "grouping_basis",
            "member_tags",
        }:
            raise CatalogValidationError(f"{label} has invalid fields")
        family_id = _integer(record.get("family_id"), f"{label}.family_id", 1)
        family_tag = _text(record.get("tag"), f"{label}.tag")
        match = FAMILY_TAG.fullmatch(family_tag)
        if match is None or int(match.group(1)) != family_id:
            raise CatalogValidationError(
                f"{label}: family_id and tag disagree"
            )
        if family_id in seen_family_ids or family_tag in seen_family_tags:
            raise CatalogValidationError(f"{label}: duplicate family identity")
        seen_family_ids.add(family_id)
        seen_family_tags.add(family_tag)

        basis = record.get("grouping_basis")
        if basis not in {"independent-design", "shared-lineage"}:
            raise CatalogValidationError(
                f"{label}: invalid grouping_basis {basis!r}"
            )
        member_tags = _text_list(
            record.get("member_tags"),
            f"{label}.member_tags",
            minimum=1,
        )
        if basis == "independent-design" and len(member_tags) != 1:
            raise CatalogValidationError(
                f"{label}: independent-design must have one member"
            )
        if basis == "shared-lineage" and len(member_tags) < 2:
            raise CatalogValidationError(
                f"{label}: shared-lineage must have multiple members"
            )
        member_ids: list[int] = []
        for member_tag in member_tags:
            member_match = TAG.fullmatch(member_tag)
            if member_match is None:
                raise CatalogValidationError(
                    f"{label}: invalid member tag {member_tag!r}"
                )
            member_ids.append(int(member_match.group(1)))
        if family_id != min(member_ids):
            raise CatalogValidationError(
                f"{label}: family ID must equal its lowest member ID"
            )
        unknown = set(member_tags) - known_ship_tags
        if unknown:
            raise CatalogValidationError(
                f"{label}: unknown members: {', '.join(sorted(unknown))}"
            )
        for member_tag in member_tags:
            previous = membership.get(member_tag)
            if previous is not None:
                raise CatalogValidationError(
                    f"{label}: {member_tag} is already in family-{previous}"
                )
            membership[member_tag] = family_id
        if len(member_tags) == 1:
            singleton += 1
        else:
            shared += 1

    missing = known_ship_tags - set(membership)
    if missing:
        raise CatalogValidationError(
            f"{path}: ships without families: {', '.join(sorted(missing))}"
        )
    return membership, shared, singleton


def _upgrade_path_membership(
    path: Path,
    known_ship_tags: set[str],
) -> dict[str, int]:
    try:
        registry = load_toml(path)
    except DesignError as error:
        raise CatalogValidationError(str(error)) from error
    allowed = {
        "schema_version",
        "catalog_revision",
        "path_count",
        "open_game_content_designations",
        "path",
    }
    extra = set(registry) - allowed
    if extra:
        raise CatalogValidationError(
            f"{path}: unknown field(s): {', '.join(sorted(extra))}"
        )
    if registry.get("schema_version") != 1:
        raise CatalogValidationError(f"{path}: schema_version must be 1")
    _integer(registry.get("catalog_revision"), f"{path}.catalog_revision", 1)
    _text_list(
        registry.get("open_game_content_designations"),
        f"{path}.open_game_content_designations",
        minimum=1,
    )
    records = registry.get("path")
    if not isinstance(records, list):
        raise CatalogValidationError(f"{path}: path must be an array")
    path_count = _integer(registry.get("path_count"), f"{path}.path_count", 1)
    if path_count != len(records):
        raise CatalogValidationError(f"{path}: path_count is stale")
    if path_count != 9:
        raise CatalogValidationError(f"{path}: exactly nine paths are required")

    expected_axes = {
        1: ("trade", "orderly"),
        2: ("trade", "contested"),
        3: ("trade", "chaotic"),
        4: ("mixed", "orderly"),
        5: ("mixed", "contested"),
        6: ("mixed", "chaotic"),
        7: ("combat", "orderly"),
        8: ("combat", "contested"),
        9: ("combat", "chaotic"),
    }
    membership: dict[str, int] = {}
    seen_path_ids: set[int] = set()
    for position, record in enumerate(records, 1):
        label = f"{path}: path {position}"
        if not isinstance(record, dict):
            raise CatalogValidationError(f"{label} must be a table")
        if set(record) != {
            "path_id",
            "tag",
            "trade_emphasis",
            "institutional_order",
            "specialty",
            "native_design_tags",
        }:
            raise CatalogValidationError(f"{label} has invalid fields")
        path_id = _integer(record.get("path_id"), f"{label}.path_id", 1)
        path_tag = _text(record.get("tag"), f"{label}.tag")
        match = UPGRADE_PATH_TAG.fullmatch(path_tag)
        if match is None or int(match.group(1)) != path_id:
            raise CatalogValidationError(f"{label}: path_id and tag disagree")
        if path_id in seen_path_ids:
            raise CatalogValidationError(f"{label}: duplicate path identity")
        seen_path_ids.add(path_id)
        trade_emphasis = record.get("trade_emphasis")
        institutional_order = record.get("institutional_order")
        if expected_axes.get(path_id) != (
            trade_emphasis,
            institutional_order,
        ):
            raise CatalogValidationError(
                f"{label}: path ID has incorrect polity axes"
            )
        _text(record.get("specialty"), f"{label}.specialty")
        design_tags = _text_list(
            record.get("native_design_tags"),
            f"{label}.native_design_tags",
            minimum=1,
        )
        design_ids: list[int] = []
        for design_tag in design_tags:
            design_match = TAG.fullmatch(design_tag)
            if design_match is None:
                raise CatalogValidationError(
                    f"{label}: invalid design tag {design_tag!r}"
                )
            design_ids.append(int(design_match.group(1)))
        if design_ids != sorted(design_ids):
            raise CatalogValidationError(
                f"{label}: native_design_tags must be in numeric order"
            )
        unknown = set(design_tags) - known_ship_tags
        if unknown:
            raise CatalogValidationError(
                f"{label}: unknown designs: {', '.join(sorted(unknown))}"
            )
        for design_tag in design_tags:
            previous = membership.get(design_tag)
            if previous is not None:
                raise CatalogValidationError(
                    f"{label}: {design_tag} is already in upgrade-path-{previous}"
                )
            membership[design_tag] = path_id

    if seen_path_ids != set(expected_axes):
        raise CatalogValidationError(f"{path}: path identities are incomplete")
    missing = known_ship_tags - set(membership)
    if missing:
        raise CatalogValidationError(
            f"{path}: ships without native paths: {', '.join(sorted(missing))}"
        )
    return membership


def _canonical_names(
    path: Path,
    known_ship_tags: set[str],
    known_family_ids: set[int],
    known_path_ids: set[int],
    ship_family_ids: dict[str, int],
) -> dict[str, str]:
    try:
        registry = load_toml(path)
    except DesignError as error:
        raise CatalogValidationError(str(error)) from error
    allowed = {
        "schema_version",
        "catalog_revision",
        "path_name_count",
        "family_name_count",
        "design_name_count",
        "open_game_content_designations",
        "path_name",
        "family_name",
        "design_name",
    }
    extra = set(registry) - allowed
    if extra:
        raise CatalogValidationError(
            f"{path}: unknown field(s): {', '.join(sorted(extra))}"
        )
    if registry.get("schema_version") != 1:
        raise CatalogValidationError(f"{path}: schema_version must be 1")
    _integer(registry.get("catalog_revision"), f"{path}.catalog_revision", 1)
    _text_list(
        registry.get("open_game_content_designations"),
        f"{path}.open_game_content_designations",
        minimum=1,
    )

    path_records = registry.get("path_name")
    if not isinstance(path_records, list):
        raise CatalogValidationError(f"{path}: path_name must be an array")
    if registry.get("path_name_count") != len(path_records):
        raise CatalogValidationError(f"{path}: path_name_count is stale")
    seen_path_ids: set[int] = set()
    seen_path_names: set[str] = set()
    seen_manufacturer_names: set[str] = set()
    for position, record in enumerate(path_records, 1):
        label = f"{path}: path_name {position}"
        if not isinstance(record, dict) or set(record) != {
            "path_id",
            "display_name",
            "manufacturer_name",
            "naming_sequence",
        }:
            raise CatalogValidationError(f"{label} has invalid fields")
        path_id = _integer(record.get("path_id"), f"{label}.path_id", 1)
        if path_id in seen_path_ids:
            raise CatalogValidationError(f"{label}: duplicate path ID")
        seen_path_ids.add(path_id)
        path_name = _text(
            record.get("display_name"),
            f"{label}.display_name",
        ).strip().casefold()
        manufacturer_name = _text(
            record.get("manufacturer_name"),
            f"{label}.manufacturer_name",
        ).strip().casefold()
        if path_name in seen_path_names:
            raise CatalogValidationError(f"{label}: duplicate path name")
        if manufacturer_name in seen_manufacturer_names:
            raise CatalogValidationError(
                f"{label}: duplicate manufacturer name"
            )
        seen_path_names.add(path_name)
        seen_manufacturer_names.add(manufacturer_name)
        _text_list(
            record.get("naming_sequence"),
            f"{label}.naming_sequence",
            minimum=6,
            maximum=6,
        )
    if seen_path_ids != known_path_ids:
        raise CatalogValidationError(f"{path}: path names are incomplete")

    family_records = registry.get("family_name")
    if not isinstance(family_records, list):
        raise CatalogValidationError(f"{path}: family_name must be an array")
    if registry.get("family_name_count") != len(family_records):
        raise CatalogValidationError(f"{path}: family_name_count is stale")
    seen_family_ids: set[int] = set()
    seen_family_names: set[str] = set()
    for position, record in enumerate(family_records, 1):
        label = f"{path}: family_name {position}"
        if not isinstance(record, dict) or set(record) != {
            "family_id",
            "display_name",
        }:
            raise CatalogValidationError(f"{label} has invalid fields")
        family_id = _integer(record.get("family_id"), f"{label}.family_id", 1)
        if family_id in seen_family_ids:
            raise CatalogValidationError(f"{label}: duplicate family ID")
        seen_family_ids.add(family_id)
        family_name = _text(
            record.get("display_name"),
            f"{label}.display_name",
        ).strip().casefold()
        if family_name in seen_family_names:
            raise CatalogValidationError(f"{label}: duplicate family name")
        seen_family_names.add(family_name)
    if seen_family_ids != known_family_ids:
        raise CatalogValidationError(f"{path}: family names are incomplete")

    design_records = registry.get("design_name")
    if not isinstance(design_records, list):
        raise CatalogValidationError(f"{path}: design_name must be an array")
    if registry.get("design_name_count") != len(design_records):
        raise CatalogValidationError(f"{path}: design_name_count is stale")
    design_names: dict[str, str] = {}
    design_name_families: dict[str, int] = {}
    for position, record in enumerate(design_records, 1):
        label = f"{path}: design_name {position}"
        if not isinstance(record, dict) or set(record) != {
            "tag",
            "display_name",
        }:
            raise CatalogValidationError(f"{label} has invalid fields")
        tag = _text(record.get("tag"), f"{label}.tag")
        if TAG.fullmatch(tag) is None:
            raise CatalogValidationError(f"{label}: invalid ship tag {tag!r}")
        if tag not in known_ship_tags:
            raise CatalogValidationError(f"{label}: unknown ship tag {tag!r}")
        if tag in design_names:
            raise CatalogValidationError(f"{label}: duplicate ship tag")
        display_name = _text(record.get("display_name"), f"{label}.display_name")
        if PLACEHOLDER_NAME.fullmatch(display_name):
            raise CatalogValidationError(
                f"{label}: placeholder design name is not canonical"
            )
        normalized_name = display_name.strip().casefold()
        previous_family = design_name_families.get(normalized_name)
        current_family = ship_family_ids[tag]
        if previous_family is not None and previous_family != current_family:
            raise CatalogValidationError(
                f"{label}: design name duplicates another family"
            )
        design_name_families[normalized_name] = current_family
        design_names[tag] = display_name
    if set(design_names) != known_ship_tags:
        raise CatalogValidationError(f"{path}: design names are incomplete")
    return design_names


def _expected_progression_stage(
    vessel_kind: str,
    hull_millitons: int,
) -> str:
    if vessel_kind == "small-craft":
        return "auxiliary"
    hull_tons = hull_millitons // 1000
    if hull_tons <= 400:
        return "starter"
    if hull_tons <= 999:
        return "light"
    if hull_tons <= 1999:
        return "medium"
    if hull_tons <= 4999:
        return "heavy"
    return "capital"


def validate(
    catalog_dir: Path,
    rules_dir: Path,
    sources_path: Path,
    runtime_index: Path | None = None,
) -> list[str]:
    _, sources, _, required_source_ids = load_registry(sources_path)
    known_source_ids = {source.source_id for source in sources}
    core = load_toml(rules_dir / "ce-core.toml")
    small = load_toml(rules_dir / "ce-small-craft.toml")
    extended_small = load_toml(rules_dir / "af3-small-compatible.toml")
    composed = compose_shipbuilding_rules(rules_dir)
    rulesets = {
        core["ruleset_id"]: core,
        composed["ruleset_id"]: composed,
    }

    paths = sorted(
        catalog_dir.glob("ship-*.toml"),
        key=lambda path: int(path.stem.split("-")[1]),
    )
    if not paths:
        raise CatalogValidationError("active ship catalog is empty")
    index_path = catalog_dir / "index.toml"
    try:
        index = load_toml(index_path)
    except DesignError as error:
        raise CatalogValidationError(str(error)) from error
    if index.get("schema_version") != 1:
        raise CatalogValidationError(f"{index_path}: schema_version must be 1")
    _integer(index.get("catalog_revision"), f"{index_path}.catalog_revision", 1)
    index_records = index.get("ship")
    if not isinstance(index_records, list):
        raise CatalogValidationError(f"{index_path}: ship must be an array")
    if index.get("entry_count") != len(paths) or len(index_records) != len(paths):
        raise CatalogValidationError(f"{index_path}: entry_count is stale")

    path_by_tag: dict[str, Path] = {}
    design_by_path: dict[Path, dict[str, Any]] = {}
    for path in paths:
        design = load_toml(path)
        design_by_path[path] = design
        metadata = design.get("catalog")
        if not isinstance(metadata, dict):
            raise CatalogValidationError(f"{path}: missing [catalog]")
        tag = _text(metadata.get("tag"), f"{path}.tag")
        if tag in path_by_tag:
            raise CatalogValidationError(f"{path}: duplicate catalog tag {tag!r}")
        path_by_tag[tag] = path
    family_membership, shared_families, singleton_families = _family_membership(
        catalog_dir / "families.toml",
        set(path_by_tag),
    )
    upgrade_path_membership = _upgrade_path_membership(
        catalog_dir / "upgrade-paths.toml",
        set(path_by_tag),
    )
    canonical_design_names = _canonical_names(
        catalog_dir / "names.toml",
        set(path_by_tag),
        set(family_membership.values()),
        set(upgrade_path_membership.values()),
        family_membership,
    )

    evaluation_paths: list[Path] = []
    visiting: set[Path] = set()
    visited: set[Path] = set()

    def schedule(path: Path) -> None:
        if path in visited:
            return
        if path in visiting:
            raise CatalogValidationError(
                f"{path}: cyclic carried-craft dependency"
            )
        visiting.add(path)
        for craft in design_by_path[path].get("carried_craft", []):
            if not isinstance(craft, dict):
                raise CatalogValidationError(
                    f"{path}: carried_craft must contain tables"
                )
            tag = _text(craft.get("tag"), f"{path}.carried_craft.tag")
            dependency = path_by_tag.get(tag)
            if dependency is None:
                raise CatalogValidationError(
                    f"{path}: carried craft {tag!r} has no catalog entry"
                )
            schedule(dependency)
        visiting.remove(path)
        visited.add(path)
        evaluation_paths.append(path)

    for path in paths:
        schedule(path)

    seen_ids: set[int] = set()
    evaluated: dict[str, dict[str, Any]] = {}
    active = 0
    summaries: list[str] = []
    for path in evaluation_paths:
        design = design_by_path[path]
        (
            catalog_id,
            vessel_kind,
            family_id,
            upgrade_path_id,
            progression_stage,
        ) = _catalog_metadata(path, design, known_source_ids)
        if catalog_id in seen_ids:
            raise CatalogValidationError(f"{path}: duplicate catalog ID")
        seen_ids.add(catalog_id)
        metadata = design["catalog"]
        expected_family_id = family_membership[metadata["tag"]]
        if family_id != expected_family_id:
            raise CatalogValidationError(
                f"{path}: family_id is {family_id}, expected "
                f"{expected_family_id}"
            )
        expected_upgrade_path_id = upgrade_path_membership[metadata["tag"]]
        if upgrade_path_id != expected_upgrade_path_id:
            raise CatalogValidationError(
                f"{path}: upgrade_path_id is {upgrade_path_id}, expected "
                f"{expected_upgrade_path_id}"
            )
        expected_display_name = canonical_design_names[metadata["tag"]]
        if metadata["display_name"] != expected_display_name:
            raise CatalogValidationError(
                f"{path}: display_name is {metadata['display_name']!r}, "
                f"expected {expected_display_name!r}"
            )
        if metadata["status"] != "active":
            continue
        active += 1
        missing_required = set(required_source_ids) - set(design["source_ids"])
        if missing_required:
            raise CatalogValidationError(
                f"{path}: missing required source IDs: "
                f"{', '.join(sorted(missing_required))}"
            )

        ruleset_id = design.get("ruleset_id")
        try:
            if ruleset_id == small["ruleset_id"]:
                result = evaluate_small_craft(core, small, design)
            elif ruleset_id == extended_small["ruleset_id"]:
                result = evaluate_small_craft(
                    core,
                    small,
                    design,
                    extension=extended_small,
                    component_rules=composed,
                )
            else:
                result = evaluate(
                    rulesets[ruleset_id],
                    design,
                    carried_craft_results=evaluated,
                )
        except KeyError as error:
            raise CatalogValidationError(
                f"{path}: unknown ruleset {ruleset_id!r}"
            ) from error
        except DesignError as error:
            raise CatalogValidationError(f"{path}: {error}") from error

        expected_kind = "starship" if result.get("jump_rating", 0) else "ship"
        if ruleset_id in {small["ruleset_id"], extended_small["ruleset_id"]}:
            expected_kind = "small-craft"
        if vessel_kind != expected_kind:
            raise CatalogValidationError(
                f"{path}: vessel_kind is {vessel_kind!r}, expected "
                f"{expected_kind!r}"
            )
        if vessel_kind == "starship" and result["provision_capacity_persons"] == 0:
            raise CatalogValidationError(
                f"{path}: active starship has no provisioned accommodation"
            )
        expected_progression_stage = _expected_progression_stage(
            vessel_kind,
            result["hull_millitons"],
        )
        if progression_stage != expected_progression_stage:
            raise CatalogValidationError(
                f"{path}: progression_stage is {progression_stage!r}, "
                f"expected {expected_progression_stage!r}"
            )
        fixed_hangar_capacity = {
            "cutter-hangar": 30000,
            "lifeboat-hangar": 20000,
            "pinnace-hangar": 40000,
            "ships-boat-hangar": 30000,
            "shuttle-hangar": 90000,
        }
        hangar_capacity = 0
        for equipment in design.get("equipment", []):
            hangar_capacity += (
                fixed_hangar_capacity.get(equipment.get("id"), 0)
                * equipment.get("quantity", 0)
            )
        for equipment in design.get("parameterized_equipment", []):
            if equipment.get("id") == "custom-hangar":
                hangar_capacity += equipment.get("contained_millitons", 0)
        for hangar in design.get("hangars", []):
            hangar_capacity += (
                hangar.get("contained_millitons", 0)
                * hangar.get("quantity", 1)
            )
        docking_capacity = 0
        fixed_docking_capacity = {
            "docking-clamp-30": 30000,
            "docking-clamp-90": 90000,
            "docking-clamp-300": 300000,
            "docking-clamp-2000": 2000000,
        }
        for clamp in design.get("docking_clamps", []):
            docking_capacity += (
                fixed_docking_capacity.get(clamp.get("id"), 0)
                * clamp.get("quantity", 1)
            )
        carried_volume = 0
        for craft in design.get("carried_craft", []):
            carried_volume += (
                evaluated[craft["tag"]]["hull_millitons"] * craft["quantity"]
            )
        if carried_volume > hangar_capacity + docking_capacity:
            raise CatalogValidationError(
                f"{path}: carried craft displace {carried_volume} millitons "
                f"but hangars and clamps hold only "
                f"{hangar_capacity + docking_capacity}"
            )
        evaluated[metadata["tag"]] = result
        summaries.append(
            f"{metadata['tag']} {result['hull_millitons']} "
            f"{result['construction_price_credits']}"
        )

    indexed = []
    for position, record in enumerate(index_records, 1):
        if not isinstance(record, dict):
            raise CatalogValidationError(
                f"{index_path}: ship {position} must be a table"
            )
        if set(record) != {"catalog_id", "tag", "file", "status"}:
            raise CatalogValidationError(
                f"{index_path}: ship {position} has invalid fields"
            )
        indexed.append(
            (
                _integer(record.get("catalog_id"), "index catalog_id", 1),
                _text(record.get("tag"), "index tag"),
                _text(record.get("file"), "index file"),
                _text(record.get("status"), "index status"),
            )
        )
    expected_index = [
        (
            design["catalog"]["catalog_id"],
            design["catalog"]["tag"],
            path.name,
            design["catalog"]["status"],
        )
        for path in paths
        for design in [load_toml(path)]
    ]
    if indexed != expected_index:
        raise CatalogValidationError(f"{index_path}: records are stale or reordered")
    if index.get("active_count") != active:
        raise CatalogValidationError(f"{index_path}: active_count is stale")

    if runtime_index is not None:
        records: list[tuple[int, dict[str, Any], dict[str, Any]]] = []
        for path in paths:
            design = design_by_path[path]
            metadata = design["catalog"]
            if metadata["status"] != "active" or metadata["vessel_kind"] != "starship":
                continue
            records.append(
                (metadata["catalog_id"], design, evaluated[metadata["tag"]])
            )
        records.sort(key=lambda record: record[0])
        lines = [
            "# Validated runtime projection of the authoritative ship designs.",
            "# Regenerate with tools/validate_ship_catalog.py --runtime-index catalog/ship-runtime.toml.",
            "schema_version = 2",
            f"catalog_revision = {index['catalog_revision']}",
            "",
        ]
        for catalog_id, design, result in records:
            metadata = design["catalog"]
            lines.extend(
                [
                    "[[ship]]",
                    f"catalog_id = {catalog_id}",
                    f"display_name = {json.dumps(metadata['display_name'])}",
                    f"tech_level = {design['tech_level']}",
                    f"construction_price_credits = {result['construction_price_credits']}",
                    f"displacement_millitons = {result['hull_millitons']}",
                    f"jump_rating = {result['jump_rating']}",
                    f"thrust_g = {result['thrust_g']}",
                    f"fuel_millitons = {result['fuel_millitons']}",
                    f"jump_fuel_millitons = {result['jump_fuel_millitons']}",
                    f"cargo_millitons = {design.get('cargo_millitons', 0)}",
                    f"minimum_crew = {result['minimum_crew']}",
                    f"crew_accommodation_capacity = {result['crew_accommodation_capacity']}",
                    f"passenger_accommodation_berths = {result['passenger_accommodation_berths']}",
                    f"provision_capacity_persons = {result['provision_capacity_persons']}",
                    f"low_berths = {result['low_berths']}",
                    f"monthly_life_support_credits = {result['monthly_life_support_credits']}",
                    "",
                ]
            )
        for catalog_id, _design, result in records:
            for item in result["line_items"]:
                lines.extend(
                    [
                        "[[component]]",
                        f"catalog_id = {catalog_id}",
                        f"kind = {json.dumps(item['kind'])}",
                        f"component_id = {json.dumps(item['id'])}",
                        f"quantity = {item.get('quantity', 1)}",
                        f"displacement_millitons = {item['displacement_millitons']}",
                        f"price_credits = {item['price_credits']}",
                        f"pack_units = {item.get('pack_units', 1)}",
                        "",
                    ]
                )
        runtime_index.write_text("\n".join(lines), encoding="utf-8")

    return [
        f"validated {len(paths)} rule-derived ship catalog entries "
        f"({active} active) in {shared_families + singleton_families} "
        f"families ({shared_families} shared lineages, "
        f"{singleton_families} singleton designs) across 9 upgrade paths "
        f"with canonical names",
        *summaries,
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    parser.add_argument("--rules", type=Path, default=DEFAULT_RULES)
    parser.add_argument("--sources", type=Path, default=DEFAULT_SOURCES)
    parser.add_argument("--runtime-index", type=Path)
    args = parser.parse_args()
    try:
        messages = validate(args.catalog, args.rules, args.sources, args.runtime_index)
    except (
        CatalogValidationError,
        DesignError,
        RuleCompositionError,
        OSError,
        tomllib.TOMLDecodeError,
    ) as error:
        print(f"ship catalog error: {error}", file=sys.stderr)
        return 1
    print(messages[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
