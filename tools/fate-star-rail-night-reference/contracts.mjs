#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const output = path.join(root,
  "content-reference/fate-star-rail-night-v1/contracts.json");
const manifestPath = path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

const workbooks = [
  workbook("FateStarRailNight.xlsx", [
    "Profiles", "Areas", "Difficulties", "Phases", "BattleZones",
    "Progress", "CaseBoards", "CaseBoardNodes", "Participants", "Teams",
    "Owners", "Traits", "Levels", "Unlocks",
  ]),
  workbook("FateStarRailNightCombat.xlsx", [
    "Stages", "BattleAreas", "Encounters", "Waves", "EnemySlots",
    "EnemyVariants", "EnemyTemplates", "EnemySkills", "EnemyStatuses",
    "Buffs", "MazeBuffs", "BattleEvents", "BattleTargets",
  ]),
  workbook("FateStarRailNightBindings.xlsx", [
    "Masters", "Servants", "NoblePhantasms", "NoblePhantasmLevels",
    "Rarities", "Tags", "Keywords", "Decks", "DeckRecommendations",
    "CommandSpells", "CommandSpellAffixes", "Resources", "RuleBindings",
    "LifecycleBindings",
  ]),
  workbook("FateStarRailNightReview.xlsx", [
    "Sources", "ContentAudit", "Coverage", "ResearchGaps",
    "Reconciliation", "ReviewFixtures", "PackFiles",
  ]),
];

const document = {
  schema_revision: "starclock.fate-star-rail-night-contracts.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P0-B4",
  manifest_binding: {
    path: "content-manifests/fate-star-rail-night-v1/content-manifest.json",
    obligations: manifest.counts.obligations,
    canonical_obligations_sha256: manifest.canonical_obligations_sha256,
  },
  canonical_encoding: {
    text: "UTF-8",
    line_endings: "LF",
    json_indent_spaces: 2,
    trailing_newline: true,
    object_key_order: "schema-declared",
    set_order: "stable-id-ascending",
    sequence_order: "source-semantic-order",
    integer_transport: "exact-decimal-string-when-not-bounded-schema-integer",
    decimal_transport: "canonical-decimal-string-no-exponent",
    forbidden_authoritative_types: ["f32", "f64", "usize", "Excel float"],
    stable_id_pattern: "^[a-z0-9]+(?:[.-][a-z0-9]+)*$",
  },
  common_record_envelope: [
    "stable_id", "family", "name_zh", "name_en", "summary_zh",
    "summary_en", "ownership", "disposition", "enabled", "source_refs",
    "evidence_quality", "mechanism_quality", "confidence", "notes",
  ],
  evidence_record: {
    required: [
      "source_id", "repository_or_url", "revision_or_access_date",
      "game_version", "path_or_page", "row_locator", "sha256",
      "evidence_quality", "mechanism_quality", "note",
    ],
    quality_values: [
      "ExactStructured", "ExactPublicText", "Observed",
      "ApproximateFromReleasedText", "ProjectPolicy",
    ],
    approximation_required: [
      "unavailable_fact", "selected_policy", "rejected_alternatives",
      "rationale", "affected_fixtures", "replacement_condition",
    ],
  },
  reconciliation_record: {
    identity: ["source_path", "row_locator", "source_sha256"],
    required: [
      "peer_goal", "peer_stable_id", "local_stable_id", "classification",
      "semantic_result", "decision", "note",
    ],
    result_values: ["SharedIdentical", "DistinctModeCopy", "EvidenceOnly", "Conflict"],
  },
  fixture_record: {
    required: [
      "fixture_id", "mechanic_family", "initial_state", "commands",
      "expected_facts", "source_refs", "mechanism_quality",
    ],
    fact_language: ["equals", "contains", "ordered_equals", "absent"],
    execution_boundary: "reference-fact-evaluator-only-no-runtime-lowering",
  },
  authoring: {
    editable_format: "xlsx",
    adapter: "python-openpyxl-3.1.5",
    schema_codegen_export: "sora-cli-0.3.0",
    workbook_policy: "complete-clean-target-no-overwrite",
    workbooks,
    total_sheets: workbooks.reduce((sum, row) => sum + row.sheets.length, 0),
    json_runtime_loading: false,
  },
  output_roots: {
    normalized: "content-reference/fate-star-rail-night-v1/",
    sora_project: "config/fate-star-rail-night/",
    generated: "config/fate-star-rail-night-generated/",
    evidence: "evidence/fate-star-rail-night-reference-v1/",
  },
};
document.contract_sha256 = digest(`${JSON.stringify(document)}\n`);
const serialized = `${JSON.stringify(document, null, 2)}\n`;

if (check) {
  assert(fs.existsSync(output), "Goal 19 contracts are missing");
  assert(fs.readFileSync(output, "utf8") === serialized,
    "Goal 19 contracts drift");
  assert(document.manifest_binding.obligations === 1904,
    "Goal 19 denominator drift");
  assert(new Set(workbooks.flatMap(({ sheets }) => sheets)).size
    === document.authoring.total_sheets, "duplicate workbook sheet ownership");
  console.log(`Goal 19 contracts verified (${document.authoring.total_sheets} sheets, ${document.contract_sha256}).`);
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, serialized);
  console.log(`Wrote Goal 19 contracts (${document.authoring.total_sheets} sheets, ${document.contract_sha256}).`);
}

function workbook(name, sheets) {
  return { name, sheets };
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
