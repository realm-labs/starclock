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
assert(partitionId === "G07-P3-M12-S01",
  `${partitionId}: negative Curio receipt profile is not implemented`);

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family === "curio-negative-error-repair-replacement",
  `${partitionId}: not a negative Curio partition`);
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const rules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const fixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);

const provenanceEvidence = [
  { path: "content-reference/standard-universe-v1/curios.json" },
  { path: "content-reference/standard-universe-v1/curio-states.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
];
const executionEvidence = [
  { path: "crates/starclock-mode-universe/src/curio_activity/negative.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/negative_curio_commands.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_negative_s01.rs" },
  { path: "crates/starclock-mode-universe/tests/mechanic_battle_integration/curio_negative_s01.rs" },
];
const reviewEvidence = [
  { path: "docs/goal-07-negative-curio-s01.md" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_negative_s01.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/negative_curio_commands.rs" },
];

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: "2026-07-27",
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
    ...fixtureEvidence(id),
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
      id: "universe.curio.108.fission-chance",
      disposition: "ExternalDecision",
      rationale:
        "Released evidence confirms a split chance but publishes no probability. The replay command records NoSplit or Split and enforces the exact three-copy cap.",
    },
    {
      id: "universe.curio.115.higher-rarity-chance",
      disposition: "ExternalDecision",
      rationale:
        "Released evidence confirms a chance for higher-rarity Blessings but publishes no probability. The replay command records and validates the complete replacement mapping.",
    },
  ],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-curio-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-mode-universe --lib runtime::negative_curio_commands::tests --all-features",
      "cargo test -p starclock-mode-universe --test mechanic_battle_integration curio_negative_s01 --all-features",
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
  console.log(`Goal 07 negative Curio receipt ${partitionId} matches generated evidence.`);
}

function fixtureEvidence(id) {
  const runtime = "crates/starclock-mode-universe/src/runtime/negative_curio_commands.rs";
  const combat =
    "crates/starclock-mode-universe/tests/mechanic_battle_integration/curio_negative_s01.rs";
  return {
    "universe.fixture.curio-state.fixed": {
      test_path: combat,
      test_marker: "code_state_rules_retain_exact_repairing_and_fixed_operations",
    },
    "universe.fixture.curio-state.repairing": {
      test_path: runtime,
      test_marker: "repairing_codes_transition_only_after_three_won_battles",
    },
    "universe.fixture.curio-tag.enhance": {
      test_path: runtime,
      test_marker: "fools_mask_preserves_enhancement_levels_and_validates_complete_mapping",
    },
    "universe.fixture.curio-tag.negative": {
      test_path: combat,
      test_marker: "goal07_p3_m12_s01_executes_assigned_states_without_native_handlers",
    },
    "universe.fixture.curio-tag.repair": {
      test_path: runtime,
      test_marker: "void_wick_repairs_two_tracked_destroyed_curios_on_reward_rng",
    },
    "universe.fixture.curio-tag.replacement": {
      test_path: runtime,
      test_marker: "shining_die_replaces_every_owned_curio_in_one_atomic_random_boundary",
    },
  }[id] ?? fail(`${id}: no fixture evidence`);
}

function nativeDecision(id) {
  const stable = id.split(".")[3];
  return {
    "108": "A bounded Activity copy counter, immutable battle contribution and ordinary ATK modifier express the exact stack and three-copy cap; the unpublished split chance remains a replay decision.",
    "115": "A complete replay-recorded mapping plus generic inventory operations preserves enhancement and validates same-or-higher rarity without an unpublished distribution.",
    "17": "Destroyed-Curio identity counters and the generic Reward-stream random boundary restore up to two distinct Curios atomically.",
    "21": "A generic random boundary tears down all owned Curios and acquires the same number of distinct unowned replacements atomically.",
    "45": "Generic repair charges transition after three won battles, while typed WeaknessBroken Rule IR sets Energy to zero or maximum by state.",
    "47": "Generic repair charges transition after three won battles, while typed post-Ultimate Rule IR consumes or heals 30% current HP by state.",
    "49": "Eight ordinary mitigation modifiers express the fixed state's exact 50% reduction; the repairing state is assigned to M12-S02.",
  }[stable] ?? "Generic Activity and Rule IR primitives express the assigned negative Curio state.";
}

function disposition(planned, runtimeDisposition, workbookEvidence) {
  assert(planned, "retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbookEvidence,
    provenance_evidence: provenanceEvidence,
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
function fail(message) { throw new Error(message); }
function assert(condition, message) { if (!condition) fail(message); }
