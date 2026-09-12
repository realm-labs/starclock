"""Regression checks for source-authoring membership, not Rust orchestration."""

from copy import deepcopy
import unittest

from persona_reachability import ROOT, audit_rows, load_sources, promotion_defects, reconciliation_defects


class PersonaReachabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tables, _ = load_sources(ROOT / ".cache/content-reference/turnbasedgamedata")

    def test_all_source_rows_accounted_once_without_prefix_admission(self):
        report = audit_rows(self.tables)
        self.assertEqual(report["summary"], {
            "source_tables": 11, "source_rows": 547, "current_areas": 28,
            "current_layers": 11, "current_layer_reference_rows": 78,
            "pending_selector_rows": 469,
        })
        self.assertEqual(len({row["source"] for row in report["rows"]}), 547)
        self.assertTrue(all(row["parent_sources"] for row in report["rows"]
                            if row["reachability"] == "CurrentLayerReference"))
        self.assertTrue(all(not row["parent_sources"] for row in report["rows"]
                            if row["reachability"] == "PendingSelectorProof"))
        self.assertEqual(report["rejected_shortcuts"]["area_ILPNADCAIBL_is_style_selector"]["missing_style_keys"],
                         [501, 601, 701, 801])
        self.assertTrue(all(row["reachability"] == "PendingSelectorProof"
                            for row in report["rows"] if "RoguePersonaStyle.json" in row["source"]))

    def test_missing_layer_duplicate_ordinal_and_dangling_preset_fail_closed(self):
        for mutation in ["missing", "duplicate", "dangling", "gap"]:
            with self.subTest(mutation=mutation):
                tables = deepcopy(self.tables)
                rows = tables["RoguePersonaLayerRoom"]
                if mutation == "missing":
                    rows[:] = [row for row in rows if row["CBCHIHEOEGK"] != 103]
                elif mutation == "duplicate":
                    rows.append(deepcopy(rows[0]))
                elif mutation == "dangling":
                    rows[0]["BKHDBIFFIKP"] = 999999
                else:
                    rows[0]["EEPIDJJJMAH"] = 99
                with self.assertRaises(ValueError):
                    audit_rows(tables)

    def test_composition_level_type_and_attributes_require_reviewed_closure(self):
        for table, field, value in [
            ("RoguePersonaRoomPreset", "LLICIMBCNPF", 999),
            ("RoguePersonaRoomPreset", "AAGKEBFHLMC", 999),
            ("RoguePersonaRoomPreset", "FJIKMHCJMKH", [101]),
            ("RoguePersonaRoomCompType", "LLICIMBCNPF", 999),
            ("RoguePersonaRoomComposition", "AAGKEBFHLMC", 999),
        ]:
            with self.subTest(table=table, field=field):
                tables = deepcopy(self.tables)
                tables[table][0][field] = value
                with self.assertRaises(ValueError):
                    audit_rows(tables)

    def test_promotion_gate_requires_actual_normalized_coverage(self):
        report = audit_rows(self.tables)
        coverage = [{"source_locator": row["source"]} for row in report["rows"]]
        self.assertEqual(promotion_defects(report, coverage), [])
        self.assertTrue(promotion_defects(report, coverage[:-1]))
        self.assertTrue(promotion_defects(report, [*coverage, coverage[0]]))

    def test_release_gate_rejects_blanket_exclusion_and_unaccounted_rows(self):
        report = audit_rows(self.tables)
        manifest = {"exclusions": {"mode_prefixes": [], "named_mode_source_files": [], "historical_rows": []},
                    "categories": {"test": {"records": [{"source": row["source"]} for row in report["rows"]]}}}
        self.assertEqual(reconciliation_defects(report, manifest), [])
        for mutation in ["prefix", "file", "missing", "duplicate", "contradiction"]:
            with self.subTest(mutation=mutation):
                edited = deepcopy(manifest)
                if mutation == "prefix":
                    edited["exclusions"]["mode_prefixes"] = ["RoguePersona"]
                elif mutation == "file":
                    edited["exclusions"]["named_mode_source_files"] = [{"source": "ExcelOutput/RoguePersonaLayerRoom.json"}]
                elif mutation == "missing":
                    edited["categories"]["test"]["records"].pop()
                elif mutation == "duplicate":
                    edited["categories"]["test"]["records"].append(deepcopy(edited["categories"]["test"]["records"][0]))
                else:
                    source = next(row["source"] for row in report["rows"] if row["reachability"] == "CurrentLayerReference")
                    edited["exclusions"]["historical_rows"] = [{"source": source, "reason": "unsupported claim", "reachability": "Excluded"}]
                self.assertTrue(reconciliation_defects(report, edited))


if __name__ == "__main__":
    unittest.main()
