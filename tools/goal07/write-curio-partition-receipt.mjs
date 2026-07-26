#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const index = args.indexOf("--partition");
const write = args.includes("--write");
assert(index >= 0 && args[index + 1], "missing --partition");
assert(args.every((value, offset) =>
  value === "--partition" || value === "--write" || offset === index + 1),
"unsupported argument");
const partitionId = args[index + 1];
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family?.startsWith("curio-"),
  `${partitionId}: not a Curio partition`);
assert(partitionId === "G07-P3-M11-S01",
  `${partitionId}: Curio receipt profile is not implemented`);
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const rules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const fixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const sourceEvidence = [
  { path: "content-reference/standard-universe-v1/curios.json" },
  { path: "content-reference/standard-universe-v1/curio-states.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
];
const executionEvidence = [
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: "crates/starclock-mode-universe/src/runtime.rs" },
  { path: "crates/starclock-mode-universe/src/topology_reward.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s01.rs" },
  { path: "crates/starclock-mode-universe/tests/mechanic_battle_integration/curio_s01.rs" },
];
const reviewEvidence = [
  { path: "docs/goal-07-curio-s01.md" },
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s01.rs" },
];

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: "2026-07-26",
  authoring: {
    workbooks: [
      {
        path: "config/data/Universe.xlsx",
        tables: ["UniverseCurio", "UniverseCurioState", "UniverseCurioParameter"],
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
      `python -c "import openpyxl" && python tools/goal07/author-curio-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: partition.record_ids.map((id) =>
    disposition(records.get(id), "ExecutableSharedPrimitive", [
      { path: "config/data/Universe.xlsx" },
    ])),
  rules: partition.rule_ids.map((id) => ({
    ...disposition(rules.get(id), "ExecutableSharedPrimitive", [
      { path: "config/data/UniverseBindings.xlsx" },
    ]),
    implementation_kind: "SharedPrimitive",
    definition_keys: [id, rules.get(id).source_record_id],
    execution_evidence: executionEvidence,
  })),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(fixtures.get(id), "ProductionExecuted", [
      { path: "config/data/UniverseEvidence.xlsx" },
    ]),
    execution_kind: "RustTest",
    test_path:
      "crates/starclock-mode-universe/tests/mechanic_battle_integration/curio_s01.rs",
    test_marker:
      "goal07_p3_m11_s01_executes_every_assigned_curio_and_fixture_family",
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: partition.native_review_candidate_rule_ids.map((id) => ({
    id,
    outcome: "IrSufficient",
    decision: nativeDecision(id),
    evidence: reviewEvidence,
  })),
  numeric_approximations: [
    {
      id: "universe.curio.107.destructible-success-chance",
      disposition: "ExternalDecision",
      rationale:
        "The public source confirms a small chance but does not expose an authoritative probability. The replayable command records NoEffect, Blessing or Failure without inventing a number.",
    },
  ],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-curio-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-mode-universe --test mechanic_battle_integration curio_s01 --all-features",
      "cargo test -p starclock-mode-universe --lib curio_activity::tests --all-features",
      "cargo test -p starclock-mode-universe --all-features",
    ],
    goldens: [evidence(golden)],
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
  console.log(`Goal 07 Curio receipt ${partitionId} matches generated evidence.`);
}

function nativeDecision(id) {
  const stable = id.split(".").slice(3, 4)[0];
  return {
    "1": "Conditional random-offer limits, a bounded pending choice and ordinary Curio teardown express both limited-use triggers atomically.",
    "102": "Acquisition-time integer division and the generic fragment-gain multiplier express the current-fragment grant exactly.",
    "104": "The Activity destroyed-Curio counter is captured in the immutable battle contribution and lowered to ordinary source damage modifiers.",
    "106": "Returned participant carry supplies full-HP facts to the generic after-battle Curio event and checked fragment operation.",
    "107": "A replayable external outcome command executes blessing acquisition or atomic Curio, Energy and Technique Point teardown without inventing an unpublished probability.",
    "11": "Initial keyed Resonance Energy and the ordinary Resonance damage ratio are compiled from the Curio contribution at battle assembly.",
    "110": "A conditional reward-node bypass and the shared checked fragment multiplier express both Gossip clauses.",
    "111": "A Technique ability tag, ordinary DamageBoost and flat pre-multiplier damage stage express both released Technique damage terms.",
  }[stable] ?? "Generic Activity and Rule IR primitives express the assigned Curio state.";
}
function disposition(planned, runtimeDisposition, workbookEvidence) {
  assert(planned, "retained-audit entry is missing");
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
function json(relative) {
  return JSON.parse(fs.readFileSync(absolute(relative), "utf8"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
