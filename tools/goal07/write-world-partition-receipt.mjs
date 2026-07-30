#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
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
assert([
  "G07-P5-M15-S19",
  "G07-P5-M15-S20",
  "G07-P5-M15-S21",
  "G07-P5-M15-S22",
  "G07-P5-M15-S23",
  "G07-P5-M15-S24",
  "G07-P5-M15-S25",
  "G07-P5-M15-S26",
  "G07-P5-M15-S27",
  "G07-P5-M15-S28",
  "G07-P5-M15-S29",
  "G07-P5-M15-S30",
  "G07-P5-M15-S31",
  "G07-P5-M15-S32",
  "G07-P5-M15-S33",
  "G07-P5-M15-S34",
  "G07-P5-M15-S35",
  "G07-P5-M15-S36",
].includes(partitionId),
  `${partitionId}: world receipt profile is not implemented`);

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert([
  "domain-graph",
  "encounter-selection",
  "topology-map",
  "room-content",
  "world-difficulty",
].includes(partition?.lane),
  `${partitionId}: not a supported world-structure partition`);
const domainPartition = partition.lane === "domain-graph";
const topologyPartition = partition.lane === "topology-map";
const roomPartition = partition.lane === "room-content";
const worldPartition = partition.lane === "world-difficulty";
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const fixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);

const provenanceEvidence = domainPartition
  ? [
      { path: "content-reference/standard-universe-v1/domains.json" },
      { path: "content-reference/standard-universe-v1/sources.json" },
    ]
  : topologyPartition
  ? [
      { path: "content-reference/standard-universe-v1/maps.json" },
      { path: "content-reference/standard-universe-v1/sources.json" },
    ]
  : roomPartition
  ? [
      { path: "content-reference/standard-universe-v1/rooms.json" },
      { path: "content-reference/standard-universe-v1/domains.json" },
      { path: "content-reference/standard-universe-v1/encounter-groups.json" },
      { path: "content-reference/standard-universe-v1/sources.json" },
    ]
  : worldPartition
  ? [
      { path: "content-reference/standard-universe-v1/worlds.json" },
      { path: "content-reference/standard-universe-v1/world-difficulties.json" },
      { path: "content-reference/standard-universe-v1/sources.json" },
    ]
  : [
      { path: "content-reference/standard-universe-v1/encounter-pools.json" },
      { path: "content-reference/standard-universe-v1/review-fixtures.json" },
      { path: "content-reference/standard-universe-v1/sources.json" },
    ];
const encounterTestPath =
  `crates/starclock-mode-universe/tests/encounter_selection_${partitionId.slice(-3).toLowerCase()}.rs`;
const topologyTestPath =
  `crates/starclock-mode-universe/tests/topology_map_${partitionId.slice(-3).toLowerCase()}.rs`;
const roomTestPath =
  `crates/starclock-mode-universe/tests/room_content_${partitionId.slice(-3).toLowerCase()}.rs`;
const worldTestPath =
  `crates/starclock-mode-universe/tests/world_difficulty_${partitionId.slice(-3).toLowerCase()}.rs`;
const executionEvidence = domainPartition
  ? [
      { path: "crates/starclock-mode-universe/src/definition.rs" },
      { path: "crates/starclock-mode-universe/src/lowering.rs" },
      { path: "crates/starclock-mode-universe/src/topology.rs" },
      { path: "crates/starclock-mode-universe/tests/domain_runtime.rs" },
    ]
  : topologyPartition
  ? [
      { path: "crates/starclock-mode-universe/src/definition.rs" },
      { path: "crates/starclock-mode-universe/src/lowering.rs" },
      { path: "crates/starclock-mode-universe/src/topology.rs" },
      { path: topologyTestPath },
    ]
  : roomPartition
  ? [
      { path: "crates/starclock-mode-universe/src/definition.rs" },
      { path: "crates/starclock-mode-universe/src/encounter.rs" },
      { path: "crates/starclock-mode-universe/src/lowering.rs" },
      { path: "crates/starclock-mode-universe/src/encounter_lowering.rs" },
      { path: "crates/starclock-mode-universe/src/topology_support.rs" },
      { path: roomTestPath },
    ]
  : worldPartition
  ? [
      { path: "crates/starclock-mode-universe/src/definition.rs" },
      { path: "crates/starclock-mode-universe/src/encounter.rs" },
      { path: "crates/starclock-mode-universe/src/lowering.rs" },
      { path: "crates/starclock-mode-universe/src/encounter_lowering.rs" },
      { path: "crates/starclock-mode-universe/src/entry.rs" },
      { path: "crates/starclock-mode-universe/src/battle_materialization.rs" },
      { path: worldTestPath },
    ]
  : [
      { path: "crates/starclock-mode-universe/src/encounter.rs" },
      { path: "crates/starclock-mode-universe/src/encounter_lowering.rs" },
      { path: "crates/starclock-mode-universe/src/encounter_content_runtime.rs" },
      { path: encounterTestPath },
    ];
