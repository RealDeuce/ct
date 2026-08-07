#!/usr/bin/env python3

from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from ship_design import DesignError, load_toml  # noqa: E402
from shipbuilding_rules import compose_shipbuilding_rules  # noqa: E402
from small_craft_design import evaluate_small_craft  # noqa: E402


CORE = ROOT / "catalog/shipbuilding/ce-core.toml"
SMALL = ROOT / "catalog/shipbuilding/ce-small-craft.toml"
LAUNCH = ROOT / "catalog/ships/ship-194.toml"
EXTENSION = ROOT / "catalog/shipbuilding/af3-small-compatible.toml"
SOURCE_BOAT = ROOT / "catalog/ships/ship-125.toml"


class SmallCraftDesignTests(unittest.TestCase):
    def setUp(self) -> None:
        self.core = load_toml(CORE)
        self.small = load_toml(SMALL)
        self.launch = load_toml(LAUNCH)
        self.composed = compose_shipbuilding_rules(
            ROOT / "catalog" / "shipbuilding"
        )

    def evaluate(self, design: dict[str, object]) -> dict[str, object]:
        return evaluate_small_craft(self.core, self.small, design)

    def test_launch_matches_published_design(self) -> None:
        result = self.evaluate(self.launch)
        self.assertEqual(result["accounted_displacement_millitons"], 20_000)
        self.assertEqual(result["construction_price_credits"], 4_797_000)
        self.assertEqual(result["thrust_g"], 1)

    def test_small_craft_has_exactly_one_hardpoint(self) -> None:
        design = deepcopy(self.launch)
        del design["assertions"]
        design["unused_fire_control_stations"] = 2
        with self.assertRaisesRegex(DesignError, "only one hardpoint"):
            self.evaluate(design)

    def test_small_power_plant_must_cover_maneuver_drive(self) -> None:
        design = deepcopy(self.launch)
        del design["assertions"]
        design["drives"]["maneuver"] = "sB"
        with self.assertRaisesRegex(DesignError, "rated below"):
            self.evaluate(design)

    def test_small_craft_cannot_install_jump_software(self) -> None:
        design = deepcopy(self.launch)
        del design["assertions"]
        design["software"] = [{"id": "jump-control", "level": 1}]
        with self.assertRaisesRegex(DesignError, "cannot install Jump"):
            self.evaluate(design)

    def test_volume_adjustments_are_not_permitted(self) -> None:
        design = deepcopy(self.launch)
        del design["assertions"]
        design["cargo_millitons"] -= 100
        with self.assertRaisesRegex(DesignError, "differs from hull by -100"):
            self.evaluate(design)

    def test_extension_whole_point_armor_is_rules_derived(self) -> None:
        result = evaluate_small_craft(
            self.core,
            self.small,
            load_toml(SOURCE_BOAT),
            extension=load_toml(EXTENSION),
            component_rules=self.composed,
        )
        self.assertEqual(result["armor_points"], 1)
        armor = next(
            item for item in result["line_items"] if item["kind"] == "armor"
        )
        self.assertEqual(armor["displacement_millitons"], 750)
        self.assertEqual(armor["price_credits"], 32_500)

    def test_small_craft_can_install_rule_derived_medical_ward(self) -> None:
        design = deepcopy(self.launch)
        del design["assertions"]
        design["ruleset_id"] = "cepheus-trader.small-craft"
        design["source_ids"] = [
            "cepheus-engine-srd-2016",
            "ship-source-a-f",
            "cepheus-trader-2026",
        ]
        design["cargo_millitons"] -= 4_000
        design["parameterized_equipment"] = [
            {"id": "medical-ward", "beds": 3}
        ]
        result = evaluate_small_craft(
            self.core,
            self.small,
            design,
            extension=load_toml(EXTENSION),
            component_rules=self.composed,
        )
        ward = next(
            item
            for item in result["line_items"]
            if item["id"] == "medical-ward"
        )
        self.assertEqual(ward["displacement_millitons"], 4_000)
        self.assertEqual(ward["price_credits"], 1_500_000)


if __name__ == "__main__":
    unittest.main()
