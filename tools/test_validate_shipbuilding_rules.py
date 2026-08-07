import tempfile
from pathlib import Path
import unittest

from validate_shipbuilding_rules import DEFAULT_RULES, DEFAULT_SOURCES, validate


class ValidateShipbuildingRulesTests(unittest.TestCase):
    def test_repository_rules_validate(self):
        self.assertEqual(validate(DEFAULT_RULES, DEFAULT_SOURCES), [])

    def test_z_drive_cannot_become_active_extension(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            for source in DEFAULT_RULES.glob("*.toml"):
                (target / source.name).write_bytes(source.read_bytes())
            extension = target / "af3-components.toml"
            extension.write_text(
                extension.read_text()
                + '\n[[equipment]]\nid = "zimm-emitter"\n'
                + 'unit = "installation"\n'
                + "displacement_millitons_per_unit = 0\n"
                + "price_credits_per_unit = 0\n",
                encoding="utf-8",
            )
            errors = validate(target, DEFAULT_SOURCES)
            self.assertTrue(any("active forbidden rule" in error for error in errors))

    def test_conversion_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            for source in DEFAULT_RULES.glob("*.toml"):
                (target / source.name).write_bytes(source.read_bytes())
            policy = target / "ct-ruleset.toml"
            policy.write_text(
                policy.read_text().replace(
                    'hull_tons = 500\nsource_z_code = "G"\n'
                    'source_preserving_j_code = "G"\noptimized_j_code = "E"',
                    'hull_tons = 500\nsource_z_code = "G"\n'
                    'source_preserving_j_code = "G"\noptimized_j_code = "F"',
                ),
                encoding="utf-8",
            )
            errors = validate(target, DEFAULT_SOURCES)
            self.assertTrue(any("minimum CE Jump-2" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