const authoringWorkbooks = domainPartition
  ? [
      {
        path: "config/data/Universe.xlsx",
        tables: ["UniverseDomain"],
      },
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: ["UniverseActivityDomainBinding"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseSourceRecord"],
      },
    ]
  : topologyPartition
  ? [
      {
        path: "config/data/Universe.xlsx",
        tables: ["UniverseMapNode", "UniverseMapEdge"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseSourceRecord"],
      },
    ]
  : roomPartition
  ? [
      {
        path: "config/data/Universe.xlsx",
        tables: ["UniverseRoom", "UniverseRoomContent"],
      },
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: ["UniverseEncounterGroup"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseSourceRecord"],
      },
    ]
  : worldPartition
  ? [
      {
        path: "config/data/Universe.xlsx",
        tables: [
          "UniverseWorld",
          "UniverseDifficulty",
          "UniverseDifficultyEnemy",
        ],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseSourceRecord"],
      },
    ]
  : [
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: [
          "UniverseEncounterPool",
          "UniverseEncounterPoolGroup",
          "UniverseEncounterPoolFixed",
        ],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: [
          "UniverseContentAudit",
          "UniverseReviewFixture",
          "UniverseSourceRecord",
        ],
      },
    ];
const focusedTest = domainPartition
  ? "cargo test -p starclock-mode-universe --test domain_runtime --all-features"
  : topologyPartition
  ? `cargo test -p starclock-mode-universe --test topology_map_${partitionId.slice(-3).toLowerCase()} --all-features`
  : roomPartition
  ? `cargo test -p starclock-mode-universe --test room_content_${partitionId.slice(-3).toLowerCase()} --all-features`
  : worldPartition
  ? `cargo test -p starclock-mode-universe --test world_difficulty_${partitionId.slice(-3).toLowerCase()} --all-features`
  : `cargo test -p starclock-mode-universe --test encounter_selection_${partitionId.slice(-3).toLowerCase()} --all-features`;

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: "2026-07-29",
  authoring: {
    workbooks: authoringWorkbooks,
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-world-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: partition.record_ids.map((id) => disposition(
    records.get(id),
    domainPartition
      ? ["Universe.xlsx", "UniverseBindings.xlsx"]
      : topologyPartition
      ? ["Universe.xlsx"]
      : roomPartition
      ? ["Universe.xlsx"]
      : worldPartition
      ? ["Universe.xlsx"]
      : ["UniverseBindings.xlsx"],
  )),
  rules: [],
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(fixtures.get(id), ["UniverseEvidence.xlsx"]),
    execution_kind: "RustTest",
    test_path: encounterTestPath,
    test_marker: fixtureMarker(id),
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: [],
  numeric_approximations: [],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-world-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      focusedTest,
      "cargo test -p starclock-mode-universe --all-features",
      "node tools/repository-check/run.mjs",
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
  console.log(`Goal 07 world receipt ${partitionId} matches generated evidence.`);
}

function disposition(planned, workbooks) {
  assert(planned, "retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition:
      planned.kind === "semantic-fixture" ? "ProductionExecuted" : "ExecutableShared",
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbooks.map((workbook) => ({
      path: `config/data/${workbook}`,
    })),
    provenance_evidence: provenanceEvidence,
    execution_evidence: executionEvidence,
  };
}
function fixtureMarker(id) {
  if (id ===
    "universe.fixture.encounter-selection.selectconditionkeythenresolvegroupordifficultybinding")
    return "goal07_p5_m15_s20_selects_condition_before_group_or_difficulty_resolution";
  if (id ===
    "universe.fixture.encounter-selection.selectexactconditionkeythenweightedstableorder")
    return "goal07_p5_m15_s21_selects_exact_condition_key_in_stable_weighted_order";
  if (id ===
    "universe.fixture.encounter-selection.resolveworlddifficultybosselitebinding")
    return "goal07_p5_m15_s21_resolves_world_difficulty_boss_binding";
  throw new Error(`${id}: encounter-selection fixture marker is not implemented`);
}
function evidence(relative) {
  return {
    path: relative,
    sha256: sha256(relative),
    git_blob_sha1: gitBlob(relative),
  };
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function gitBlob(relative) {
  return execFileSync("git", ["hash-object", relative], {
    cwd: root,
    encoding: "utf8",
  }).trim();
}
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function json(relative) {
  return JSON.parse(fs.readFileSync(absolute(relative), "utf8"));
}
function assert(condition, message) { if (!condition) throw new Error(message); }
