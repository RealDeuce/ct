#!/usr/bin/env python3

from pathlib import Path
import sys
import tempfile
import unittest


sys.path.insert(0, str(Path(__file__).resolve().parent))
from compile_catalog_ogl import (
    CatalogLicenseError,
    compile_notices,
    replace_section15,
)


REGISTRY = """\
schema_version = 1
catalog_source_ids = ["ct-source"]
required_entry_source_ids = ["rules-source"]

[[notice]]
notice_id = "ogl"
title = "OGL"
source_descriptions = ["license"]
open_game_content_descriptions = []
excluded_product_identity_descriptions = []
texts = ["OGL notice"]

[[notice]]
notice_id = "srd"
title = "SRD"
source_descriptions = ["rules"]
open_game_content_descriptions = ["mechanics"]
excluded_product_identity_descriptions = []
texts = ["SRD notice"]

[[notice]]
notice_id = "book"
title = "Book"
source_descriptions = ["ship"]
open_game_content_descriptions = ["design"]
excluded_product_identity_descriptions = ["names"]
texts = ["Book notice"]

[[notice]]
notice_id = "ct"
title = "CT"
source_descriptions = ["catalog"]
open_game_content_descriptions = ["all"]
excluded_product_identity_descriptions = []
texts = ["CT notice"]

[[source]]
source_id = "rules-source"
title = "Rules"
source_descriptions = ["rules source"]
open_game_content_descriptions = ["mechanics"]
excluded_product_identity_descriptions = []
notice_ids = ["ogl", "srd"]

[[source]]
source_id = "book-source"
title = "Book"
source_descriptions = ["ship source"]
open_game_content_descriptions = ["design"]
excluded_product_identity_descriptions = ["names"]
notice_ids = ["ogl", "srd", "book"]

[[source]]
source_id = "ct-source"
title = "CT"
source_descriptions = ["catalog source"]
open_game_content_descriptions = ["all"]
excluded_product_identity_descriptions = []
notice_ids = ["ogl", "ct"]
"""


def entry(source_ids: str) -> str:
    return f"""\
schema_version = 1
design_id = "ship-1"
source_ids = [{source_ids}]

[catalog]
status = "active"
open_game_content_designations = ["The complete entry is OGC."]
"""


class CompileCatalogOglTests(unittest.TestCase):
    def compile(self, entry_text: str) -> str:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            registry = root / "sources.toml"
            catalog = root / "ships"
            catalog.mkdir()
            registry.write_text(REGISTRY, encoding="utf-8")
            (catalog / "ship-1.toml").write_text(entry_text, encoding="utf-8")
            return compile_notices(registry, catalog, include_drafts=False)

    def test_merges_in_master_order_and_adds_catalog_notice(self) -> None:
        output = self.compile(entry('"rules-source", "book-source"'))
        self.assertLess(output.index("OGL notice"), output.index("SRD notice"))
        self.assertLess(output.index("SRD notice"), output.index("Book notice"))
        self.assertLess(output.index("Book notice"), output.index("CT notice"))

    def test_rejects_single_source_shortcut(self) -> None:
        with self.assertRaises(CatalogLicenseError):
            self.compile(entry('"book-source"'))

    def test_rejects_omitted_ogl(self) -> None:
        with self.assertRaises(CatalogLicenseError):
            self.compile(entry('"book-source", "ct-source"'))

    def test_rejects_unknown_source(self) -> None:
        with self.assertRaises(CatalogLicenseError):
            self.compile(entry('"rules-source", "missing"'))

    def test_replaces_only_section_15(self) -> None:
        original = (
            "license terms\n\n15. COPYRIGHT NOTICE\n\nold notice\n\n"
            "END OF LICENSE\n"
        )
        section = "15. COPYRIGHT NOTICE\n\nnew notice\n"
        self.assertEqual(
            replace_section15(original, section),
            "license terms\n\n15. COPYRIGHT NOTICE\n\nnew notice\n\n"
            "END OF LICENSE\n",
        )

    def test_rejects_license_without_end_marker(self) -> None:
        with self.assertRaises(CatalogLicenseError):
            replace_section15("15. COPYRIGHT NOTICE\n", "replacement\n")


if __name__ == "__main__":
    unittest.main()
