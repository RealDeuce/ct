#!/usr/bin/env python3
"""Compose the active Cepheus Trader ship-construction rules.

Construction modules remain separate source transcriptions on disk.  This
module performs the deterministic merge used by design evaluation; it never
derives components from published ship specifications.
"""

from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import tomllib
from typing import Any


class RuleCompositionError(ValueError):
    pass


def _load(path: Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise RuleCompositionError(f"{path}: {error}") from error


def _document_id(document: dict[str, Any], path: Path) -> str:
    document_id = document.get("ruleset_id") or document.get("module_id")
    if not isinstance(document_id, str) or not document_id:
        raise RuleCompositionError(f"{path}: missing ruleset_id or module_id")
    return document_id


def _record_id(record: dict[str, Any], kind: str) -> str:
    key = {
        "drive": "code",
        "drive_performance": "hull_tons",
        "hull_section": "hull_tons",
    }.get(kind, "id")
    record_id = record.get(key)
    if isinstance(record_id, int) and kind in {
        "drive_performance",
        "hull_section",
    }:
        return str(record_id)
    if not isinstance(record_id, str) or not record_id:
        raise RuleCompositionError(f"{kind} record lacks {key}")
    return record_id


def _merge_records(
    result: dict[str, Any],
    kind: str,
    additions: list[dict[str, Any]],
) -> None:
    existing = result.setdefault(kind, [])
    if not isinstance(existing, list):
        raise RuleCompositionError(f"cannot merge records into scalar {kind}")
    indexed: dict[str, int] = {}
    for position, record in enumerate(existing):
        if not isinstance(record, dict):
            raise RuleCompositionError(f"{kind}[{position}] is not a table")
        indexed[_record_id(record, kind)] = position

    for source_record in additions:
        if not isinstance(source_record, dict):
            raise RuleCompositionError(f"{kind} addition is not a table")
        record = deepcopy(source_record)
        record_id = _record_id(record, kind)
        superseded = record.pop("supersedes_id", None)
        if superseded is not None:
            if not isinstance(superseded, str) or superseded not in indexed:
                raise RuleCompositionError(
                    f"{kind} {record_id!r} supersedes unknown {superseded!r}"
                )
            removed_position = indexed.pop(superseded)
            existing.pop(removed_position)
            indexed = {
                _record_id(item, kind): position
                for position, item in enumerate(existing)
            }
        if record_id in indexed:
            raise RuleCompositionError(f"duplicate composed {kind} {record_id!r}")
        indexed[record_id] = len(existing)
        existing.append(record)


# These are active construction concepts.  Source-audit, blocked, and excluded
# records are deliberately absent.
ACTIVE_RECORD_KINDS = {
    "ammunition",
    "armor_extension",
    "barbette",
    "bay",
    "bridge_option",
    "cargo_option",
    "computer",
    "configuration",
    "docking_clamp",
    "drive_performance",
    "electronics",
    "electronics_upgrade",
    "equipment",
    "equipment_option",
    "hangar",
    "hull",
    "hull_option",
    "hull_section",
    "launch_facility",
    "livestock_hold",
    "magazine_option",
    "mount",
    "mount_option",
    "parameterized_equipment",
    "point_defense_mount",
    "point_defense_weapon",
    "power_option",
    "screen",
    "software",
    "spinal_weapon",
    "spinal_weapon_improvement",
    "structural_option",
    "weapon",
}


def compose_shipbuilding_rules(
    rules_dir: Path,
    policy_id: str = "cepheus-trader.shipbuilding",
) -> dict[str, Any]:
    documents: dict[str, tuple[Path, dict[str, Any]]] = {}
    for path in sorted(rules_dir.glob("*.toml")):
        document = _load(path)
        document_id = _document_id(document, path)
        if document_id in documents:
            raise RuleCompositionError(f"duplicate rules document {document_id!r}")
        documents[document_id] = (path, document)

    try:
        policy_path, policy = documents[policy_id]
    except KeyError as error:
        raise RuleCompositionError(f"unknown composition policy {policy_id!r}") from error

    large_ship_bases = [
        document_id
        for document_id in policy.get("extends_ruleset_ids", [])
        if document_id.endswith(".shipbuilding")
    ]
    if len(large_ship_bases) != 1:
        raise RuleCompositionError(
            f"{policy_path}: expected one large-ship base ruleset"
        )
    base_id = large_ship_bases[0]
    try:
        _, base = documents[base_id]
    except KeyError as error:
        raise RuleCompositionError(
            f"{policy_path}: unknown base ruleset {base_id!r}"
        ) from error

    result = deepcopy(base)
    result["ruleset_id"] = policy_id
    result["source_ids"] = deepcopy(policy.get("source_ids", []))
    result["composed_from"] = [base_id]

    for module_id in policy.get("extension_module_ids", []):
        try:
            module_path, module = documents[module_id]
        except KeyError as error:
            raise RuleCompositionError(
                f"{policy_path}: unknown extension module {module_id!r}"
            ) from error
        result["composed_from"].append(module_id)
        for kind in ACTIVE_RECORD_KINDS:
            additions = module.get(kind)
            if additions is None:
                continue
            if not isinstance(additions, list):
                raise RuleCompositionError(
                    f"{module_path}: active {kind} must be an array"
                )
            _merge_records(result, kind, additions)

        for extension in module.get("bridge_extension", []):
            if not isinstance(extension, dict):
                raise RuleCompositionError(
                    f"{module_path}: bridge_extension must contain tables"
                )
            sizes = extension.get("sizes")
            if not isinstance(sizes, list):
                raise RuleCompositionError(
                    f"{module_path}: bridge_extension lacks sizes"
                )
            bridge = result.get("bridge")
            if not isinstance(bridge, dict) or not isinstance(
                bridge.get("sizes"), list
            ):
                raise RuleCompositionError("base rules lack bridge sizes")
            bridge["sizes"].extend(deepcopy(sizes))
            bridge["sizes"].sort(key=lambda row: row["maximum_hull_tons"])

    return result
