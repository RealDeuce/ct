#!/usr/bin/env python3
"""Validate the canonical new-player starting-offer design mapping."""

from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path
import re
import sys
import tomllib
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OFFERS = ROOT / "catalog" / "starting-offers.toml"
DEFAULT_SHIPS = ROOT / "catalog" / "ships"
OFFER_TAG = re.compile(r"starting-offer-([1-9][0-9]*)\Z")


class StartingOfferValidationError(ValueError):
    pass


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as source:
        return tomllib.load(source)


def text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise StartingOfferValidationError(f"{label} must be non-empty text")
    return value


def integer(value: object, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise StartingOfferValidationError(
            f"{label} must be an integer >= {minimum}"
        )
    return value


def validate(offers_path: Path, ships_dir: Path) -> str:
    registry = load_toml(offers_path)
    allowed = {
        "schema_version",
        "catalog_revision",
        "offer_count",
        "open_game_content_designations",
        "offer",
    }
    extra = set(registry) - allowed
    if extra:
        raise StartingOfferValidationError(
            f"{offers_path}: unknown field(s): {', '.join(sorted(extra))}"
        )
    if registry.get("schema_version") != 1:
        raise StartingOfferValidationError(
            f"{offers_path}: schema_version must be 1"
        )
    integer(
        registry.get("catalog_revision"),
        f"{offers_path}.catalog_revision",
        1,
    )
    designations = registry.get("open_game_content_designations")
    if (
        not isinstance(designations, list)
        or not designations
        or any(not isinstance(item, str) or not item.strip() for item in designations)
    ):
        raise StartingOfferValidationError(
            f"{offers_path}: open_game_content_designations must be non-empty"
        )
    records = registry.get("offer")
    if not isinstance(records, list):
        raise StartingOfferValidationError(f"{offers_path}: offer must be an array")
    if registry.get("offer_count") != len(records) or len(records) != 27:
        raise StartingOfferValidationError(
            f"{offers_path}: offer_count must describe exactly 27 offers"
        )

    ships: dict[str, dict[str, Any]] = {}
    for path in ships_dir.glob("ship-*.toml"):
        design = load_toml(path)
        catalog = design.get("catalog")
        if isinstance(catalog, dict):
            ships[text(catalog.get("tag"), f"{path}.catalog.tag")] = design

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
    package_kinds = {
        "trader": "independent-commercial-charter",
        "privateer": "private-armed-charter",
        "navy": "public-service-commission",
    }
    cell_careers: defaultdict[int, set[str]] = defaultdict(set)
    cell_ships: defaultdict[int, set[str]] = defaultdict(set)
    seen_offer_ids: set[int] = set()
    selected_ship_tags: set[str] = set()

    for position, record in enumerate(records, 1):
        label = f"{offers_path}: offer {position}"
        if not isinstance(record, dict) or set(record) != {
            "offer_id",
            "tag",
            "home_path_id",
            "trade_emphasis",
            "institutional_order",
            "career",
            "package_kind",
            "ship_tag",
            "selection_rationale",
        }:
            raise StartingOfferValidationError(f"{label} has invalid fields")
        offer_id = integer(record.get("offer_id"), f"{label}.offer_id", 1)
        offer_tag = text(record.get("tag"), f"{label}.tag")
        match = OFFER_TAG.fullmatch(offer_tag)
        if match is None or int(match.group(1)) != offer_id:
            raise StartingOfferValidationError(
                f"{label}: offer_id and tag disagree"
            )
        if offer_id != position or offer_id in seen_offer_ids:
            raise StartingOfferValidationError(
                f"{label}: offer IDs must be unique and sequential"
            )
        seen_offer_ids.add(offer_id)

        home_path_id = integer(
            record.get("home_path_id"),
            f"{label}.home_path_id",
            1,
        )
        axes = (
            text(record.get("trade_emphasis"), f"{label}.trade_emphasis"),
            text(
                record.get("institutional_order"),
                f"{label}.institutional_order",
            ),
        )
        if expected_axes.get(home_path_id) != axes:
            raise StartingOfferValidationError(
                f"{label}: home path and polity axes disagree"
            )
        career = text(record.get("career"), f"{label}.career")
        if package_kinds.get(career) != record.get("package_kind"):
            raise StartingOfferValidationError(
                f"{label}: career and package_kind disagree"
            )
        if career in cell_careers[home_path_id]:
            raise StartingOfferValidationError(
                f"{label}: duplicate career in home-path-{home_path_id}"
            )
        cell_careers[home_path_id].add(career)

        ship_tag = text(record.get("ship_tag"), f"{label}.ship_tag")
        design = ships.get(ship_tag)
        if design is None:
            raise StartingOfferValidationError(
                f"{label}: unknown catalog design {ship_tag!r}"
            )
        catalog = design["catalog"]
        if (
            catalog.get("status") != "active"
            or catalog.get("vessel_kind") != "starship"
            or catalog.get("progression_stage") not in {"starter", "light"}
            or "jump" not in design.get("drives", {})
        ):
            raise StartingOfferValidationError(
                f"{label}: selected design must be an active Jump-capable "
                "starter or light starship"
            )
        fuel = design.get("fuel")
        if (
            not isinstance(fuel, dict)
            or integer(
                fuel.get("jump_distance"),
                f"{label}: selected design fuel.jump_distance",
            )
            < 2
            or integer(
                fuel.get("jump_count"),
                f"{label}: selected design fuel.jump_count",
            )
            < 1
        ):
            raise StartingOfferValidationError(
                f"{label}: selected design must be fitted and fueled for "
                "at least one Jump-2 transit"
            )
        if ship_tag in cell_ships[home_path_id]:
            raise StartingOfferValidationError(
                f"{label}: one cell cannot offer the same design twice"
            )
        cell_ships[home_path_id].add(ship_tag)
        selected_ship_tags.add(ship_tag)

        text(record.get("selection_rationale"), f"{label}.selection_rationale")

    expected_careers = set(package_kinds)
    if set(cell_careers) != set(expected_axes) or any(
        careers != expected_careers for careers in cell_careers.values()
    ):
        raise StartingOfferValidationError(
            f"{offers_path}: every path must offer trader, privateer, and navy"
        )
    return (
        f"validated 27 starting offers in 9 polity cells using "
        f"{len(selected_ship_tags)} catalog designs"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--offers", type=Path, default=DEFAULT_OFFERS)
    parser.add_argument("--ships", type=Path, default=DEFAULT_SHIPS)
    args = parser.parse_args()
    try:
        print(validate(args.offers, args.ships))
    except (OSError, tomllib.TOMLDecodeError, StartingOfferValidationError) as error:
        print(f"starting-offer error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
