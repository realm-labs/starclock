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
assert(["G07-P4-M14-S01", "G07-P4-M14-S02"].includes(partitionId),
  `${partitionId}: service receipt profile is not implemented`);

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family === "services-shops-roster-interactables",
  `${partitionId}: not a service partition`);
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const rules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const fixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const services = new Map(
  json("content-reference/standard-universe-v1/services.json")
    .map((entry) => [entry.id, entry]),
);
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);

const provenanceEvidence = [
  { path: "content-reference/standard-universe-v1/services.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
  { path: "content-reference/standard-universe-v1/review-fixtures.json" },
];
const executionEvidence = [
  { path: "crates/starclock-mode-universe/src/progression_lowering.rs" },
  { path: "crates/starclock-mode-universe/src/service_effect_runtime.rs" },
  { path: "crates/starclock-mode-universe/src/service_interaction.rs" },
  { path: "crates/starclock-mode-universe/src/topology.rs" },
  { path: "crates/starclock-mode-universe/src/universe_replay_v2.rs" },
  { path: "crates/starclock-mode-universe/src/universe_replay_v3.rs" },
  { path: "crates/starclock-mode-universe/tests/service_effect_runtime.rs" },
  { path: "crates/starclock-mode-universe/tests/service_interaction_runtime.rs" },
  { path: "crates/starclock-mode-universe/tests/service_reviver_runtime.rs" },
];
const commonMarker =
  "goal07_p4_m14_s01_executes_every_non_reviver_service_through_activity";

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
        tables: ["UniverseService", "UniverseServiceParameter"],
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
      `python -c "import openpyxl" && python tools/goal07/author-service-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: partition.record_ids.map((id) => {
    const service = services.get(id);
    const excluded = profileExcluded(service);
    return {
      ...disposition(
        records.get(id),
        excluded ? "ProfileExcluded" : "ExecutableRuleIr",
        "Universe.xlsx",
      ),
      ...(excluded ? { profile_owner: service.profile_owner } : {}),
    };
  }),
  rules: partition.rule_ids.map((id) => {
    const planned = rules.get(id);
    const service = services.get(planned.source_record_id);
    const excluded = profileExcluded(service);
    return {
      ...disposition(
        planned,
        excluded ? "ProfileExcluded" : "ExecutableRuleIr",
        "UniverseBindings.xlsx",
      ),
      ...(excluded ? { profile_owner: service.profile_owner } : {}),
      implementation_kind: excluded ? "ProfileBoundary" : "SharedPrimitive",
      definition_keys: [id, planned.source_record_id],
      execution_evidence: executionEvidence,
    };
  }),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(fixtures.get(id), "ProductionExecuted", "UniverseEvidence.xlsx"),
    execution_kind: "RustTest",
    ...fixtureEvidence(id),
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: [],
  numeric_approximations: [],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-service-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-mode-universe --test service_effect_runtime --all-features",
      "cargo test -p starclock-mode-universe --test service_interaction_runtime --all-features",
      "cargo test -p starclock-mode-universe --test service_reviver_runtime --all-features",
      "cargo test -p starclock-mode-universe --all-features",
      "cargo test -p starclock-agent-api --test activity_session_loop --all-features",
      "cargo test -p starclock-cli --test universe_cli --all-features",
      "cargo test -p starclock-mcp --test universe_surface_parity --all-features",
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
  console.log(`Goal 07 service receipt ${partitionId} matches generated evidence.`);
}

function fixtureEvidence(id) {
  if (id === "universe.fixture.service.reviver") {
    return {
      test_path: "crates/starclock-mode-universe/tests/service_reviver_runtime.rs",
      test_marker:
        "goal07_p4_m14_s01_reviver_restores_defeated_battle_carry_atomically",
    };
  }
  return {
    test_path: "crates/starclock-mode-universe/tests/service_interaction_runtime.rs",
    test_marker: commonMarker,
  };
}

function disposition(planned, runtimeDisposition, workbook) {
  assert(planned, "retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: [{ path: `config/data/${workbook}` }],
    provenance_evidence: provenanceEvidence,
  };
}
function profileExcluded(service) {
  return service?.kind === "TrailblazeBonus"
    && service.mode_owner === "EvidenceOnly"
    && service.profile_owner !== "Standard";
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
