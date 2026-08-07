#!/usr/bin/env python3

from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from ship_design import DesignError, evaluate, load_toml  # noqa: E402
from shipbuilding_rules import compose_shipbuilding_rules  # noqa: E402
from small_craft_design import evaluate_small_craft  # noqa: E402


RULES_DIR = ROOT / "catalog" / "shipbuilding"
MERCHANT = ROOT / "catalog" / "ships" / "ship-192.toml"
FRONTIER = ROOT / "catalog" / "ships" / "ship-193.toml"
PATROL = ROOT / "catalog" / "ships" / "ship-95.toml"
BOAT = ROOT / "catalog" / "ships" / "ship-18.toml"


class ComposedShipbuildingRuleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.rules = compose_shipbuilding_rules(RULES_DIR)

    def _convert_design(self, path: Path) -> dict:
        design = deepcopy(load_toml(path))
        design["ruleset_id"] = "cepheus-trader.shipbuilding"
        design["source_ids"] = self.rules["source_ids"]
        design.pop("assertions", None)
        return design

    def test_composition_contains_active_components_only(self) -> None:
        equipment = {record["id"] for record in self.rules["equipment"]}
        bays = {record["id"] for record in self.rules["bay"]}
        self.assertIn("gymnasium", equipment)
        self.assertNotIn("mineralogy-suite", equipment)
        self.assertNotIn("fusion-power-fuel-cells", equipment)
        self.assertNotIn("railgun-bay-500", bays)
        self.assertNotIn("tech-heavy-railgun-bay", bays)

    def test_superseded_component_is_replaced(self) -> None:
        bays = {record["id"] for record in self.rules["bay"]}
        self.assertNotIn("particle-beam-bay-500", bays)
        self.assertIn("tech-heavy-particle-beam-bay", bays)

    def test_fixed_expansion_equipment_is_executable(self) -> None:
        design = self._convert_design(MERCHANT)
        design["cargo_millitons"] -= 4000
        design["equipment"].append({"id": "gymnasium", "quantity": 1})
        result = evaluate(self.rules, design)
        self.assertEqual(result["accounted_displacement_millitons"], 200_000)
        self.assertEqual(result["construction_price_credits"], 52_119_000)

    def test_reload_pack_price_and_volume_are_executable(self) -> None:
        design = self._convert_design(FRONTIER)
        design["cargo_millitons"] -= 1000
        design["ammunition"].append(
            {"id": "turret-railgun-basic", "quantity": 30}
        )
        result = evaluate(self.rules, design)
        item = next(
            line
            for line in result["line_items"]
            if line["id"] == "turret-railgun-basic"
        )
        self.assertEqual(item["displacement_millitons"], 1000)
        self.assertEqual(item["price_credits"], 5000)
        self.assertEqual(result["construction_price_credits"], 85_564_000)

    def test_partial_reload_pack_is_rejected(self) -> None:
        design = self._convert_design(FRONTIER)
        design["cargo_millitons"] -= 34
        design["ammunition"].append(
            {"id": "turret-railgun-basic", "quantity": 1}
        )
        with self.assertRaisesRegex(DesignError, "packs of 30"):
            evaluate(self.rules, design)

    def test_patrol_source_options_are_executable_components(self) -> None:
        core = load_toml(RULES_DIR / "ce-core.toml")
        small = load_toml(RULES_DIR / "ce-small-craft.toml")
        extension = load_toml(RULES_DIR / "af3-small-compatible.toml")
        boat = evaluate_small_craft(
            core,
            small,
            load_toml(BOAT),
            extension=extension,
        )
        result = evaluate(
            self.rules,
            load_toml(PATROL),
            carried_craft_results={"ship-18": boat},
        )
        self.assertEqual(result["structure_points"], 14)
        self.assertEqual(result["hardpoints_used"], 6)
        self.assertEqual(result["point_defense_nodes"], 6)
        self.assertEqual(
            result["carried_craft_displacement_millitons"], 30_000
        )
        self.assertEqual(result["carried_craft_price_credits"], 20_580_000)
        self.assertEqual(result["construction_price_credits"], 521_750_000)

    def test_jump_capable_parasite_craft_is_allowed_when_capacity_exists(
        self,
    ) -> None:
        core = load_toml(RULES_DIR / "ce-core.toml")
        small = load_toml(RULES_DIR / "ce-small-craft.toml")
        extension = load_toml(RULES_DIR / "af3-small-compatible.toml")
        boat = evaluate_small_craft(
            core,
            small,
            load_toml(BOAT),
            extension=extension,
        )
        boat["jump_rating"] = 1
        result = evaluate(
            self.rules,
            load_toml(PATROL),
            carried_craft_results={"ship-18": boat},
        )
        self.assertEqual(result["carried_craft_count"], 1)

    def test_carried_craft_must_fit_total_stowage_capacity(self) -> None:
        core = load_toml(RULES_DIR / "ce-core.toml")
        small = load_toml(RULES_DIR / "ce-small-craft.toml")
        extension = load_toml(RULES_DIR / "af3-small-compatible.toml")
        boat = evaluate_small_craft(
            core,
            small,
            load_toml(BOAT),
            extension=extension,
        )
        boat["hull_millitons"] = 40_000
        with self.assertRaisesRegex(DesignError, "hangars provide 30000"):
            evaluate(
                self.rules,
                load_toml(PATROL),
                carried_craft_results={"ship-18": boat},
            )

    def test_redundant_computer_quantity_is_not_only_descriptive(self) -> None:
        design = self._convert_design(MERCHANT)
        baseline = evaluate(self.rules, design)
        design["computer"]["quantity"] = 2
        duplicate = evaluate(self.rules, design)
        self.assertEqual(
            duplicate["construction_price_credits"]
            - baseline["construction_price_credits"],
            144_000,
        )


if __name__ == "__main__":
    unittest.main()
