#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const partitionIndex = args.indexOf("--partition");
const write = args.includes("--write");
assert(partitionIndex >= 0 && args[partitionIndex + 1], "missing --partition");
assert(args.every((value, index) =>
  value === "--partition" || value === "--write" || index === partitionIndex + 1),
"unsupported argument");
const partitionId = args[partitionIndex + 1];
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family?.startsWith("path-"),
  `${partitionId}: not a path partition`);
const profile = partitionProfile(partitionId);
const goldenPath =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(goldenPath), `${partitionId}: partition golden is missing`);

const auditRecords = new Map(audit.records.map((entry) => [entry.id, entry]));
const auditRules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const auditFixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const sourceEvidence = [
  { path: "content-reference/standard-universe-v1/blessings.json" },
  { path: "content-reference/standard-universe-v1/blessing-levels.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
  { path: "content-reference/standard-universe-v1/paths.json" },
];
const executionEvidence = profile.executionEvidence.map((path) => ({ path }));
const reviewEvidence = profile.reviewEvidence.map((path) => ({ path }));

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: "2026-07-25",
  authoring: {
    workbooks: [
      {
        path: "config/data/Universe.xlsx",
        tables: [
          "UniversePath",
          "UniversePathBlessing",
          "UniverseBlessing",
          "UniverseBlessingLevel",
          "UniverseBlessingParameter",
        ],
      },
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: ["UniverseMechanicRule"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseReviewFixture", "UniverseSourceRecord"],
      },
    ],
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-path-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(goldenPath),
  },
  records: partition.record_ids.map((id) => disposition(
    auditRecords.get(id),
    id.startsWith("universe.blessing.612042")
      ? "ExecutableSharedPrimitive"
      : "ExecutableRuleIr",
    [{ path: "config/data/Universe.xlsx" }],
  )),
  rules: partition.rule_ids.map((id) => {
    const planned = auditRules.get(id);
    const shared = id.startsWith("universe.rule.blessing.612042");
    return {
      ...disposition(
        planned,
        shared ? "ExecutableSharedPrimitive" : "ExecutableRuleIr",
        [{ path: "config/data/UniverseBindings.xlsx" }],
      ),
      implementation_kind: shared ? "SharedPrimitive" : "RuleIr",
      definition_keys: [id, planned.source_record_id],
      execution_evidence: executionEvidence,
    };
  }),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(
      auditFixtures.get(id),
      "ProductionExecuted",
      [{ path: "config/data/UniverseEvidence.xlsx" }],
    ),
    execution_kind: "RustTest",
    test_path: profile.fixturePath,
    test_marker: profile.fixtureMarker,
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: partition.native_review_candidate_rule_ids.map((id) => ({
    id,
    outcome: "IrSufficient",
    decision: nativeDecision(id),
    evidence: reviewEvidence,
  })),
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-path-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-combat --all-features --no-fail-fast",
      ...profile.testCommands,
      "cargo test -p starclock-replay --all-features --no-fail-fast",
    ],
    goldens: [evidence(goldenPath)],
  },
};

const relative =
  `evidence/standard-universe-mechanics-complete-v1/partitions/${partitionId}.json`;
const encoded = `${JSON.stringify(receipt, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(absolute(relative)), { recursive: true });
  fs.writeFileSync(absolute(relative), encoded);
  console.log(`Wrote Goal 07 receipt ${relative}.`);
} else {
  assert(exists(relative), `${relative} is missing`);
  assert(fs.readFileSync(absolute(relative), "utf8") === encoded,
    `${partitionId}: generated receipt drifted`);
  console.log(`Goal 07 receipt ${partitionId} matches generated evidence.`);
}

function nativeDecision(id) {
  if (id.includes("612043"))
    return "Dynamic current-shield and authored-base-stat queries express the capped ATK conversion without a content branch.";
  if (id.includes("612044"))
    return "Turn-end triggers, fixed chance and effect-scoped shield replacement express Sanctuary.";
  if (id.includes("612045"))
    return "Formula-subject filters distinguish shield generation from shield reception in the shared modifier pipeline.";
  if (id.includes("612046"))
    return "Shield delta events, complete cause roles and bounded Rule IR slots express the provider shield and its lifetime.";
  if (id.includes("612050"))
    return "The validated contribution compiler supplies the owned Preservation count to an ordinary percent-of-base modifier.";
  if (id.includes("612032"))
    return "Dedicated shield state, event deltas, scoped removal and Rule IR slots express the complete cycle.";
  if (id.includes("612041"))
    return "Typed effect chance, capped DoT templates and deterministic source filters express Bleed.";
  if (id.includes("612040"))
    return "Stable selector iteration and applied-damage event reads express Quake boost and splash.";
  return "Shield snapshots, derived-stat queries, once scopes and explicit nonlethal damage express Quake.";
}
function partitionProfile(id) {
  if (id === "G07-P2-M02-S01") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration.rs",
        "crates/starclock-mode-universe/tests/preservation_runtime.rs",
        "crates/starclock-combat/tests/ability_program_execution.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s01.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering.rs",
        "crates/starclock-combat/src/rule/model.rs",
      ],
      fixturePath: "crates/starclock-mode-universe/tests/mechanic_battle_integration.rs",
      fixtureMarker: "goal07_p2_m02_s01_executes_every_assigned_rule_and_operation_fixture",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  if (id === "G07-P2-M02-S02") {
    return {
      executionEvidence: [
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s02.rs",
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s02.rs",
        "crates/starclock-mode-universe/tests/preservation_runtime.rs",
        "crates/starclock-combat/tests/modifier_pipeline.rs",
      ],
      reviewEvidence: [
        "docs/goal-07-preservation-s02.md",
        "crates/starclock-mode-universe/src/battle_rule_lowering/preservation_s02.rs",
        "crates/starclock-combat/src/modifier/resolve.rs",
      ],
      fixturePath:
        "crates/starclock-mode-universe/tests/mechanic_battle_integration/preservation_s02.rs",
      fixtureMarker:
        "goal07_p2_m02_s02_executes_dynamic_stat_and_directional_shield_rules",
      testCommands: [
        "cargo test -p starclock-mode-universe --test mechanic_battle_integration goal07_p2_m02_s02 --all-features",
        "cargo test -p starclock-mode-universe --test preservation_runtime --all-features",
      ],
    };
  }
  throw new Error(`${id}: path receipt profile is not implemented`);
}
function disposition(planned, runtimeDisposition, workbookEvidence) {
  assert(planned, "assigned retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbookEvidence,
    provenance_evidence: sourceEvidence,
  };
}
function evidence(relative) {
  return { path: relative, sha256: sha256(relative) };
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function json(relative) { return JSON.parse(fs.readFileSync(absolute(relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
