#!/usr/bin/env python3

from __future__ import annotations

import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from check_repository import validate_release_notes  # noqa: E402


VALID_NOTES = """## Compatibility notice

Cepheus Trader v0.7.10 advances to CT-RPC 8.

## Highlights

- Added a complete release-note guard.

**Full changelog:** https://github.com/RealDeuce/ct/compare/v0.7.9...v0.7.10
"""


class ReleaseNotesTests(unittest.TestCase):
    def test_complete_release_notes_pass(self) -> None:
        self.assertEqual(validate_release_notes("0.7.10", VALID_NOTES), [])

    def test_compatibility_notice_is_required(self) -> None:
        notes = VALID_NOTES.replace("## Compatibility notice", "## Upgrade")
        self.assertIn("Compatibility notice section", validate_release_notes("0.7.10", notes))

    def test_curated_highlight_is_required(self) -> None:
        notes = VALID_NOTES.replace("- Added a complete release-note guard.\n", "")
        self.assertIn("at least one curated highlight", validate_release_notes("0.7.10", notes))

    def test_changelog_must_end_at_the_product_version(self) -> None:
        notes = VALID_NOTES.replace("...v0.7.10", "...v0.7.9")
        self.assertIn(
            "version-matched full changelog link",
            validate_release_notes("0.7.10", notes),
        )


if __name__ == "__main__":
    unittest.main()
