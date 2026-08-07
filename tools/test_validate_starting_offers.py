#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import shutil
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from validate_starting_offers import validate  # noqa: E402


class ValidateStartingOffersTests(unittest.TestCase):
    def test_repository_offer_mapping_validates(self) -> None:
        message = validate(
            ROOT / "catalog" / "starting-offers.toml",
            ROOT / "catalog" / "ships",
        )
        self.assertEqual(
            message,
            "validated 27 starting offers in 9 polity cells using "
            "19 catalog designs",
        )

    def test_jump_one_design_is_rejected_as_a_starting_offer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ships = Path(directory) / "ships"
            shutil.copytree(ROOT / "catalog" / "ships", ships)
            hudson = ships / "ship-192.toml"
            source = hudson.read_text()
            source = source.replace(
                "jump_distance = 2",
                "jump_distance = 1",
                1,
            )
            hudson.write_text(source)
            with self.assertRaisesRegex(
                ValueError,
                "must be fitted and fueled for at least one Jump-2 transit",
            ):
                validate(ROOT / "catalog" / "starting-offers.toml", ships)


if __name__ == "__main__":
    unittest.main()
