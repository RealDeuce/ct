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

    def test_ariel_luxury_boat_sets_stevenson_hangar_clearance(self) -> None:
        ariel = self.records[142]
        self.assertEqual(ariel["family_id"], 142)
        self.assertEqual(ariel["tons"], 30)
        self.assertEqual(ariel["configuration"], "Streamlined")
        self.assertEqual(ariel["maneuver_drive"], "sJ")
        self.assertEqual(ariel["armor_points"], 4)
        self.assertEqual(ariel["control"], "Two Person Control Cabin")
        self.assertEqual(ariel["airlocks"], 1)
        self.assertEqual(ariel["cargo"], "4.2 tons")
        self.assertIn("8 passenger seats", ariel["equipment"])
        self.assertIn("2 × Small Craft Stateroom", ariel["equipment"])
        self.assertEqual(ariel["armament"], "None installed")
        self.assertIsNone(ariel["jump_drive"])
        self.assertEqual(ariel["art_path"], "assets/ships/ship-142-ariel.webp")
        self.assertEqual(ariel["length_m"], 19.0)

    def test_bellerophon_sets_mitchell_strike_fighter_clearance(self) -> None:
        bellerophon = self.records[25]
        self.assertEqual(bellerophon["family_id"], 25)
        self.assertEqual(bellerophon["tons"], 15)
        self.assertEqual(bellerophon["configuration"], "Streamlined")
        self.assertEqual(bellerophon["electronics"], "Basic Military")
        self.assertEqual(bellerophon["armor_points"], 2)
        self.assertEqual(bellerophon["control"], "Two Person Cockpit")
        self.assertEqual(bellerophon["additional_passengers"], 0)
        self.assertEqual(bellerophon["maneuver_drive"], "sE")
        self.assertEqual(bellerophon["power_plant"], "sE")
        self.assertIsNone(bellerophon["thrust_g"])
        self.assertIn("Thrust 6", bellerophon["mission_tags"])
        self.assertEqual(bellerophon["endurance"], 1)
        self.assertEqual(bellerophon["cargo"], "1.925 tons")
        self.assertEqual(bellerophon["airlocks"], 0)
        self.assertEqual(bellerophon["armament"], "Single Turret: Missile Rack")
        self.assertEqual(bellerophon["ammunition"], "12 × Standard Missiles")
        self.assertIsNone(bellerophon["jump_drive"])
        self.assertEqual(bellerophon["length_m"], 14.8)
        self.assertEqual(
            bellerophon["art_path"],
            "assets/ships/ship-025-bellerophon.webp",
        )

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

    def test_hanse_generations_share_two_exterior_fits(self) -> None:
        lubecks = [self.records[catalog_id] for catalog_id in (49, 51)]
        hamburgs = [self.records[catalog_id] for catalog_id in (50, 52)]
        self.assertEqual(
            {record["art_path"] for record in lubecks},
            {"assets/ships/family-049-lubeck.webp"},
        )
        self.assertEqual(
            {record["art_path"] for record in hamburgs},
            {"assets/ships/family-049-hamburg.webp"},
        )
        for record in (*lubecks, *hamburgs):
            self.assertEqual(record["tons"], 300)
            self.assertEqual(record["length_m"], 53.0)
            self.assertEqual(record["armament"], "Single Turret: Beam Laser")
        for record in lubecks:
            self.assertEqual(record["jump_drive"], "C")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["cargo"], "131.5 tons")
        for record in hamburgs:
            self.assertIsNone(record["jump_drive"])
            self.assertEqual(record["cargo"], "211.5 tons")

    def test_verne_records_resolve_to_seven_audited_exterior_fits(self) -> None:
        fit_groups = {
            "assets/ships/family-054-fogg.webp": (54, 164),
            "assets/ships/family-054-hatteras.webp": (57, 58, 161),
            "assets/ships/family-054-aouda.webp": (59, 162),
            "assets/ships/family-054-passepartout.webp": (60, 163),
            "assets/ships/family-054-stahlstadt.webp": (62, 167),
            "assets/ships/family-054-nemo.webp": (65, 166),
            "assets/ships/ship-168-robur.webp": (168,),
        }
        vernes = []
        for art_path, catalog_ids in fit_groups.items():
            records = [self.records[catalog_id] for catalog_id in catalog_ids]
            vernes.extend(records)
            self.assertEqual({record["art_path"] for record in records}, {art_path})
        for record in vernes:
            self.assertEqual(record["family_id"], 54)
            self.assertEqual(record["tons"], 300)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["jump_drive"], "C")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["length_m"], 50.0)

        stahlstadt = self.records[62]
        self.assertIn("Missile Bank", stahlstadt["armament"])
        self.assertEqual(
            stahlstadt["ammunition"],
            "240 × Standard Missiles · 40 × Sandcaster Canisters",
        )

        nemo, source_variant = (
            self.records[catalog_id] for catalog_id in (65, 166)
        )
        for record in (nemo, source_variant):
            self.assertIn("Particle Beam Barbette", record["armament"])
            self.assertIn("Full Hangar (30 tons contained)", record["equipment"])
        self.assertIn("Carried Craft: Jason (ship-17)", nemo["equipment"])
        self.assertIn(
            "Carried Craft: Wayfarer Armed (ship-165)", source_variant["equipment"]
        )

        robur = self.records[168]
        self.assertEqual(robur["maneuver_drive"], "E")
        self.assertEqual(robur["power_plant"], "E")
        self.assertEqual(robur["thrust_g"], 3)
        self.assertIn("3 × Point Defense Node Mount", robur["armament"])

    def test_stevenson_generations_share_one_fast_clipper_chassis(self) -> None:
        stevensons = [
            self.records[catalog_id] for catalog_id in (68, 69, 72, 139, 140, 141)
        ]
        for record in stevensons:
            self.assertEqual(record["family_id"], 68)
            self.assertEqual(record["tons"], 400)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["jump_drive"], "D")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "H")
            self.assertEqual(record["power_plant"], "H")
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["length_m"], 58.0)
            self.assertIn("Full Hangar (30 tons contained)", record["equipment"])

        for catalog_id in (68, 69, 72):
            self.assertIn(
                "Carried Craft: Jason (ship-17)", self.records[catalog_id]["equipment"]
            )
        for catalog_id in (139, 140, 141):
            self.assertIn(
                "Carried Craft: Ariel (ship-142)",
                self.records[catalog_id]["equipment"],
            )

        self.assertEqual(self.records[68]["armament"], self.records[72]["armament"])
        self.assertNotEqual(self.records[68]["art_path"], self.records[72]["art_path"])
        self.assertIn("2 × Single Turret: Particle Beam", self.records[140]["armament"])
        self.assertNotIn("Barbette", self.records[140]["armament"])
        self.assertIn("2 × Particle Beam Barbette", self.records[141]["armament"])
        self.assertIn("3 × Point Defense Node Mount", self.records[141]["armament"])
        self.assertEqual(self.records[141]["endurance"], 4)
        equipment_names = {
            entry["name"] for entry in self.records[141]["equipment_entries"]
        }
        self.assertIn("Barracks", equipment_names)
        self.assertNotIn("Barrackss", equipment_names)

    def test_klondike_modules_share_one_distributed_tender_frame(self) -> None:
        bonanza, skagway = (self.records[catalog_id] for catalog_id in (78, 79))
        for record in (bonanza, skagway):
            self.assertEqual(record["family_id"], 78)
            self.assertEqual(record["tons"], 800)
            self.assertEqual(record["configuration"], "Distributed")
            self.assertEqual(record["jump_drive"], "K")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "K")
            self.assertEqual(record["armor_points"], 2)
            self.assertEqual(record["external_load"], "110 tons")
            self.assertEqual(record["length_m"], 76.0)
            self.assertIn("2 × Carried Craft: Proteus Cargo (ship-20)", record["equipment"])
            self.assertIn("Carried Craft: Charon (ship-158)", record["equipment"])
            self.assertIn("Docking Clamp 300", record["equipment"])
        self.assertEqual(bonanza["cargo"], "369 tons")
        self.assertEqual(skagway["cargo"], "263 tons")
        self.assertIn("2 × Single Turret: Pulse Laser", bonanza["armament"])
        self.assertIn("4 × Double Turret: Beam Laser", skagway["armament"])

    def test_homeric_records_share_one_standard_modular_frame(self) -> None:
        catalog_ids = (80, 82, 83, 86, 88, 152, 153, 154, 155, 156, 157)
        homerics = [self.records[catalog_id] for catalog_id in catalog_ids]
        for record in homerics:
            self.assertEqual(record["family_id"], 80)
            self.assertEqual(record["tons"], 800)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["hull_options"], ["Self Sealing"])
            self.assertEqual(record["jump_drive"], "J")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["armor_points"], 0)
            self.assertEqual(record["length_m"], 72.0)

        self.assertEqual(
            {self.records[catalog_id]["art_path"] for catalog_id in (80, 152)},
            {"assets/ships/family-080-ithaca-external.webp"},
        )
        self.assertEqual(self.records[80]["cargo"], "373 tons")
        self.assertEqual(self.records[152]["cargo"], "365 tons")
        for catalog_id in (80, 82, 152):
            self.assertEqual(self.records[catalog_id]["external_load"], "40 tons")
            self.assertIn(
                "Carried Craft: Albatross (ship-151)",
                self.records[catalog_id]["equipment"],
            )

        port_freighter = self.records[86]
        self.assertEqual(port_freighter["cargo"], "386 tons")
        self.assertEqual(port_freighter["external_load"], "0 tons")
        self.assertNotIn("Docking Clamp", port_freighter["equipment"])
        self.assertNotIn("Carried Craft", port_freighter["equipment"])

        self.assertIn("Carried Craft: Jason (ship-17)", self.records[83]["equipment"])
        self.assertIn("Carried Craft: Jason (ship-17)", self.records[88]["equipment"])
        for catalog_id in (153, 154, 155, 156, 157):
            self.assertIn(
                "Carried Craft: Wayfarer Cargo (ship-187)",
                self.records[catalog_id]["equipment"],
            )

        calypso = self.records[153]
        self.assertEqual(calypso["jump_count"], 2)
        self.assertEqual(calypso["endurance"], 3)

        for catalog_id in (88, 157):
            cyclops = self.records[catalog_id]
            self.assertEqual(cyclops["maneuver_drive"], "L")
            self.assertEqual(cyclops["power_plant"], "L")
            self.assertIn("6 × Missile Bank", cyclops["armament"])
            self.assertIn("504 × Standard Missiles", cyclops["ammunition"])

        armed_odysseus = self.records[156]
        self.assertIn("2 × Double Turret: Beam Laser", armed_odysseus["armament"])
        self.assertIn("2 × Double Turret: Missile Rack", armed_odysseus["armament"])
        self.assertIn("2 × Double Turret: Sandcaster", armed_odysseus["armament"])
        self.assertIn("2 × Particle Beam Barbette", armed_odysseus["armament"])
        self.assertEqual(armed_odysseus["unused_fire_control_stations"], 0)

    def test_hawkwood_conversion_preserves_the_patrol_frigate_chassis(self) -> None:
        hawkwood, condottiere = (
            self.records[catalog_id] for catalog_id in (90, 96)
        )
        for record in (hawkwood, condottiere):
            self.assertEqual(record["family_id"], 90)
            self.assertEqual(record["tons"], 550)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(record["bridge_options"], ["Hardened Bridge"])
            self.assertEqual(record["length_m"], 64.0)
            self.assertIn("Full Hangar (20 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Caduceus (ship-7)", record["equipment"])
            self.assertIn("2 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("Triple Turret: Missile Rack", record["armament"])
            self.assertIn("Triple Turret: Sandcaster", record["armament"])
            self.assertIn("5 × Point Defense Node Mount", record["armament"])

        self.assertEqual(hawkwood["jump_drive"], "F")
        self.assertEqual(hawkwood["jump_distance"], 2)
        self.assertEqual(hawkwood["armor_points"], 8)
        self.assertEqual(hawkwood["thrust_g"], 4)
        self.assertIn("Particle Beam Barbette", hawkwood["armament"])
        self.assertNotIn("Meson Gun Bay", hawkwood["armament"])

        self.assertIsNone(condottiere["jump_drive"])
        self.assertEqual(condottiere["jump_distance"], 0)
        self.assertEqual(condottiere["armor_points"], 11)
        self.assertIn("Meson Gun Bay", condottiere["armament"])
        self.assertNotIn("Particle Beam Barbette", condottiere["armament"])

    def test_cook_refit_fairs_the_same_patrol_chassis(self) -> None:
        frigate, corvette = (self.records[catalog_id] for catalog_id in (91, 95))
        for record in (frigate, corvette):
            self.assertEqual(record["family_id"], 91)
            self.assertEqual(record["tons"], 600)
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "S")
            self.assertEqual(record["power_plant"], "S")
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["length_m"], 68.0)
            self.assertIn("Full Hangar (30 tons contained)", record["equipment"])
            self.assertIn("2 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("Triple Turret: Missile Rack", record["armament"])
            self.assertIn("Triple Turret: Sandcaster", record["armament"])
            self.assertIn("2 × Particle Beam Barbette", record["armament"])

        self.assertEqual(frigate["configuration"], "Standard")
        self.assertEqual(frigate["jump_drive"], "H")
        self.assertEqual(frigate["endurance"], 2)
        self.assertIn("Carried Craft: Jason (ship-17)", frigate["equipment"])
        self.assertIn("6 × Point Defense Node Mount", frigate["armament"])

        self.assertEqual(corvette["configuration"], "Streamlined")
        self.assertEqual(corvette["jump_drive"], "F")
        self.assertEqual(corvette["endurance"], 4)
        self.assertEqual(corvette["hull_options"], ["Radiation Shielding"])
        self.assertEqual(
            corvette["structural_options"], ["Reinforced Structure (1 increment)"]
        )
        self.assertIn("Carried Craft: Wayfarer Utility (ship-18)", corvette["equipment"])
        self.assertIn(
            "3 × Point Defense Node Mount: Point Defense Gatling Laser",
            corvette["armament"],
        )
        self.assertIn(
            "3 × Point Defense Node Mount: Point Defense Laser",
            corvette["armament"],
        )

    def test_nightingale_hospital_refit_keeps_the_clinical_chassis(self) -> None:
        early, late = (self.records[catalog_id] for catalog_id in (93, 160))
        for record in (early, late):
            self.assertEqual(record["family_id"], 93)
            self.assertEqual(record["tons"], 1000)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["jump_drive"], "H")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["length_m"], 78.0)
            self.assertIn("Full Hangar (110 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Charon (ship-158)", record["equipment"])
            self.assertIn("2 × Carried Craft: Proteus Mercy (ship-159)", record["equipment"])
            self.assertIn("6 × Double Turret: Sandcaster", record["armament"])
            self.assertIn("10 × Point Defense Node Mount", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 4)
        self.assertEqual(early["maneuver_drive"], "L")
        self.assertEqual(early["power_plant"], "L")
        self.assertEqual(early["endurance"], 2)
        self.assertEqual(early["cargo"], "106.5 tons")
        self.assertEqual(late["maneuver_drive"], "P")
        self.assertEqual(late["power_plant"], "P")
        self.assertEqual(late["endurance"], 6)
        self.assertEqual(late["cargo"], "31.5 tons")

    def test_baltic_league_uses_two_audited_logistics_fits(self) -> None:
        riga, visby, novgorod = (
            self.records[catalog_id] for catalog_id in (97, 98, 101)
        )
        for record in (riga, visby, novgorod):
            self.assertEqual(record["family_id"], 97)
            self.assertEqual(record["tons"], 2000)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["jump_drive"], "Q")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "N")
            self.assertEqual(record["power_plant"], "Q")
            self.assertEqual(record["armor_points"], 2)
            self.assertEqual(record["length_m"], 104.0)
            self.assertIn("Full Hangar (30 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Jason (ship-17)", record["equipment"])
            self.assertIn("5 × Triple Turret", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 15)

        self.assertEqual(
            {riga["art_path"], novgorod["art_path"]},
            {"assets/ships/family-097-baltic-logistics.webp"},
        )
        for record in (riga, novgorod):
            self.assertIn("10 × Underway Replenishment System", record["equipment"])
            self.assertIn("20 × Fuel Processor", record["equipment"])
        self.assertEqual(riga["cargo"], "1,154 tons")
        self.assertEqual(novgorod["cargo"], "1,142 tons")

        self.assertEqual(visby["art_path"], "assets/ships/ship-098-visby.webp")
        self.assertNotIn("Underway Replenishment", visby["equipment"])
        self.assertIn("Air Raft Hangar", visby["equipment"])
        self.assertIn("5 × Fuel Processor", visby["equipment"])
        self.assertEqual(visby["cargo"], "1,122 tons")

    def test_corbett_configurations_share_pressure_volumes_not_silhouettes(self) -> None:
        distributed, close_structure = (
            self.records[catalog_id] for catalog_id in (99, 107)
        )
        for record in (distributed, close_structure):
            self.assertEqual(record["family_id"], 99)
            self.assertEqual(record["tons"], 1500)
            self.assertEqual(record["jump_drive"], "L")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "P")
            self.assertEqual(record["power_plant"], "P")
            self.assertEqual(record["endurance"], 3)
            self.assertEqual(record["armor_points"], 2)
            self.assertEqual(record["hull_options"], ["Radiation Shielding"])
            self.assertEqual(record["length_m"], 96.0)
            self.assertIn("Full Hangar (70 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Archimedes (ship-124)", record["equipment"])
            self.assertIn("6 × Particle Beam Barbette", record["armament"])
            self.assertIn("2 × Railgun Barbette", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 0)
            self.assertNotIn("Fuel Scoop", record["equipment"])

        self.assertEqual(distributed["configuration"], "Distributed")
        self.assertEqual(distributed["cargo"], "568.5 tons")
        self.assertIn(
            "2 × Carried Craft: Wayfarer Cargo (ship-187)",
            distributed["equipment"],
        )
        self.assertIn("2 × Triple Turret: Beam Laser", distributed["armament"])
        self.assertIn("2 × Triple Turret: Missile Rack", distributed["armament"])
        self.assertIn("3 × Triple Turret: Sandcaster", distributed["armament"])

        self.assertEqual(close_structure["configuration"], "Close Structure")
        self.assertEqual(close_structure["cargo"], "541.7 tons")
        self.assertIn(
            "2 × Carried Craft: Wayfarer Utility (ship-18)",
            close_structure["equipment"],
        )
        self.assertIn("3 × Triple Turret: Beam Laser", close_structure["armament"])
        self.assertIn("2 × Triple Turret: Missile Rack", close_structure["armament"])
        self.assertIn("2 × Triple Turret: Sandcaster", close_structure["armament"])

    def test_duncan_command_refit_preserves_the_fleet_escort_chassis(self) -> None:
        duncan, duncan_ii = (self.records[catalog_id] for catalog_id in (100, 102))
        for record in (duncan, duncan_ii):
            self.assertEqual(record["family_id"], 100)
            self.assertEqual(record["tons"], 1000)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(
                record["bridge_options"],
                ["Hardened Bridge", "Holographic Controls"],
            )
            self.assertEqual(record["jump_drive"], "K")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "P")
            self.assertEqual(record["power_plant"], "P")
            self.assertEqual(record["endurance"], 3)
            self.assertEqual(record["armor_points"], 8)
            self.assertEqual(record["cargo"], "81.5 tons")
            self.assertEqual(record["length_m"], 76.0)
            self.assertIn("Fuel Scoop", record["equipment"])
            self.assertIn("Full Hangar (50 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Jason (ship-17)", record["equipment"])
            self.assertIn("2 × Triple Turret: Particle Beam", record["armament"])
            self.assertIn("2 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("2 × Triple Turret: Sandcaster", record["armament"])
            self.assertIn("2 × Meson Gun Bay", record["armament"])
            self.assertIn("2 × Particle Beam Bay", record["armament"])
            self.assertIn("10 × Point Defense Node Mount", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 0)

        self.assertEqual(duncan["art_path"], "assets/ships/ship-100-duncan.webp")
        self.assertEqual(
            duncan_ii["art_path"], "assets/ships/ship-102-duncan-ii.webp"
        )

    def test_hephaestus_clamp_fits_account_for_the_external_flotilla(self) -> None:
        enyo, ares = (self.records[catalog_id] for catalog_id in (104, 105))
        for record in (enyo, ares):
            self.assertEqual(record["family_id"], 104)
            self.assertEqual(record["tons"], 1400)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(
                record["bridge_options"],
                ["Command Bridge", "Hardened Bridge", "Holographic Controls"],
            )
            self.assertEqual(record["armor_points"], 1)
            self.assertEqual(record["endurance"], 3)
            self.assertEqual(record["length_m"], 94.0)
            self.assertIn("Full Hangar (20 tons contained)", record["equipment"])
            self.assertIn("Carried Craft: Caduceus (ship-7)", record["equipment"])
            self.assertIn("6 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("4 × Triple Turret: Missile Rack", record["armament"])
            self.assertIn("4 × Triple Turret: Sandcaster", record["armament"])
            self.assertIn("14 × Point Defense Node Mount", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 0)

        self.assertEqual(enyo["jump_drive"], "K")
        self.assertEqual(enyo["jump_distance"], 2)
        self.assertEqual(enyo["maneuver_drive"], "N")
        self.assertEqual(enyo["power_plant"], "P")
        self.assertEqual(enyo["cargo"], "499.5 tons")
        self.assertEqual(enyo["external_load"], "0 tons")
        self.assertIn("10 × Docking Clamp 90", enyo["equipment"])
        self.assertNotIn("Fuel Scoop", enyo["equipment"])
        self.assertNotIn("Triton", enyo["equipment"])

        self.assertIsNone(ares["jump_drive"])
        self.assertEqual(ares["jump_distance"], 0)
        self.assertEqual(ares["maneuver_drive"], "W")
        self.assertEqual(ares["power_plant"], "W")
        self.assertEqual(ares["cargo"], "892.5 tons")
        self.assertEqual(ares["external_load"], "1,140 tons")
        self.assertIn("4 × Docking Clamp 300", ares["equipment"])
        self.assertIn("Fuel Scoop", ares["equipment"])
        self.assertIn("10 × Fuel Processor", ares["equipment"])
        self.assertIn("12 × Carried Craft: Triton (ship-48)", ares["equipment"])

    def test_vauban_path_fits_preserve_the_capital_battery(self) -> None:
        citadel, ravelin = (self.records[catalog_id] for catalog_id in (116, 118))
        for record in (citadel, ravelin):
            self.assertEqual(record["family_id"], 116)
            self.assertEqual(record["tons"], 2500)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(
                record["bridge_options"],
                ["Command Bridge", "Hardened Bridge", "Holographic Controls"],
            )
            self.assertEqual(record["armor_points"], 8)
            self.assertEqual(record["maneuver_drive"], "Z")
            self.assertEqual(record["power_plant"], "Z")
            self.assertEqual(record["length_m"], 124.0)
            self.assertNotIn("Fuel Scoop", record["equipment"])
            self.assertIn("5 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("5 × Triple Turret: Missile Rack", record["armament"])
            self.assertIn("5 × Triple Turret: Sandcaster", record["armament"])
            self.assertIn("4 × Meson Gun Bay", record["armament"])
            self.assertIn("4 × Particle Beam Bay", record["armament"])
            self.assertIn("2 × Torpedo Bay 100", record["armament"])
            self.assertIn("25 × Point Defense Node Mount", record["armament"])
            self.assertEqual(record["unused_fire_control_stations"], 0)

        self.assertIsNone(citadel["jump_drive"])
        self.assertEqual(citadel["jump_distance"], 0)
        self.assertEqual(citadel["endurance"], 3)
        self.assertEqual(citadel["cargo"], "805 tons")
        self.assertIn("Full Hangar (100 tons contained)", citadel["equipment"])
        self.assertIn("Carried Craft: Proteus Cargo (ship-20)", citadel["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-7)", citadel["equipment"])

        self.assertEqual(ravelin["jump_drive"], "T")
        self.assertEqual(ravelin["jump_distance"], 2)
        self.assertEqual(ravelin["endurance"], 2)
        self.assertEqual(ravelin["cargo"], "7.75 tons")
        self.assertIn("Reinforced Structure (1 increment)", ravelin["structural_options"])
        self.assertIn("Full Hangar (50 tons contained)", ravelin["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-189)", ravelin["equipment"])
        self.assertIn("Carried Craft: Grapnel (ship-211)", ravelin["equipment"])

    def test_aviator_blocks_grow_one_carrier_module_at_a_time(self) -> None:
        lilienthal, wright, mitchell = (
            self.records[catalog_id] for catalog_id in (117, 119, 120)
        )
        for record in (lilienthal, wright, mitchell):
            self.assertEqual(record["family_id"], 117)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(
                record["bridge_options"],
                ["Command Bridge", "Holographic Controls"],
            )
            self.assertEqual(record["hull_options"], ["Radiation Shielding"])
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["jump_count"], 1)
            self.assertEqual(record["endurance"], 2)
            self.assertIn("Armored Bulkheads (Bridge)", record["structural_options"])
            self.assertIn("Armored Bulkheads (Drives)", record["structural_options"])
            self.assertIn(
                "Armored Bulkheads (Ordnance Magazines)",
                record["structural_options"],
            )
            self.assertEqual(record["magazine_options"], ["Improved Magazine"])
            self.assertEqual(record["power_options"], ["Emergency Power"])
            self.assertIn("Fuel Scoop", record["equipment"])
            self.assertIn("15 × Fuel Processor", record["equipment"])
            self.assertIn("2 × Flight Deck", record["equipment"])
            self.assertIn("Recovery Deck", record["equipment"])
            self.assertIn("2 × Carried Craft: Caduceus (ship-185)", record["equipment"])
            self.assertIn(
                "2 × Carried Craft: Wayfarer Cargo (ship-187)",
                record["equipment"],
            )
            self.assertEqual(record["unused_fire_control_stations"], 0)

        self.assertEqual(lilienthal["tons"], 2500)
        self.assertEqual(lilienthal["length_m"], 118.0)
        self.assertEqual(lilienthal["jump_drive"], "T")
        self.assertEqual(lilienthal["maneuver_drive"], "W")
        self.assertEqual(lilienthal["power_plant"], "W")
        self.assertEqual(lilienthal["cargo"], "49.1 tons")
        self.assertIn("Full Hangar (400 tons contained)", lilienthal["equipment"])
        self.assertIn("30 × Carried Craft: Icarus I (ship-9)", lilienthal["equipment"])
        self.assertIn("10 × Triple Turret: Beam Laser", lilienthal["armament"])
        self.assertIn("9 × Triple Turret: Missile Rack", lilienthal["armament"])
        self.assertIn("25 × Point Defense Node Mount", lilienthal["armament"])
        self.assertEqual(
            lilienthal["ammunition"],
            "324 × Standard Missiles · 360 × Sandcaster Canisters",
        )

        self.assertEqual(wright["tons"], 3000)
        self.assertEqual(wright["length_m"], 132.0)
        self.assertEqual(wright["jump_drive"], "T")
        self.assertEqual(wright["maneuver_drive"], "W")
        self.assertEqual(wright["power_plant"], "W")
        self.assertEqual(wright["cargo"], "91.98 tons")
        self.assertIn("Full Hangar (600 tons contained)", wright["equipment"])
        self.assertIn("50 × Carried Craft: Icarus II (ship-11)", wright["equipment"])
        self.assertIn("14 × Triple Turret: Beam Laser", wright["armament"])
        self.assertIn("10 × Triple Turret: Missile Rack", wright["armament"])
        self.assertIn("30 × Point Defense Node Mount", wright["armament"])
        self.assertEqual(
            wright["ammunition"],
            "360 × Standard Missiles · 384 × Sandcaster Canisters",
        )

        self.assertEqual(mitchell["tons"], 3800)
        self.assertEqual(mitchell["length_m"], 150.0)
        self.assertEqual(mitchell["jump_drive"], "Y")
        self.assertEqual(mitchell["maneuver_drive"], "Z")
        self.assertEqual(mitchell["power_plant"], "Z")
        self.assertEqual(mitchell["cargo"], "175.36 tons")
        self.assertIn("Full Hangar (825 tons contained)", mitchell["equipment"])
        self.assertIn("50 × Carried Craft: Icarus II (ship-11)", mitchell["equipment"])
        self.assertIn(
            "15 × Carried Craft: Bellerophon (ship-25)",
            mitchell["equipment"],
        )
        self.assertIn("14 × Triple Turret: Beam Laser", mitchell["armament"])
        self.assertIn("12 × Triple Turret: Missile Rack", mitchell["armament"])
        self.assertIn("33 × Point Defense Node Mount", mitchell["armament"])
        self.assertEqual(
            mitchell["ammunition"],
            "432 × Standard Missiles · 768 × Sandcaster Canisters",
        )

    def test_bulwark_conversion_preserves_the_fast_escort_chassis(self) -> None:
        curtain, keep = (self.records[catalog_id] for catalog_id in (121, 122))
        for record in (curtain, keep):
            self.assertEqual(record["family_id"], 121)
            self.assertEqual(record["tons"], 500)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(
                record["bridge_options"],
                ["Hardened Bridge", "Holographic Controls"],
            )
            self.assertEqual(
                record["hull_options"],
                ["Radiation Shielding", "Self Sealing"],
            )
            self.assertEqual(record["maneuver_drive"], "Q")
            self.assertEqual(record["power_plant"], "Q")
            self.assertEqual(record["endurance"], 3)
            self.assertEqual(record["length_m"], 62.0)
            self.assertIn("Fuel Scoop", record["equipment"])
            self.assertIn("8 × Fuel Processor", record["equipment"])
            self.assertIn("Standard Hangar (30 tons contained)", record["equipment"])
            self.assertIn(
                "Carried Craft: Wayfarer Boarding (ship-212)",
                record["equipment"],
            )
            self.assertIn("2 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("Triple Turret: Missile Rack", record["armament"])
            self.assertIn("5 × Point Defense Node Mount", record["armament"])
            self.assertIn("40 × Sandcaster Canisters", record["ammunition"])

        self.assertEqual(curtain["armor_points"], 4)
        self.assertEqual(curtain["jump_drive"], "G")
        self.assertEqual(curtain["jump_distance"], 2)
        self.assertEqual(curtain["cargo"], "4 tons")
        self.assertIn("2 × Particle Beam Barbette", curtain["armament"])
        self.assertNotIn("Particle Beam Bay", curtain["armament"])
        self.assertIn("84 × Standard Missiles", curtain["ammunition"])
        self.assertEqual(
            curtain["art_path"],
            "assets/ships/ship-121-curtain.webp",
        )

        self.assertEqual(keep["armor_points"], 12)
        self.assertIsNone(keep["jump_drive"])
        self.assertEqual(keep["jump_distance"], 0)
        self.assertEqual(keep["cargo"], "50.9 tons")
        self.assertIn("Particle Beam Barbette", keep["armament"])
        self.assertIn("Particle Beam Bay", keep["armament"])
        self.assertNotIn("2 × Particle Beam Barbette", keep["armament"])
        self.assertIn("96 × Standard Missiles", keep["ammunition"])
        self.assertEqual(keep["art_path"], "assets/ships/ship-122-keep.webp")

    def test_cradle_exposes_its_empty_two_thousand_ton_tender_clamp(self) -> None:
        cradle = self.records[123]
        self.assertEqual(cradle["family_id"], 123)
        self.assertEqual(cradle["tons"], 350)
        self.assertEqual(cradle["configuration"], "Standard")
        self.assertEqual(cradle["electronics"], "Standard")
        self.assertEqual(
            cradle["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(cradle["hull_options"], ["Self Sealing"])
        self.assertEqual(cradle["armor_points"], 2)
        self.assertEqual(cradle["jump_drive"], "D")
        self.assertEqual(cradle["jump_distance"], 2)
        self.assertEqual(cradle["jump_count"], 1)
        self.assertEqual(cradle["maneuver_drive"], "D")
        self.assertEqual(cradle["power_plant"], "D")
        self.assertEqual(cradle["endurance"], 3)
        self.assertEqual(cradle["cargo"], "148.75 tons")
        self.assertEqual(cradle["external_load"], "0 tons")
        self.assertIn("Fuel Scoop", cradle["equipment"])
        self.assertIn("4 × Fuel Processor", cradle["equipment"])
        self.assertIn("Repair Drones", cradle["equipment"])
        self.assertIn("Docking Clamp 2000", cradle["equipment"])
        self.assertEqual(
            cradle["armament"],
            "3 × Point Defense Node Mount: Point Defense Laser",
        )
        self.assertEqual(cradle["ammunition"], "None carried")
        self.assertEqual(cradle["length_m"], 54.0)
        self.assertEqual(cradle["art_path"], "assets/ships/ship-123-cradle.webp")

    def test_nausicaa_sets_leviathan_boat_and_hangar_clearance(self) -> None:
        nausicaa = self.records[125]
        self.assertEqual(nausicaa["family_id"], 125)
        self.assertEqual(nausicaa["tons"], 30)
        self.assertEqual(nausicaa["configuration"], "Streamlined")
        self.assertEqual(nausicaa["electronics"], "Standard")
        self.assertEqual(
            nausicaa["hull_options"],
            ["Small Craft Radiation Shielding"],
        )
        self.assertEqual(nausicaa["armor_points"], 1)
        self.assertEqual(nausicaa["control"], "Two Person Cockpit")
        self.assertEqual(nausicaa["additional_passengers"], 0)
        self.assertEqual(nausicaa["airlocks"], 1)
        self.assertEqual(nausicaa["maneuver_drive"], "sE")
        self.assertEqual(nausicaa["power_plant"], "sE")
        self.assertEqual(nausicaa["endurance"], 2)
        self.assertEqual(nausicaa["cargo"], "16.75 tons")
        self.assertIn("4 passenger seats", nausicaa["equipment"])
        self.assertEqual(nausicaa["armament"], "None installed")
        self.assertEqual(nausicaa["ammunition"], "None carried")
        self.assertIsNone(nausicaa["jump_drive"])
        self.assertEqual(nausicaa["length_m"], 19.5)
        self.assertEqual(
            nausicaa["art_path"],
            "assets/ships/ship-125-nausicaa.webp",
        )

    def test_leviathan_family_exposes_jump_and_freight_variant_geometry(self) -> None:
        leviathan = self.records[126]
        behemoth = self.records[127]

        for record in (leviathan, behemoth):
            self.assertEqual(record["family_id"], 126)
            self.assertEqual(record["tons"], 1900)
            self.assertEqual(record["configuration"], "Distributed")
            self.assertEqual(record["electronics"], "Basic Civilian")
            self.assertEqual(
                record["hull_options"],
                ["Self Sealing", "Radiation Shielding"],
            )
            self.assertEqual(record["armor_points"], 1)
            self.assertEqual(record["maneuver_drive"], "K")
            self.assertEqual(record["endurance"], 4)
            self.assertEqual(record["external_load"], "40 tons")
            self.assertIn("10 × Fuel Processor", record["equipment"])
            self.assertIn("2 × Docking Clamp 30", record["equipment"])
            self.assertIn("Carried Craft: Archimedes (ship-124)", record["equipment"])
            self.assertIn("Carried Craft: Nausicaa (ship-125)", record["equipment"])
            self.assertIn("2 × Double Turret: Missile Rack", record["armament"])
            self.assertIn("6 × Double Turret: Beam Laser", record["armament"])
            self.assertEqual(
                record["ammunition"],
                "24 × Standard Missiles · 40 × Sandcaster Canisters",
            )
            self.assertEqual(record["length_m"], 112.0)

        self.assertEqual(leviathan["jump_drive"], "Q")
        self.assertEqual(leviathan["jump_distance"], 2)
        self.assertEqual(leviathan["jump_count"], 1)
        self.assertEqual(leviathan["power_plant"], "Q")
        self.assertEqual(leviathan["cargo"], "1,076.9 tons")
        self.assertEqual(
            leviathan["art_path"],
            "assets/ships/ship-126-leviathan.webp",
        )

        self.assertIsNone(behemoth["jump_drive"])
        self.assertEqual(behemoth["jump_distance"], 0)
        self.assertEqual(behemoth["jump_count"], 0)
        self.assertEqual(behemoth["power_plant"], "K")
        self.assertEqual(behemoth["cargo"], "1,581.4 tons")
        self.assertEqual(
            behemoth["art_path"],
            "assets/ships/ship-127-behemoth.webp",
        )

    def test_grapnel_is_the_armored_boat_for_ravelins_hangar(self) -> None:
        grapnel = self.records[211]
        self.assertEqual(grapnel["family_id"], 211)
        self.assertEqual(grapnel["tons"], 30)
        self.assertEqual(grapnel["configuration"], "Streamlined")
        self.assertEqual(grapnel["electronics"], "Standard")
        self.assertEqual(grapnel["armor_points"], 4)
        self.assertEqual(grapnel["control"], "Two Person Control Cabin")
        self.assertEqual(grapnel["additional_passengers"], 0)
        self.assertEqual(grapnel["maneuver_drive"], "sJ")
        self.assertEqual(grapnel["power_plant"], "sJ")
        self.assertEqual(grapnel["endurance"], 2)
        self.assertEqual(grapnel["cargo"], "2.5 tons")
        self.assertEqual(grapnel["airlocks"], 1)
        self.assertIn("14 passenger seats", grapnel["equipment"])
        self.assertIn("Small Craft Fuel Processor", grapnel["equipment"])
        self.assertIn("Fixed Single Turret: Beam Laser", grapnel["armament"])
        self.assertIsNone(grapnel["jump_drive"])
        self.assertEqual(grapnel["length_m"], 18.6)
        self.assertEqual(grapnel["art_path"], "assets/ships/ship-211-grapnel.webp")

    def test_archimedes_is_the_unarmed_corbett_work_pod(self) -> None:
        archimedes = self.records[124]
        self.assertEqual(archimedes["family_id"], 124)
        self.assertEqual(archimedes["tons"], 10)
        self.assertEqual(archimedes["configuration"], "Standard")
        self.assertEqual(archimedes["hull_options"], ["Small Craft Radiation Shielding"])
        self.assertEqual(archimedes["control"], "Two Person Cockpit")
        self.assertEqual(archimedes["additional_passengers"], 0)
        self.assertEqual(archimedes["maneuver_drive"], "sA")
        self.assertEqual(archimedes["power_plant"], "sA")
        self.assertEqual(archimedes["endurance"], 1)
        self.assertEqual(archimedes["cargo"], "1.9 tons")
        self.assertIn("4 passenger seats", archimedes["equipment"])
        self.assertEqual(archimedes["armament"], "None installed")
        self.assertIsNone(archimedes["jump_drive"])
        self.assertEqual(archimedes["airlocks"], 1)
        self.assertEqual(archimedes["length_m"], 8.4)


if __name__ == "__main__":
    unittest.main()
