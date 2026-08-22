#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from validate_ship_catalog import validate  # noqa: E402


class ValidateShipCatalogTests(unittest.TestCase):
    def test_repository_catalog_and_forward_craft_dependencies_validate(
        self,
    ) -> None:
        messages = validate(
            ROOT / "catalog" / "ships",
            ROOT / "catalog" / "shipbuilding",
            ROOT / "catalog" / "ogl-sources.toml",
        )
        self.assertEqual(
            messages[0],
            "validated 215 rule-derived ship catalog entries (215 active) "
            "in 114 families (39 shared lineages, 75 singleton designs) "
            "across 9 upgrade paths with canonical names",
        )


if __name__ == "__main__":
    unittest.main()
