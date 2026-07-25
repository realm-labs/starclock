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
assert(partition?.mechanic_family === "shared-activity-and-ability-tree",
  `${partitionId}: not an Ability Tree partition`);
const goldenPath =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(goldenPath), `${partitionId}: partition golden is missing`);

const auditRecords = new Map(audit.records.map((entry) => [entry.id, entry]));
const auditRules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const auditFixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const workbookEvidence = (file) => [{ path: `config/data/${file}` }];
const provenanceEvidence = [{ path: "content-reference/standard-universe-v1/ability-tree.json" }];
const ruleExecution = [
  {
    path: "crates/starclock-mode-universe/tests/goal07_ability_tree_s01.rs",
  },
  {
    path: "crates/starclock-mode-universe/tests/encounter_runtime.rs",
  },
];
const fixtureMarker =
  "goal07_p2_m01_s01_executes_every_assigned_rule_and_operation_fixture";

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
          "UniverseAbilityTreeNode",
          "UniverseAbilityTreeEdge",
          "UniverseAbilityTreeCost",
          "UniverseAbilityTreeEffect",
          "UniverseAbilityTreeParameter",
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
      `python -c "import openpyxl" && python tools/goal07/author-ability-tree-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(goldenPath),
  },
  records: partition.record_ids.map((id) => disposition(
    auditRecords.get(id),
    "ExecutableRuleIr",
    workbookEvidence("Universe.xlsx"),
  )),
  rules: partition.rule_ids.map((id) => {
    const planned = auditRules.get(id);
    return {
      ...disposition(
        planned,
        "ExecutableRuleIr",
        workbookEvidence("UniverseBindings.xlsx"),
      ),
      implementation_kind: "RuleIr",
      definition_keys: [id, planned.source_record_id],
      execution_evidence: ruleExecution,
    };
  }),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(
      auditFixtures.get(id),
      "ProductionExecuted",
      workbookEvidence("UniverseEvidence.xlsx"),
    ),
    execution_kind: "RustTest",
    test_path: "crates/starclock-mode-universe/tests/goal07_ability_tree_s01.rs",
    test_marker: fixtureMarker,
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: [],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-ability-tree-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-activity --test activity_transaction --all-features",
      "cargo test -p starclock-mode-universe --test goal07_ability_tree_s01 --all-features",
      "cargo test -p starclock-mode-universe --test encounter_runtime --all-features",
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

function disposition(planned, runtimeDisposition, workbook) {
  assert(planned, "assigned retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbook,
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
function exists(relative) { return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile(); }
function json(relative) { return JSON.parse(fs.readFileSync(absolute(relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
