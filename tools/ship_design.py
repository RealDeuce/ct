#!/usr/bin/env python3
"""Evaluate a ship bill of materials against rule-derived construction data.

This module never creates construction rules from a published ship
description.  A design may select rules and state published assertions, but
it cannot override a rule's displacement, price, performance, or TL.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import sys
import tomllib
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RULES = ROOT / "catalog/shipbuilding/ce-core.toml"
DEFAULT_SMALL_CRAFT_RULES = (
    ROOT / "catalog/shipbuilding/ce-small-craft.toml"
)
DEFAULT_COMPOSED_RULES_DIR = ROOT / "catalog" / "shipbuilding"


class DesignError(ValueError):
    pass


def _integer(value: object, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise DesignError(f"{label} must be an integer >= {minimum}")
    return value


def _text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise DesignError(f"{label} must be non-empty text")
    return value


def _table(value: object, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise DesignError(f"{label} must be a table")
    return value


def _records(value: object, label: str) -> list[dict[str, Any]]:
    if value is None:
        return []
    if not isinstance(value, list) or any(not isinstance(x, dict) for x in value):
        raise DesignError(f"{label} must be an array of tables")
    return value


def _text_list(value: object, label: str, minimum: int = 0) -> list[str]:
    if (
        not isinstance(value, list)
        or len(value) < minimum
        or any(not isinstance(item, str) or not item for item in value)
        or len(value) != len(set(value))
    ):
        raise DesignError(
            f"{label} must be a unique text list with at least {minimum} item(s)"
        )
    return value


def _index(records: object, key: str, label: str) -> dict[str, dict[str, Any]]:
    found: dict[str, dict[str, Any]] = {}
    for position, record in enumerate(_records(records, label), 1):
        item_id = _text(record.get(key), f"{label}[{position}].{key}")
        if item_id in found:
            raise DesignError(f"{label} contains duplicate {key} {item_id!r}")
        found[item_id] = record
    return found


def _check_keys(record: dict[str, Any], allowed: set[str], label: str) -> None:
    extra = set(record) - allowed
    if extra:
        raise DesignError(f"{label} has unknown field(s): {', '.join(sorted(extra))}")


@dataclass
class Totals:
    displacement_millitons: int = 0
    discountable_credits: int = 0
    undiscounted_credits: int = 0

    def add(
        self,
        displacement_millitons: int,
        price_credits: int,
        *,
        discountable: bool = True,
    ) -> None:
        self.displacement_millitons += displacement_millitons
        if discountable:
            self.discountable_credits += price_credits
        else:
            self.undiscounted_credits += price_credits


def load_toml(path: Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise DesignError(f"{path}: {error}") from error


def _require_tl(design_tl: int, rule: dict[str, Any], label: str) -> None:
    minimum = _integer(rule.get("min_tl", 0), f"{label}.min_tl")
    if design_tl < minimum:
        raise DesignError(f"{label} requires TL{minimum}; design is TL{design_tl}")


def evaluate(
    rules: dict[str, Any],
    design: dict[str, Any],
    *,
    carried_craft_results: dict[str, dict[str, Any]] | None = None,
) -> dict[str, Any]:
    if rules.get("schema_version") != 1:
        raise DesignError("construction rules schema_version must be 1")
    if design.get("schema_version") != 1:
        raise DesignError("design schema_version must be 1")

    _check_keys(
        design,
        {
            "schema_version",
            "design_id",
            "revision",
            "ruleset_id",
            "source_ids",
            "catalog",
            "tech_level",
            "standard_design",
            "include_architect_fee",
            "hull",
            "armor",
            "hull_options",
            "structural_options",
            "drives",
            "power_options",
            "fuel",
            "bridge_options",
            "computer",
            "software",
            "electronics",
            "equipment",
            "parameterized_equipment",
            "hangars",
            "launch_facilities",
            "docking_clamps",
            "unused_fire_control_stations",
            "mounts",
            "barbettes",
            "point_defense",
            "bays",
            "screens",
            "ammunition",
            "magazine_options",
            "carried_craft",
            "external_load_millitons",
            "crew",
            "cargo_millitons",
            "thrust_g",
            "assertions",
        },
        "design",
    )
    ruleset_id = _text(rules.get("ruleset_id"), "rules.ruleset_id")
    if design.get("ruleset_id") != ruleset_id:
        raise DesignError(f"design must select ruleset_id {ruleset_id!r}")
    rule_sources = set(
        _text_list(rules.get("source_ids"), "rules.source_ids", minimum=1)
    )
    design_sources = set(
        _text_list(design.get("source_ids"), "design.source_ids", minimum=1)
    )
    if not rule_sources.issubset(design_sources):
        missing = ", ".join(sorted(rule_sources - design_sources))
        raise DesignError(f"design.source_ids omits rules source(s): {missing}")
    _text(design.get("design_id"), "design.design_id")
    _integer(design.get("revision"), "design.revision", 1)
    design_tl = _integer(design.get("tech_level"), "design.tech_level", 1)
    standard_design = design.get("standard_design")
    if not isinstance(standard_design, bool):
        raise DesignError("design.standard_design must be boolean")
    architect_fee = design.get("include_architect_fee", False)
    if not isinstance(architect_fee, bool):
        raise DesignError("design.include_architect_fee must be boolean")
    if standard_design and architect_fee:
        raise DesignError("a standard design cannot include a new-design architect fee")

    hulls = _index(rules.get("hull"), "id", "rules.hull")
    configurations = _index(
        rules.get("configuration"), "id", "rules.configuration"
    )
    armors = _index(rules.get("armor"), "id", "rules.armor")
    armor_extensions = _index(
        rules.get("armor_extension"), "id", "rules.armor_extension"
    )
    hull_options = _index(
        rules.get("hull_option"), "id", "rules.hull_option"
    )
    structural_options = _index(
        rules.get("structural_option"), "id", "rules.structural_option"
    )
    power_options = _index(
        rules.get("power_option"), "id", "rules.power_option"
    )
    drives_by_code = _index(rules.get("drive"), "code", "rules.drive")
    computers = _index(rules.get("computer"), "id", "rules.computer")
    computer_options = _index(
        rules.get("computer_option"), "id", "rules.computer_option"
    )
    bridge_options = _index(
        rules.get("bridge_option"), "id", "rules.bridge_option"
    )
    software_rules = _index(rules.get("software"), "id", "rules.software")
    electronics_rules = _index(
        rules.get("electronics"), "id", "rules.electronics"
    )
    equipment_rules = _index(rules.get("equipment"), "id", "rules.equipment")
    parameterized_equipment_rules = _index(
        rules.get("parameterized_equipment"),
        "id",
        "rules.parameterized_equipment",
    )
    hangar_rules = _index(rules.get("hangar"), "id", "rules.hangar")
    launch_facility_rules = _index(
        rules.get("launch_facility"), "id", "rules.launch_facility"
    )
    docking_clamp_rules = _index(
        rules.get("docking_clamp"), "id", "rules.docking_clamp"
    )
    mount_rules = _index(rules.get("mount"), "id", "rules.mount")
    weapon_rules = _index(rules.get("weapon"), "id", "rules.weapon")
    barbette_rules = _index(rules.get("barbette"), "id", "rules.barbette")
    point_defense_mount_rules = _index(
        rules.get("point_defense_mount"),
        "id",
        "rules.point_defense_mount",
    )
    point_defense_weapon_rules = _index(
        rules.get("point_defense_weapon"),
        "id",
        "rules.point_defense_weapon",
    )
    bay_rules = _index(rules.get("bay"), "id", "rules.bay")
    screen_rules = _index(rules.get("screen"), "id", "rules.screen")
    ammunition_rules = _index(
        rules.get("ammunition"), "id", "rules.ammunition"
    )
    magazine_option_rules = _index(
        rules.get("magazine_option"), "id", "rules.magazine_option"
    )
    crew_roles = _index(rules.get("crew_role"), "id", "rules.crew_role")

    hull_choice = _table(design.get("hull"), "design.hull")
    _check_keys(hull_choice, {"id", "configuration"}, "design.hull")
    hull_id = _text(hull_choice.get("id"), "design.hull.id")
    try:
        hull = hulls[hull_id]
    except KeyError as error:
        raise DesignError(f"unknown hull {hull_id!r}") from error
    configuration_id = _text(
        hull_choice.get("configuration"), "design.hull.configuration"
    )
    try:
        configuration = configurations[configuration_id]
    except KeyError as error:
        raise DesignError(f"unknown configuration {configuration_id!r}") from error

    hull_tons = _integer(hull.get("tons"), f"rules.hull.{hull_id}.tons", 1)
    hull_millitons = hull_tons * 1000
    hull_code = _text(hull.get("code"), f"rules.hull.{hull_id}.code")
    hull_tons_by_code = {
        _text(record.get("code"), "rules.hull.code"): _integer(
            record.get("tons"), "rules.hull.tons", 1
        )
        for record in hulls.values()
    }
    base_hull_price = _integer(
        hull.get("price_credits"), f"rules.hull.{hull_id}.price_credits"
    )
    configuration_percent = _integer(
        configuration.get("hull_price_percent"),
        f"rules.configuration.{configuration_id}.hull_price_percent",
    )
    configured_hull_price = base_hull_price * configuration_percent // 100
    totals = Totals()
    totals.add(0, configured_hull_price)
    line_items: list[dict[str, Any]] = [
        {
            "kind": "hull",
            "id": hull_id,
            "quantity": 1,
            "displacement_millitons": 0,
            "price_credits": configured_hull_price,
        }
    ]

    armor_points = 0
    armor_choice = design.get("armor")
    if armor_choice is not None:
        armor_choice = _table(armor_choice, "design.armor")
        _check_keys(armor_choice, {"id", "layers", "points"}, "design.armor")
        armor_id = _text(armor_choice.get("id"), "design.armor.id")
        has_layers = "layers" in armor_choice
        has_points = "points" in armor_choice
        if has_layers == has_points:
            raise DesignError("design.armor selects exactly one of layers or points")
        if has_layers:
            layers = _integer(
                armor_choice.get("layers"), "design.armor.layers", 1
            )
            try:
                armor = armors[armor_id]
            except KeyError as error:
                raise DesignError(f"unknown armor {armor_id!r}") from error
            _require_tl(design_tl, armor, f"armor {armor_id}")
            volume_percent = _integer(
                armor.get("volume_percent_per_layer"),
                f"rules.armor.{armor_id}.volume_percent_per_layer",
            )
            minimum = _integer(
                armor.get("minimum_millitons_per_layer"),
                f"rules.armor.{armor_id}.minimum_millitons_per_layer",
            )
            per_layer_volume = max(
                minimum, hull_millitons * volume_percent // 100
            )
            price_percent = _integer(
                armor.get("base_hull_price_percent_per_layer"),
                f"rules.armor.{armor_id}.base_hull_price_percent_per_layer",
            )
            armor_price = base_hull_price * price_percent * layers // 100
            armor_volume = per_layer_volume * layers
            armor_points = (
                _integer(
                    armor.get("protection_per_layer"),
                    f"rules.armor.{armor_id}.protection_per_layer",
                )
                * layers
            )
            armor_quantity = layers
        else:
            points = _integer(
                armor_choice.get("points"), "design.armor.points", 1
            )
            try:
                extension = armor_extensions[armor_id]
            except KeyError as error:
                raise DesignError(f"unknown whole-point armor {armor_id!r}") from error
            base_id = _text(
                extension.get("base_armor_id"),
                f"rules.armor_extension.{armor_id}.base_armor_id",
            )
            try:
                armor = armors[base_id]
            except KeyError as error:
                raise DesignError(
                    f"whole-point armor {armor_id!r} has unknown base {base_id!r}"
                ) from error
            _require_tl(design_tl, armor, f"armor {armor_id}")
            volume_basis = _integer(
                extension.get("volume_basis_points_per_point"),
                f"rules.armor_extension.{armor_id}."
                "volume_basis_points_per_point",
                1,
            )
            price_basis = _integer(
                extension.get("base_hull_price_basis_points_per_point"),
                f"rules.armor_extension.{armor_id}."
                "base_hull_price_basis_points_per_point",
                1,
            )
            armor_volume = (
                hull_millitons * volume_basis * points + 9999
            ) // 10000
            armor_price = base_hull_price * price_basis * points // 10000
            formula = _text(
                extension.get("maximum_protection_formula"),
                f"rules.armor_extension.{armor_id}.maximum_protection_formula",
            )
            ceiling = {
                "minimum-of-tech-level-and-9": min(design_tl, 9),
                "minimum-of-tech-level-and-12": min(design_tl, 12),
            }.get(formula)
            if ceiling is None:
                raise DesignError(f"unknown armor maximum formula {formula!r}")
            if points > ceiling:
                raise DesignError(
                    f"whole-point armor {armor_id!r} exceeds maximum {ceiling}"
                )
            armor_points = points
            armor_quantity = points
        totals.add(armor_volume, armor_price)
        line_items.append(
            {
                "kind": "armor",
                "id": armor_id,
                "quantity": armor_quantity,
                "displacement_millitons": armor_volume,
                "price_credits": armor_price,
            }
        )

    installed_hull_options: set[str] = set()
    for position, choice in enumerate(
        _records(design.get("hull_options"), "design.hull_options"), 1
    ):
        _check_keys(choice, {"id", "quantity"}, f"design.hull_options[{position}]")
        option_id = _text(choice.get("id"), f"design.hull_options[{position}].id")
        quantity = _integer(
            choice.get("quantity", 1),
            f"design.hull_options[{position}].quantity",
            1,
        )
        try:
            option = hull_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown hull option {option_id!r}") from error
        if option_id in installed_hull_options:
            raise DesignError(
                f"hull option {option_id!r} must use one quantity record"
            )
        installed_hull_options.add(option_id)
        _require_tl(design_tl, option, f"hull option {option_id}")
        maximum = option.get("maximum_quantity")
        if maximum is not None and quantity > _integer(
            maximum, f"rules.hull_option.{option_id}.maximum_quantity", 1
        ):
            raise DesignError(f"hull option {option_id!r} exceeds maximum quantity")
        price = (
            _integer(
                option.get("price_credits_per_hull_ton"),
                f"rules.hull_option.{option_id}.price_credits_per_hull_ton",
            )
            * hull_tons
            * quantity
        )
        totals.add(0, price)
        line_items.append(
            {
                "kind": "hull-option",
                "id": option_id,
                "quantity": quantity,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )

    structure_bonus = 0
    deferred_structural_options: list[
        tuple[str, dict[str, Any], str]
    ] = []
    installed_structural_options: set[str] = set()
    for position, choice in enumerate(
        _records(design.get("structural_options"), "design.structural_options"),
        1,
    ):
        label = f"design.structural_options[{position}]"
        _check_keys(
            choice,
            {"id", "increments", "protected_component_id"},
            label,
        )
        option_id = _text(choice.get("id"), f"{label}.id")
        increments = _integer(
            choice.get("increments", 1), f"{label}.increments", 1
        )
        try:
            option = structural_options[option_id]
        except KeyError as error:
            raise DesignError(
                f"unknown structural option {option_id!r}"
            ) from error
        formula = _text(
            option.get("formula"),
            f"rules.structural_option.{option_id}.formula",
        )
        if formula == "protected-system-volume-percent":
            protected_component_id = _text(
                choice.get("protected_component_id"),
                f"{label}.protected_component_id",
            )
            if increments != 1:
                raise DesignError(
                    f"structural option {option_id!r} does not take increments"
                )
            installed_key = f"{option_id}:{protected_component_id}"
            if installed_key in installed_structural_options:
                raise DesignError(
                    f"structural option {option_id!r} already protects "
                    f"{protected_component_id!r}"
                )
            installed_structural_options.add(installed_key)
            deferred_structural_options.append(
                (option_id, option, protected_component_id)
            )
            continue
        if formula != "hull-volume-percent":
            raise DesignError(
                f"structural option formula {formula!r} is not implemented"
            )
        if "protected_component_id" in choice:
            raise DesignError(
                f"structural option {option_id!r} does not protect a component"
            )
        if option_id in installed_structural_options:
            raise DesignError(
                f"structural option {option_id!r} must use one increment record"
            )
        installed_structural_options.add(option_id)
        volume_percent = _integer(
            option.get("volume_percent_per_increment"),
            f"rules.structural_option.{option_id}.volume_percent_per_increment",
            1,
        )
        volume = hull_millitons * volume_percent * increments // 100
        price = (
            volume
            * _integer(
                option.get("price_credits_per_installed_ton"),
                f"rules.structural_option.{option_id}."
                "price_credits_per_installed_ton",
            )
            // 1000
        )
        bonuses = option.get("bonuses")
        if not isinstance(bonuses, list):
            raise DesignError(
                f"rules.structural_option.{option_id}.bonuses must be an array"
            )
        points_per_increment = None
        for bonus_position, raw_bonus in enumerate(bonuses, 1):
            bonus = _table(
                raw_bonus,
                f"rules.structural_option.{option_id}.bonuses[{bonus_position}]",
            )
            if hull_tons <= _integer(
                bonus.get("maximum_hull_tons"),
                f"rules.structural_option.{option_id}."
                f"bonuses[{bonus_position}].maximum_hull_tons",
                1,
            ):
                points_per_increment = _integer(
                    bonus.get("points_per_increment"),
                    f"rules.structural_option.{option_id}."
                    f"bonuses[{bonus_position}].points_per_increment",
                    1,
                )
                break
        if points_per_increment is None:
            raise DesignError(
                f"structural option {option_id!r} has no bonus for "
                f"{hull_tons}-ton hull"
            )
        option_bonus = points_per_increment * increments
        structure_bonus += option_bonus
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "structural-option",
                "id": option_id,
                "quantity": increments,
                "structure_points": option_bonus,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    installed_protection: set[tuple[str, str]] = set()

    def install_protected_system(
        protected_component_id: str,
        protected_volume: int,
    ) -> None:
        for option_id, option, target in deferred_structural_options:
            if target != protected_component_id:
                continue
            key = (option_id, target)
            volume = (
                protected_volume
                * _integer(
                    option.get("protected_system_volume_percent"),
                    f"rules.structural_option.{option_id}."
                    "protected_system_volume_percent",
                    1,
                )
                // 100
            )
            price = (
                volume
                * _integer(
                    option.get("price_credits_per_installed_ton"),
                    f"rules.structural_option.{option_id}."
                    "price_credits_per_installed_ton",
                )
                // 1000
            )
            totals.add(volume, price)
            line_items.append(
                {
                    "kind": "structural-option",
                    "id": option_id,
                    "protected_component_id": target,
                    "quantity": 1,
                    "displacement_millitons": volume,
                    "price_credits": price,
                }
            )
            installed_protection.add(key)

    performance_rows: dict[int, dict[str, int]] = {}
    for position, row in enumerate(
        _records(rules.get("drive_performance"), "rules.drive_performance"), 1
    ):
        row_hull = _integer(
            row.get("hull_tons"),
            f"rules.drive_performance[{position}].hull_tons",
            1,
        )
        if row_hull in performance_rows:
            raise DesignError(f"duplicate drive-performance row for {row_hull} tons")
        values = _table(
            row.get("values"), f"rules.drive_performance[{position}].values"
        )
        performance_rows[row_hull] = {
            _text(code, "drive code"): _integer(
                value, f"drive performance {row_hull}/{code}", 1
            )
            for code, value in values.items()
        }
    external_load_millitons = _integer(
        design.get("external_load_millitons", 0),
        "design.external_load_millitons",
    )
    effective_displacement_millitons = (
        hull_millitons + external_load_millitons
    )
    effective_displacement_tons = (
        effective_displacement_millitons + 999
    ) // 1000
    try:
        drive_performance_tons = next(
            tons
            for tons in sorted(performance_rows)
            if tons >= effective_displacement_tons
        )
    except StopIteration as error:
        raise DesignError(
            "no drive-performance row for loaded displacement "
            f"{effective_displacement_tons} tons"
        ) from error
    drive_choices = _table(design.get("drives"), "design.drives")
    _check_keys(drive_choices, {"jump", "maneuver", "power"}, "design.drives")
    selected_codes: dict[str, str | None] = {}
    drive_performance: dict[str, int] = {}
    drive_rank = {code: rank for rank, code in enumerate(drives_by_code)}
    selected_drive_volume = 0
    for kind in ("jump", "maneuver", "power"):
        raw_code = drive_choices.get(kind)
        if raw_code is None and kind != "power":
            selected_codes[kind] = None
            drive_performance[kind] = 0
            continue
        code = _text(raw_code, f"design.drives.{kind}")
        try:
            drive = drives_by_code[code]
        except KeyError as error:
            raise DesignError(f"unknown {kind} drive code {code!r}") from error
        selected_codes[kind] = code
        if kind != "power":
            try:
                drive_performance[kind] = performance_rows[
                    drive_performance_tons
                ][code]
            except KeyError as error:
                raise DesignError(
                    f"{kind} drive {code} has no performance for loaded "
                    f"{effective_displacement_tons} tons "
                    f"(using {drive_performance_tons}-ton row)"
                ) from error
        displacement = _integer(
            drive.get(f"{kind}_millitons"),
            f"rules.drive.{code}.{kind}_millitons",
        )
        price = _integer(
            drive.get(f"{kind}_price_credits"),
            f"rules.drive.{code}.{kind}_price_credits",
        )
        totals.add(displacement, price)
        selected_drive_volume += displacement
        line_items.append(
            {
                "kind": f"{kind}-drive" if kind != "power" else "power-plant",
                "id": code,
                "quantity": 1,
                "displacement_millitons": displacement,
                "price_credits": price,
            }
        )
        if kind == "power":
            main_power_volume = displacement
            main_power_price = price
    install_protected_system("drives", selected_drive_volume)
    power_code = selected_codes["power"]
    assert power_code is not None
    selected_power_options = _text_list(
        design.get("power_options", []),
        "design.power_options",
    )
    for option_id in selected_power_options:
        try:
            option = power_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown power option {option_id!r}") from error
        formula = _text(
            option.get("formula"), f"rules.power_option.{option_id}.formula"
        )
        if formula != "main-power-plant-percent":
            raise DesignError(
                f"power option formula {formula!r} is not implemented"
            )
        volume = (
            main_power_volume
            * _integer(
                option.get("volume_percent"),
                f"rules.power_option.{option_id}.volume_percent",
                1,
            )
            // 100
        )
        price = (
            main_power_price
            * _integer(
                option.get("price_percent"),
                f"rules.power_option.{option_id}.price_percent",
                1,
            )
            // 100
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "power-option",
                "id": option_id,
                "quantity": 1,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )
    for kind in ("jump", "maneuver"):
        code = selected_codes[kind]
        if code is not None and drive_rank[power_code] < drive_rank[code]:
            raise DesignError(
                f"power plant {power_code} is rated below {kind} drive {code}"
            )

    fuel = _table(design.get("fuel"), "design.fuel")
    _check_keys(
        fuel,
        {"jump_distance", "jump_count", "power_plant_weeks", "reserve_millitons"},
        "design.fuel",
    )
    jump_distance = _integer(fuel.get("jump_distance", 0), "design.fuel.jump_distance")
    jump_count = _integer(fuel.get("jump_count", 0), "design.fuel.jump_count")
    power_weeks = _integer(
        fuel.get("power_plant_weeks"), "design.fuel.power_plant_weeks", 2
    )
    reserve_fuel = _integer(
        fuel.get("reserve_millitons", 0), "design.fuel.reserve_millitons"
    )
    jump_rating = drive_performance["jump"]
    if jump_distance > jump_rating:
        raise DesignError(
            f"jump fuel distance {jump_distance} exceeds Jump-{jump_rating}"
        )
    if (jump_distance == 0) != (jump_count == 0):
        raise DesignError("jump_distance and jump_count must both be zero or nonzero")
    jump_fuel_numerator = (
        effective_displacement_millitons * jump_distance * jump_count
    )
    if jump_fuel_numerator % 10:
        raise DesignError("loaded displacement produces fractional jump fuel")
    jump_fuel = jump_fuel_numerator // 10
    power_drive = drives_by_code[power_code]
    weekly_power_fuel = _integer(
        power_drive.get("power_fuel_millitons_per_week"),
        f"rules.drive.{power_code}.power_fuel_millitons_per_week",
    )
    power_fuel = weekly_power_fuel * power_weeks
    total_fuel = jump_fuel + power_fuel + reserve_fuel
    totals.add(total_fuel, 0, discountable=False)
    line_items.append(
        {
            "kind": "fuel",
            "id": "tankage",
            "quantity": 1,
            "displacement_millitons": total_fuel,
            "price_credits": 0,
        }
    )

    bridge = _table(rules.get("bridge"), "rules.bridge")
    bridge_sizes = bridge.get("sizes")
    if not isinstance(bridge_sizes, list):
        raise DesignError("rules.bridge.sizes must be an array")
    bridge_volume = None
    for position, row in enumerate(bridge_sizes, 1):
        row = _table(row, f"rules.bridge.sizes[{position}]")
        if hull_tons <= _integer(
            row.get("maximum_hull_tons"),
            f"rules.bridge.sizes[{position}].maximum_hull_tons",
            1,
        ):
            bridge_volume = _integer(
                row.get("displacement_millitons"),
                f"rules.bridge.sizes[{position}].displacement_millitons",
                1,
            )
            break
    if bridge_volume is None:
        raise DesignError(f"no bridge-size rule for {hull_tons}-ton hull")
    bridge_price = (
        hull_tons
        // 100
        * _integer(
            bridge.get("price_credits_per_100_hull_tons"),
            "rules.bridge.price_credits_per_100_hull_tons",
        )
    )
    install_protected_system("bridge", bridge_volume)
    base_bridge_volume = bridge_volume
    base_bridge_price = bridge_price
    selected_bridge_options = _text_list(
        design.get("bridge_options", []),
        "design.bridge_options",
    )
    for option_id in selected_bridge_options:
        try:
            option = bridge_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown bridge option {option_id!r}") from error
        volume_percent = _integer(
            option.get("volume_percent"),
            f"rules.bridge_option.{option_id}.volume_percent",
            1,
        )
        price_percent = _integer(
            option.get("price_percent"),
            f"rules.bridge_option.{option_id}.price_percent",
            1,
        )
        option_volume = base_bridge_volume * (volume_percent - 100) // 100
        option_price = base_bridge_price * (price_percent - 100) // 100
        bridge_volume += option_volume
        bridge_price += option_price
        line_items.append(
            {
                "kind": "bridge-option",
                "id": option_id,
                "quantity": 1,
                "displacement_millitons": option_volume,
                "price_credits": option_price,
            }
        )
    totals.add(bridge_volume, bridge_price)
    line_items.append(
        {
            "kind": "bridge",
            "id": "bridge",
            "quantity": 1,
            "displacement_millitons": base_bridge_volume,
            "price_credits": base_bridge_price,
        }
    )

    computer_choice = _table(design.get("computer"), "design.computer")
    _check_keys(computer_choice, {"id", "options", "quantity"}, "design.computer")
    computer_id = _text(computer_choice.get("id"), "design.computer.id")
    try:
        computer = computers[computer_id]
    except KeyError as error:
        raise DesignError(f"unknown computer {computer_id!r}") from error
    _require_tl(design_tl, computer, f"computer {computer_id}")
    minimum_computer_hull = computer.get("minimum_hull_tons")
    if minimum_computer_hull is not None and hull_tons < _integer(
        minimum_computer_hull,
        f"rules.computer.{computer_id}.minimum_hull_tons",
        1,
    ):
        raise DesignError(f"computer {computer_id!r} is too large for this hull")
    maximum_computer_hull = computer.get("maximum_hull_tons")
    if maximum_computer_hull is not None and hull_tons > _integer(
        maximum_computer_hull,
        f"rules.computer.{computer_id}.maximum_hull_tons",
        1,
    ):
        raise DesignError(f"computer {computer_id!r} is too small for this hull")
    computer_quantity = _integer(
        computer_choice.get("quantity", 1), "design.computer.quantity", 1
    )
    computer_price = _integer(
        computer.get("price_credits"), f"rules.computer.{computer_id}.price_credits"
    )
    computer_rating = _integer(
        computer.get("rating"), f"rules.computer.{computer_id}.rating", 1
    )
    option_names = computer_choice.get("options", [])
    if (
        not isinstance(option_names, list)
        or any(not isinstance(option, str) for option in option_names)
        or len(option_names) != len(set(option_names))
    ):
        raise DesignError("design.computer.options must be a unique text list")
    jump_rating_bonus = 0
    option_percent = 0
    for option_id in option_names:
        try:
            option = computer_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown computer option {option_id!r}") from error
        option_percent += _integer(
            option.get("price_percent_of_computer"),
            f"rules.computer_option.{option_id}.price_percent_of_computer",
        )
        jump_rating_bonus += _integer(
            option.get("jump_control_rating_bonus", 0),
            f"rules.computer_option.{option_id}.jump_control_rating_bonus",
        )
    computer_price += computer_price * option_percent // 100
    computer_price *= computer_quantity
    totals.add(0, computer_price)
    line_items.append(
        {
            "kind": "computer",
            "id": computer_id,
            "quantity": computer_quantity,
            "displacement_millitons": 0,
            "price_credits": computer_price,
        }
    )

    jump_control_level = 0
    installed_software: set[str] = set()
    for position, choice in enumerate(
        _records(design.get("software"), "design.software"), 1
    ):
        _check_keys(choice, {"id", "level"}, f"design.software[{position}]")
        software_id = _text(choice.get("id"), f"design.software[{position}].id")
        if software_id in installed_software:
            raise DesignError(f"software {software_id!r} may be selected once")
        installed_software.add(software_id)
        try:
            software = software_rules[software_id]
        except KeyError as error:
            raise DesignError(f"unknown software {software_id!r}") from error
        _require_tl(design_tl, software, f"software {software_id}")
        if "rating" in software:
            if "level" in choice:
                raise DesignError(
                    f"fixed-rating software {software_id!r} has no level"
                )
            level = 1
            rating = _integer(
                software.get("rating"),
                f"rules.software.{software_id}.rating",
                1,
            )
            price = _integer(
                software.get("price_credits"),
                f"rules.software.{software_id}.price_credits",
            )
        else:
            level = _integer(
                choice.get("level"), f"design.software[{position}].level", 1
            )
            maximum = _integer(
                software.get("maximum_level"),
                f"rules.software.{software_id}.maximum_level",
                1,
            )
            if level > maximum:
                raise DesignError(
                    f"software {software_id!r} exceeds level {maximum}"
                )
            rating = _integer(
                software.get("base_rating", 0),
                f"rules.software.{software_id}.base_rating",
            ) + level * _integer(
                software.get("rating_per_level"),
                f"rules.software.{software_id}.rating_per_level",
            )
            price = level * _integer(
                software.get("price_credits_per_level"),
                f"rules.software.{software_id}.price_credits_per_level",
            )
        discountable = software.get("discountable", True)
        if not isinstance(discountable, bool):
            raise DesignError(f"rules.software.{software_id}.discountable is invalid")
        totals.add(0, price, discountable=discountable)
        if software_id == "jump-control":
            jump_control_level = max(jump_control_level, level)
            if rating > computer_rating + jump_rating_bonus:
                raise DesignError(
                    f"Jump Control/{level} requires computer rating {rating}"
                )
        else:
            if rating > computer_rating:
                raise DesignError(
                    f"software {software_id!r} requires computer rating {rating}"
                )
        line_items.append(
            {
                "kind": "software",
                "id": software_id,
                "quantity": level,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )
    if jump_rating and jump_control_level < jump_rating:
        raise DesignError(
            f"Jump-{jump_rating} drive requires Jump Control/{jump_rating}"
        )

    electronics_id = _text(design.get("electronics"), "design.electronics")
    try:
        electronics = electronics_rules[electronics_id]
    except KeyError as error:
        raise DesignError(f"unknown electronics {electronics_id!r}") from error
    _require_tl(design_tl, electronics, f"electronics {electronics_id}")
    electronics_volume = _integer(
        electronics.get("displacement_millitons"),
        f"rules.electronics.{electronics_id}.displacement_millitons",
    )
    electronics_price = _integer(
        electronics.get("price_credits"),
        f"rules.electronics.{electronics_id}.price_credits",
    )
    totals.add(electronics_volume, electronics_price)
    line_items.append(
        {
            "kind": "electronics",
            "id": electronics_id,
            "quantity": 1,
            "displacement_millitons": electronics_volume,
            "price_credits": electronics_price,
        }
    )

    installed_equipment: dict[str, int] = {}
    crew_accommodation_capacity = 0
    dedicated_crew_accommodation_capacity = 0
    passenger_accommodation_berths = 0
    shared_accommodations: list[tuple[int, int, int, str]] = []
    provision_capacity_persons = 0
    low_berths = 0
    monthly_life_support_credits = 0
    hangar_capacity_millitons = 0
    for position, choice in enumerate(
        _records(design.get("equipment"), "design.equipment"), 1
    ):
        _check_keys(choice, {"id", "quantity"}, f"design.equipment[{position}]")
        equipment_id = _text(choice.get("id"), f"design.equipment[{position}].id")
        quantity = _integer(
            choice.get("quantity"), f"design.equipment[{position}].quantity", 1
        )
        if equipment_id in installed_equipment:
            raise DesignError(
                f"equipment {equipment_id!r} must use one quantity-bearing record"
            )
        installed_equipment[equipment_id] = quantity
        try:
            equipment = equipment_rules[equipment_id]
        except KeyError as error:
            raise DesignError(f"unknown equipment {equipment_id!r}") from error
        _require_tl(design_tl, equipment, f"equipment {equipment_id}")
        minimum = equipment.get("minimum_quantity")
        if minimum is not None and quantity < _integer(
            minimum, f"rules.equipment.{equipment_id}.minimum_quantity", 1
        ):
            raise DesignError(
                f"equipment {equipment_id!r} requires at least {minimum}"
            )
        maximum = equipment.get("maximum_quantity")
        if maximum is not None and quantity > _integer(
            maximum, f"rules.equipment.{equipment_id}.maximum_quantity", 1
        ):
            raise DesignError(f"equipment {equipment_id!r} exceeds maximum quantity")
        minimum_hull = equipment.get("minimum_hull_tons")
        if minimum_hull is not None and hull_tons < _integer(
            minimum_hull,
            f"rules.equipment.{equipment_id}.minimum_hull_tons",
            1,
        ):
            raise DesignError(
                f"equipment {equipment_id!r} requires a hull of at least "
                f"{minimum_hull} tons"
            )
        volume = quantity * _integer(
            equipment.get("displacement_millitons_per_unit"),
            f"rules.equipment.{equipment_id}.displacement_millitons_per_unit",
        )
        price = quantity * _integer(
            equipment.get("price_credits_per_unit"),
            f"rules.equipment.{equipment_id}.price_credits_per_unit",
        )
        totals.add(volume, price)
        crew_capacity_per_unit = equipment.get("crew_capacity_per_unit")
        if crew_capacity_per_unit is not None:
            crew_capacity = _integer(
                crew_capacity_per_unit,
                f"rules.equipment.{equipment_id}.crew_capacity_per_unit",
                1,
            )
            crew_accommodation_capacity += quantity * crew_capacity
        else:
            crew_capacity = 0
        passenger_berths_per_unit = equipment.get("passenger_berths_per_unit")
        if passenger_berths_per_unit is not None:
            passenger_berths = _integer(
                passenger_berths_per_unit,
                f"rules.equipment.{equipment_id}.passenger_berths_per_unit",
                1,
            )
            if crew_capacity:
                shared_accommodations.append(
                    (crew_capacity, passenger_berths, quantity, equipment_id)
                )
            else:
                passenger_accommodation_berths += quantity * passenger_berths
        elif crew_capacity:
            dedicated_crew_accommodation_capacity += quantity * crew_capacity
        provision_capacity_per_unit = equipment.get("provision_capacity_per_unit")
        if provision_capacity_per_unit is not None:
            provision_capacity_persons += quantity * _integer(
                provision_capacity_per_unit,
                f"rules.equipment.{equipment_id}.provision_capacity_per_unit",
                1,
            )
        low_berths_per_unit = equipment.get("low_berths_per_unit")
        if low_berths_per_unit is not None:
            low_berths += quantity * _integer(
                low_berths_per_unit,
                f"rules.equipment.{equipment_id}.low_berths_per_unit",
                1,
            )
        monthly_life_support_per_unit = equipment.get(
            "monthly_life_support_credits_per_unit"
        )
        if monthly_life_support_per_unit is not None:
            monthly_life_support_credits += quantity * _integer(
                monthly_life_support_per_unit,
                "rules.equipment."
                f"{equipment_id}.monthly_life_support_credits_per_unit",
                1,
            )
        craft_capacity_per_unit = equipment.get(
            "craft_capacity_millitons_per_unit"
        )
        if craft_capacity_per_unit is not None:
            hangar_capacity_millitons += quantity * _integer(
                craft_capacity_per_unit,
                f"rules.equipment.{equipment_id}."
                "craft_capacity_millitons_per_unit",
                1,
            )
        line_items.append(
            {
                "kind": "equipment",
                "id": equipment_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )
    included_scoops = configuration.get("includes_fuel_scoops")
    may_install_scoops = configuration.get("may_install_fuel_scoops")
    if not isinstance(included_scoops, bool) or not isinstance(
        may_install_scoops, bool
    ):
        raise DesignError("configuration fuel-scoop flags must be boolean")
    purchased_scoops = installed_equipment.get("fuel-scoop", 0) > 0
    if included_scoops and purchased_scoops:
        raise DesignError("streamlined hull already includes fuel scoops")
    if purchased_scoops and not may_install_scoops:
        raise DesignError(f"{configuration_id} hull cannot install fuel scoops")

    installed_parameterized: set[str] = set()
    for position, choice in enumerate(
        _records(
            design.get("parameterized_equipment"),
            "design.parameterized_equipment",
        ),
        1,
    ):
        label = f"design.parameterized_equipment[{position}]"
        equipment_id = _text(choice.get("id"), f"{label}.id")
        if equipment_id in installed_parameterized:
            raise DesignError(
                f"parameterized equipment {equipment_id!r} must use one record"
            )
        installed_parameterized.add(equipment_id)
        try:
            equipment = parameterized_equipment_rules[equipment_id]
        except KeyError as error:
            raise DesignError(
                f"unknown parameterized equipment {equipment_id!r}"
            ) from error
        _require_tl(design_tl, equipment, f"equipment {equipment_id}")
        formula = _text(
            equipment.get("formula"),
            f"rules.parameterized_equipment.{equipment_id}.formula",
        )
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        if formula == "crew-ratio-with-minimum-and-administration":
            parameter = _text(
                equipment.get("parameter"),
                f"rules.parameterized_equipment.{equipment_id}.parameter",
            )
            _check_keys(choice, {"id", "quantity", parameter}, label)
            crew_count = _integer(
                choice.get(parameter),
                f"{label}.{parameter}",
                1,
            )
            crew_per_quarter_ton = _integer(
                equipment.get("crew_per_quarter_ton"),
                f"rules.parameterized_equipment.{equipment_id}."
                "crew_per_quarter_ton",
                1,
            )
            service_volume = (
                (crew_count + crew_per_quarter_ton - 1)
                // crew_per_quarter_ton
                * 250
            )
            service_volume = max(
                service_volume,
                _integer(
                    equipment.get("minimum_millitons"),
                    f"rules.parameterized_equipment.{equipment_id}."
                    "minimum_millitons",
                    1,
                ),
            )
            volume = (
                service_volume
                + _integer(
                    equipment.get("administration_millitons"),
                    f"rules.parameterized_equipment.{equipment_id}."
                    "administration_millitons",
                )
            ) * quantity
        elif formula == "contained-volume-percent":
            parameter = _text(
                equipment.get("parameter"),
                f"rules.parameterized_equipment.{equipment_id}.parameter",
            )
            _check_keys(choice, {"id", "quantity", parameter}, label)
            contained = _integer(choice.get(parameter), f"{label}.{parameter}", 1)
            percent = _integer(
                equipment.get("contained_volume_percent"),
                f"rules.parameterized_equipment.{equipment_id}."
                "contained_volume_percent",
                1,
            )
            volume = (contained * percent + 99) // 100 * quantity
        elif formula == "contained-volume-multiple":
            parameter = _text(
                equipment.get("parameter"),
                f"rules.parameterized_equipment.{equipment_id}.parameter",
            )
            _check_keys(choice, {"id", "quantity", parameter}, label)
            contained = _integer(choice.get(parameter), f"{label}.{parameter}", 1)
            multiple = _integer(
                equipment.get("contained_volume_multiple"),
                f"rules.parameterized_equipment.{equipment_id}."
                "contained_volume_multiple",
                1,
            )
            volume = contained * multiple * quantity
        elif formula == "hull-volume-percent":
            _check_keys(choice, {"id", "quantity"}, label)
            percent = _integer(
                equipment.get("hull_volume_percent"),
                f"rules.parameterized_equipment.{equipment_id}."
                "hull_volume_percent",
                1,
            )
            volume = (hull_millitons * percent + 99) // 100 * quantity
        elif formula == "first-unit-plus-additional-units":
            parameter = _text(
                equipment.get("parameter"),
                f"rules.parameterized_equipment.{equipment_id}.parameter",
            )
            _check_keys(choice, {"id", parameter}, label)
            units = _integer(choice.get(parameter), f"{label}.{parameter}", 1)
            maximum = equipment.get("maximum_units")
            if maximum is not None and units > _integer(
                maximum,
                f"rules.parameterized_equipment.{equipment_id}.maximum_units",
                1,
            ):
                raise DesignError(
                    f"parameterized equipment {equipment_id!r} exceeds "
                    f"maximum {maximum}"
                )
            volume = _integer(
                equipment.get("first_unit_millitons"),
                f"rules.parameterized_equipment.{equipment_id}."
                "first_unit_millitons",
                1,
            ) + (units - 1) * _integer(
                equipment.get("additional_unit_millitons"),
                f"rules.parameterized_equipment.{equipment_id}."
                "additional_unit_millitons",
            )
            price = units * _integer(
                equipment.get("price_credits_per_unit"),
                f"rules.parameterized_equipment.{equipment_id}."
                "price_credits_per_unit",
            )
            totals.add(volume, price)
            line_items.append(
                {
                    "kind": "parameterized-equipment",
                    "id": equipment_id,
                    "quantity": units,
                    "displacement_millitons": volume,
                    "price_credits": price,
                }
            )
            continue
        else:
            raise DesignError(
                f"unknown parameterized-equipment formula {formula!r}"
            )
        price_per_ton = _integer(
            equipment.get("price_credits_per_installed_ton"),
            f"rules.parameterized_equipment.{equipment_id}."
            "price_credits_per_installed_ton",
        )
        price = volume * price_per_ton // 1000
        totals.add(volume, price)
        if equipment_id == "custom-hangar":
            hangar_capacity_millitons += contained * quantity
        line_items.append(
            {
                "kind": "parameterized-equipment",
                "id": equipment_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    for position, choice in enumerate(
        _records(design.get("hangars"), "design.hangars"), 1
    ):
        label = f"design.hangars[{position}]"
        _check_keys(choice, {"id", "contained_millitons", "quantity"}, label)
        hangar_id = _text(choice.get("id"), f"{label}.id")
        contained = _integer(
            choice.get("contained_millitons"),
            f"{label}.contained_millitons",
            1,
        )
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        try:
            hangar = hangar_rules[hangar_id]
        except KeyError as error:
            raise DesignError(f"unknown hangar {hangar_id!r}") from error
        formula = _text(hangar.get("formula"), f"rules.hangar.{hangar_id}.formula")
        if formula != "contained-volume-percent":
            raise DesignError(f"hangar formula {formula!r} is not implemented")
        percent = _integer(
            hangar.get("contained_volume_percent"),
            f"rules.hangar.{hangar_id}.contained_volume_percent",
            1,
        )
        raw_volume = (contained * percent + 99) // 100
        rounding = hangar.get("rounding")
        if rounding != "nearest-whole-ton":
            raise DesignError(f"hangar {hangar_id!r} has unknown rounding")
        installed_volume = ((raw_volume + 500) // 1000) * 1000
        volume = installed_volume * quantity
        price = (
            volume
            * _integer(
                hangar.get("price_credits_per_installed_ton"),
                f"rules.hangar.{hangar_id}.price_credits_per_installed_ton",
            )
            // 1000
        )
        hangar_capacity_millitons += contained * quantity
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "hangar",
                "id": hangar_id,
                "quantity": quantity,
                "contained_millitons": contained * quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    for position, choice in enumerate(
        _records(
            design.get("launch_facilities"),
            "design.launch_facilities",
        ),
        1,
    ):
        label = f"design.launch_facilities[{position}]"
        _check_keys(
            choice,
            {"id", "largest_craft_millitons", "quantity"},
            label,
        )
        facility_id = _text(choice.get("id"), f"{label}.id")
        largest_craft = _integer(
            choice.get("largest_craft_millitons"),
            f"{label}.largest_craft_millitons",
            1,
        )
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        try:
            facility = launch_facility_rules[facility_id]
        except KeyError as error:
            raise DesignError(
                f"unknown launch facility {facility_id!r}"
            ) from error
        formula = _text(
            facility.get("formula"),
            f"rules.launch_facility.{facility_id}.formula",
        )
        if formula != "largest-craft-volume-multiple":
            raise DesignError(
                f"launch facility formula {formula!r} is not implemented"
            )
        multiple = _integer(
            facility.get("contained_volume_multiple"),
            f"rules.launch_facility.{facility_id}."
            "contained_volume_multiple",
            1,
        )
        volume = largest_craft * multiple * quantity
        price = (
            volume
            * _integer(
                facility.get("price_credits_per_installed_ton"),
                f"rules.launch_facility.{facility_id}."
                "price_credits_per_installed_ton",
            )
            // 1000
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "launch-facility",
                "id": facility_id,
                "quantity": quantity,
                "largest_craft_millitons": largest_craft,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    docking_capacity_millitons = 0
    for position, choice in enumerate(
        _records(design.get("docking_clamps"), "design.docking_clamps"), 1
    ):
        label = f"design.docking_clamps[{position}]"
        _check_keys(choice, {"id", "quantity"}, label)
        clamp_id = _text(choice.get("id"), f"{label}.id")
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        try:
            clamp = docking_clamp_rules[clamp_id]
        except KeyError as error:
            raise DesignError(f"unknown docking clamp {clamp_id!r}") from error
        _require_tl(design_tl, clamp, f"docking clamp {clamp_id}")
        maximum_attached = clamp.get("maximum_attached_millitons")
        if maximum_attached is None:
            raise DesignError(
                f"docking clamp {clamp_id!r} has no finite craft capacity"
            )
        volume = quantity * _integer(
            clamp.get("displacement_millitons"),
            f"rules.docking_clamp.{clamp_id}.displacement_millitons",
        )
        price = quantity * _integer(
            clamp.get("price_credits"),
            f"rules.docking_clamp.{clamp_id}.price_credits",
        )
        docking_capacity_millitons += quantity * _integer(
            maximum_attached,
            f"rules.docking_clamp.{clamp_id}.maximum_attached_millitons",
            1,
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "docking-clamp",
                "id": clamp_id,
                "quantity": quantity,
                "attached_capacity_millitons": (
                    quantity * maximum_attached
                ),
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    unused_fire_control = _integer(
        design.get("unused_fire_control_stations", 0),
        "design.unused_fire_control_stations",
    )
    fire_control_volume = unused_fire_control * 1000
    if unused_fire_control:
        totals.add(fire_control_volume, 0)
        line_items.append(
            {
                "kind": "fire-control",
                "id": "unarmed-station",
                "quantity": unused_fire_control,
                "displacement_millitons": fire_control_volume,
                "price_credits": 0,
            }
        )

    hardpoints_used = 0
    mount_count = 0
    for position, choice in enumerate(_records(design.get("mounts"), "design.mounts"), 1):
        _check_keys(
            choice,
            {"id", "weapons", "quantity", "pop_up", "fixed"},
            f"design.mounts[{position}]",
        )
        mount_id = _text(choice.get("id"), f"design.mounts[{position}].id")
        quantity = _integer(
            choice.get("quantity", 1),
            f"design.mounts[{position}].quantity",
            1,
        )
        try:
            mount = mount_rules[mount_id]
        except KeyError as error:
            raise DesignError(f"unknown mount {mount_id!r}") from error
        _require_tl(design_tl, mount, f"mount {mount_id}")
        weapons = choice.get("weapons")
        if not isinstance(weapons, list) or any(
            not isinstance(weapon, str) for weapon in weapons
        ):
            raise DesignError(f"design.mounts[{position}].weapons must be text list")
        capacity = _integer(
            mount.get("weapon_capacity"), f"rules.mount.{mount_id}.weapon_capacity", 1
        )
        if len(weapons) > capacity:
            raise DesignError(f"mount {position} exceeds its {capacity}-weapon capacity")
        mount_volume = _integer(
            mount.get("displacement_millitons"),
            f"rules.mount.{mount_id}.displacement_millitons",
        )
        mount_price = _integer(
            mount.get("price_credits"), f"rules.mount.{mount_id}.price_credits"
        )
        pop_up = choice.get("pop_up", False)
        fixed = choice.get("fixed", False)
        if not isinstance(pop_up, bool) or not isinstance(fixed, bool):
            raise DesignError(f"design.mounts[{position}] flags must be boolean")
        if pop_up and fixed:
            raise DesignError("a mount cannot be both pop-up and fixed")
        if pop_up:
            if design_tl < 10:
                raise DesignError("pop-up turret requires TL10")
            mount_volume = 2000
            mount_price += 1000000
        if fixed:
            mount_volume = 0
            mount_price //= 2
        weapon_price = 0
        weapon_counts: dict[str, int] = {}
        for weapon_id in weapons:
            try:
                weapon = weapon_rules[weapon_id]
            except KeyError as error:
                raise DesignError(f"unknown turret weapon {weapon_id!r}") from error
            _require_tl(design_tl, weapon, f"weapon {weapon_id}")
            weapon_price += _integer(
                weapon.get("price_credits"),
                f"rules.weapon.{weapon_id}.price_credits",
            )
            weapon_counts[weapon_id] = weapon_counts.get(weapon_id, 0) + 1
            maximum_formula = weapon.get("maximum_count_formula")
            if maximum_formula == "mount-capacity-minus-one":
                if weapon_counts[weapon_id] > capacity - 1:
                    raise DesignError(
                        f"mount {position} may carry at most {capacity - 1} "
                        f"{weapon_id!r} weapons"
                    )
            elif maximum_formula is not None:
                raise DesignError(
                    f"unknown weapon maximum formula {maximum_formula!r}"
                )
        total_price = (mount_price + weapon_price) * quantity
        total_volume = mount_volume * quantity
        totals.add(total_volume, total_price)
        hardpoints_used += quantity * _integer(
            mount.get("hardpoints"), f"rules.mount.{mount_id}.hardpoints", 1
        )
        mount_count += quantity
        fire_control_volume += total_volume
        line_items.append(
            {
                "kind": "weapon-mount",
                "id": mount_id,
                "quantity": quantity,
                "weapons": weapons,
                "displacement_millitons": total_volume,
                "price_credits": total_price,
            }
        )

    bay_count = 0
    for position, choice in enumerate(
        _records(design.get("barbettes"), "design.barbettes"), 1
    ):
        label = f"design.barbettes[{position}]"
        _check_keys(choice, {"id", "quantity"}, label)
        barbette_id = _text(choice.get("id"), f"{label}.id")
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        try:
            barbette = barbette_rules[barbette_id]
        except KeyError as error:
            raise DesignError(f"unknown barbette {barbette_id!r}") from error
        _require_tl(design_tl, barbette, f"barbette {barbette_id}")
        volume = quantity * _integer(
            barbette.get("displacement_millitons"),
            f"rules.barbette.{barbette_id}.displacement_millitons",
        )
        price = quantity * _integer(
            barbette.get("price_credits"),
            f"rules.barbette.{barbette_id}.price_credits",
        )
        hardpoints = quantity * _integer(
            barbette.get("hardpoints"),
            f"rules.barbette.{barbette_id}.hardpoints",
            1,
        )
        totals.add(volume, price)
        hardpoints_used += hardpoints
        fire_control_volume += volume
        bay_count += quantity
        line_items.append(
            {
                "kind": "barbette",
                "id": barbette_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    point_defense_count = 0
    for position, choice in enumerate(
        _records(design.get("point_defense"), "design.point_defense"), 1
    ):
        label = f"design.point_defense[{position}]"
        _check_keys(choice, {"mount_id", "weapon_id", "quantity"}, label)
        mount_id = _text(choice.get("mount_id"), f"{label}.mount_id")
        weapon_id = _text(choice.get("weapon_id"), f"{label}.weapon_id")
        quantity = _integer(choice.get("quantity", 1), f"{label}.quantity", 1)
        try:
            point_defense_mount = point_defense_mount_rules[mount_id]
        except KeyError as error:
            raise DesignError(
                f"unknown point-defense mount {mount_id!r}"
            ) from error
        try:
            point_defense_weapon = point_defense_weapon_rules[weapon_id]
        except KeyError as error:
            raise DesignError(
                f"unknown point-defense weapon {weapon_id!r}"
            ) from error
        _require_tl(
            design_tl, point_defense_mount, f"point-defense mount {mount_id}"
        )
        _require_tl(
            design_tl, point_defense_weapon, f"point-defense weapon {weapon_id}"
        )
        capacity = _integer(
            point_defense_mount.get("hardpoint_node_capacity"),
            f"rules.point_defense_mount.{mount_id}.hardpoint_node_capacity",
            1,
        )
        if capacity != 1:
            raise DesignError(
                f"point-defense mount {mount_id!r} capacity {capacity} "
                "is not implemented"
            )
        volume = quantity * (
            _integer(
                point_defense_mount.get("displacement_millitons"),
                f"rules.point_defense_mount.{mount_id}.displacement_millitons",
            )
            + _integer(
                point_defense_weapon.get("displacement_millitons", 0),
                f"rules.point_defense_weapon.{weapon_id}.displacement_millitons",
            )
        )
        price = quantity * (
            _integer(
                point_defense_mount.get("price_credits"),
                f"rules.point_defense_mount.{mount_id}.price_credits",
            )
            + _integer(
                point_defense_weapon.get("price_credits"),
                f"rules.point_defense_weapon.{weapon_id}.price_credits",
            )
        )
        totals.add(volume, price)
        point_defense_count += quantity
        line_items.append(
            {
                "kind": "point-defense",
                "id": mount_id,
                "weapon_id": weapon_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    screen_count = 0
    selected_screens: set[str] = set()
    for collection_name, registry in (("bays", bay_rules), ("screens", screen_rules)):
        for position, choice in enumerate(
            _records(design.get(collection_name), f"design.{collection_name}"), 1
        ):
            _check_keys(choice, {"id", "quantity"}, f"design.{collection_name}[{position}]")
            item_id = _text(
                choice.get("id"), f"design.{collection_name}[{position}].id"
            )
            quantity = _integer(
                choice.get("quantity", 1),
                f"design.{collection_name}[{position}].quantity",
                1,
            )
            try:
                item = registry[item_id]
            except KeyError as error:
                raise DesignError(f"unknown {collection_name[:-1]} {item_id!r}") from error
            _require_tl(design_tl, item, f"{collection_name[:-1]} {item_id}")
            if collection_name == "bays":
                minimum_power = item.get("minimum_power_plant_rating")
                if minimum_power is not None:
                    plant_rating = performance_rows[hull_tons][power_code]
                    if plant_rating < _integer(
                        minimum_power,
                        f"rules.bays.{item_id}.minimum_power_plant_rating",
                        1,
                    ):
                        raise DesignError(
                            f"bay {item_id!r} requires power-plant rating "
                            f"{minimum_power}"
                        )
            else:
                selected_screens.add(item_id)
                minimum_code = item.get("minimum_hull_code")
                if minimum_code is not None:
                    minimum_tons = hull_tons_by_code.get(str(minimum_code))
                    if minimum_tons is None or hull_tons < minimum_tons:
                        raise DesignError(
                            f"screen {item_id!r} requires hull code "
                            f"{minimum_code} or larger"
                        )
                maximum_code = item.get("maximum_hull_code")
                if maximum_code is not None:
                    maximum_tons = hull_tons_by_code.get(str(maximum_code))
                    if maximum_tons is None or hull_tons > maximum_tons:
                        raise DesignError(
                            f"screen {item_id!r} requires hull code "
                            f"{maximum_code} or smaller"
                        )
            volume = quantity * _integer(
                item.get("displacement_millitons"),
                f"rules.{collection_name}.{item_id}.displacement_millitons",
            )
            price = quantity * _integer(
                item.get("price_credits"),
                f"rules.{collection_name}.{item_id}.price_credits",
            )
            totals.add(volume, price)
            if collection_name == "bays":
                bay_count += quantity
                hardpoints_used += quantity * _integer(
                    item.get("hardpoints"),
                    f"rules.bays.{item_id}.hardpoints",
                    1,
                )
                fire_control_volume += quantity * 1000
            else:
                screen_count += quantity
            line_items.append(
                {
                    "kind": collection_name[:-1],
                    "id": item_id,
                    "quantity": quantity,
                    "displacement_millitons": volume,
                    "price_credits": price,
                }
            )

    for option_id in installed_hull_options:
        exclusions = hull_options[option_id].get("mutually_exclusive_with", [])
        if not isinstance(exclusions, list):
            raise DesignError(
                f"rules.hull_option.{option_id}.mutually_exclusive_with "
                "must be a list"
            )
        conflict = selected_screens.intersection(exclusions)
        if conflict:
            raise DesignError(
                f"hull option {option_id!r} conflicts with "
                f"{sorted(conflict)[0]!r}"
            )
    for screen_id in selected_screens:
        exclusions = screen_rules[screen_id].get("mutually_exclusive_with", [])
        if not isinstance(exclusions, list):
            raise DesignError(
                f"rules.screen.{screen_id}.mutually_exclusive_with must be a list"
            )
        conflict = installed_hull_options.intersection(exclusions)
        if conflict:
            raise DesignError(
                f"screen {screen_id!r} conflicts with {sorted(conflict)[0]!r}"
            )

    ammunition_volume = 0
    for position, choice in enumerate(
        _records(design.get("ammunition"), "design.ammunition"), 1
    ):
        _check_keys(choice, {"id", "quantity"}, f"design.ammunition[{position}]")
        ammunition_id = _text(
            choice.get("id"), f"design.ammunition[{position}].id"
        )
        quantity = _integer(
            choice.get("quantity"), f"design.ammunition[{position}].quantity", 1
        )
        try:
            ammunition = ammunition_rules[ammunition_id]
        except KeyError as error:
            raise DesignError(f"unknown ammunition {ammunition_id!r}") from error
        _require_tl(design_tl, ammunition, f"ammunition {ammunition_id}")
        volume_scales = (
            ("units_per_ton", 1000),
            ("units_per_five_tons", 5000),
            ("units_per_twenty_tons", 20000),
        )
        present_scales = [
            (field, millitons)
            for field, millitons in volume_scales
            if field in ammunition
        ]
        if len(present_scales) != 1:
            raise DesignError(
                f"ammunition {ammunition_id!r} must define exactly one "
                "volume scale"
            )
        scale_field, scale_millitons = present_scales[0]
        units_per_scale = _integer(
            ammunition.get(scale_field),
            f"rules.ammunition.{ammunition_id}.{scale_field}",
            1,
        )
        volume = (
            quantity * scale_millitons + units_per_scale - 1
        ) // units_per_scale

        pack_units = 1
        if "price_credits_per_unit" in ammunition:
            price = quantity * _integer(
                ammunition.get("price_credits_per_unit"),
                f"rules.ammunition.{ammunition_id}.price_credits_per_unit",
            )
        elif "price_credits_per_pack" in ammunition:
            pack_units = _integer(
                ammunition.get("pack_units"),
                f"rules.ammunition.{ammunition_id}.pack_units",
                1,
            )
            if quantity % pack_units:
                raise DesignError(
                    f"ammunition {ammunition_id!r} must be purchased in "
                    f"packs of {pack_units}"
                )
            price = quantity // pack_units * _integer(
                ammunition.get("price_credits_per_pack"),
                f"rules.ammunition.{ammunition_id}.price_credits_per_pack",
            )
        else:
            raise DesignError(
                f"ammunition {ammunition_id!r} lacks a price formula"
            )
        totals.add(volume, price, discountable=False)
        ammunition_volume += volume
        line_items.append(
            {
                "kind": "ammunition",
                "id": ammunition_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
                "pack_units": pack_units,
            }
        )

    selected_magazine_options = _text_list(
        design.get("magazine_options", []),
        "design.magazine_options",
    )
    for option_id in selected_magazine_options:
        try:
            option = magazine_option_rules[option_id]
        except KeyError as error:
            raise DesignError(f"unknown magazine option {option_id!r}") from error
        price = (
            ammunition_volume
            * _integer(
                option.get("price_credits_per_installed_ton"),
                f"rules.magazine_option.{option_id}."
                "price_credits_per_installed_ton",
            )
            // 1000
        )
        totals.add(0, price)
        line_items.append(
            {
                "kind": "magazine-option",
                "id": option_id,
                "quantity": 1,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )

    install_protected_system("ordnance-magazines", ammunition_volume)
    expected_protection = {
        (option_id, target)
        for option_id, _, target in deferred_structural_options
    }
    unsupported_protection = expected_protection - installed_protection
    if unsupported_protection:
        _, target = sorted(unsupported_protection)[0]
        raise DesignError(
            f"armored bulkheads cannot yet protect {target!r}"
        )

    maximum_hardpoints = hull_tons // _integer(
        _table(rules.get("derived"), "rules.derived").get("hardpoint_tons"),
        "rules.derived.hardpoint_tons",
        1,
    )
    if hardpoints_used + unused_fire_control > maximum_hardpoints:
        raise DesignError(
            f"design uses/reserves {hardpoints_used + unused_fire_control} "
            f"hardpoints; hull permits {maximum_hardpoints}"
        )

    crew_manifest: dict[str, int] = {}
    for position, record in enumerate(_records(design.get("crew"), "design.crew"), 1):
        label = f"design.crew[{position}]"
        _check_keys(record, {"role", "quantity"}, label)
        role = _text(record.get("role"), f"{label}.role")
        quantity = _integer(record.get("quantity"), f"{label}.quantity", 1)
        if role not in crew_roles:
            raise DesignError(f"unknown crew role {role!r}")
        if role in crew_manifest:
            raise DesignError(f"crew role {role!r} must use one quantity record")
        crew_manifest[role] = quantity
    minimum_formula_values = {
        "none": 0,
        "one": 1,
        "one-with-drives": 1,
        "one-per-turret": mount_count,
        "one-per-bay": bay_count,
        "one-per-screen": screen_count,
    }
    minimum_by_role: dict[str, int] = {}
    for role, rule in crew_roles.items():
        formula = _text(
            rule.get("minimum_formula"),
            f"rules.crew_role.{role}.minimum_formula",
        )
        try:
            required = minimum_formula_values[formula]
        except KeyError as error:
            raise DesignError(f"unknown minimum crew formula {formula!r}") from error
        if required:
            minimum_by_role[role] = required
    for role, required in minimum_by_role.items():
        actual = crew_manifest.get(role, 0)
        if actual < required:
            raise DesignError(
                f"crew role {role!r} has {actual}; fitted ship requires {required}"
            )
    minimum_crew = sum(minimum_by_role.values())
    crew_total = sum(crew_manifest.values())
    if crew_total > crew_accommodation_capacity:
        raise DesignError(
            f"{crew_total} crew exceed accommodation capacity of "
            f"{crew_accommodation_capacity}"
        )
    crew_needing_shared_accommodation = max(
        0, crew_total - dedicated_crew_accommodation_capacity
    )
    for crew_capacity, passenger_berths, quantity, _equipment_id in sorted(
        shared_accommodations, key=lambda item: (-item[0], item[3])
    ):
        occupied_units = min(
            quantity,
            (crew_needing_shared_accommodation + crew_capacity - 1)
            // crew_capacity,
        )
        crew_needing_shared_accommodation = max(
            0,
            crew_needing_shared_accommodation - occupied_units * crew_capacity,
        )
        passenger_accommodation_berths += (
            quantity - occupied_units
        ) * passenger_berths
    if crew_needing_shared_accommodation:
        raise DesignError("crew accommodation allocation did not converge")

    cargo = _integer(design.get("cargo_millitons", 0), "design.cargo_millitons")
    totals.add(cargo, 0)
    if cargo:
        line_items.append(
            {
                "kind": "cargo",
                "id": "cargo-hold",
                "quantity": 1,
                "displacement_millitons": cargo,
                "price_credits": 0,
            }
        )

    carried_craft_count = 0
    carried_craft_price = 0
    carried_craft_displacement = 0
    carried_tags: set[str] = set()
    for position, record in enumerate(
        _records(design.get("carried_craft"), "design.carried_craft"), 1
    ):
        label = f"design.carried_craft[{position}]"
        _check_keys(record, {"tag", "quantity"}, label)
        tag = _text(record.get("tag"), f"{label}.tag")
        quantity = _integer(record.get("quantity"), f"{label}.quantity", 1)
        if tag in carried_tags:
            raise DesignError(f"carried craft {tag!r} must use one quantity record")
        carried_tags.add(tag)
        if carried_craft_results is None or tag not in carried_craft_results:
            raise DesignError(f"carried craft {tag!r} has no evaluated catalog entry")
        craft = carried_craft_results[tag]
        craft_displacement = _integer(
            craft.get("hull_millitons"),
            f"carried craft {tag}.hull_millitons",
            1,
        ) * quantity
        price = _integer(
            craft.get("construction_price_credits"),
            f"carried craft {tag}.construction_price_credits",
        ) * quantity
        carried_craft_count += quantity
        carried_craft_price += price
        carried_craft_displacement += craft_displacement
        # A carried craft is already a completed standard or custom design.
        # Its own construction treatment has already been applied, so the
        # parent vessel's standard-design discount must not be applied again.
        totals.add(0, price, discountable=False)
        line_items.append(
            {
                "kind": "carried-craft",
                "id": tag,
                "quantity": quantity,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )
    inferred_external_load = max(
        0, carried_craft_displacement - hangar_capacity_millitons
    )
    if inferred_external_load > docking_capacity_millitons:
        raise DesignError(
            f"carried craft require {carried_craft_displacement} millitons but "
            f"hangars provide {hangar_capacity_millitons} and docking clamps "
            f"provide {docking_capacity_millitons}"
        )
    if inferred_external_load != external_load_millitons:
        raise DesignError(
            f"external_load_millitons is {external_load_millitons}, but "
            f"carried-craft stowage implies {inferred_external_load}"
        )
    if totals.displacement_millitons != hull_millitons:
        relation = (
            "over"
            if totals.displacement_millitons > hull_millitons
            else "under"
        )
        difference = abs(totals.displacement_millitons - hull_millitons)
        raise DesignError(
            f"design is {difference} millitons {relation} its {hull_millitons}-"
            "milliton hull; no displacement adjustment is permitted"
        )

    discount_percent = 0
    if standard_design:
        construction = _table(rules.get("construction"), "rules.construction")
        discount_percent = _integer(
            construction.get("standard_design_discount_percent"),
            "rules.construction.standard_design_discount_percent",
        )
    discounted_credits = (
        totals.discountable_credits * (100 - discount_percent) // 100
    )
    architect_fee_credits = 0
    if architect_fee:
        plan_percent = _integer(
            _table(rules.get("construction"), "rules.construction").get(
                "new_design_plan_percent"
            ),
            "rules.construction.new_design_plan_percent",
        )
        architect_fee_credits = (
            totals.discountable_credits + totals.undiscounted_credits
        ) * plan_percent // 100
    final_price = (
        discounted_credits
        + totals.undiscounted_credits
        + architect_fee_credits
    )

    derived = _table(rules.get("derived"), "rules.derived")
    hull_divisor = _integer(
        derived.get("hull_points_per_tons"),
        "rules.derived.hull_points_per_tons",
        1,
    )
    structure_divisor = _integer(
        derived.get("structure_points_per_tons"),
        "rules.derived.structure_points_per_tons",
        1,
    )
    result: dict[str, Any] = {
        "design_id": _text(design.get("design_id"), "design.design_id"),
        "ruleset_id": ruleset_id,
        "hull_millitons": hull_millitons,
        "effective_displacement_millitons": effective_displacement_millitons,
        "accounted_displacement_millitons": totals.displacement_millitons,
        "hull_points": hull_tons // hull_divisor,
        "structure_points": (hull_tons + structure_divisor - 1)
        // structure_divisor
        + structure_bonus,
        "armor_points": armor_points,
        "jump_rating": jump_rating,
        "thrust_g": drive_performance["maneuver"],
        "fuel_millitons": total_fuel,
        "jump_fuel_millitons": jump_fuel,
        "power_fuel_millitons": power_fuel,
        "hardpoints": maximum_hardpoints,
        "hardpoints_used": hardpoints_used,
        "point_defense_nodes": point_defense_count,
        "fire_control_millitons": fire_control_volume,
        "minimum_crew": minimum_crew,
        "crew": crew_total,
        "crew_accommodation_capacity": crew_accommodation_capacity,
        "passenger_accommodation_berths": passenger_accommodation_berths,
        "provision_capacity_persons": provision_capacity_persons,
        "low_berths": low_berths,
        "monthly_life_support_credits": monthly_life_support_credits,
        "carried_craft_count": carried_craft_count,
        "carried_craft_displacement_millitons": carried_craft_displacement,
        "carried_craft_price_credits": carried_craft_price,
        "hangar_capacity_millitons": hangar_capacity_millitons,
        "docking_capacity_millitons": docking_capacity_millitons,
        "construction_weeks": _integer(
            hull.get("construction_weeks"),
            f"rules.hull.{hull_id}.construction_weeks",
            1,
        ),
        "pre_discount_price_credits": (
            totals.discountable_credits + totals.undiscounted_credits
        ),
        "discount_credits": totals.discountable_credits - discounted_credits,
        "architect_fee_credits": architect_fee_credits,
        "construction_price_credits": final_price,
        "line_items": line_items,
    }
    if "thrust_g" in design:
        displayed_thrust = _integer(design["thrust_g"], "design.thrust_g", 1)
        if displayed_thrust != result["thrust_g"]:
            raise DesignError(
                f"design.thrust_g={displayed_thrust} does not match "
                f"rules-derived value {result['thrust_g']}"
            )

    assertions = _table(design.get("assertions", {}), "design.assertions")
    allowed_assertions = set(result) - {"line_items", "design_id", "ruleset_id"}
    _check_keys(assertions, allowed_assertions, "design.assertions")
    for field, expected in assertions.items():
        expected_integer = _integer(expected, f"design.assertions.{field}")
        actual = result[field]
        if actual != expected_integer:
            raise DesignError(
                f"published assertion {field}={expected_integer} does not match "
                f"rules-derived value {actual}"
            )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("design", type=Path)
    parser.add_argument("--rules", type=Path)
    parser.add_argument(
        "--small-craft-rules",
        type=Path,
        default=DEFAULT_SMALL_CRAFT_RULES,
    )
    parser.add_argument("--pretty", action="store_true")
    arguments = parser.parse_args(argv)
    try:
        design = load_toml(arguments.design)
        if arguments.rules is not None:
            core_rules = load_toml(arguments.rules)
        elif design.get("ruleset_id") == "cepheus-trader.shipbuilding":
            from shipbuilding_rules import (
                RuleCompositionError,
                compose_shipbuilding_rules,
            )

            try:
                core_rules = compose_shipbuilding_rules(
                    DEFAULT_COMPOSED_RULES_DIR
                )
            except RuleCompositionError as error:
                raise DesignError(str(error)) from error
        else:
            core_rules = load_toml(DEFAULT_RULES)
        if design.get("ruleset_id") == "ce-srd-2016.small-craft":
            from small_craft_design import evaluate_small_craft

            result = evaluate_small_craft(
                core_rules,
                load_toml(arguments.small_craft_rules),
                design,
            )
        else:
            result = evaluate(core_rules, design)
    except DesignError as error:
        print(f"ship design error: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2 if arguments.pretty else None, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
