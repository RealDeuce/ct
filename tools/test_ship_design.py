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


RULES_PATH = ROOT / "catalog/shipbuilding/ce-core.toml"
DESIGN_PATH = ROOT / "catalog/ships/ship-192.toml"
ARMED_DESIGN_PATH = ROOT / "catalog/ships/ship-193.toml"


class ShipDesignTests(unittest.TestCase):
    def setUp(self) -> None:
        self.rules = load_toml(RULES_PATH)
        self.design = load_toml(DESIGN_PATH)

    def test_merchant_starter_revision_is_jump_two(self) -> None:
        result = evaluate(self.rules, self.design)
        self.assertEqual(result["accounted_displacement_millitons"], 200_000)
        self.assertEqual(result["construction_price_credits"], 51_219_000)
        self.assertEqual(result["jump_rating"], 2)
        self.assertEqual(result["thrust_g"], 1)
        self.assertEqual(result["crew"], 3)

    def test_frontier_starter_is_freight_first_and_armed(self) -> None:
        result = evaluate(self.rules, load_toml(ARMED_DESIGN_PATH))
        self.assertEqual(result["accounted_displacement_millitons"], 300_000)
        self.assertEqual(result["jump_rating"], 2)
        self.assertEqual(result["hardpoints_used"], 3)
        self.assertEqual(result["minimum_crew"], 6)
        self.assertEqual(result["crew"], 7)
        self.assertEqual(result["construction_price_credits"], 85_559_000)

    def test_accommodation_capacities_distinguish_people_from_rooms(self) -> None:
        crusoe = evaluate(
            self.rules, load_toml(ROOT / "catalog/ships/ship-193.toml")
        )
        self.assertEqual(crusoe["crew_accommodation_capacity"], 24)
        self.assertEqual(crusoe["provision_capacity_persons"], 24)
        self.assertEqual(crusoe["passenger_accommodation_berths"], 8)
        self.assertEqual(crusoe["low_berths"], 12)
        self.assertEqual(crusoe["monthly_life_support_credits"], 25_200)

        rules = compose_shipbuilding_rules(ROOT / "catalog/shipbuilding")
        trafalgar_design = load_toml(ROOT / "catalog/ships/ship-180.toml")
        trafalgar_design.pop("assertions", None)
        trafalgar_design["carried_craft"] = []
        trafalgar = evaluate(rules, trafalgar_design)
        self.assertEqual(trafalgar["crew_accommodation_capacity"], 322)
        self.assertEqual(trafalgar["provision_capacity_persons"], 322)
        self.assertEqual(trafalgar["passenger_accommodation_berths"], 12)
        self.assertEqual(trafalgar["monthly_life_support_credits"], 171_000)

    def test_steerage_places_are_commercial_berths_and_provision_places(self) -> None:
        rules = compose_shipbuilding_rules(ROOT / "catalog/shipbuilding")
        design = evaluate(rules, load_toml(ROOT / "catalog/ships/ship-39.toml"))
        self.assertEqual(design["passenger_accommodation_berths"], 17)
        self.assertEqual(design["provision_capacity_persons"], 23)

    def test_unknown_specification_text_cannot_become_a_component(self) -> None:
        design = deepcopy(self.design)
        design["equipment"].append({"id": "maintenance-cost", "quantity": 1})
        with self.assertRaisesRegex(DesignError, "unknown equipment"):
            evaluate(self.rules, design)

    def test_design_cannot_override_component_numbers(self) -> None:
        design = deepcopy(self.design)
        design["equipment"][0]["price_credits"] = 1
        with self.assertRaisesRegex(DesignError, "unknown field"):
            evaluate(self.rules, design)

    def test_unaccounted_volume_is_an_error_not_a_delta(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["cargo_millitons"] -= 1_000
        with self.assertRaisesRegex(DesignError, "1000 millitons under"):
            evaluate(self.rules, design)

    def test_rule_change_breaks_published_assertion(self) -> None:
        rules = deepcopy(self.rules)
        next(
            item for item in rules["equipment"] if item["id"] == "fuel-processor"
        )["price_credits_per_unit"] = 60_000
        with self.assertRaisesRegex(
            DesignError, "published assertion pre_discount_price_credits"
        ):
            evaluate(rules, self.design)

    def test_power_plant_must_cover_drive_code(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["drives"]["power"] = "A"
        design["cargo_millitons"] += 3_000
        with self.assertRaisesRegex(DesignError, "power plant A is rated below"):
            evaluate(self.rules, design)

    def test_jump_drive_requires_jump_control_software(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["software"] = []
        with self.assertRaisesRegex(DesignError, "requires Jump Control/2"):
            evaluate(self.rules, design)

    def test_each_turret_requires_a_gunner(self) -> None:
        design = load_toml(ARMED_DESIGN_PATH)
        del design["assertions"]
        next(
            role for role in design["crew"] if role["role"] == "turret-gunner"
        )["quantity"] = 2
        with self.assertRaisesRegex(DesignError, "requires 3"):
            evaluate(self.rules, design)

    def test_identical_turrets_use_a_quantity_not_duplicate_components(self) -> None:
        design = load_toml(ARMED_DESIGN_PATH)
        design["mounts"][0]["quantity"] = 2
        del design["mounts"][1]
        result = evaluate(self.rules, design)
        self.assertEqual(result["hardpoints_used"], 3)
        self.assertEqual(result["construction_price_credits"], 85_559_000)
        pulse_mount = next(
            item
            for item in result["line_items"]
            if item["kind"] == "weapon-mount"
            and item["weapons"] == ["pulse-laser"] * 3
        )
        self.assertEqual(pulse_mount["quantity"], 2)

    def test_standard_hull_scoops_are_real_equipment(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["equipment"] = [
            item for item in design["equipment"] if item["id"] != "fuel-scoop"
        ]
        result = evaluate(self.rules, design)
        self.assertEqual(result["construction_price_credits"], 50_319_000)

    def test_streamlined_hull_rejects_duplicate_scoops(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["hull"]["configuration"] = "streamlined"
        with self.assertRaisesRegex(DesignError, "already includes fuel scoops"):
            evaluate(self.rules, design)

    def test_custom_hangar_uses_rule_formula_not_a_design_override(self) -> None:
        design = deepcopy(self.design)
        del design["assertions"]
        design["cargo_millitons"] -= 13_000
        design["parameterized_equipment"] = [
            {
                "id": "custom-hangar",
                "contained_millitons": 10_000,
                "quantity": 1,
            }
        ]
        result = evaluate(self.rules, design)
        item = next(
            line
            for line in result["line_items"]
            if line["id"] == "custom-hangar"
        )
        self.assertEqual(item["displacement_millitons"], 13_000)
        self.assertEqual(item["price_credits"], 2_600_000)

    def test_carrier_facilities_and_commissary_use_rule_formulas(self) -> None:
        rules = compose_shipbuilding_rules(ROOT / "catalog/shipbuilding")
        design = load_toml(ARMED_DESIGN_PATH)
        del design["assertions"]
        design["ruleset_id"] = rules["ruleset_id"]
        design["source_ids"] = list(rules["source_ids"])
        design["cargo_millitons"] -= 62_500
        design["launch_facilities"] = [
            {
                "id": "flight-deck",
                "largest_craft_millitons": 10_000,
                "quantity": 2,
            }
        ]
        design["parameterized_equipment"] = [
            {"id": "commissary", "crew": 3, "quantity": 1}
        ]
        result = evaluate(rules, design)
        flight_deck = next(
            item
            for item in result["line_items"]
            if item["kind"] == "launch-facility"
        )
        commissary = next(
            item
            for item in result["line_items"]
            if item["id"] == "commissary"
        )
        self.assertEqual(flight_deck["displacement_millitons"], 60_000)
        self.assertEqual(flight_deck["price_credits"], 30_000_000)
        self.assertEqual(commissary["displacement_millitons"], 2_500)
        self.assertEqual(commissary["price_credits"], 250_000)


if __name__ == "__main__":
    unittest.main()
