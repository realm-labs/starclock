#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const outputPath = path.join(
  root,
  "evidence",
  "unknowable-domain-reference-v1",
  "reconciliation-checkpoints.json",
);
const categories = ["boss_choices", "curios", "occurrences"];
const definitions = [
  {
    goal: "Goal08",
    goal_id: "gold-and-gears-reference-v1",
    commit: "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    registration_commit: "2688624c34a564d87076cadb405c8da506efd373",
    manifest_path:
      "content-manifests/gold-and-gears-v1/content-manifest.json",
    manifest_sha256:
      "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
    source_manifest_records: 7913,
    source_transport: "LocalCommittedReleaseRegistration",
  },
  {
    goal: "Goal09",
    goal_id: "swarm-disaster-reference-v1",
    commit: "b8da6744a63cd92554b45f8e780d79a1be131f50",
    manifest_path:
      "content-manifests/swarm-disaster-v1/content-manifest.json",
    manifest_sha256:
      "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850",
    source_manifest_records: 6963,
    source_transport: "RemoteBranch",
    remote_ref: "origin/codex/goal09-swarm-disaster-reference",
  },
];

const checkpoints = write
  ? definitions.map(buildCheckpoint)
  : loadAndValidateCheckpoints();
const report = {
  schema_revision: "starclock.unknowable-domain-reconciliation-checkpoints.v1",
  goal_id: "unknowable-domain-reference-v1",
  frozen_at: "2026-07-29",
  purpose:
    "Compact ownership-only proof for Goal 08/09 source-path, row, digest and ownership reconciliation; no foreign gameplay data is imported.",
  result: "pass",
  checkpoints,
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(
    fs.readFileSync(outputPath, "utf8") === encoded,
    "checkpoint evidence drifted",
  );
}
console.log(
  `Unknowable Domain reconciliation checkpoints verified ` +
    `(${checkpoints.map(({ goal, reconciliation_record_count: count }) =>
      `${goal}=${count}`).join("; ")} ownership-only records).`,
);

function buildCheckpoint(definition) {
  const bytes = gitBlob(definition.commit, definition.manifest_path);
  assert(
    sha256(bytes) === definition.manifest_sha256,
    `${definition.goal}: manifest digest drift`,
  );
  const manifest = JSON.parse(bytes);
  assert(
    manifest.counts.records === definition.source_manifest_records,
    `${definition.goal}: source manifest denominator drift`,
  );
  const records = categories
    .flatMap((categoryId) =>
      manifest.categories[categoryId].records.map((record) => ({
        category_id: categoryId,
        id: String(record.id),
        source: record.source,
        evidence_sha256: record.evidence_sha256,
        ownership: record.ownership,
      })),
    )
    .sort(
      (left, right) =>
        left.category_id.localeCompare(right.category_id) ||
        left.source.localeCompare(right.source) ||
        left.id.localeCompare(right.id),
    );
  return {
    goal: definition.goal,
    goal_id: definition.goal_id,
    commit: definition.commit,
    ...(definition.registration_commit
      ? { registration_commit: definition.registration_commit }
      : {}),
    manifest_path: definition.manifest_path,
    manifest_sha256: definition.manifest_sha256,
    source_manifest_records: definition.source_manifest_records,
    source_transport: definition.source_transport,
    ...(definition.remote_ref ? { remote_ref: definition.remote_ref } : {}),
    frozen_categories: categories,
    reconciliation_record_count: records.length,
    reconciliation_records_sha256: sha256(canonical(records)),
    reconciliation_records: records,
  };
}

function loadAndValidateCheckpoints() {
  assert(fs.existsSync(outputPath), "checkpoint evidence is missing; run with --write");
  const stored = JSON.parse(fs.readFileSync(outputPath, "utf8"));
  assert(
    stored.schema_revision ===
      "starclock.unknowable-domain-reconciliation-checkpoints.v1" &&
      stored.goal_id === "unknowable-domain-reference-v1" &&
      stored.result === "pass",
    "checkpoint evidence envelope drifted",
  );
  assert(
    stored.checkpoints.length === definitions.length,
    "checkpoint evidence denominator drifted",
  );
  for (const definition of definitions) {
    const checkpoint = stored.checkpoints.find(
      ({ goal }) => goal === definition.goal,
    );
    assert(checkpoint, `${definition.goal}: checkpoint evidence missing`);
    for (const field of [
      "goal_id",
      "commit",
      "manifest_path",
      "manifest_sha256",
      "source_manifest_records",
      "source_transport",
    ]) {
      assert(
        checkpoint[field] === definition[field],
        `${definition.goal}: checkpoint ${field} drifted`,
      );
    }
    assert(
      checkpoint.reconciliation_record_count ===
        checkpoint.reconciliation_records.length &&
        checkpoint.reconciliation_records_sha256 ===
          sha256(canonical(checkpoint.reconciliation_records)),
      `${definition.goal}: compact record proof drifted`,
    );
    if (gitObjectExists(definition.commit)) {
      assert(
        canonical(checkpoint) === canonical(buildCheckpoint(definition)),
        `${definition.goal}: checkpoint differs from available Git object`,
      );
    }
  }
  return stored.checkpoints;
}

function gitBlob(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function gitObjectExists(commit) {
  try {
    execFileSync("git", ["cat-file", "-e", `${commit}^{commit}`], {
      cwd: root,
      stdio: "ignore",
    });
    return true;
  } catch {
    return false;
  }
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
