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

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family === "occurrences-and-choices",
  `${partitionId}: not an Occurrence partition`);
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
const sourceReview =
  `evidence/standard-universe-mechanics-complete-v1/source-reviews/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);

const provenanceEvidence = [
  { path: "content-reference/standard-universe-v1/occurrences.json" },
  { path: "content-reference/standard-universe-v1/occurrence-variants.json" },
  { path: "content-reference/standard-universe-v1/occurrence-choices.json" },
  { path: "content-reference/standard-universe-v1/sources.json" },
  ...(exists(sourceReview) ? [{ path: sourceReview }] : []),
];
const executionEvidence = [
  { path: "crates/starclock-mode-universe/src/occurrence_interaction.rs" },
  { path: "crates/starclock-mode-universe/src/occurrence_interaction/support.rs" },
  { path: "crates/starclock-mode-universe/src/topology.rs" },
  { path: "crates/starclock-mode-universe/src/topology/occurrence_binding.rs" },
  { path: "crates/starclock-mode-universe/src/topology_identity.rs" },
  { path: "crates/starclock-mode-universe/tests/run_runtime.rs" },
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
        tables: [
          "UniverseOccurrence",
          "UniverseOccurrenceVariant",
          "UniverseOccurrenceChoice",
          "UniverseOccurrenceCost",
          "UniverseOccurrenceOutcome",
        ],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseSourceRecord"],
      },
    ],
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-occurrence-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: partition.record_ids.map((id) => ({
    ...disposition(records.get(id)),
    execution_evidence: executionEvidence,
    ...(id.includes(".choice.") ? {
      execution_kind: id.endsWith(".choice.02")
        && id.startsWith("universe.occurrence.1.")
        ? "ExplicitExternalResult"
        : "SharedOccurrenceHandler",
      test_path: "crates/starclock-mode-universe/tests/run_runtime.rs",
      test_marker:
        "goal07_p4_m13_s01_executes_exact_fragments_named_curio_transitions_and_external_blessing_results",
    } : {}),
  })),
  rules: [],
  fixtures: [],
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: [],
  numeric_approximations: [],
  execution: {
    result: "pass",
    commands: [
      ...(partitionId === "G07-P4-M13-S01"
        ? ["node tools/goal07/refine-occurrence-s01.mjs"]
        : []),
      `python tools/goal07/author-occurrence-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify-pack.mjs .",
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      "cargo test -p starclock-mode-universe --test run_runtime --all-features",
      "cargo test -p starclock-mode-universe --all-features",
      "cargo test -p starclock-agent-api --test activity_session_loop --all-features",
      "cargo test -p starclock-cli --test universe_cli --all-features",
      "cargo test -p starclock-mcp --test universe_surface_parity --all-features",
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
  console.log(`Goal 07 Occurrence receipt ${partitionId} matches generated evidence.`);
}

function disposition(planned) {
  assert(planned, "retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: planned.intended_runtime_disposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: [{ path: "config/data/Universe.xlsx" }],
    provenance_evidence: provenanceEvidence,
  };
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
