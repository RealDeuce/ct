#!/usr/bin/env python3
"""Regression tests for player-facing site catalog records."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("ct_site_build", ROOT / "site" / "build.py")
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load site/build.py")
SITE_BUILD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SITE_BUILD)


class CatalogRecordTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.records = {
            record["catalog_id"]: record for record in SITE_BUILD.catalog_records()
        }

    def test_small_craft_retains_explicit_control_and_no_jump_drive(self) -> None:
        albatross = self.records[22]
        self.assertEqual(albatross["control"], "Two Person Cockpit")
        self.assertEqual(albatross["additional_passengers"], 0)
        self.assertIsNone(albatross["jump_drive"])
        self.assertEqual(albatross["airlocks"], 1)

    def test_starship_exposes_jump_and_bridge_fields(self) -> None:
        pym = self.records[26]
        self.assertEqual(pym["jump_drive"], "A")
        self.assertEqual(pym["jump_distance"], 2)
        self.assertEqual(pym["jump_count"], 1)
        self.assertEqual(pym["control"], "Standard bridge")
        self.assertEqual(pym["additional_passengers"], "Not applicable")
        self.assertEqual(pym["bridge_options"], ["Holographic Controls"])
        self.assertIsNone(pym["airlocks"])

    def test_starship_special_fits_are_searchable_and_visible(self) -> None:
        ligeia = self.records[33]
        self.assertEqual(ligeia["hull_options"], ["Stealth"])
        self.assertIn("Standard Hangar (6 tons contained)", ligeia["equipment"])
        self.assertIn("Point Defense Node Mount", ligeia["armament"])
        self.assertIn("Point Defense Laser", ligeia["armament"])
        self.assertIn("Point Defense 2", ligeia["software"])

    def test_exterior_equivalent_mercators_share_one_plate(self) -> None:
        mercators = [self.records[catalog_id] for catalog_id in (27, 28, 31)]
        self.assertEqual(
            {record["art_path"] for record in mercators},
            {"assets/ships/family-027-mercator.webp"},
        )
        self.assertTrue(
            all(record["unused_fire_control_stations"] == 1 for record in mercators)
        )
        self.assertEqual(mercators[0]["ammunition"], "12 × Standard Missiles")
        self.assertEqual(mercators[1]["ammunition"], "None carried")
        self.assertEqual(mercators[2]["ammunition"], "None carried")

    def test_exterior_equivalent_goliaths_share_one_plate(self) -> None:
        goliaths = [self.records[catalog_id] for catalog_id in (30, 32)]
        self.assertEqual(
            {record["art_path"] for record in goliaths},
            {"assets/ships/family-030-goliath.webp"},
        )
        for record in goliaths:
            self.assertEqual(record["tons"], 90)
            self.assertIn("50 passenger seats", record["equipment"])
            self.assertEqual(record["airlocks"], 2)
            self.assertEqual(record["cargo"], "17.6 tons")
            self.assertIsNone(record["jump_drive"])

    def test_source_fit_humboldts_share_one_hull_plate(self) -> None:
        humboldts = [self.records[catalog_id] for catalog_id in (34, 43)]
        self.assertEqual(
            {record["art_path"] for record in humboldts},
            {"assets/ships/family-034-humboldt.webp"},
        )
        for record in humboldts:
            self.assertEqual(record["tons"], 200)
            self.assertEqual(record["jump_drive"], "B")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(
                record["armament"], "Double Turret: Beam Laser · Sandcaster"
            )
        self.assertIn(
            "Standard Hangar (45 tons contained)", humboldts[0]["equipment"]
        )
        self.assertIn(
            "Standard Hangar (50 tons contained)", humboldts[1]["equipment"]
        )
        self.assertEqual(humboldts[0]["cargo"], "31 tons")
        self.assertEqual(humboldts[1]["cargo"], "27 tons")
        self.assertEqual(humboldts[0]["ammunition"], "20 × Sandcaster Canisters")
        self.assertEqual(humboldts[1]["ammunition"], "40 × Sandcaster Canisters")

    def test_congreve_variants_keep_shared_hull_scale(self) -> None:
        strike, escort = (self.records[catalog_id] for catalog_id in (35, 36))
        for record in (strike, escort):
            self.assertEqual(record["tons"], 80)
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["length_m"], 29.0)
            self.assertEqual(record["airlocks"], 1)
            self.assertIsNone(record["jump_drive"])
        self.assertEqual(
            strike["art_path"], "assets/ships/ship-035-congreve-strike.webp"
        )
        self.assertEqual(
            escort["art_path"], "assets/ships/ship-036-congreve-escort.webp"
        )
        self.assertEqual(
            strike["armament"], "Fixed Double Turret: Missile Rack · Missile Rack"
        )
        self.assertEqual(escort["armament"], "Single Turret: Missile Rack")
        self.assertEqual(strike["ammunition"], "144 × Standard Missiles")
        self.assertEqual(escort["ammunition"], "12 × Standard Missiles")

    def test_polo_internal_fits_keep_shared_trader_chassis(self) -> None:
        marco, niccolo, maffeo = (
            self.records[catalog_id] for catalog_id in (38, 39, 40)
        )
        for record in (marco, niccolo, maffeo):
            self.assertEqual(record["tons"], 200)
            self.assertEqual(record["jump_drive"], "B")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "C")
            self.assertEqual(record["length_m"], 44.0)
            self.assertEqual(record["armament"], "Double Turret: Beam Laser")
        self.assertEqual(marco["cargo"], "88.7 tons")
        self.assertEqual(niccolo["cargo"], "68.7 tons")
        self.assertEqual(maffeo["cargo"], "67.7 tons")
        self.assertNotIn("Steerage", marco["equipment"])
        self.assertIn("15 × Steerage", niccolo["equipment"])
        self.assertIn("8 × Stateroom", maffeo["equipment"])

    def test_exterior_equivalent_sinbads_share_one_plate(self) -> None:
        sinbads = [self.records[catalog_id] for catalog_id in (45, 46)]
        self.assertEqual(
            {record["art_path"] for record in sinbads},
            {"assets/ships/family-045-sinbad.webp"},
        )
        for record in sinbads:
            self.assertEqual(record["tons"], 200)
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["jump_drive"], "B")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["length_m"], 43.0)
            self.assertEqual(
                record["armament"],
                "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
            )
            self.assertEqual(
                record["ammunition"],
                "12 × Standard Missiles · 20 × Sandcaster Canisters",
            )
        self.assertEqual(sinbads[0]["cargo"], "42 tons")
        self.assertEqual(sinbads[1]["cargo"], "41 tons")
        self.assertEqual(sinbads[0]["electronics"], "Basic Civilian")
        self.assertEqual(sinbads[1]["electronics"], "Basic Military")

    def test_trident_weapon_fits_keep_shared_attack_boat_scale(self) -> None:
        triton, nereus, glaucus = (
            self.records[catalog_id] for catalog_id in (48, 55, 56)
        )
        for record in (triton, nereus, glaucus):
            self.assertEqual(record["tons"], 95)
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["maneuver_drive"], "sW")
            self.assertEqual(record["airlocks"], 2)
            self.assertEqual(record["length_m"], 31.0)
            self.assertIsNone(record["jump_drive"])
        self.assertEqual(triton["armament"], "Single Turret: Missile Rack")
        self.assertEqual(triton["ammunition"], "60 × Standard Missiles")
        self.assertEqual(nereus["armament"], "Single Turret: Particle Beam")
        self.assertEqual(nereus["ammunition"], "None carried")
        self.assertEqual(glaucus["hull_options"], ["Stealth"])
        self.assertEqual(glaucus["ammunition"], "48 × Standard Missiles")


if __name__ == "__main__":
    unittest.main()
