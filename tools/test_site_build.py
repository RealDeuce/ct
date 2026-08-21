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

    def test_argo_exposes_armored_boarding_launch_fit(self) -> None:
        argo = self.records[10]
        self.assertEqual(argo["family_id"], 10)
        self.assertEqual(argo["tons"], 20)
        self.assertEqual(argo["configuration"], "Streamlined")
        self.assertEqual(argo["tech_level"], 11)
        self.assertFalse(argo["standard_design"])
        self.assertEqual(argo["electronics"], "Standard")
        self.assertEqual(argo["armor_points"], 4)
        self.assertEqual(argo["control"], "One Person Cockpit")
        self.assertEqual(argo["additional_passengers"], 0)
        self.assertEqual(argo["maneuver_drive"], "sF")
        self.assertEqual(argo["power_plant"], "sG")
        self.assertEqual(argo["assertions"]["thrust_g"], 6)
        self.assertEqual(argo["endurance"], 2)
        self.assertEqual(argo["cargo"], "2.5 tons")
        self.assertEqual(argo["crew"], 1)
        self.assertEqual(argo["airlocks"], 1)
        self.assertIn("10 passenger seats", argo["equipment"])
        self.assertEqual(argo["armament"], "Single Turret: Beam Laser")
        self.assertIsNone(argo["jump_drive"])
        self.assertEqual(argo["length_m"], 16.5)
        self.assertEqual(argo["art_path"], "assets/ships/ship-010-argo.webp")

    def test_orion_exposes_local_patrol_fighter_fit(self) -> None:
        orion = self.records[13]
        self.assertEqual(orion["family_id"], 13)
        self.assertEqual(orion["tons"], 10)
        self.assertEqual(orion["configuration"], "Streamlined")
        self.assertEqual(orion["tech_level"], 11)
        self.assertFalse(orion["standard_design"])
        self.assertEqual(orion["electronics"], "Basic Military")
        self.assertEqual(orion["armor_points"], 1)
        self.assertEqual(orion["control"], "One Person Cockpit")
        self.assertEqual(orion["additional_passengers"], 0)
        self.assertEqual(orion["maneuver_drive"], "sC")
        self.assertEqual(orion["power_plant"], "sC")
        self.assertEqual(orion["endurance"], 1)
        self.assertEqual(orion["cargo"], "0.475 tons")
        self.assertEqual(orion["crew"], 1)
        self.assertEqual(orion["airlocks"], 0)
        self.assertEqual(orion["armament"], "Single Turret: Missile Rack")
        self.assertEqual(orion["ammunition"], "12 × Standard Missiles")
        self.assertIsNone(orion["jump_drive"])
        self.assertEqual(orion["length_m"], 10.8)
        self.assertEqual(orion["art_path"], "assets/ships/ship-013-orion.webp")

    def test_apollo_exposes_unarmored_energy_fighter_fit(self) -> None:
        apollo = self.records[14]
        self.assertEqual(apollo["family_id"], 14)
        self.assertEqual(apollo["tons"], 10)
        self.assertEqual(apollo["configuration"], "Streamlined")
        self.assertEqual(apollo["tech_level"], 11)
        self.assertFalse(apollo["standard_design"])
        self.assertEqual(apollo["electronics"], "Basic Military")
        self.assertEqual(apollo["armor_points"], 0)
        self.assertEqual(apollo["control"], "One Person Cockpit")
        self.assertEqual(apollo["additional_passengers"], 0)
        self.assertEqual(apollo["maneuver_drive"], "sC")
        self.assertEqual(apollo["power_plant"], "sG")
        self.assertEqual(apollo["endurance"], 1)
        self.assertEqual(apollo["cargo"], "0 tons")
        self.assertEqual(apollo["crew"], 1)
        self.assertEqual(apollo["airlocks"], 0)
        self.assertEqual(apollo["armament"], "Single Turret: Beam Laser")
        self.assertEqual(apollo["ammunition"], "None carried")
        self.assertIsNone(apollo["jump_drive"])
        self.assertEqual(apollo["length_m"], 12.8)
        self.assertEqual(apollo["art_path"], "assets/ships/ship-014-apollo.webp")

    def test_horatius_exposes_unarmored_laser_interceptor_fit(self) -> None:
        horatius = self.records[16]
        self.assertEqual(horatius["family_id"], 16)
        self.assertEqual(horatius["tons"], 10)
        self.assertEqual(horatius["configuration"], "Streamlined")
        self.assertEqual(horatius["tech_level"], 11)
        self.assertFalse(horatius["standard_design"])
        self.assertEqual(horatius["electronics"], "Basic Military")
        self.assertEqual(horatius["armor_points"], 0)
        self.assertEqual(horatius["control"], "One Person Cockpit")
        self.assertEqual(horatius["additional_passengers"], 0)
        self.assertEqual(horatius["maneuver_drive"], "sC")
        self.assertEqual(horatius["power_plant"], "sG")
        self.assertEqual(horatius["endurance"], 1)
        self.assertEqual(horatius["cargo"], "0 tons")
        self.assertEqual(horatius["crew"], 1)
        self.assertEqual(horatius["airlocks"], 0)
        self.assertEqual(horatius["armament"], "Single Turret: Beam Laser")
        self.assertEqual(horatius["ammunition"], "None carried")
        self.assertIsNone(horatius["jump_drive"])
        self.assertEqual(horatius["length_m"], 11.4)
        self.assertEqual(
            horatius["art_path"],
            "assets/ships/ship-016-horatius.webp",
        )

    def test_pegasus_exposes_passenger_and_freight_boat_fit(self) -> None:
        pegasus = self.records[23]
        self.assertEqual(pegasus["family_id"], 23)
        self.assertEqual(pegasus["tons"], 50)
        self.assertEqual(pegasus["configuration"], "Streamlined")
        self.assertEqual(pegasus["tech_level"], 11)
        self.assertFalse(pegasus["standard_design"])
        self.assertEqual(pegasus["electronics"], "Standard")
        self.assertEqual(pegasus["armor_points"], 2)
        self.assertEqual(pegasus["control"], "Two Person Cockpit")
        self.assertEqual(pegasus["additional_passengers"], 0)
        self.assertEqual(pegasus["maneuver_drive"], "sK")
        self.assertEqual(pegasus["power_plant"], "sK")
        self.assertEqual(pegasus["endurance"], 1)
        self.assertEqual(pegasus["cargo"], "22.05 tons")
        self.assertEqual(pegasus["crew"], 2)
        self.assertEqual(pegasus["airlocks"], 1)
        self.assertIn("24 passenger seats", pegasus["equipment"])
        self.assertIn("Small Craft Fuel Processor", pegasus["equipment"])
        self.assertEqual(pegasus["armament"], "None installed")
        self.assertEqual(pegasus["ammunition"], "None carried")
        self.assertIsNone(pegasus["jump_drive"])
        self.assertEqual(pegasus["length_m"], 23.0)
        self.assertEqual(
            pegasus["art_path"],
            "assets/ships/ship-023-pegasus.webp",
        )

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

    def test_caravanserai_family_exposes_trade_replenishment_and_assault_fits(self) -> None:
        samarkand = self.records[128]
        bukhara = self.records[129]
        merv = self.records[130]

        for record in (samarkand, bukhara, merv):
            self.assertEqual(record["family_id"], 128)
            self.assertEqual(record["tons"], 600)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["bridge_options"], ["Holographic Controls"])
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["jump_drive"], "H")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["jump_count"], 1)
            self.assertEqual(record["maneuver_drive"], "J")
            self.assertEqual(record["power_plant"], "J")
            self.assertEqual(record["endurance"], 2)
            self.assertEqual(record["external_load"], "0 tons")
            self.assertIn("Fuel Scoop", record["equipment"])
            self.assertIn("2 × Fuel Processor", record["equipment"])
            self.assertIn(
                "6 × Point Defense Node Mount: Point Defense Laser",
                record["armament"],
            )
            self.assertEqual(record["ammunition"], "20 × Sandcaster Canisters")
            self.assertEqual(record["length_m"], 72.0)

        self.assertEqual(samarkand["electronics"], "Basic Civilian")
        self.assertEqual(samarkand["cargo"], "248 tons")
        self.assertIn("5 × Triple Turret: Beam Laser", samarkand["armament"])
        self.assertIn("Single Turret: Sandcaster", samarkand["armament"])
        self.assertNotIn("Particle Beam", samarkand["armament"])
        self.assertEqual(
            samarkand["art_path"],
            "assets/ships/ship-128-samarkand.webp",
        )

        self.assertEqual(bukhara["electronics"], "Basic Military")
        self.assertEqual(bukhara["cargo"], "244 tons")
        self.assertIn("Underway Replenishment System", bukhara["equipment"])
        self.assertEqual(bukhara["armament"], samarkand["armament"])
        self.assertEqual(
            bukhara["art_path"],
            "assets/ships/ship-129-bukhara.webp",
        )

        self.assertEqual(merv["electronics"], "Basic Military")
        self.assertEqual(merv["cargo"], "5 tons")
        self.assertIn("60 × Barracks", merv["equipment"])
        self.assertIn("Full Hangar (70 tons contained)", merv["equipment"])
        self.assertIn("3 × Triple Turret: Beam Laser", merv["armament"])
        self.assertIn("2 × Single Turret: Particle Beam", merv["armament"])
        self.assertIn("Single Turret: Sandcaster", merv["armament"])
        self.assertEqual(merv["art_path"], "assets/ships/ship-130-merv.webp")

    def test_exterior_equivalent_bellamy_generations_share_one_plate(self) -> None:
        bellamys = [self.records[catalog_id] for catalog_id in (131, 132, 133)]

        self.assertEqual(
            {record["art_path"] for record in bellamys},
            {"assets/ships/family-131-bellamy.webp"},
        )
        for record in bellamys:
            self.assertEqual(record["family_id"], 131)
            self.assertEqual(record["tons"], 400)
            self.assertEqual(record["configuration"], "Standard")
            self.assertEqual(record["electronics"], "Basic Civilian")
            self.assertEqual(
                record["hull_options"],
                ["Radiation Shielding", "Self Sealing"],
            )
            self.assertEqual(record["armor_points"], 1)
            self.assertEqual(record["jump_drive"], "D")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["maneuver_drive"], "B")
            self.assertEqual(record["power_plant"], "D")
            self.assertEqual(record["endurance"], 3)
            self.assertEqual(record["cargo"], "172 tons")
            self.assertEqual(record["unused_fire_control_stations"], 1)
            self.assertIn("2 × Fuel Processor", record["equipment"])
            self.assertNotIn("Fuel Scoop", record["equipment"])
            self.assertEqual(
                record["armament"],
                "2 × Double Turret: Beam Laser · Beam Laser · "
                "Double Turret: Sandcaster · Missile Rack",
            )
            self.assertEqual(
                record["ammunition"],
                "20 × Sandcaster Canisters · 12 × Standard Missiles",
            )
            self.assertEqual(record["length_m"], 60.0)

    def test_roman_family_exposes_four_distinct_capital_bay_fits(self) -> None:
        scipio = self.records[135]
        onager = self.records[136]
        corvus = self.records[137]
        vesuvius = self.records[138]

        for record in (scipio, onager, corvus, vesuvius):
            self.assertEqual(record["family_id"], 135)
            self.assertEqual(record["tons"], 1000)
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
            self.assertEqual(record["armor_points"], 5)
            self.assertEqual(record["jump_drive"], "K")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["jump_count"], 1)
            self.assertEqual(record["maneuver_drive"], "S")
            self.assertEqual(record["power_plant"], "S")
            self.assertEqual(record["endurance"], 4)
            self.assertIn("Fuel Scoop", record["equipment"])
            self.assertIn("4 × Fuel Processor", record["equipment"])
            self.assertIn("Full Hangar (30 tons contained)", record["equipment"])
            self.assertIn(
                "Carried Craft: Wayfarer Armed (ship-134)",
                record["equipment"],
            )
            self.assertIn("4 × Triple Turret: Beam Laser", record["armament"])
            self.assertIn("2 × Triple Turret: Missile Rack", record["armament"])
            self.assertIn(
                "10 × Point Defense Node Mount: Point Defense Laser",
                record["armament"],
            )
            self.assertEqual(record["length_m"], 82.0)

        self.assertEqual(scipio["cargo"], "2.6 tons")
        self.assertIn("2 × Meson Gun Bay", scipio["armament"])
        self.assertIn("2 × Particle Beam Bay", scipio["armament"])
        self.assertNotIn("Torpedo Bay", scipio["armament"])
        self.assertEqual(scipio["art_path"], "assets/ships/ship-135-scipio.webp")

        self.assertEqual(onager["cargo"], "3 tons")
        self.assertIn("2 × Torpedo Bay 50", onager["armament"])
        self.assertIn("2 × Particle Beam Bay", onager["armament"])
        self.assertEqual(onager["art_path"], "assets/ships/ship-136-onager.webp")

        self.assertEqual(corvus["cargo"], "2.6 tons")
        self.assertIn("4 × Particle Beam Bay", corvus["armament"])
        self.assertNotIn("Meson Gun Bay", corvus["armament"])
        self.assertEqual(corvus["art_path"], "assets/ships/ship-137-corvus.webp")

        self.assertEqual(vesuvius["cargo"], "2.233 tons")
        self.assertIn("2 × Missile Bank", vesuvius["armament"])
        self.assertIn("2 × Particle Beam Bay", vesuvius["armament"])
        self.assertEqual(
            vesuvius["ammunition"],
            "100 × Standard Missiles · 80 × Sandcaster Canisters",
        )
        self.assertEqual(
            vesuvius["art_path"],
            "assets/ships/ship-138-vesuvius.webp",
        )

    def test_moriarty_exposes_its_raider_fit_without_inventing_a_scoop(self) -> None:
        moriarty = self.records[143]

        self.assertEqual(moriarty["family_id"], 143)
        self.assertEqual(moriarty["tons"], 800)
        self.assertEqual(moriarty["configuration"], "Streamlined")
        self.assertEqual(moriarty["electronics"], "Advanced")
        self.assertEqual(
            moriarty["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(moriarty["armor_points"], 4)
        self.assertEqual(moriarty["jump_drive"], "J")
        self.assertEqual(moriarty["jump_distance"], 2)
        self.assertEqual(moriarty["jump_count"], 1)
        self.assertEqual(moriarty["maneuver_drive"], "N")
        self.assertEqual(moriarty["power_plant"], "R")
        self.assertEqual(moriarty["endurance"], 3)
        self.assertEqual(moriarty["cargo"], "143.2 tons")
        self.assertEqual(moriarty["unused_fire_control_stations"], 0)
        self.assertIn("8 × Ships Brig", moriarty["equipment"])
        self.assertNotIn("Fuel Scoop", moriarty["equipment"])
        self.assertNotIn("Fuel Processor", moriarty["equipment"])
        self.assertIn("2 × Particle Beam Bay", moriarty["armament"])
        self.assertIn("2 × Plasma Barbette", moriarty["armament"])
        self.assertIn(
            "8 × Point Defense Node Mount: Point Defense Laser",
            moriarty["armament"],
        )
        self.assertEqual(
            moriarty["ammunition"],
            "192 × Standard Missiles · 80 × Sandcaster Canisters",
        )
        self.assertEqual(moriarty["length_m"], 78.0)
        self.assertEqual(
            moriarty["art_path"],
            "assets/ships/ship-143-moriarty.webp",
        )

    def test_vidocq_exposes_system_cutter_and_caduceus_hangar_fit(self) -> None:
        vidocq = self.records[144]

        self.assertEqual(vidocq["family_id"], 144)
        self.assertEqual(vidocq["tons"], 200)
        self.assertEqual(vidocq["configuration"], "Standard")
        self.assertEqual(vidocq["electronics"], "Basic Military")
        self.assertEqual(
            vidocq["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(vidocq["armor_points"], 4)
        self.assertIsNone(vidocq["jump_drive"])
        self.assertEqual(vidocq["jump_distance"], 0)
        self.assertEqual(vidocq["jump_count"], 0)
        self.assertEqual(vidocq["maneuver_drive"], "F")
        self.assertEqual(vidocq["power_plant"], "F")
        self.assertIsNone(vidocq["thrust_g"])
        self.assertIn("Six Gravity", vidocq["mission_tags"])
        self.assertEqual(vidocq["endurance"], 2)
        self.assertEqual(vidocq["cargo"], "32.1 tons")
        self.assertIn("Fuel Scoop", vidocq["equipment"])
        self.assertIn("Fuel Processor", vidocq["equipment"])
        self.assertIn("Full Hangar (20 tons contained)", vidocq["equipment"])
        self.assertIn(
            "Carried Craft: Caduceus (ship-145)",
            vidocq["equipment"],
        )
        self.assertIn(
            "Triple Turret: Turret Plasma Gun · Turret Plasma Gun",
            vidocq["armament"],
        )
        self.assertIn(
            "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
            vidocq["armament"],
        )
        self.assertIn(
            "2 × Point Defense Node Mount: Point Defense Gatling Laser",
            vidocq["armament"],
        )
        self.assertEqual(
            vidocq["ammunition"],
            "24 × Standard Missiles · 20 × Sandcaster Canisters",
        )
        self.assertEqual(vidocq["length_m"], 46.0)
        self.assertEqual(
            vidocq["art_path"],
            "assets/ships/ship-144-vidocq.webp",
        )

    def test_banks_family_exposes_modular_merchant_and_tizard_fits(self) -> None:
        banks = self.records[147]
        solander = self.records[148]
        parkinson = self.records[149]
        survey = self.records[150]
        scout = self.records[190]

        for record in (banks, solander, parkinson, survey, scout):
            self.assertEqual(record["family_id"], 147)
            self.assertEqual(record["tons"], 300)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["bridge_options"], ["Holographic Controls"])
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["endurance"], 2)
            self.assertIn("Fuel Processor", record["equipment"])
            self.assertNotIn("Fuel Scoop", record["equipment"])
            self.assertEqual(
                record["ammunition"],
                "12 × Standard Missiles · 20 × Sandcaster Canisters",
            )
            self.assertEqual(record["length_m"], 58.0)

        merchants = (banks, solander, parkinson)
        for record in merchants:
            self.assertEqual(record["electronics"], "Basic Civilian")
            self.assertEqual(record["jump_drive"], "C")
            self.assertEqual(record["jump_count"], 1)
            self.assertEqual(record["maneuver_drive"], "E")
            self.assertEqual(record["power_plant"], "E")
            self.assertEqual(record["external_load"], "0 tons")
            self.assertEqual(record["unused_fire_control_stations"], 2)
            self.assertIn("Standard Hangar (4 tons contained)", record["equipment"])
            self.assertEqual(
                record["armament"],
                "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
            )

        self.assertEqual(banks["cargo"], "84.5 tons")
        self.assertEqual(solander["cargo"], "35.5 tons")
        self.assertEqual(parkinson["cargo"], "3 tons")
        self.assertEqual(banks["art_path"], "assets/ships/ship-147-banks.webp")
        self.assertEqual(
            solander["art_path"],
            "assets/ships/ship-148-solander.webp",
        )
        self.assertEqual(
            parkinson["art_path"],
            "assets/ships/ship-149-parkinson.webp",
        )

        for record in (survey, scout):
            self.assertEqual(record["electronics"], "Advanced")
            self.assertEqual(record["unused_fire_control_stations"], 0)
            self.assertIn(
                "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
                record["armament"],
            )
            self.assertIn(
                "2 × Triple Turret: Beam Laser · Beam Laser · Beam Laser",
                record["armament"],
            )

        self.assertEqual(survey["jump_drive"], "C")
        self.assertEqual(survey["jump_count"], 2)
        self.assertEqual(survey["maneuver_drive"], "E")
        self.assertEqual(survey["power_plant"], "E")
        self.assertEqual(survey["cargo"], "2.5 tons")
        self.assertEqual(survey["external_load"], "0 tons")
        self.assertIn("Standard Hangar (24 tons contained)", survey["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-146)", survey["equipment"])
        self.assertEqual(
            survey["art_path"],
            "assets/ships/ship-150-tizard-survey.webp",
        )

        self.assertEqual(scout["jump_drive"], "D")
        self.assertEqual(scout["jump_count"], 1)
        self.assertEqual(scout["maneuver_drive"], "F")
        self.assertEqual(scout["power_plant"], "F")
        self.assertEqual(scout["cargo"], "59.5 tons")
        self.assertEqual(scout["external_load"], "20 tons")
        self.assertIn("Docking Clamp 30", scout["equipment"])
        self.assertIn("Air Raft Hangar", scout["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-189)", scout["equipment"])
        self.assertEqual(
            scout["art_path"],
            "assets/ships/ship-190-tizard-scout.webp",
        )

    def test_franklin_family_exposes_unarmed_light_trader_fits(self) -> None:
        records = {catalog_id: self.records[catalog_id] for catalog_id in range(169, 176)}

        for record in records.values():
            self.assertEqual(record["family_id"], 169)
            self.assertEqual(record["tons"], 100)
            self.assertEqual(record["configuration"], "Streamlined")
            self.assertEqual(record["bridge_options"], ["Holographic Controls"])
            self.assertEqual(record["armor_points"], 4)
            self.assertEqual(record["jump_drive"], "A")
            self.assertEqual(record["jump_distance"], 2)
            self.assertEqual(record["endurance"], 2)
            self.assertEqual(record["external_load"], "0 tons")
            self.assertEqual(record["unused_fire_control_stations"], 1)
            self.assertIn("Fuel Processor", record["equipment"])
            self.assertIn("Autodoc", record["equipment"])
            self.assertNotIn("Fuel Scoop", record["equipment"])
            self.assertEqual(record["armament"], "None installed")
            self.assertEqual(record["ammunition"], "None carried")
            self.assertEqual(record["length_m"], 38.0)

        exterior_equivalents = (records[169], records[171], records[172])
        self.assertEqual(
            {record["art_path"] for record in exterior_equivalents},
            {"assets/ships/family-169-franklin-commerce.webp"},
        )
        for record in exterior_equivalents:
            self.assertEqual(record["electronics"], "Basic Civilian")
            self.assertEqual(record["jump_count"], 1)
            self.assertEqual(record["maneuver_drive"], "B")
            self.assertEqual(record["power_plant"], "B")

        self.assertEqual(records[169]["cargo"], "19.5 tons")
        self.assertEqual(records[171]["cargo"], "19.5 tons")
        self.assertEqual(records[172]["cargo"], "27.5 tons")

        poor_richard = records[170]
        self.assertEqual(poor_richard["maneuver_drive"], "A")
        self.assertEqual(poor_richard["power_plant"], "A")
        self.assertEqual(poor_richard["cargo"], "21.5 tons")
        self.assertIn("Standard Hangar (4 tons contained)", poor_richard["equipment"])
        self.assertEqual(
            poor_richard["art_path"],
            "assets/ships/ship-170-poor-richard.webp",
        )

        deborah = records[173]
        self.assertEqual(deborah["cargo"], "7.5 tons")
        self.assertIn("6 × Stateroom", deborah["equipment"])
        self.assertEqual(deborah["art_path"], "assets/ships/ship-173-deborah.webp")

        postmaster = records[174]
        self.assertEqual(postmaster["maneuver_drive"], "C")
        self.assertEqual(postmaster["power_plant"], "C")
        self.assertEqual(postmaster["cargo"], "2.5 tons")
        self.assertEqual(
            postmaster["art_path"],
            "assets/ships/ship-174-postmaster.webp",
        )

        gulf_stream = records[175]
        self.assertEqual(gulf_stream["electronics"], "Advanced")
        self.assertEqual(gulf_stream["jump_count"], 2)
        self.assertEqual(gulf_stream["maneuver_drive"], "B")
        self.assertEqual(gulf_stream["power_plant"], "B")
        self.assertEqual(gulf_stream["cargo"], "1.5 tons")
        self.assertEqual(
            gulf_stream["art_path"],
            "assets/ships/ship-175-gulf-stream.webp",
        )

    def test_trafalgar_exposes_distributed_dreadnought_battery(self) -> None:
        trafalgar = self.records[180]

        self.assertEqual(trafalgar["family_id"], 180)
        self.assertEqual(trafalgar["tons"], 5000)
        self.assertEqual(trafalgar["configuration"], "Streamlined")
        self.assertEqual(trafalgar["electronics"], "Advanced")
        self.assertEqual(
            trafalgar["bridge_options"],
            ["Command Bridge", "Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(trafalgar["hull_options"], ["Radiation Shielding"])
        self.assertEqual(trafalgar["armor_points"], 10)
        self.assertEqual(trafalgar["jump_drive"], "Z")
        self.assertEqual(trafalgar["jump_distance"], 2)
        self.assertEqual(trafalgar["jump_count"], 1)
        self.assertEqual(trafalgar["maneuver_drive"], "Z")
        self.assertEqual(trafalgar["power_plant"], "Z")
        self.assertEqual(trafalgar["endurance"], 3)
        self.assertEqual(trafalgar["cargo"], "959.1 tons")
        self.assertEqual(trafalgar["unused_fire_control_stations"], 0)
        self.assertIn("Full Hangar (100 tons contained)", trafalgar["equipment"])
        self.assertIn("Carried Craft: Boreas (ship-176)", trafalgar["equipment"])
        self.assertIn("Carried Craft: Zephyrus (ship-177)", trafalgar["equipment"])
        self.assertIn("2 × Carried Craft: Castor (ship-178)", trafalgar["equipment"])
        self.assertNotIn("Fuel Scoop", trafalgar["equipment"])
        self.assertNotIn("Fuel Processor", trafalgar["equipment"])
        self.assertIn("12 × Triple Turret: Beam Laser", trafalgar["armament"])
        self.assertIn("10 × Triple Turret: Missile Rack", trafalgar["armament"])
        self.assertIn("8 × Plasma Barbette", trafalgar["armament"])
        self.assertIn("6 × Railgun Barbette", trafalgar["armament"])
        self.assertIn("2 × Torpedo Bay 100", trafalgar["armament"])
        self.assertIn("6 × Meson Gun Bay", trafalgar["armament"])
        self.assertIn("6 × Particle Beam Bay", trafalgar["armament"])
        self.assertIn(
            "50 × Point Defense Node Mount: Point Defense Laser",
            trafalgar["armament"],
        )
        self.assertEqual(
            trafalgar["ammunition"],
            "12 × Torpedo Basics · 576 × Railgun Basics · "
            "1440 × Standard Missiles · 400 × Sandcaster Canisters",
        )
        self.assertEqual(trafalgar["length_m"], 160.0)
        self.assertEqual(
            trafalgar["art_path"],
            "assets/ships/ship-180-trafalgar.webp",
        )

    def test_silk_road_exposes_carrier_cargo_and_reserved_hardpoints(self) -> None:
        silk_road = self.records[182]

        self.assertEqual(silk_road["family_id"], 182)
        self.assertEqual(silk_road["tons"], 4500)
        self.assertEqual(silk_road["configuration"], "Standard")
        self.assertEqual(silk_road["electronics"], "Basic Civilian")
        self.assertEqual(silk_road["bridge_options"], ["Holographic Controls"])
        self.assertEqual(silk_road["armor_points"], 4)
        self.assertEqual(silk_road["jump_drive"], "Z")
        self.assertEqual(silk_road["jump_distance"], 2)
        self.assertEqual(silk_road["jump_count"], 1)
        self.assertEqual(silk_road["maneuver_drive"], "Z")
        self.assertEqual(silk_road["power_plant"], "Z")
        self.assertEqual(silk_road["endurance"], 2)
        self.assertEqual(silk_road["cargo"], "2,647.2 tons")
        self.assertEqual(silk_road["unused_fire_control_stations"], 25)
        self.assertIn("Full Hangar (60 tons contained)", silk_road["equipment"])
        self.assertIn(
            "2 × Carried Craft: Wayfarer Cargo (ship-181)",
            silk_road["equipment"],
        )
        self.assertNotIn("Fuel Scoop", silk_road["equipment"])
        self.assertNotIn("Fuel Processor", silk_road["equipment"])
        self.assertIn(
            "20 × Double Turret: Beam Laser · Beam Laser",
            silk_road["armament"],
        )
        self.assertIn(
            "45 × Point Defense Node Mount: Point Defense Laser",
            silk_road["armament"],
        )
        self.assertEqual(silk_road["ammunition"], "None carried")
        self.assertEqual(silk_road["length_m"], 150.0)
        self.assertEqual(
            silk_road["art_path"],
            "assets/ships/ship-182-silk-road.webp",
        )

    def test_zheng_he_exposes_patrol_battery_and_grapnel_hangar(self) -> None:
        zheng_he = self.records[183]

        self.assertEqual(zheng_he["family_id"], 183)
        self.assertEqual(zheng_he["tons"], 400)
        self.assertEqual(zheng_he["configuration"], "Standard")
        self.assertEqual(zheng_he["electronics"], "Advanced")
        self.assertEqual(
            zheng_he["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(zheng_he["armor_points"], 4)
        self.assertEqual(zheng_he["jump_drive"], "D")
        self.assertEqual(zheng_he["jump_distance"], 2)
        self.assertEqual(zheng_he["jump_count"], 1)
        self.assertEqual(zheng_he["maneuver_drive"], "F")
        self.assertEqual(zheng_he["power_plant"], "F")
        self.assertEqual(zheng_he["endurance"], 4)
        self.assertEqual(zheng_he["cargo"], "5 tons")
        self.assertIn("Fuel Scoop", zheng_he["equipment"])
        self.assertIn("5 × Fuel Processor", zheng_he["equipment"])
        self.assertIn("Standard Hangar (30 tons contained)", zheng_he["equipment"])
        self.assertIn("Carried Craft: Grapnel (ship-211)", zheng_he["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Sandcaster",
            zheng_he["armament"],
        )
        self.assertIn("2 × Particle Beam Barbette", zheng_he["armament"])
        self.assertIn("Missile Bank", zheng_he["armament"])
        self.assertIn(
            "2 × Point Defense Node Mount: Point Defense Minigun",
            zheng_he["armament"],
        )
        self.assertIn(
            "2 × Point Defense Node Mount: Point Defense Laser",
            zheng_he["armament"],
        )
        self.assertEqual(
            zheng_he["ammunition"],
            "120 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(zheng_he["length_m"], 62.0)
        self.assertEqual(
            zheng_he["art_path"],
            "assets/ships/ship-183-zheng-he.webp",
        )

    def test_rampart_exposes_no_jump_armored_system_patrol_fit(self) -> None:
        rampart = self.records[184]

        self.assertEqual(rampart["family_id"], 184)
        self.assertEqual(rampart["tons"], 400)
        self.assertEqual(rampart["configuration"], "Streamlined")
        self.assertEqual(rampart["electronics"], "Advanced")
        self.assertEqual(
            rampart["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(rampart["hull_options"], ["Radiation Shielding"])
        self.assertEqual(rampart["armor_points"], 10)
        self.assertIsNone(rampart["jump_drive"])
        self.assertEqual(rampart["jump_distance"], 0)
        self.assertEqual(rampart["jump_count"], 0)
        self.assertEqual(rampart["maneuver_drive"], "M")
        self.assertEqual(rampart["power_plant"], "M")
        self.assertEqual(rampart["endurance"], 8)
        self.assertEqual(rampart["cargo"], "7.25 tons")
        self.assertIn("8 × Fuel Processor", rampart["equipment"])
        self.assertNotIn("Fuel Scoop", rampart["equipment"])
        self.assertIn("Full Hangar (30 tons contained)", rampart["equipment"])
        self.assertIn("Carried Craft: Grapnel (ship-211)", rampart["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            rampart["armament"],
        )
        self.assertIn(
            "Triple Turret: Missile Rack · Missile Rack · Sandcaster",
            rampart["armament"],
        )
        self.assertIn("2 × Particle Beam Barbette", rampart["armament"])
        self.assertIn(
            "4 × Point Defense Node Mount: Point Defense Laser",
            rampart["armament"],
        )
        self.assertEqual(
            rampart["ammunition"],
            "144 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(rampart["length_m"], 58.0)
        self.assertEqual(
            rampart["art_path"],
            "assets/ships/ship-184-rampart.webp",
        )

    def test_cerberus_exposes_jump_one_patrol_and_external_caduceus(self) -> None:
        cerberus = self.records[186]

        self.assertEqual(cerberus["family_id"], 186)
        self.assertEqual(cerberus["tons"], 400)
        self.assertEqual(cerberus["configuration"], "Streamlined")
        self.assertEqual(cerberus["electronics"], "Advanced")
        self.assertEqual(cerberus["bridge_options"], ["Holographic Controls"])
        self.assertEqual(cerberus["hull_options"], ["Radiation Shielding"])
        self.assertEqual(cerberus["armor_points"], 10)
        self.assertEqual(cerberus["jump_drive"], "C")
        self.assertEqual(cerberus["jump_distance"], 1)
        self.assertEqual(cerberus["jump_count"], 1)
        self.assertEqual(cerberus["maneuver_drive"], "Q")
        self.assertEqual(cerberus["power_plant"], "Q")
        self.assertEqual(cerberus["endurance"], 2)
        self.assertEqual(cerberus["cargo"], "0.9 tons")
        self.assertEqual(cerberus["external_load"], "20 tons")
        self.assertIn("2 × Fuel Processor", cerberus["equipment"])
        self.assertNotIn("Fuel Scoop", cerberus["equipment"])
        self.assertNotIn("Hangar", cerberus["equipment"])
        self.assertIn("Docking Clamp 30", cerberus["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-185)", cerberus["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            cerberus["armament"],
        )
        self.assertIn(
            "Triple Turret: Missile Rack · Missile Rack · Sandcaster",
            cerberus["armament"],
        )
        self.assertIn("Particle Beam Barbette", cerberus["armament"])
        self.assertIn("Particle Beam Bay", cerberus["armament"])
        self.assertIn(
            "4 × Point Defense Node Mount: Point Defense Laser",
            cerberus["armament"],
        )
        self.assertEqual(
            cerberus["ammunition"],
            "120 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(cerberus["length_m"], 66.0)
        self.assertEqual(
            cerberus["art_path"],
            "assets/ships/ship-186-cerberus.webp",
        )

    def test_faraday_exposes_exploration_carrier_and_distributed_battery(self) -> None:
        faraday = self.records[191]

        self.assertEqual(faraday["family_id"], 191)
        self.assertEqual(faraday["tons"], 2500)
        self.assertEqual(faraday["configuration"], "Standard")
        self.assertEqual(faraday["electronics"], "Advanced")
        self.assertEqual(faraday["bridge_options"], ["Holographic Controls"])
        self.assertEqual(faraday["hull_options"], ["Self Sealing"])
        self.assertEqual(faraday["armor_points"], 2)
        self.assertEqual(faraday["jump_drive"], "T")
        self.assertEqual(faraday["jump_distance"], 2)
        self.assertEqual(faraday["jump_count"], 2)
        self.assertEqual(faraday["maneuver_drive"], "T")
        self.assertEqual(faraday["power_plant"], "T")
        self.assertEqual(faraday["endurance"], 2)
        self.assertEqual(faraday["cargo"], "2 tons")
        self.assertEqual(faraday["external_load"], "80 tons")
        self.assertEqual(faraday["crew"], 66)
        self.assertIn("Fuel Scoop", faraday["equipment"])
        self.assertIn("20 × Fuel Processor", faraday["equipment"])
        self.assertIn("Full Hangar (600 tons contained)", faraday["equipment"])
        self.assertIn("Docking Clamp 30", faraday["equipment"])
        self.assertIn("Docking Clamp 90", faraday["equipment"])
        self.assertIn(
            "2 × Carried Craft: Tizard (ship-190)",
            faraday["equipment"],
        )
        self.assertIn(
            "Carried Craft: Wayfarer Cargo (ship-187)",
            faraday["equipment"],
        )
        self.assertIn(
            "Carried Craft: Proteus Surveyor (ship-188)",
            faraday["equipment"],
        )
        self.assertIn(
            "20 × Double Turret: Beam Laser · Beam Laser",
            faraday["armament"],
        )
        self.assertIn(
            "5 × Double Turret: Missile Rack · Sandcaster",
            faraday["armament"],
        )
        self.assertIn(
            "25 × Point Defense Node Mount: Point Defense Laser",
            faraday["armament"],
        )
        self.assertEqual(faraday["unused_fire_control_stations"], 0)
        self.assertEqual(
            faraday["ammunition"],
            "120 × Standard Missiles · 200 × Sandcaster Canisters",
        )
        self.assertEqual(faraday["length_m"], 132.0)
        self.assertEqual(
            faraday["art_path"],
            "assets/ships/ship-191-faraday.webp",
        )

    def test_hudson_exposes_unarmed_jump_two_mixed_trader(self) -> None:
        hudson = self.records[192]

        self.assertEqual(hudson["family_id"], 192)
        self.assertEqual(hudson["tons"], 200)
        self.assertEqual(hudson["configuration"], "Standard")
        self.assertEqual(hudson["electronics"], "Basic Civilian")
        self.assertTrue(hudson["standard_design"])
        self.assertEqual(hudson["armor_points"], 2)
        self.assertEqual(hudson["jump_drive"], "B")
        self.assertEqual(hudson["jump_distance"], 2)
        self.assertEqual(hudson["jump_count"], 1)
        self.assertEqual(hudson["maneuver_drive"], "A")
        self.assertEqual(hudson["power_plant"], "B")
        self.assertEqual(hudson["endurance"], 4)
        self.assertEqual(hudson["cargo"], "53 tons")
        self.assertEqual(hudson["external_load"], "0 tons")
        self.assertEqual(hudson["crew"], 3)
        self.assertIn("10 × Stateroom", hudson["equipment"])
        self.assertIn("20 × Low Berth", hudson["equipment"])
        self.assertIn("2 × Fuel Processor", hudson["equipment"])
        self.assertIn("Fuel Scoop", hudson["equipment"])
        self.assertEqual(hudson["armament"], "None installed")
        self.assertEqual(hudson["ammunition"], "None carried")
        self.assertEqual(hudson["unused_fire_control_stations"], 2)
        self.assertEqual(hudson["length_m"], 48.0)
        self.assertEqual(
            hudson["art_path"],
            "assets/ships/ship-192-hudson.webp",
        )

    def test_crusoe_exposes_armed_frontier_freight_trader(self) -> None:
        crusoe = self.records[193]

        self.assertEqual(crusoe["family_id"], 193)
        self.assertEqual(crusoe["tons"], 300)
        self.assertEqual(crusoe["configuration"], "Standard")
        self.assertEqual(crusoe["electronics"], "Basic Civilian")
        self.assertTrue(crusoe["standard_design"])
        self.assertEqual(crusoe["armor_points"], 2)
        self.assertEqual(crusoe["jump_drive"], "C")
        self.assertEqual(crusoe["jump_distance"], 2)
        self.assertEqual(crusoe["jump_count"], 1)
        self.assertEqual(crusoe["maneuver_drive"], "C")
        self.assertEqual(crusoe["power_plant"], "C")
        self.assertEqual(crusoe["endurance"], 4)
        self.assertEqual(crusoe["cargo"], "92 tons")
        self.assertEqual(crusoe["external_load"], "0 tons")
        self.assertEqual(crusoe["crew"], 7)
        self.assertIn("12 × Stateroom", crusoe["equipment"])
        self.assertIn("12 × Low Berth", crusoe["equipment"])
        self.assertIn("3 × Fuel Processor", crusoe["equipment"])
        self.assertIn("Fuel Scoop", crusoe["equipment"])
        self.assertEqual(
            crusoe["armament"].count(
                "Triple Turret: Pulse Laser · Pulse Laser · Pulse Laser"
            ),
            2,
        )
        self.assertIn(
            "Triple Turret: Sandcaster · Sandcaster · Sandcaster",
            crusoe["armament"],
        )
        self.assertEqual(crusoe["unused_fire_control_stations"], 0)
        self.assertEqual(crusoe["ammunition"], "100 × Sandcaster Canisters")
        self.assertEqual(crusoe["length_m"], 58.0)
        self.assertEqual(
            crusoe["art_path"],
            "assets/ships/ship-193-crusoe.webp",
        )

    def test_dory_exposes_unarmed_standard_utility_launch(self) -> None:
        dory = self.records[194]

        self.assertEqual(dory["family_id"], 194)
        self.assertEqual(dory["tons"], 20)
        self.assertEqual(dory["configuration"], "Standard")
        self.assertEqual(dory["electronics"], "Standard")
        self.assertTrue(dory["standard_design"])
        self.assertEqual(dory["armor_points"], 0)
        self.assertEqual(dory["control"], "Two Person Control Cabin")
        self.assertEqual(dory["additional_passengers"], 0)
        self.assertEqual(dory["maneuver_drive"], "sA")
        self.assertEqual(dory["power_plant"], "sA")
        self.assertEqual(dory["endurance"], 1)
        self.assertEqual(dory["cargo"], "10.9 tons")
        self.assertEqual(dory["external_load"], "0 tons")
        self.assertEqual(dory["crew"], 1)
        self.assertIn("Priority cargo module", dory["equipment"])
        self.assertEqual(dory["armament"], "None installed")
        self.assertEqual(dory["ammunition"], "None carried")
        self.assertEqual(dory["unused_fire_control_stations"], 1)
        self.assertEqual(dory["length_m"], 14.5)
        self.assertEqual(
            dory["art_path"],
            "assets/ships/ship-194-dory.webp",
        )

    def test_decatur_exposes_stealth_corvette_and_missile_battery(self) -> None:
        decatur = self.records[195]

        self.assertEqual(decatur["family_id"], 195)
        self.assertEqual(decatur["tons"], 300)
        self.assertEqual(decatur["configuration"], "Standard")
        self.assertEqual(decatur["electronics"], "Advanced")
        self.assertTrue(decatur["standard_design"])
        self.assertEqual(decatur["armor_points"], 8)
        self.assertEqual(decatur["hull_options"], ["Stealth"])
        self.assertEqual(decatur["jump_drive"], "C")
        self.assertEqual(decatur["jump_distance"], 2)
        self.assertEqual(decatur["jump_count"], 1)
        self.assertEqual(decatur["maneuver_drive"], "J")
        self.assertEqual(decatur["power_plant"], "J")
        self.assertEqual(decatur["endurance"], 4)
        self.assertEqual(decatur["cargo"], "17 tons")
        self.assertEqual(decatur["external_load"], "0 tons")
        self.assertEqual(decatur["crew"], 18)
        self.assertIn("9 × Stateroom", decatur["equipment"])
        self.assertIn("5 × Emergency Low Berth", decatur["equipment"])
        self.assertIn("Armory", decatur["equipment"])
        self.assertIn("4 × Detention Cell", decatur["equipment"])
        self.assertIn("5 × Fuel Processor", decatur["equipment"])
        self.assertIn("Fuel Scoop", decatur["equipment"])
        self.assertEqual(
            decatur["armament"].count(
                "Triple Turret: Missile Rack · Missile Rack · Missile Rack"
            ),
            2,
        )
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            decatur["armament"],
        )
        self.assertEqual(decatur["unused_fire_control_stations"], 0)
        self.assertEqual(decatur["ammunition"], "120 × Smart Missiles")
        self.assertEqual(decatur["length_m"], 60.0)
        self.assertEqual(
            decatur["art_path"],
            "assets/ships/ship-195-decatur.webp",
        )

    def test_fugger_exposes_unarmed_bulk_freight_capacity(self) -> None:
        fugger = self.records[196]

        self.assertEqual(fugger["family_id"], 196)
        self.assertEqual(fugger["tons"], 400)
        self.assertEqual(fugger["configuration"], "Standard")
        self.assertEqual(fugger["electronics"], "Basic Civilian")
        self.assertTrue(fugger["standard_design"])
        self.assertEqual(fugger["armor_points"], 2)
        self.assertEqual(fugger["jump_drive"], "B")
        self.assertEqual(fugger["jump_distance"], 1)
        self.assertEqual(fugger["jump_count"], 1)
        self.assertEqual(fugger["maneuver_drive"], "B")
        self.assertEqual(fugger["power_plant"], "B")
        self.assertEqual(fugger["endurance"], 4)
        self.assertEqual(fugger["cargo"], "261 tons")
        self.assertEqual(fugger["external_load"], "0 tons")
        self.assertEqual(fugger["crew"], 3)
        self.assertIn("4 × Stateroom", fugger["equipment"])
        self.assertIn("2 × Emergency Low Berth", fugger["equipment"])
        self.assertIn("3 × Fuel Processor", fugger["equipment"])
        self.assertIn("Fuel Scoop", fugger["equipment"])
        self.assertEqual(fugger["armament"], "None installed")
        self.assertEqual(fugger["ammunition"], "None carried")
        self.assertEqual(fugger["unused_fire_control_stations"], 4)
        self.assertEqual(fugger["length_m"], 64.0)
        self.assertEqual(
            fugger["art_path"],
            "assets/ships/ship-196-fugger.webp",
        )

    def test_pullman_exposes_unarmed_scheduled_passenger_liner(self) -> None:
        pullman = self.records[197]

        self.assertEqual(pullman["family_id"], 197)
        self.assertEqual(pullman["tons"], 300)
        self.assertEqual(pullman["configuration"], "Standard")
        self.assertEqual(pullman["electronics"], "Basic Civilian")
        self.assertTrue(pullman["standard_design"])
        self.assertEqual(pullman["armor_points"], 2)
        self.assertEqual(pullman["jump_drive"], "B")
        self.assertEqual(pullman["jump_distance"], 1)
        self.assertEqual(pullman["jump_count"], 1)
        self.assertEqual(pullman["maneuver_drive"], "B")
        self.assertEqual(pullman["power_plant"], "B")
        self.assertEqual(pullman["endurance"], 4)
        self.assertEqual(pullman["cargo"], "46 tons")
        self.assertEqual(pullman["external_load"], "0 tons")
        self.assertEqual(pullman["crew"], 7)
        self.assertIn("35 × Stateroom", pullman["equipment"])
        self.assertIn("20 × Low Berth", pullman["equipment"])
        self.assertIn("2 × Fuel Processor", pullman["equipment"])
        self.assertIn("Fuel Scoop", pullman["equipment"])
        self.assertEqual(pullman["armament"], "None installed")
        self.assertEqual(pullman["ammunition"], "None carried")
        self.assertEqual(pullman["unused_fire_control_stations"], 3)
        self.assertEqual(pullman["length_m"], 72.0)
        self.assertEqual(
            pullman["art_path"],
            "assets/ships/ship-197-pullman.webp",
        )

    def test_pony_express_exposes_no_scoop_jump_two_courier(self) -> None:
        courier = self.records[198]

        self.assertEqual(courier["family_id"], 198)
        self.assertEqual(courier["tons"], 100)
        self.assertEqual(courier["configuration"], "Streamlined")
        self.assertEqual(courier["electronics"], "Basic Civilian")
        self.assertTrue(courier["standard_design"])
        self.assertEqual(courier["armor_points"], 2)
        self.assertEqual(courier["jump_drive"], "A")
        self.assertEqual(courier["jump_distance"], 2)
        self.assertEqual(courier["jump_count"], 1)
        self.assertEqual(courier["maneuver_drive"], "B")
        self.assertEqual(courier["power_plant"], "B")
        self.assertEqual(courier["endurance"], 4)
        self.assertEqual(courier["cargo"], "16 tons")
        self.assertEqual(courier["external_load"], "0 tons")
        self.assertEqual(courier["crew"], 3)
        self.assertIn("4 × Stateroom", courier["equipment"])
        self.assertIn("Emergency Low Berth", courier["equipment"])
        self.assertIn("2 × Fuel Processor", courier["equipment"])
        self.assertNotIn("Fuel Scoop", courier["equipment"])
        self.assertEqual(courier["armament"], "None installed")
        self.assertEqual(courier["ammunition"], "None carried")
        self.assertEqual(courier["unused_fire_control_stations"], 1)
        self.assertEqual(courier["length_m"], 38.0)
        self.assertEqual(
            courier["art_path"],
            "assets/ships/ship-198-pony-express.webp",
        )

    def test_cleopatra_exposes_no_scoop_unarmed_diplomatic_yacht(self) -> None:
        yacht = self.records[199]

        self.assertEqual(yacht["family_id"], 199)
        self.assertEqual(yacht["tons"], 100)
        self.assertEqual(yacht["configuration"], "Streamlined")
        self.assertEqual(yacht["electronics"], "Basic Civilian")
        self.assertTrue(yacht["standard_design"])
        self.assertEqual(yacht["armor_points"], 2)
        self.assertEqual(yacht["jump_drive"], "A")
        self.assertEqual(yacht["jump_distance"], 2)
        self.assertEqual(yacht["jump_count"], 1)
        self.assertEqual(yacht["maneuver_drive"], "A")
        self.assertEqual(yacht["power_plant"], "A")
        self.assertEqual(yacht["endurance"], 4)
        self.assertEqual(yacht["cargo"], "12 tons")
        self.assertEqual(yacht["external_load"], "0 tons")
        self.assertEqual(yacht["crew"], 3)
        self.assertIn("6 × Stateroom", yacht["equipment"])
        self.assertIn("3 × Emergency Low Berth", yacht["equipment"])
        self.assertIn("2 × Fuel Processor", yacht["equipment"])
        self.assertIn("2 × Luxuries", yacht["equipment"])
        self.assertNotIn("Fuel Scoop", yacht["equipment"])
        self.assertEqual(yacht["armament"], "None installed")
        self.assertEqual(yacht["ammunition"], "None carried")
        self.assertEqual(yacht["unused_fire_control_stations"], 1)
        self.assertEqual(yacht["length_m"], 40.0)
        self.assertEqual(
            yacht["art_path"],
            "assets/ships/ship-199-cleopatra.webp",
        )

    def test_thermopylae_exposes_fixed_pulse_interceptor(self) -> None:
        fighter = self.records[200]

        self.assertEqual(fighter["family_id"], 200)
        self.assertEqual(fighter["tons"], 10)
        self.assertEqual(fighter["configuration"], "Streamlined")
        self.assertEqual(fighter["electronics"], "Standard")
        self.assertTrue(fighter["standard_design"])
        self.assertEqual(fighter["armor_points"], 0)
        self.assertEqual(fighter["control"], "One Person Cockpit")
        self.assertEqual(fighter["additional_passengers"], 0)
        self.assertEqual(fighter["maneuver_drive"], "sC")
        self.assertEqual(fighter["power_plant"], "sL")
        self.assertEqual(fighter["endurance"], 1)
        self.assertEqual(fighter["cargo"], "0 tons")
        self.assertEqual(fighter["external_load"], "0 tons")
        self.assertEqual(fighter["crew"], 1)
        self.assertEqual(fighter["equipment"], "")
        self.assertEqual(fighter["armament"], "Fixed Single Turret: Pulse Laser")
        self.assertEqual(fighter["ammunition"], "None carried")
        self.assertEqual(fighter["unused_fire_control_stations"], 0)
        self.assertEqual(fighter["length_m"], 11.8)
        self.assertEqual(
            fighter["art_path"],
            "assets/ships/ship-200-thermopylae.webp",
        )

    def test_lighter_exposes_unarmed_five_gravity_cargo_pinnace(self) -> None:
        lighter = self.records[201]

        self.assertEqual(lighter["family_id"], 201)
        self.assertEqual(lighter["tons"], 40)
        self.assertEqual(lighter["configuration"], "Standard")
        self.assertEqual(lighter["electronics"], "Standard")
        self.assertTrue(lighter["standard_design"])
        self.assertEqual(lighter["armor_points"], 0)
        self.assertEqual(lighter["control"], "One Person Control Cabin")
        self.assertEqual(lighter["additional_passengers"], 0)
        self.assertEqual(lighter["maneuver_drive"], "sK")
        self.assertEqual(lighter["power_plant"], "sL")
        self.assertEqual(lighter["endurance"], 1)
        self.assertEqual(lighter["cargo"], "25 tons")
        self.assertEqual(lighter["external_load"], "0 tons")
        self.assertEqual(lighter["crew"], 1)
        self.assertIn("Priority cargo module", lighter["equipment"])
        self.assertEqual(lighter["armament"], "None installed")
        self.assertEqual(lighter["ammunition"], "None carried")
        self.assertEqual(lighter["unused_fire_control_stations"], 1)
        self.assertEqual(lighter["length_m"], 20.5)
        self.assertEqual(
            lighter["art_path"],
            "assets/ships/ship-201-lighter.webp",
        )

    def test_tender_exposes_unarmed_six_gravity_ships_boat(self) -> None:
        tender = self.records[202]
        self.assertEqual(tender["family_id"], 202)
        self.assertEqual(tender["tons"], 30)
        self.assertEqual(tender["configuration"], "Standard")
        self.assertEqual(tender["armor_points"], 0)
        self.assertEqual(tender["control"], "One Person Control Cabin")
        self.assertEqual(tender["maneuver_drive"], "sJ")
        self.assertEqual(tender["power_plant"], "sJ")
        self.assertEqual(tender["endurance"], 1)
        self.assertEqual(tender["cargo"], "16.7 tons")
        self.assertEqual(tender["crew"], 1)
        self.assertIn("Priority cargo module", tender["equipment"])
        self.assertEqual(tender["armament"], "None installed")
        self.assertEqual(tender["ammunition"], "None carried")
        self.assertEqual(tender["unused_fire_control_stations"], 1)
        self.assertEqual(tender["length_m"], 17.5)
        self.assertEqual(tender["art_path"], "assets/ships/ship-202-tender.webp")

    def test_bactrian_exposes_twin_hump_bulk_shuttle(self) -> None:
        shuttle = self.records[203]
        self.assertEqual(shuttle["family_id"], 203)
        self.assertEqual(shuttle["tons"], 90)
        self.assertEqual(shuttle["configuration"], "Standard")
        self.assertEqual(shuttle["armor_points"], 0)
        self.assertEqual(shuttle["control"], "Two Person Control Cabin")
        self.assertEqual(shuttle["maneuver_drive"], "sN")
        self.assertEqual(shuttle["power_plant"], "sN")
        self.assertEqual(shuttle["endurance"], 1)
        self.assertEqual(shuttle["cargo"], "67.4 tons")
        self.assertEqual(shuttle["crew"], 2)
        self.assertIn("Priority cargo module", shuttle["equipment"])
        self.assertEqual(shuttle["armament"], "None installed")
        self.assertEqual(shuttle["ammunition"], "None carried")
        self.assertEqual(shuttle["unused_fire_control_stations"], 1)
        self.assertEqual(shuttle["length_m"], 30.0)
        self.assertEqual(shuttle["art_path"], "assets/ships/ship-203-bactrian.webp")

    def test_perry_exposes_stealth_patrol_frigate_and_fighter_hangar(self) -> None:
        frigate = self.records[204]
        self.assertEqual(frigate["family_id"], 204)
        self.assertEqual(frigate["tons"], 300)
        self.assertEqual(frigate["configuration"], "Standard")
        self.assertEqual(frigate["electronics"], "Advanced")
        self.assertTrue(frigate["standard_design"])
        self.assertEqual(frigate["armor_points"], 8)
        self.assertIn("Stealth", frigate["hull_options"])
        self.assertEqual(frigate["jump_drive"], "C")
        self.assertEqual(frigate["jump_distance"], 2)
        self.assertEqual(frigate["jump_count"], 1)
        self.assertEqual(frigate["maneuver_drive"], "F")
        self.assertEqual(frigate["power_plant"], "F")
        self.assertEqual(frigate["thrust_g"], 4)
        self.assertEqual(frigate["endurance"], 4)
        self.assertEqual(frigate["cargo"], "22 tons")
        self.assertEqual(frigate["crew"], 20)
        self.assertIn("Custom Hangar (20 tons contained)", frigate["equipment"])
        self.assertIn(
            "2 × Carried Craft: Thermopylae (ship-200)",
            frigate["equipment"],
        )
        self.assertEqual(frigate["armament"].count("Triple Turret"), 3)
        self.assertEqual(frigate["armament"].count("Missile Rack"), 6)
        self.assertEqual(frigate["armament"].count("Beam Laser"), 3)
        self.assertEqual(frigate["ammunition"], "120 × Smart Missiles")
        self.assertEqual(frigate["unused_fire_control_stations"], 0)
        self.assertEqual(frigate["length_m"], 64.0)
        self.assertEqual(frigate["art_path"], "assets/ships/ship-204-perry.webp")

    def test_casemate_exposes_armored_system_defense_boat(self) -> None:
        boat = self.records[205]
        self.assertEqual(boat["family_id"], 205)
        self.assertEqual(boat["tons"], 400)
        self.assertEqual(boat["configuration"], "Streamlined")
        self.assertEqual(boat["electronics"], "Basic Civilian")
        self.assertTrue(boat["standard_design"])
        self.assertEqual(boat["armor_points"], 8)
        self.assertEqual(boat["hull_options"], [])
        self.assertIsNone(boat["jump_drive"])
        self.assertEqual(boat["jump_distance"], 0)
        self.assertEqual(boat["jump_count"], 0)
        self.assertEqual(boat["maneuver_drive"], "M")
        self.assertEqual(boat["power_plant"], "M")
        self.assertEqual(boat["assertions"]["thrust_g"], 6)
        self.assertEqual(boat["endurance"], 4)
        self.assertEqual(boat["cargo"], "107 tons")
        self.assertEqual(boat["external_load"], "0 tons")
        self.assertEqual(boat["crew"], 19)
        self.assertIn("10 × Stateroom", boat["equipment"])
        self.assertIn("5 × Emergency Low Berth", boat["equipment"])
        self.assertIn("3 × Fuel Processor", boat["equipment"])
        self.assertNotIn("Fuel Scoop", boat["equipment"])
        self.assertEqual(boat["armament"].count("Triple Turret"), 4)
        self.assertEqual(boat["armament"].count("Missile Rack"), 6)
        self.assertEqual(boat["armament"].count("Beam Laser"), 6)
        self.assertEqual(boat["ammunition"], "360 × Smart Missiles")
        self.assertEqual(boat["unused_fire_control_stations"], 0)
        self.assertEqual(boat["length_m"], 61.0)
        self.assertEqual(boat["art_path"], "assets/ships/ship-205-casemate.webp")

    def test_blackbeard_exposes_raider_and_three_craft_hangar(self) -> None:
        raider = self.records[206]
        self.assertEqual(raider["family_id"], 206)
        self.assertEqual(raider["tons"], 600)
        self.assertEqual(raider["configuration"], "Standard")
        self.assertEqual(raider["electronics"], "Basic Civilian")
        self.assertTrue(raider["standard_design"])
        self.assertEqual(raider["armor_points"], 8)
        self.assertEqual(raider["hull_options"], [])
        self.assertEqual(raider["jump_drive"], "F")
        self.assertEqual(raider["jump_distance"], 2)
        self.assertEqual(raider["jump_count"], 1)
        self.assertEqual(raider["maneuver_drive"], "M")
        self.assertEqual(raider["power_plant"], "M")
        self.assertEqual(raider["thrust_g"], 4)
        self.assertEqual(raider["endurance"], 4)
        self.assertEqual(raider["cargo"], "55 tons")
        self.assertEqual(raider["crew"], 24)
        self.assertIn("4 × Detention Cell", raider["equipment"])
        self.assertIn("6 × Fuel Processor", raider["equipment"])
        self.assertIn("Fuel Scoop", raider["equipment"])
        self.assertIn("Custom Hangar (50 tons contained)", raider["equipment"])
        self.assertIn(
            "2 × Carried Craft: Thermopylae (ship-200)",
            raider["equipment"],
        )
        self.assertIn("Carried Craft: Tender (ship-202)", raider["equipment"])
        self.assertEqual(raider["armament"].count("Triple Turret"), 6)
        self.assertEqual(raider["armament"].count("Beam Laser"), 18)
        self.assertEqual(raider["ammunition"], "None carried")
        self.assertEqual(raider["unused_fire_control_stations"], 0)
        self.assertEqual(raider["length_m"], 76.0)
        self.assertEqual(
            raider["art_path"],
            "assets/ships/ship-206-blackbeard.webp",
        )

    def test_bacon_exposes_research_labs_probes_and_two_dory_hangars(self) -> None:
        research = self.records[207]
        self.assertEqual(research["family_id"], 207)
        self.assertEqual(research["tons"], 200)
        self.assertEqual(research["configuration"], "Standard")
        self.assertEqual(research["electronics"], "Basic Civilian")
        self.assertTrue(research["standard_design"])
        self.assertEqual(research["armor_points"], 2)
        self.assertEqual(research["hull_options"], [])
        self.assertEqual(research["jump_drive"], "A")
        self.assertEqual(research["jump_distance"], 1)
        self.assertEqual(research["jump_count"], 1)
        self.assertEqual(research["maneuver_drive"], "A")
        self.assertEqual(research["power_plant"], "A")
        self.assertEqual(research["assertions"]["thrust_g"], 1)
        self.assertEqual(research["endurance"], 4)
        self.assertEqual(research["cargo"], "29 tons")
        self.assertEqual(research["crew"], 9)
        self.assertIn("2 × Lifeboat Hangar", research["equipment"])
        self.assertIn("3 × Probe Drones", research["equipment"])
        self.assertIn("6 × Laboratory", research["equipment"])
        self.assertIn("2 × Fuel Processor", research["equipment"])
        self.assertIn("Fuel Scoop", research["equipment"])
        self.assertIn(
            "2 × Carried Craft: Dory (ship-194)",
            research["equipment"],
        )
        self.assertEqual(research["armament"], "None installed")
        self.assertEqual(research["ammunition"], "None carried")
        self.assertEqual(research["unused_fire_control_stations"], 2)
        self.assertEqual(research["length_m"], 50.0)
        self.assertEqual(research["art_path"], "assets/ships/ship-207-bacon.webp")

    def test_galileo_exposes_armed_survey_endurance_and_dory_hangars(self) -> None:
        survey = self.records[208]
        self.assertEqual(survey["family_id"], 208)
        self.assertEqual(survey["tons"], 300)
        self.assertEqual(survey["configuration"], "Standard")
        self.assertEqual(survey["electronics"], "Basic Civilian")
        self.assertTrue(survey["standard_design"])
        self.assertEqual(survey["armor_points"], 2)
        self.assertEqual(survey["hull_options"], [])
        self.assertEqual(survey["jump_drive"], "B")
        self.assertEqual(survey["jump_distance"], 1)
        self.assertEqual(survey["jump_count"], 2)
        self.assertEqual(survey["maneuver_drive"], "C")
        self.assertEqual(survey["power_plant"], "C")
        self.assertEqual(survey["assertions"]["thrust_g"], 2)
        self.assertEqual(survey["endurance"], 4)
        self.assertEqual(survey["cargo"], "39 tons")
        self.assertEqual(survey["crew"], 14)
        self.assertIn("2 × Lifeboat Hangar", survey["equipment"])
        self.assertIn("4 × Probe Drones", survey["equipment"])
        self.assertIn("6 × Laboratory", survey["equipment"])
        self.assertIn("4 × Fuel Processor", survey["equipment"])
        self.assertIn("Fuel Scoop", survey["equipment"])
        self.assertIn(
            "2 × Carried Craft: Dory (ship-194)",
            survey["equipment"],
        )
        self.assertEqual(survey["armament"].count("Triple Turret"), 3)
        self.assertEqual(survey["armament"].count("Beam Laser"), 9)
        self.assertEqual(survey["ammunition"], "None carried")
        self.assertEqual(survey["unused_fire_control_stations"], 0)
        self.assertEqual(survey["length_m"], 59.0)
        self.assertEqual(
            survey["art_path"],
            "assets/ships/ship-208-galileo.webp",
        )

    def test_jutland_exposes_dreadnought_battery_screens_and_air_group(self) -> None:
        dreadnought = self.records[210]
        self.assertEqual(dreadnought["family_id"], 210)
        self.assertEqual(dreadnought["tons"], 5000)
        self.assertEqual(dreadnought["configuration"], "Standard")
        self.assertEqual(dreadnought["electronics"], "Very Advanced")
        self.assertTrue(dreadnought["standard_design"])
        self.assertEqual(dreadnought["armor_points"], 12)
        self.assertIn("Stealth", dreadnought["hull_options"])
        self.assertEqual(dreadnought["jump_drive"], "Z")
        self.assertEqual(dreadnought["jump_distance"], 2)
        self.assertEqual(dreadnought["jump_count"], 1)
        self.assertEqual(dreadnought["maneuver_drive"], "Z")
        self.assertEqual(dreadnought["power_plant"], "Z")
        self.assertEqual(dreadnought["assertions"]["thrust_g"], 2)
        self.assertEqual(dreadnought["endurance"], 4)
        self.assertEqual(dreadnought["cargo"], "635 tons")
        self.assertEqual(dreadnought["crew"], 223)
        self.assertIn("60 × Barracks", dreadnought["equipment"])
        self.assertIn("223 × Escape Pod", dreadnought["equipment"])
        self.assertIn("54 × Fuel Processor", dreadnought["equipment"])
        self.assertIn("Fuel Scoop", dreadnought["equipment"])
        self.assertIn(
            "Custom Hangar (300 tons contained)",
            dreadnought["equipment"],
        )
        self.assertIn(
            "20 × Carried Craft: Thermopylae (ship-200)",
            dreadnought["equipment"],
        )
        self.assertIn(
            "2 × Carried Craft: Proteus Modular (ship-209)",
            dreadnought["equipment"],
        )
        self.assertIn("Meson Screen", dreadnought["equipment"])
        self.assertIn("Nuclear Damper", dreadnought["equipment"])
        self.assertEqual(
            [entry["quantity"] for entry in dreadnought["mount_entries"]],
            [35, 10, 5],
        )
        self.assertEqual(
            dreadnought["mount_entries"][0]["weapons"],
            ["Beam Laser", "Beam Laser", "Beam Laser"],
        )
        self.assertEqual(dreadnought["ammunition"], "3600 × Smart Missiles")
        self.assertEqual(dreadnought["unused_fire_control_stations"], 0)
        self.assertEqual(dreadnought["length_m"], 168.0)
        self.assertEqual(
            dreadnought["art_path"],
            "assets/ships/ship-210-jutland.webp",
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

    def test_cutlass_is_revenants_armed_assault_cutter(self) -> None:
        cutlass = self.records[213]
        self.assertEqual(cutlass["family_id"], 213)
        self.assertEqual(cutlass["tons"], 50)
        self.assertEqual(cutlass["configuration"], "Streamlined")
        self.assertEqual(cutlass["electronics"], "Standard")
        self.assertEqual(cutlass["armor_points"], 2)
        self.assertEqual(cutlass["control"], "Two Person Control Cabin")
        self.assertEqual(cutlass["additional_passengers"], 0)
        self.assertEqual(cutlass["maneuver_drive"], "sK")
        self.assertEqual(cutlass["power_plant"], "sK")
        self.assertEqual(cutlass["assertions"]["thrust_g"], 4)
        self.assertEqual(cutlass["endurance"], 2)
        self.assertEqual(cutlass["cargo"], "17.25 tons")
        self.assertEqual(cutlass["crew"], 2)
        self.assertEqual(cutlass["airlocks"], 1)
        self.assertIn("24 passenger seats", cutlass["equipment"])
        self.assertIn("Fixed Single Turret: Beam Laser", cutlass["armament"])
        self.assertIsNone(cutlass["jump_drive"])
        self.assertEqual(cutlass["length_m"], 22.0)
        self.assertEqual(
            cutlass["art_path"],
            "assets/ships/ship-213-cutlass.webp",
        )

    def test_myrmidon_exposes_armored_strike_boat_fit(self) -> None:
        myrmidon = self.records[29]
        self.assertEqual(myrmidon["family_id"], 29)
        self.assertEqual(myrmidon["tons"], 40)
        self.assertEqual(myrmidon["configuration"], "Streamlined")
        self.assertEqual(myrmidon["tech_level"], 11)
        self.assertFalse(myrmidon["standard_design"])
        self.assertEqual(myrmidon["electronics"], "Basic Military")
        self.assertEqual(myrmidon["armor_points"], 4)
        self.assertEqual(myrmidon["control"], "Four Person Control Cabin")
        self.assertEqual(myrmidon["additional_passengers"], 0)
        self.assertEqual(myrmidon["maneuver_drive"], "sL")
        self.assertEqual(myrmidon["power_plant"], "sL")
        self.assertEqual(myrmidon["endurance"], 1)
        self.assertEqual(myrmidon["cargo"], "6 tons")
        self.assertEqual(myrmidon["crew"], 4)
        self.assertEqual(myrmidon["airlocks"], 1)
        self.assertIn("2 × Small Craft Stateroom", myrmidon["equipment"])
        self.assertEqual(myrmidon["armament"], "Single Turret: Beam Laser")
        self.assertEqual(myrmidon["ammunition"], "None carried")
        self.assertIsNone(myrmidon["jump_drive"])
        self.assertEqual(myrmidon["length_m"], 24.5)
        self.assertEqual(
            myrmidon["art_path"],
            "assets/ships/ship-029-myrmidon.webp",
        )

    def test_scheherazade_exposes_armed_executive_yacht_fit(self) -> None:
        scheherazade = self.records[37]
        self.assertEqual(scheherazade["family_id"], 37)
        self.assertEqual(scheherazade["tons"], 100)
        self.assertEqual(scheherazade["configuration"], "Streamlined")
        self.assertEqual(scheherazade["tech_level"], 12)
        self.assertFalse(scheherazade["standard_design"])
        self.assertEqual(scheherazade["electronics"], "Basic Civilian")
        self.assertEqual(scheherazade["armor_points"], 4)
        self.assertEqual(scheherazade["control"], "Standard bridge")
        self.assertEqual(scheherazade["bridge_options"], ["Holographic Controls"])
        self.assertEqual(scheherazade["jump_drive"], "A")
        self.assertEqual(scheherazade["maneuver_drive"], "B")
        self.assertEqual(scheherazade["power_plant"], "B")
        self.assertEqual(scheherazade["assertions"]["jump_rating"], 2)
        self.assertEqual(scheherazade["assertions"]["thrust_g"], 4)
        self.assertEqual(scheherazade["endurance"], 2)
        self.assertEqual(scheherazade["cargo"], "0.5 tons")
        self.assertEqual(scheherazade["crew"], 6)
        self.assertIn("4 × Stateroom", scheherazade["equipment"])
        self.assertIn("2 × High Class Stateroom", scheherazade["equipment"])
        self.assertIn("Repair Drones", scheherazade["equipment"])
        self.assertEqual(
            scheherazade["armament"],
            "Double Turret: Beam Laser · Beam Laser · Point Defense Node "
            "Mount: Point Defense Laser",
        )
        self.assertEqual(scheherazade["assertions"]["point_defense_nodes"], 1)
        self.assertEqual(scheherazade["ammunition"], "None carried")
        self.assertEqual(scheherazade["length_m"], 36.0)
        self.assertEqual(
            scheherazade["art_path"],
            "assets/ships/ship-037-scheherazade.webp",
        )

    def test_warden_exposes_customs_cutter_and_caduceus_hangar(self) -> None:
        warden = self.records[41]
        self.assertEqual(warden["family_id"], 41)
        self.assertEqual(warden["tons"], 100)
        self.assertEqual(warden["configuration"], "Streamlined")
        self.assertEqual(warden["tech_level"], 11)
        self.assertFalse(warden["standard_design"])
        self.assertEqual(warden["electronics"], "Basic Military")
        self.assertEqual(warden["armor_points"], 4)
        self.assertEqual(warden["control"], "Standard bridge")
        self.assertEqual(warden["maneuver_drive"], "C")
        self.assertEqual(warden["power_plant"], "C")
        self.assertIsNone(warden["jump_drive"])
        self.assertEqual(warden["endurance"], 2)
        self.assertEqual(warden["cargo"], "13 tons")
        self.assertEqual(warden["crew"], 8)
        self.assertIn("2 × Fuel Processor", warden["equipment"])
        self.assertIn("2 × Ships Brig", warden["equipment"])
        self.assertIn("Medical Bay (1 bed)", warden["equipment"])
        self.assertIn("Standard Hangar (20 tons contained)", warden["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-7)", warden["equipment"])
        self.assertEqual(
            warden["armament"],
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser",
        )
        self.assertEqual(warden["ammunition"], "None carried")
        self.assertEqual(warden["length_m"], 38.0)
        self.assertEqual(warden["art_path"], "assets/ships/ship-041-warden.webp")

    def test_ballista_exposes_persistent_missile_escort_fit(self) -> None:
        ballista = self.records[42]
        self.assertEqual(ballista["family_id"], 42)
        self.assertEqual(ballista["tons"], 80)
        self.assertEqual(ballista["configuration"], "Streamlined")
        self.assertEqual(ballista["tech_level"], 11)
        self.assertFalse(ballista["standard_design"])
        self.assertEqual(ballista["electronics"], "Basic Military")
        self.assertEqual(ballista["armor_points"], 4)
        self.assertEqual(ballista["control"], "Four Person Control Cabin")
        self.assertEqual(ballista["additional_passengers"], 0)
        self.assertEqual(ballista["maneuver_drive"], "sQ")
        self.assertEqual(ballista["power_plant"], "sQ")
        self.assertEqual(ballista["endurance"], 1)
        self.assertEqual(ballista["cargo"], "32.3 tons")
        self.assertEqual(ballista["crew"], 4)
        self.assertEqual(ballista["airlocks"], 1)
        self.assertIn("3 × Small Craft Stateroom", ballista["equipment"])
        self.assertIn("Small Craft Fuel Processor", ballista["equipment"])
        self.assertIn("Emergency Low Berth", ballista["equipment"])
        self.assertEqual(ballista["armament"], "Single Turret: Missile Rack")
        self.assertEqual(ballista["ammunition"], "12 × Standard Missiles")
        self.assertIsNone(ballista["jump_drive"])
        self.assertEqual(ballista["length_m"], 32.0)
        self.assertEqual(ballista["art_path"], "assets/ships/ship-042-ballista.webp")

    def test_phileas_exposes_fast_armed_courier_fit(self) -> None:
        phileas = self.records[44]
        self.assertEqual(phileas["family_id"], 44)
        self.assertEqual(phileas["tons"], 100)
        self.assertEqual(phileas["configuration"], "Streamlined")
        self.assertEqual(phileas["tech_level"], 11)
        self.assertFalse(phileas["standard_design"])
        self.assertEqual(phileas["electronics"], "Basic Military")
        self.assertEqual(phileas["armor_points"], 4)
        self.assertEqual(phileas["control"], "Standard bridge")
        self.assertEqual(phileas["bridge_options"], ["Hardened Bridge"])
        self.assertEqual(phileas["jump_drive"], "A")
        self.assertEqual(phileas["maneuver_drive"], "C")
        self.assertEqual(phileas["power_plant"], "C")
        self.assertEqual(phileas["jump_distance"], 2)
        self.assertEqual(phileas["endurance"], 2)
        self.assertEqual(phileas["cargo"], "14 tons")
        self.assertEqual(phileas["crew"], 4)
        self.assertIn("3 × Stateroom", phileas["equipment"])
        self.assertIn("Fuel Processor", phileas["equipment"])
        self.assertIn("Additional Airlock", phileas["equipment"])
        self.assertIn("Emergency Low Berth", phileas["equipment"])
        self.assertEqual(
            phileas["armament"],
            "Triple Turret: Missile Rack · Sandcaster · Beam Laser · Point "
            "Defense Node Mount: Point Defense Laser",
        )
        self.assertEqual(
            phileas["ammunition"],
            "12 × Standard Missiles · 20 × Sandcaster Canisters",
        )
        self.assertEqual(phileas["length_m"], 41.0)
        self.assertEqual(phileas["art_path"], "assets/ships/ship-044-phileas.webp")

    def test_antaeus_exposes_planetary_transport_fit(self) -> None:
        antaeus = self.records[47]
        self.assertEqual(antaeus["family_id"], 47)
        self.assertEqual(antaeus["tons"], 200)
        self.assertEqual(antaeus["configuration"], "Streamlined")
        self.assertEqual(antaeus["tech_level"], 11)
        self.assertFalse(antaeus["standard_design"])
        self.assertEqual(antaeus["electronics"], "Basic Military")
        self.assertEqual(antaeus["armor_points"], 4)
        self.assertEqual(antaeus["jump_drive"], None)
        self.assertEqual(antaeus["maneuver_drive"], "C")
        self.assertEqual(antaeus["power_plant"], "C")
        self.assertEqual(antaeus["endurance"], 2)
        self.assertEqual(antaeus["cargo"], "113 tons")
        self.assertEqual(antaeus["crew"], 5)
        self.assertIn("3 × Stateroom", antaeus["equipment"])
        self.assertIn("25 × Steerage", antaeus["equipment"])
        self.assertIn("3 × Additional Airlock", antaeus["equipment"])
        self.assertEqual(
            antaeus["armament"],
            "2 × Single Turret: Sandcaster",
        )
        self.assertEqual(antaeus["ammunition"], "40 × Sandcaster Canisters")
        self.assertEqual(antaeus["length_m"], 48.0)
        self.assertEqual(antaeus["art_path"], "assets/ships/ship-047-antaeus.webp")

    def test_challenger_exposes_armed_survey_courier_fit(self) -> None:
        challenger = self.records[53]
        self.assertEqual(challenger["family_id"], 53)
        self.assertEqual(challenger["tons"], 120)
        self.assertEqual(challenger["configuration"], "Streamlined")
        self.assertEqual(challenger["tech_level"], 11)
        self.assertFalse(challenger["standard_design"])
        self.assertEqual(challenger["electronics"], "Basic Military")
        self.assertEqual(challenger["armor_points"], 4)
        self.assertEqual(challenger["control"], "Standard bridge")
        self.assertEqual(challenger["jump_drive"], "B")
        self.assertEqual(challenger["maneuver_drive"], "E")
        self.assertEqual(challenger["power_plant"], "E")
        self.assertEqual(challenger["jump_distance"], 2)
        self.assertEqual(challenger["thrust_g"], 5)
        self.assertEqual(challenger["endurance"], 2)
        self.assertEqual(challenger["cargo"], "2 tons")
        self.assertEqual(challenger["crew"], 4)
        self.assertIn("4 × Stateroom", challenger["equipment"])
        self.assertIn("Fuel Processor", challenger["equipment"])
        self.assertIn("Emergency Low Berth", challenger["equipment"])
        self.assertIn("Standard Hangar (5 tons contained)", challenger["equipment"])
        self.assertEqual(
            challenger["armament"],
            "Double Turret: Beam Laser · Sandcaster",
        )
        self.assertEqual(challenger["ammunition"], "20 × Sandcaster Canisters")
        self.assertEqual(challenger["length_m"], 43.0)
        self.assertEqual(
            challenger["art_path"],
            "assets/ships/ship-053-challenger.webp",
        )

    def test_argosy_exposes_frontier_merchant_freighter_fit(self) -> None:
        argosy = self.records[61]
        self.assertEqual(argosy["family_id"], 61)
        self.assertEqual(argosy["tons"], 400)
        self.assertEqual(argosy["configuration"], "Streamlined")
        self.assertEqual(argosy["tech_level"], 11)
        self.assertFalse(argosy["standard_design"])
        self.assertEqual(argosy["electronics"], "Basic Military")
        self.assertEqual(argosy["armor_points"], 4)
        self.assertEqual(argosy["control"], "Standard bridge")
        self.assertEqual(argosy["jump_drive"], "D")
        self.assertEqual(argosy["maneuver_drive"], "D")
        self.assertEqual(argosy["power_plant"], "D")
        self.assertEqual(argosy["jump_distance"], 2)
        self.assertEqual(argosy["thrust_g"], 2)
        self.assertEqual(argosy["endurance"], 2)
        self.assertEqual(argosy["cargo"], "182 tons")
        self.assertEqual(argosy["crew"], 5)
        self.assertIn("7 × Stateroom", argosy["equipment"])
        self.assertIn("2 × Fuel Processor", argosy["equipment"])
        self.assertIn("10 × Low Berth", argosy["equipment"])
        self.assertIn("Medical Bay (1 bed)", argosy["equipment"])
        self.assertEqual(
            argosy["armament"],
            "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
        )
        self.assertEqual(
            argosy["ammunition"],
            "12 × Standard Missiles · 20 × Sandcaster Canisters",
        )
        self.assertEqual(argosy["unused_fire_control_stations"], 3)
        self.assertEqual(argosy["length_m"], 68.0)
        self.assertEqual(argosy["art_path"], "assets/ships/ship-061-argosy.webp")

    def test_janus_exposes_armored_system_patrol_fit(self) -> None:
        janus = self.records[63]
        self.assertEqual(janus["family_id"], 63)
        self.assertEqual(janus["tons"], 200)
        self.assertEqual(janus["configuration"], "Streamlined")
        self.assertEqual(janus["tech_level"], 11)
        self.assertFalse(janus["standard_design"])
        self.assertEqual(janus["electronics"], "Basic Military")
        self.assertEqual(janus["armor_points"], 11)
        self.assertEqual(janus["control"], "Standard bridge")
        self.assertEqual(
            janus["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertIsNone(janus["jump_drive"])
        self.assertEqual(janus["maneuver_drive"], "F")
        self.assertEqual(janus["power_plant"], "F")
        self.assertEqual(janus["endurance"], 2)
        self.assertEqual(janus["cargo"], "49.5 tons")
        self.assertEqual(janus["crew"], 8)
        self.assertIn("2 × Ships Brig", janus["equipment"])
        self.assertIn("Emergency Low Berth", janus["equipment"])
        self.assertIn("Repair Drones", janus["equipment"])
        self.assertIn("Standard Hangar (20 tons contained)", janus["equipment"])
        self.assertIn("Carried Craft: Caduceus (ship-7)", janus["equipment"])
        self.assertEqual(
            janus["armament"],
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser · Triple "
            "Turret: Missile Rack · Missile Rack · Sandcaster · 3 × Point "
            "Defense Node Mount: Point Defense Laser",
        )
        self.assertEqual(
            janus["ammunition"],
            "84 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(janus["unused_fire_control_stations"], 0)
        self.assertEqual(janus["length_m"], 51.0)
        self.assertEqual(janus["art_path"], "assets/ships/ship-063-janus.webp")

    def test_darwin_exposes_independent_survey_scout_fit(self) -> None:
        darwin = self.records[64]
        self.assertEqual(darwin["family_id"], 64)
        self.assertEqual(darwin["tons"], 300)
        self.assertEqual(darwin["configuration"], "Streamlined")
        self.assertEqual(darwin["tech_level"], 11)
        self.assertFalse(darwin["standard_design"])
        self.assertEqual(darwin["electronics"], "Basic Military")
        self.assertEqual(darwin["armor_points"], 4)
        self.assertEqual(darwin["control"], "Standard bridge")
        self.assertEqual(darwin["bridge_options"], ["Holographic Controls"])
        self.assertEqual(darwin["jump_drive"], "C")
        self.assertEqual(darwin["maneuver_drive"], "E")
        self.assertEqual(darwin["power_plant"], "E")
        self.assertEqual(darwin["jump_distance"], 2)
        self.assertEqual(darwin["endurance"], 2)
        self.assertEqual(darwin["cargo"], "83 tons")
        self.assertEqual(darwin["crew"], 8)
        self.assertIn("8 × Stateroom", darwin["equipment"])
        self.assertIn("4 × Emergency Low Berth", darwin["equipment"])
        self.assertIn("20 × Crew Recreation", darwin["equipment"])
        self.assertIn("Gymnasium", darwin["equipment"])
        self.assertIn("2 × Laboratory", darwin["equipment"])
        self.assertIn("Medical Bay (1 bed)", darwin["equipment"])
        self.assertIn("Repair Drones", darwin["equipment"])
        self.assertEqual(
            darwin["armament"],
            "Triple Turret: Beam Laser · Sandcaster · Missile Rack",
        )
        self.assertEqual(
            darwin["ammunition"],
            "12 × Standard Missiles · 20 × Sandcaster Canisters",
        )
        self.assertEqual(darwin["unused_fire_control_stations"], 2)
        self.assertEqual(darwin["length_m"], 56.0)
        self.assertEqual(darwin["art_path"], "assets/ships/ship-064-darwin.webp")

    def test_drake_exposes_raider_and_boarding_fit(self) -> None:
        drake = self.records[66]
        self.assertEqual(drake["family_id"], 66)
        self.assertEqual(drake["tons"], 300)
        self.assertEqual(drake["configuration"], "Streamlined")
        self.assertEqual(drake["armor_points"], 4)
        self.assertEqual(drake["jump_drive"], "C")
        self.assertEqual(drake["maneuver_drive"], "F")
        self.assertEqual(drake["power_plant"], "F")
        self.assertEqual(drake["jump_distance"], 2)
        self.assertEqual(drake["thrust_g"], 4)
        self.assertEqual(drake["cargo"], "42.5 tons")
        self.assertEqual(drake["crew"], 23)
        self.assertIn("Breaching Tube", drake["equipment"])
        self.assertIn("4 × Fuel Processor", drake["equipment"])
        self.assertIn("Standard Hangar (30 tons contained)", drake["equipment"])
        self.assertIn("Carried Craft: Wayfarer Boarding (ship-212)", drake["equipment"])
        self.assertIn("Single Turret: Particle Beam", drake["armament"])
        self.assertIn("Triple Turret: Beam Laser · Beam Laser · Sandcaster", drake["armament"])
        self.assertIn("Triple Turret: Missile Rack · Missile Rack · Beam Laser", drake["armament"])
        self.assertEqual(drake["ammunition"], "24 × Standard Missiles · 20 × Sandcaster Canisters")
        self.assertEqual(drake["length_m"], 58.0)
        self.assertEqual(drake["art_path"], "assets/ships/ship-066-drake.webp")

    def test_magellan_exposes_self_reliant_exploration_fit(self) -> None:
        magellan = self.records[67]
        self.assertEqual(magellan["family_id"], 67)
        self.assertEqual(magellan["tons"], 400)
        self.assertEqual(magellan["configuration"], "Streamlined")
        self.assertEqual(magellan["tech_level"], 11)
        self.assertFalse(magellan["standard_design"])
        self.assertEqual(magellan["electronics"], "Basic Military")
        self.assertEqual(magellan["armor_points"], 4)
        self.assertEqual(magellan["control"], "Standard bridge")
        self.assertEqual(magellan["jump_drive"], "D")
        self.assertEqual(magellan["maneuver_drive"], "H")
        self.assertEqual(magellan["power_plant"], "H")
        self.assertEqual(magellan["jump_distance"], 2)
        self.assertEqual(magellan["endurance"], 2)
        self.assertEqual(magellan["cargo"], "127 tons")
        self.assertEqual(magellan["crew"], 7)
        self.assertIn("6 × Stateroom", magellan["equipment"])
        self.assertIn("10 × Fuel Processor", magellan["equipment"])
        self.assertIn("4 × Probe Drones", magellan["equipment"])
        self.assertIn("Emergency Low Berth", magellan["equipment"])
        self.assertIn("4 × Low Berth", magellan["equipment"])
        self.assertIn("Medical Bay (1 bed)", magellan["equipment"])
        self.assertIn("Repair Drones", magellan["equipment"])
        self.assertIn("Standard Hangar (14 tons contained)", magellan["equipment"])
        self.assertEqual(
            magellan["armament"],
            "2 × Double Turret: Beam Laser · Beam Laser · Double Turret: "
            "Sandcaster · Sandcaster · Double Turret: Missile Rack · Missile Rack",
        )
        self.assertEqual(
            magellan["ammunition"],
            "24 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(magellan["unused_fire_control_stations"], 0)
        self.assertEqual(magellan["length_m"], 62.0)
        self.assertEqual(magellan["art_path"], "assets/ships/ship-067-magellan.webp")

    def test_monitor_exposes_casemate_gunboat_fit(self) -> None:
        monitor = self.records[70]
        self.assertEqual(monitor["family_id"], 70)
        self.assertEqual(monitor["tons"], 300)
        self.assertEqual(monitor["configuration"], "Standard")
        self.assertEqual(monitor["tech_level"], 10)
        self.assertFalse(monitor["standard_design"])
        self.assertEqual(monitor["hull_options"], ["Radiation Shielding"])
        self.assertEqual(monitor["electronics"], "Basic Military")
        self.assertEqual(monitor["armor_points"], 4)
        self.assertEqual(monitor["control"], "Standard bridge")
        self.assertEqual(monitor["bridge_options"], ["Hardened Bridge"])
        self.assertIsNone(monitor["jump_drive"])
        self.assertEqual(monitor["maneuver_drive"], "F")
        self.assertEqual(monitor["power_plant"], "F")
        self.assertEqual(monitor["endurance"], 4)
        self.assertEqual(monitor["cargo"], "99.5 tons")
        self.assertEqual(monitor["crew"], 14)
        self.assertIn("14 × Escape Pod", monitor["equipment"])
        self.assertIn("Emergency Low Berth", monitor["equipment"])
        self.assertIn("Gymnasium", monitor["equipment"])
        self.assertIn("Repair Drones", monitor["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Missile Rack · Sandcaster",
            monitor["armament"],
        )
        self.assertIn("Railgun Barbette", monitor["armament"])
        self.assertIn("Particle Beam Bay", monitor["armament"])
        self.assertIn(
            "2 × Point Defense Node Mount: Point Defense Minigun",
            monitor["armament"],
        )
        self.assertIn(
            "Point Defense Node Mount: Point Defense Laser",
            monitor["armament"],
        )
        self.assertEqual(
            monitor["ammunition"],
            "36 × Standard Missiles · 20 × Sandcaster Canisters · 80 × Railgun Basics",
        )
        self.assertEqual(monitor["unused_fire_control_stations"], 0)
        self.assertEqual(monitor["length_m"], 44.0)
        self.assertEqual(monitor["art_path"], "assets/ships/ship-070-monitor.webp")

    def test_xanadu_exposes_luxury_resort_liner_fit(self) -> None:
        xanadu = self.records[71]
        self.assertEqual(xanadu["family_id"], 71)
        self.assertEqual(xanadu["tons"], 400)
        self.assertEqual(xanadu["configuration"], "Streamlined")
        self.assertEqual(xanadu["tech_level"], 11)
        self.assertFalse(xanadu["standard_design"])
        self.assertEqual(xanadu["electronics"], "Basic Military")
        self.assertEqual(xanadu["armor_points"], 0)
        self.assertEqual(xanadu["control"], "Standard bridge")
        self.assertEqual(xanadu["bridge_options"], ["Holographic Controls"])
        self.assertEqual(xanadu["jump_drive"], "D")
        self.assertEqual(xanadu["maneuver_drive"], "J")
        self.assertEqual(xanadu["power_plant"], "J")
        self.assertEqual(xanadu["jump_distance"], 2)
        self.assertEqual(xanadu["endurance"], 2)
        self.assertEqual(xanadu["cargo"], "69 tons")
        self.assertEqual(xanadu["crew"], 8)
        self.assertIn("19 × Stateroom", xanadu["equipment"])
        self.assertIn("100 × Crew Recreation", xanadu["equipment"])
        self.assertIn("6 × Garden Space", xanadu["equipment"])
        self.assertIn("2 × Office", xanadu["equipment"])
        self.assertIn("12 × Pool Or Spa Space", xanadu["equipment"])
        self.assertIn("16 × Theater Seat", xanadu["equipment"])
        self.assertIn("2 × Emergency Low Berth", xanadu["equipment"])
        self.assertIn("Medical Bay (1 bed)", xanadu["equipment"])
        self.assertIn("Repair Drones", xanadu["equipment"])
        self.assertEqual(
            xanadu["armament"],
            "Triple Turret: Beam Laser · Sandcaster · Missile Rack · 4 × "
            "Point Defense Node Mount: Point Defense Laser",
        )
        self.assertEqual(
            xanadu["ammunition"],
            "12 × Standard Missiles · 20 × Sandcaster Canisters",
        )
        self.assertEqual(xanadu["unused_fire_control_stations"], 3)
        self.assertEqual(xanadu["length_m"], 65.0)
        self.assertEqual(xanadu["art_path"], "assets/ships/ship-071-xanadu.webp")

    def test_aegis_exposes_reinforced_meson_cruiser_fit(self) -> None:
        aegis = self.records[73]
        self.assertEqual(aegis["family_id"], 73)
        self.assertEqual(aegis["tons"], 300)
        self.assertEqual(aegis["configuration"], "Streamlined")
        self.assertEqual(aegis["tech_level"], 11)
        self.assertFalse(aegis["standard_design"])
        self.assertEqual(
            aegis["structural_options"],
            ["Reinforced Structure (1 increment)"],
        )
        self.assertEqual(aegis["electronics"], "Advanced")
        self.assertEqual(aegis["armor_points"], 11)
        self.assertEqual(aegis["control"], "Standard bridge")
        self.assertEqual(
            aegis["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(aegis["computer"], "Model 3")
        self.assertEqual(aegis["computer_options"], ["Bis"])
        self.assertIsNone(aegis["jump_drive"])
        self.assertEqual(aegis["maneuver_drive"], "J")
        self.assertEqual(aegis["power_plant"], "J")
        self.assertEqual(aegis["endurance"], 2)
        self.assertEqual(aegis["cargo"], "57.25 tons")
        self.assertEqual(aegis["crew"], 15)
        self.assertIn("Armory", aegis["equipment"])
        self.assertIn("Office", aegis["equipment"])
        self.assertIn("2 × Fuel Processor", aegis["equipment"])
        self.assertIn("Emergency Low Berth", aegis["equipment"])
        self.assertIn("Medical Bay (1 bed)", aegis["equipment"])
        self.assertIn("Repair Drones", aegis["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Sandcaster",
            aegis["armament"],
        )
        self.assertIn(
            "Triple Turret: Missile Rack · Missile Rack · Sandcaster",
            aegis["armament"],
        )
        self.assertIn("Meson Gun Bay", aegis["armament"])
        self.assertIn(
            "3 × Point Defense Node Mount: Point Defense Laser",
            aegis["armament"],
        )
        self.assertEqual(
            aegis["ammunition"],
            "72 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(aegis["unused_fire_control_stations"], 0)
        self.assertEqual(aegis["length_m"], 48.0)
        self.assertEqual(aegis["art_path"], "assets/ships/ship-073-aegis.webp")

    def test_marque_exposes_boarding_escort_brig_fit(self) -> None:
        marque = self.records[74]
        self.assertEqual(marque["family_id"], 74)
        self.assertEqual(marque["path_name"], "Marque Marine")
        self.assertEqual(marque["tons"], 400)
        self.assertEqual(marque["configuration"], "Streamlined")
        self.assertEqual(marque["tech_level"], 12)
        self.assertFalse(marque["standard_design"])
        self.assertEqual(marque["electronics"], "Advanced")
        self.assertEqual(marque["armor_points"], 4)
        self.assertEqual(marque["computer"], "Model 3")
        self.assertEqual(marque["computer_options"], ["Bis", "Fib"])
        self.assertEqual(marque["jump_drive"], "D")
        self.assertEqual(marque["jump_distance"], 2)
        self.assertEqual(marque["maneuver_drive"], "H")
        self.assertEqual(marque["power_plant"], "H")
        self.assertEqual(marque["thrust_g"], 4)
        self.assertEqual(marque["endurance"], 3)
        self.assertEqual(marque["cargo"], "16 tons")
        self.assertEqual(marque["crew"], 25)
        self.assertIn("5 × Stateroom", marque["equipment"])
        self.assertIn("2 × Workshop", marque["equipment"])
        self.assertIn("Medical Bay (2 beds)", marque["equipment"])
        self.assertIn("Atv Hangar", marque["equipment"])
        self.assertIn("Air Raft Hangar", marque["equipment"])
        self.assertIn("Full Hangar (30 tons contained)", marque["equipment"])
        self.assertIn("Carried Craft: Grapnel (ship-211)", marque["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            marque["armament"],
        )
        self.assertIn(
            "2 × Triple Turret: Missile Rack · Missile Rack · Sandcaster",
            marque["armament"],
        )
        self.assertIn("Particle Beam Barbette", marque["armament"])
        self.assertIn(
            "4 × Point Defense Node Mount: Point Defense Gatling Laser",
            marque["armament"],
        )
        self.assertEqual(
            marque["ammunition"],
            "144 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(marque["unused_fire_control_stations"], 0)
        self.assertEqual(marque["assertions"]["hardpoints"], 4)
        self.assertEqual(marque["assertions"]["hardpoints_used"], 4)
        self.assertEqual(marque["assertions"]["hangar_capacity_millitons"], 30000)
        self.assertEqual(marque["length_m"], 61.0)
        self.assertEqual(marque["art_path"], "assets/ships/ship-074-marque.webp")

    def test_bastion_exposes_system_defense_carrier_fit(self) -> None:
        bastion = self.records[75]
        self.assertEqual(bastion["family_id"], 75)
        self.assertEqual(bastion["path_name"], "Redoubt")
        self.assertEqual(bastion["tons"], 300)
        self.assertEqual(bastion["configuration"], "Streamlined")
        self.assertEqual(bastion["tech_level"], 11)
        self.assertFalse(bastion["standard_design"])
        self.assertEqual(
            bastion["structural_options"],
            ["Reinforced Structure (1 increment)"],
        )
        self.assertEqual(bastion["electronics"], "Advanced")
        self.assertEqual(bastion["armor_points"], 8)
        self.assertEqual(
            bastion["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(bastion["computer"], "Model 3")
        self.assertEqual(bastion["computer_options"], ["Bis"])
        self.assertIsNone(bastion["jump_drive"])
        self.assertEqual(bastion["maneuver_drive"], "J")
        self.assertEqual(bastion["power_plant"], "J")
        self.assertEqual(bastion["endurance"], 2)
        self.assertEqual(bastion["cargo"], "55.5 tons")
        self.assertEqual(bastion["crew"], 15)
        self.assertIn("2 × Stateroom", bastion["equipment"])
        self.assertIn("Armory", bastion["equipment"])
        self.assertIn("2 × Fuel Processor", bastion["equipment"])
        self.assertIn("Medical Bay (1 bed)", bastion["equipment"])
        self.assertIn("Repair Drones", bastion["equipment"])
        self.assertIn("Full Hangar (10 tons contained)", bastion["equipment"])
        self.assertIn("Carried Craft: Charon (ship-158)", bastion["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Sandcaster",
            bastion["armament"],
        )
        self.assertIn(
            "Triple Turret: Missile Rack · Missile Rack · Sandcaster",
            bastion["armament"],
        )
        self.assertIn("Meson Gun Bay", bastion["armament"])
        self.assertIn(
            "3 × Point Defense Node Mount: Point Defense Laser",
            bastion["armament"],
        )
        self.assertEqual(
            bastion["ammunition"],
            "72 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(bastion["unused_fire_control_stations"], 0)
        self.assertEqual(bastion["length_m"], 42.0)
        self.assertEqual(bastion["art_path"], "assets/ships/ship-075-bastion.webp")

    def test_nansen_exposes_long_range_scout_fit(self) -> None:
        nansen = self.records[76]
        self.assertEqual(nansen["family_id"], 76)
        self.assertEqual(nansen["path_name"], "Civic Survey")
        self.assertEqual(nansen["tons"], 500)
        self.assertEqual(nansen["configuration"], "Streamlined")
        self.assertEqual(nansen["tech_level"], 11)
        self.assertFalse(nansen["standard_design"])
        self.assertEqual(nansen["electronics"], "Advanced")
        self.assertEqual(nansen["armor_points"], 4)
        self.assertEqual(nansen["computer"], "Model 3")
        self.assertEqual(nansen["computer_options"], ["Bis"])
        self.assertEqual(nansen["jump_drive"], "E")
        self.assertEqual(nansen["jump_distance"], 2)
        self.assertEqual(nansen["jump_count"], 2)
        self.assertEqual(nansen["maneuver_drive"], "H")
        self.assertEqual(nansen["power_plant"], "H")
        self.assertEqual(nansen["endurance"], 2)
        self.assertEqual(nansen["cargo"], "30 tons")
        self.assertEqual(nansen["crew"], 8)
        self.assertIn("10 × Stateroom", nansen["equipment"])
        self.assertIn("2 × Laboratory", nansen["equipment"])
        self.assertIn("6 × Fuel Processor", nansen["equipment"])
        self.assertIn("2 × Probe Drones", nansen["equipment"])
        self.assertIn("6 × Low Berth", nansen["equipment"])
        self.assertIn("2 × Emergency Low Berth", nansen["equipment"])
        self.assertIn("Medical Bay (2 beds)", nansen["equipment"])
        self.assertIn("Full Hangar (30 tons contained)", nansen["equipment"])
        self.assertIn("Standard Hangar (14 tons contained)", nansen["equipment"])
        self.assertIn("Carried Craft: Jason (ship-17)", nansen["equipment"])
        self.assertIn(
            "2 × Double Turret: Beam Laser · Beam Laser",
            nansen["armament"],
        )
        self.assertIn(
            "Double Turret: Sandcaster · Sandcaster",
            nansen["armament"],
        )
        self.assertIn(
            "Double Turret: Missile Rack · Missile Rack",
            nansen["armament"],
        )
        self.assertEqual(
            nansen["ammunition"],
            "24 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(nansen["unused_fire_control_stations"], 1)
        self.assertEqual(nansen["length_m"], 68.0)
        self.assertEqual(nansen["art_path"], "assets/ships/ship-076-nansen.webp")

    def test_sentinel_exposes_heavy_system_monitor_fit(self) -> None:
        sentinel = self.records[77]
        self.assertEqual(sentinel["family_id"], 77)
        self.assertEqual(sentinel["path_name"], "Redoubt")
        self.assertEqual(sentinel["tons"], 300)
        self.assertEqual(sentinel["configuration"], "Standard")
        self.assertEqual(sentinel["tech_level"], 11)
        self.assertFalse(sentinel["standard_design"])
        self.assertEqual(
            sentinel["structural_options"],
            ["Reinforced Structure (1 increment)"],
        )
        self.assertEqual(sentinel["electronics"], "Advanced")
        self.assertEqual(sentinel["armor_points"], 11)
        self.assertEqual(sentinel["bridge_options"], ["Hardened Bridge"])
        self.assertEqual(sentinel["computer"], "Model 3")
        self.assertEqual(sentinel["computer_options"], ["Bis"])
        self.assertIsNone(sentinel["jump_drive"])
        self.assertEqual(sentinel["maneuver_drive"], "H")
        self.assertEqual(sentinel["power_plant"], "J")
        self.assertEqual(sentinel["endurance"], 2)
        self.assertEqual(sentinel["cargo"], "76.75 tons")
        self.assertEqual(sentinel["crew"], 8)
        self.assertIn("2 × Stateroom", sentinel["equipment"])
        self.assertIn("Crew Berthing", sentinel["equipment"])
        self.assertIn("Fuel Scoop", sentinel["equipment"])
        self.assertIn("Fuel Processor", sentinel["equipment"])
        self.assertIn("Emergency Low Berth", sentinel["equipment"])
        self.assertIn("Repair Drones", sentinel["equipment"])
        self.assertIn(
            "Triple Turret: Beam Laser · Beam Laser · Sandcaster",
            sentinel["armament"],
        )
        self.assertIn("Particle Beam Barbette", sentinel["armament"])
        self.assertIn("Meson Gun Bay", sentinel["armament"])
        self.assertNotIn("Point Defense", sentinel["armament"])
        self.assertEqual(sentinel["ammunition"], "40 × Sandcaster Canisters")
        self.assertEqual(sentinel["unused_fire_control_stations"], 0)
        self.assertEqual(sentinel["length_m"], 38.0)
        self.assertEqual(
            sentinel["art_path"],
            "assets/ships/ship-077-sentinel.webp",
        )

    def test_raleigh_exposes_armored_merchant_fit(self) -> None:
        raleigh = self.records[81]
        self.assertEqual(raleigh["family_id"], 81)
        self.assertEqual(raleigh["path_name"], "Marque Marine")
        self.assertEqual(raleigh["tons"], 500)
        self.assertEqual(raleigh["configuration"], "Streamlined")
        self.assertEqual(raleigh["tech_level"], 11)
        self.assertFalse(raleigh["standard_design"])
        self.assertEqual(
            raleigh["hull_options"],
            ["Radiation Shielding", "Self Sealing"],
        )
        self.assertEqual(raleigh["electronics"], "Basic Military")
        self.assertEqual(raleigh["armor_points"], 8)
        self.assertEqual(raleigh["computer"], "Model 3")
        self.assertEqual(raleigh["computer_options"], ["Fib"])
        self.assertEqual(raleigh["jump_drive"], "E")
        self.assertEqual(raleigh["jump_distance"], 2)
        self.assertEqual(raleigh["jump_count"], 1)
        self.assertEqual(raleigh["maneuver_drive"], "E")
        self.assertEqual(raleigh["power_plant"], "E")
        self.assertEqual(raleigh["assertions"]["thrust_g"], 2)
        self.assertEqual(raleigh["endurance"], 4)
        self.assertEqual(raleigh["cargo"], "198 tons")
        self.assertEqual(raleigh["crew"], 11)
        self.assertIn("6 × Stateroom", raleigh["equipment"])
        self.assertIn("6 × Fuel Processor", raleigh["equipment"])
        self.assertIn("Armory", raleigh["equipment"])
        self.assertIn("Office", raleigh["equipment"])
        self.assertIn("Workshop", raleigh["equipment"])
        self.assertIn("Medical Bay (1 bed)", raleigh["equipment"])
        self.assertIn(
            "3 × Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            raleigh["armament"],
        )
        self.assertIn(
            "2 × Triple Turret: Beam Laser · Missile Rack · Sandcaster",
            raleigh["armament"],
        )
        self.assertNotIn("Point Defense", raleigh["armament"])
        self.assertEqual(
            raleigh["ammunition"],
            "24 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(raleigh["unused_fire_control_stations"], 0)
        self.assertEqual(raleigh["length_m"], 72.0)
        self.assertEqual(raleigh["art_path"], "assets/ships/ship-081-raleigh.webp")

    def test_aquarius_exposes_distributed_fleet_tanker_fit(self) -> None:
        aquarius = self.records[84]
        self.assertEqual(aquarius["family_id"], 84)
        self.assertEqual(aquarius["path_name"], "Admiralty Line")
        self.assertEqual(aquarius["tons"], 1000)
        self.assertEqual(aquarius["configuration"], "Distributed")
        self.assertEqual(aquarius["tech_level"], 11)
        self.assertFalse(aquarius["standard_design"])
        self.assertEqual(aquarius["electronics"], "Advanced")
        self.assertEqual(aquarius["armor_points"], 0)
        self.assertEqual(aquarius["computer"], "Model 3")
        self.assertEqual(aquarius["computer_options"], ["Bis"])
        self.assertEqual(aquarius["jump_drive"], "H")
        self.assertEqual(aquarius["jump_distance"], 2)
        self.assertEqual(aquarius["jump_count"], 1)
        self.assertEqual(aquarius["maneuver_drive"], "H")
        self.assertEqual(aquarius["power_plant"], "H")
        self.assertEqual(aquarius["endurance"], 2)
        self.assertEqual(aquarius["cargo"], "607 tons")
        self.assertEqual(aquarius["crew"], 9)
        self.assertIn("5 × Stateroom", aquarius["equipment"])
        self.assertIn("Emergency Low Berth", aquarius["equipment"])
        self.assertIn("Repair Drones", aquarius["equipment"])
        self.assertIn("Standard Hangar (20 tons contained)", aquarius["equipment"])
        self.assertIn("Carried Craft: Argo (ship-10)", aquarius["equipment"])
        self.assertIn("2 × Single Turret: Beam Laser", aquarius["armament"])
        self.assertIn("2 × Single Turret: Missile Rack", aquarius["armament"])
        self.assertIn(
            "2 × Double Turret: Sandcaster · Sandcaster",
            aquarius["armament"],
        )
        self.assertNotIn("Point Defense", aquarius["armament"])
        self.assertEqual(
            aquarius["ammunition"],
            "24 × Standard Missiles · 80 × Sandcaster Canisters",
        )
        self.assertEqual(aquarius["unused_fire_control_stations"], 4)
        self.assertEqual(aquarius["length_m"], 84.0)
        self.assertEqual(
            aquarius["art_path"],
            "assets/ships/ship-084-aquarius.webp",
        )

    def test_caravan_exposes_frontier_armed_freighter_fit(self) -> None:
        caravan = self.records[85]
        self.assertEqual(caravan["family_id"], 85)
        self.assertEqual(caravan["path_name"], "Venture Passage")
        self.assertEqual(caravan["tons"], 1000)
        self.assertEqual(caravan["configuration"], "Standard")
        self.assertEqual(caravan["tech_level"], 11)
        self.assertFalse(caravan["standard_design"])
        self.assertEqual(caravan["electronics"], "Advanced")
        self.assertEqual(caravan["armor_points"], 2)
        self.assertEqual(caravan["bridge_options"], ["Holographic Controls"])
        self.assertEqual(caravan["computer"], "Model 3")
        self.assertEqual(caravan["computer_options"], ["Bis"])
        self.assertEqual(caravan["jump_drive"], "H")
        self.assertEqual(caravan["jump_distance"], 2)
        self.assertEqual(caravan["jump_count"], 1)
        self.assertEqual(caravan["maneuver_drive"], "H")
        self.assertEqual(caravan["power_plant"], "H")
        self.assertEqual(caravan["endurance"], 2)
        self.assertEqual(caravan["cargo"], "498 tons")
        self.assertEqual(caravan["crew"], 17)
        self.assertIn("16 × Stateroom", caravan["equipment"])
        self.assertIn("20 × Low Berth", caravan["equipment"])
        self.assertIn("5 × Fuel Processor", caravan["equipment"])
        self.assertIn("Fuel Scoop", caravan["equipment"])
        self.assertIn("Medical Bay (2 beds)", caravan["equipment"])
        self.assertIn("Full Hangar (10 tons contained)", caravan["equipment"])
        self.assertIn("Carried Craft: Charon (ship-158)", caravan["equipment"])
        self.assertIn(
            "4 × Double Turret: Beam Laser · Beam Laser",
            caravan["armament"],
        )
        self.assertIn(
            "3 × Double Turret: Sandcaster · Missile Rack",
            caravan["armament"],
        )
        self.assertNotIn("Point Defense", caravan["armament"])
        self.assertEqual(
            caravan["ammunition"],
            "36 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(caravan["unused_fire_control_stations"], 3)
        self.assertEqual(caravan["length_m"], 74.0)
        self.assertEqual(caravan["art_path"], "assets/ships/ship-085-caravan.webp")

    def test_revenant_exposes_heavy_boarding_raider_fit(self) -> None:
        revenant = self.records[87]
        self.assertEqual(revenant["family_id"], 87)
        self.assertEqual(revenant["path_name"], "Rogue Tide")
        self.assertEqual(revenant["tons"], 600)
        self.assertEqual(revenant["configuration"], "Streamlined")
        self.assertEqual(revenant["tech_level"], 11)
        self.assertFalse(revenant["standard_design"])
        self.assertEqual(revenant["electronics"], "Basic Civilian")
        self.assertEqual(revenant["armor_points"], 4)
        self.assertEqual(revenant["computer"], "Model 3")
        self.assertEqual(revenant["computer_options"], ["Bis"])
        self.assertEqual(revenant["jump_drive"], "H")
        self.assertEqual(revenant["jump_distance"], 2)
        self.assertEqual(revenant["jump_count"], 1)
        self.assertEqual(revenant["maneuver_drive"], "M")
        self.assertEqual(revenant["power_plant"], "M")
        self.assertEqual(revenant["assertions"]["thrust_g"], 4)
        self.assertEqual(revenant["endurance"], 2)
        self.assertEqual(revenant["cargo"], "41.5 tons")
        self.assertEqual(revenant["crew"], 67)
        self.assertIn("50 × Barracks", revenant["equipment"])
        self.assertIn("4 × Ships Brig", revenant["equipment"])
        self.assertIn("6 × Fuel Processor", revenant["equipment"])
        self.assertIn("12 × Low Berth", revenant["equipment"])
        self.assertIn("Breaching Tube", revenant["equipment"])
        self.assertIn("Medical Bay (2 beds)", revenant["equipment"])
        self.assertIn("Repair Drones", revenant["equipment"])
        self.assertIn("Full Hangar (50 tons contained)", revenant["equipment"])
        self.assertIn("Carried Craft: Cutlass (ship-213)", revenant["equipment"])
        self.assertIn(
            "4 × Double Turret: Beam Laser · Beam Laser",
            revenant["armament"],
        )
        self.assertIn(
            "Double Turret: Missile Rack · Missile Rack",
            revenant["armament"],
        )
        self.assertIn(
            "Double Turret: Sandcaster · Sandcaster",
            revenant["armament"],
        )
        self.assertNotIn("Point Defense", revenant["armament"])
        self.assertEqual(
            revenant["ammunition"],
            "48 × Standard Missiles · 40 × Sandcaster Canisters",
        )
        self.assertEqual(revenant["unused_fire_control_stations"], 0)
        self.assertEqual(revenant["length_m"], 70.0)
        self.assertEqual(
            revenant["art_path"],
            "assets/ships/ship-087-revenant.webp",
        )

    def test_fabius_exposes_fleet_replenishment_fit(self) -> None:
        fabius = self.records[89]
        self.assertEqual(fabius["family_id"], 89)
        self.assertEqual(fabius["path_name"], "Admiralty Line")
        self.assertEqual(fabius["tons"], 1000)
        self.assertEqual(fabius["configuration"], "Standard")
        self.assertEqual(fabius["tech_level"], 11)
        self.assertFalse(fabius["standard_design"])
        self.assertEqual(fabius["electronics"], "Advanced")
        self.assertEqual(fabius["armor_points"], 4)
        self.assertEqual(fabius["bridge_options"], ["Holographic Controls"])
        self.assertEqual(fabius["computer"], "Model 3")
        self.assertEqual(fabius["computer_options"], ["Bis"])
        self.assertEqual(fabius["jump_drive"], "H")
        self.assertEqual(fabius["jump_distance"], 2)
        self.assertEqual(fabius["jump_count"], 1)
        self.assertEqual(fabius["maneuver_drive"], "H")
        self.assertEqual(fabius["power_plant"], "H")
        self.assertEqual(fabius["endurance"], 2)
        self.assertEqual(fabius["cargo"], "344.5 tons")
        self.assertEqual(fabius["crew"], 49)
        self.assertIn("5 × Fuel Processor", fabius["equipment"])
        self.assertIn("Fuel Scoop", fabius["equipment"])
        self.assertIn("2 × Workshop", fabius["equipment"])
        self.assertIn(
            "10 × Underway Replenishment System",
            fabius["equipment"],
        )
        self.assertIn("Medical Bay (2 beds)", fabius["equipment"])
        self.assertIn("Repair Drones", fabius["equipment"])
        self.assertIn("Full Hangar (110 tons contained)", fabius["equipment"])
        self.assertNotIn("Carried Craft:", fabius["equipment"])
        self.assertIn(
            "4 × Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            fabius["armament"],
        )
        self.assertIn(
            "3 × Triple Turret: Missile Rack · Missile Rack · Missile Rack",
            fabius["armament"],
        )
        self.assertIn(
            "3 × Triple Turret: Sandcaster · Sandcaster · Sandcaster",
            fabius["armament"],
        )
        self.assertIn(
            "10 × Point Defense Node Mount: Point Defense Laser",
            fabius["armament"],
        )
        self.assertEqual(
            fabius["ammunition"],
            "96 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(fabius["unused_fire_control_stations"], 0)
        self.assertEqual(fabius["length_m"], 78.0)
        self.assertEqual(fabius["art_path"], "assets/ships/ship-089-fabius.webp")

    def test_greyhound_exposes_fast_patrol_frigate_fit(self) -> None:
        greyhound = self.records[92]
        self.assertEqual(greyhound["family_id"], 92)
        self.assertEqual(greyhound["path_name"], "Admiralty Line")
        self.assertEqual(greyhound["tons"], 600)
        self.assertEqual(greyhound["configuration"], "Standard")
        self.assertEqual(greyhound["tech_level"], 11)
        self.assertFalse(greyhound["standard_design"])
        self.assertEqual(greyhound["electronics"], "Advanced")
        self.assertEqual(greyhound["armor_points"], 4)
        self.assertEqual(
            greyhound["bridge_options"],
            ["Hardened Bridge", "Holographic Controls"],
        )
        self.assertEqual(greyhound["computer"], "Model 3")
        self.assertEqual(greyhound["computer_options"], ["Bis"])
        self.assertEqual(greyhound["jump_drive"], "H")
        self.assertEqual(greyhound["jump_distance"], 2)
        self.assertEqual(greyhound["jump_count"], 1)
        self.assertEqual(greyhound["maneuver_drive"], "V")
        self.assertEqual(greyhound["power_plant"], "V")
        self.assertEqual(greyhound["endurance"], 2)
        self.assertEqual(greyhound["cargo"], "127 tons")
        self.assertEqual(greyhound["crew"], 26)
        self.assertIn("8 × Fuel Processor", greyhound["equipment"])
        self.assertIn("Fuel Scoop", greyhound["equipment"])
        self.assertIn("Probe Drones", greyhound["equipment"])
        self.assertIn("4 × Ships Brig", greyhound["equipment"])
        self.assertIn("2 × Emergency Low Berth", greyhound["equipment"])
        self.assertIn("Medical Bay (2 beds)", greyhound["equipment"])
        self.assertIn("Repair Drones", greyhound["equipment"])
        self.assertIn("Full Hangar (20 tons contained)", greyhound["equipment"])
        self.assertIn(
            "Carried Craft: Caduceus (ship-7)",
            greyhound["equipment"],
        )
        self.assertIn(
            "2 × Triple Turret: Beam Laser · Beam Laser · Beam Laser",
            greyhound["armament"],
        )
        self.assertIn(
            "Triple Turret: Missile Rack · Missile Rack · Missile Rack",
            greyhound["armament"],
        )
        self.assertIn(
            "Triple Turret: Sandcaster · Sandcaster · Sandcaster",
            greyhound["armament"],
        )
        self.assertIn("2 × Particle Beam Barbette", greyhound["armament"])
        self.assertIn(
            "6 × Point Defense Node Mount: Point Defense Laser",
            greyhound["armament"],
        )
        self.assertEqual(
            greyhound["ammunition"],
            "72 × Standard Missiles · 60 × Sandcaster Canisters",
        )
        self.assertEqual(greyhound["unused_fire_control_stations"], 0)
        self.assertEqual(greyhound["length_m"], 72.0)
        self.assertEqual(
            greyhound["art_path"],
            "assets/ships/ship-092-greyhound.webp",
        )

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
