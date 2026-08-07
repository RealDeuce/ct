#!/usr/bin/env python3
"""Validate the composed, rule-derived ship-construction catalog."""

from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path
import sys
import tomllib

from shipbuilding_rules import RuleCompositionError, compose_shipbuilding_rules


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RULES = ROOT / "catalog" / "shipbuilding"
DEFAULT_SOURCES = ROOT / "catalog" / "ogl-sources.toml"


class ValidationError(Exception):
    pass


def load(path: Path) -> dict:
    with path.open("rb") as stream:
        return tomllib.load(stream)


def validate(rules_dir: Path, sources_path: Path) -> list[str]:
    errors: list[str] = []
    documents: dict[str, tuple[Path, dict]] = {}

    source_registry = load(sources_path)
    source_ids = {entry["source_id"] for entry in source_registry.get("source", [])}

    for path in sorted(rules_dir.glob("*.toml")):
        data = load(path)
        document_id = data.get("ruleset_id") or data.get("module_id")
        if not isinstance(document_id, str):
            errors.append(f"{path}: missing ruleset_id or module_id")
            continue
        if document_id in documents:
            errors.append(f"{path}: duplicate document id {document_id!r}")
        documents[document_id] = (path, data)
        for source_id in data.get("source_ids", []):
            if source_id not in source_ids:
                errors.append(f"{path}: unknown source_id {source_id!r}")

    for document_id, (path, data) in documents.items():
        references: list[str] = []
        one = data.get("extends_ruleset_id")
        if one:
            references.append(one)
        references.extend(data.get("extends_ruleset_ids", []))
        extension = data.get("extends_module_id")
        if extension:
            references.append(extension)
        references.extend(data.get("extension_module_ids", []))
        for reference in references:
            if reference not in documents:
                errors.append(f"{path}: unknown rules document {reference!r}")

    # Rule IDs are unique within a construction concept. A configuration and
    # an electronics suite may both be called "standard"; two equipment rules
    # with that ID may not.
    ids_by_kind: dict[str, dict[str, Path]] = defaultdict(dict)
    rule_sets: dict[str, list[dict]] = defaultdict(list)
    ignored_kinds = {
        "conflict",
        "drive_conversion",
        "excluded_rule",
        "blocked_scope",
    }
    for _, (path, data) in documents.items():
        for kind, value in data.items():
            if kind in ignored_kinds or not isinstance(value, list):
                continue
            for record in value:
                if not isinstance(record, dict) or "id" not in record:
                    continue
                rule_id = record["id"]
                prior = ids_by_kind[kind].get(rule_id)
                if prior:
                    errors.append(
                        f"{path}: duplicate {kind} id {rule_id!r}; first in {prior}"
                    )
                ids_by_kind[kind][rule_id] = path
                rule_sets[kind].append(record)

    all_rule_ids = {
        rule_id for records in ids_by_kind.values() for rule_id in records
    }
    for kind, records in rule_sets.items():
        for record in records:
            target = record.get("supersedes_id")
            if target and target not in all_rule_ids:
                errors.append(
                    f"{kind} {record['id']!r}: unknown supersedes_id {target!r}"
                )

    policy_path, policy = documents.get(
        "cepheus-trader.shipbuilding", (Path("<missing>"), {})
    )
    if not policy:
        errors.append("missing cepheus-trader.shipbuilding composition policy")
        return errors

    policy_source_ids = set(policy.get("source_ids", []))
    module_source_ids: set[str] = set()
    for module_id in policy.get("extension_module_ids", []):
        module = documents.get(module_id)
        if module:
            module_source_ids.update(module[1].get("source_ids", []))
    missing_policy_sources = module_source_ids - policy_source_ids
    if missing_policy_sources:
        errors.append(
            f"{policy_path}: extension sources absent from policy: "
            f"{sorted(missing_policy_sources)}"
        )

    drive_policy = policy.get("interstellar_drive_policy", {})
    if drive_policy.get("drive") != "jump":
        errors.append(f"{policy_path}: active interstellar drive must be Jump")
    forbidden_true = (
        "allows_in_system_transit",
        "has_routes_or_points",
        "has_emitter_nodes",
        "has_recharge_cycle",
        "has_military_grade",
        "has_bubble_failure",
    )
    for field in forbidden_true:
        if drive_policy.get(field) is not False:
            errors.append(f"{policy_path}: {field} must be false")

    # Active module rule IDs must not smuggle excluded Z-drive or ansible
    # mechanics back into the catalog.
    for module_id in policy.get("extension_module_ids", []):
        module_path, module = documents[module_id]
        for kind, records in module.items():
            if kind in {"excluded_component", "blocked_component"}:
                continue
            if not isinstance(records, list):
                continue
            for record in records:
                if not isinstance(record, dict):
                    continue
                rule_id = str(record.get("id", "")).lower()
                if "zimm" in rule_id or rule_id.startswith("z-drive"):
                    errors.append(f"{module_path}: active forbidden rule {rule_id!r}")
                if "quantum-entanglement" in rule_id:
                    errors.append(f"{module_path}: active ansible rule {rule_id!r}")
                if "fuel-cell" in rule_id:
                    errors.append(
                        f"{module_path}: active one-year fuel-cell rule {rule_id!r}"
                    )
                if (
                    kind == "bay"
                    and "railgun" in rule_id
                    and record.get("displacement_millitons") == 501000
                ):
                    errors.append(
                        f"{module_path}: active 500-tonne railgun bay {rule_id!r}"
                    )
                if kind == "ammunition" and rule_id.startswith("heavy-railgun-"):
                    errors.append(
                        f"{module_path}: active heavy-railgun ammunition {rule_id!r}"
                    )

    for hull in rule_sets.get("hull", []):
        if (
            hull.get("tons", 0) > 5000
            and hull.get("interstellar_status") != "in-system-only"
        ):
            errors.append(
                f"hull {hull.get('id')!r}: hulls above 5,000 tons must be "
                "marked in-system-only"
            )

    # Verify every optimized conversion is the smallest CE drive that provides
    # Jump-2 for that hull. This catches hand-edited conversion drift.
    core = documents.get("ce-srd-2016.shipbuilding")
    if core:
        _, core_data = core
        drive_order = [row["code"] for row in core_data["drive"]]
        performance = {
            row["hull_tons"]: row["values"] for row in core_data["drive_performance"]
        }
        drive_volume = {
            row["code"]: row["jump_millitons"] for row in core_data["drive"]
        }
        seen_hulls: set[int] = set()
        for conversion in policy.get("drive_conversion", []):
            hull = conversion["hull_tons"]
            if hull in seen_hulls:
                errors.append(f"{policy_path}: duplicate conversion for {hull} tons")
                continue
            seen_hulls.add(hull)
            row = performance.get(hull)
            if row is None:
                errors.append(f"{policy_path}: no CE performance row for {hull} tons")
                continue
            minimum = next(
                (code for code in drive_order if row.get(code, 0) >= 2), None
            )
            optimized = conversion["optimized_j_code"]
            if optimized != minimum:
                errors.append(
                    f"{policy_path}: {hull}-ton optimized drive is {optimized}, "
                    f"but minimum CE Jump-2 drive is {minimum}"
                )
            source_code = conversion["source_z_code"]
            if source_code not in drive_volume or optimized not in drive_volume:
                errors.append(f"{policy_path}: unknown conversion drive code")
                continue
            recovered = drive_volume[source_code] - drive_volume[optimized]
            if recovered != conversion["recovered_millitons"]:
                errors.append(
                    f"{policy_path}: {hull}-ton recovered volume is "
                    f"{conversion['recovered_millitons']}, expected {recovered}"
                )

    conflicts = policy.get("conflict", [])
    conflict_ids = [entry.get("id") for entry in conflicts]
    if len(conflict_ids) != len(set(conflict_ids)):
        errors.append(f"{policy_path}: duplicate conflict id")
    for conflict in conflicts:
        for field in ("id", "subject", "source_value", "core_value", "resolution", "reason"):
            if not conflict.get(field):
                errors.append(
                    f"{policy_path}: conflict {conflict.get('id')!r} lacks {field}"
                )

    excluded_ids = {entry["id"] for entry in policy.get("excluded_rule", [])}
    if "quantum-entanglement-communicator" not in excluded_ids:
        errors.append(f"{policy_path}: instantaneous communicator is not excluded")
    if "fusion-power-fuel-cells" not in excluded_ids:
        errors.append(f"{policy_path}: one-year power-plant fuel cells are not excluded")

    try:
        compose_shipbuilding_rules(rules_dir)
    except RuleCompositionError as error:
        errors.append(f"ruleset composition failed: {error}")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rules", type=Path, default=DEFAULT_RULES)
    parser.add_argument("--sources", type=Path, default=DEFAULT_SOURCES)
    args = parser.parse_args()
    errors = validate(args.rules, args.sources)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"validated shipbuilding rules in {args.rules}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
