#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  process.argv[2]
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const policy = json("policy/sora-toolchain.json");
const sora = locateSora();
const project = path.join(root, "config", "unknowable-domain", "project.toml");
const schemaFile = path.join(
  root,
  "config",
  "unknowable-domain",
  "schema",
  "core.toml",
);
const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-unknowable-domain-schema-"),
);
const expected = new Map([
  ["UnknowableDomainProfile", "profiles.json"],
  ["UnknowableDomainAlignment", "alignments.json"],
  ["UnknowableDomainArea", "areas.json"],
  ["UnknowableDomainDifficultyComposition", "difficulty-compositions.json"],
  ["UnknowableDomainLayer", "layers.json"],
  ["UnknowableDomainLayerRoom", "layer-rooms.json"],
  ["UnknowableDomainRoom", "rooms.json"],
  ["UnknowableDomainStageFlow", "stage-flow.json"],
  ["UnknowableDomainFinishCondition", "finish-conditions.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(
    fs.existsSync(project) && fs.existsSync(schemaFile),
    "isolated Unknowable Domain schema is missing",
  );
  const projectText = fs.readFileSync(project, "utf8");
  assert(
    projectText.includes('includes = [\n  "schema/core.toml",\n]'),
    "P3-B1 project include set drift",
  );
  for (const forbidden of [
    "config/data",
    "config/generated",
    "config/universe-generated",
    "config/gold-and-gears",
    "config/gold-and-gears-generated",
    "config/swarm-disaster",
    "config/swarm-disaster-generated",
  ])
    assert(
      !projectText.includes(forbidden),
      `isolated project references forbidden output ${forbidden}`,
    );

  const before = fs.readFileSync(schemaFile);
  run(process.execPath, [
    "tools/unknowable-domain-reference/generate-sora-schema.mjs",
    root,
  ]);
  assert(
    before.equals(fs.readFileSync(schemaFile)),
    "core.toml generation drifted",
  );
  run(sora, ["--serial", "check", "--project", project]);
  const lock = path.join(temporary, "schema.lock");
  run(sora, [
    "--serial",
    "schema-lock",
    "--project",
    project,
    "--out",
    lock,
  ]);
  const parsed = JSON.parse(fs.readFileSync(lock, "utf8")).schema;
  assert(
    parsed.package === "starclock_unknowable_domain_reference_config",
    "schema package differs",
  );
  const tables = new Map(parsed.tables.map((table) => [table.name, table]));
  assert(
    tables.size === expected.size,
    `expected ${expected.size} core tables, found ${tables.size}`,
  );
  for (const [tableName, normalized] of expected) {
    assert(tables.has(tableName), `missing table ${tableName}`);
    assert(
      Array.isArray(json(
        `content-reference/unknowable-domain-v1/${normalized}`,
      )),
      `${normalized} has the wrong top-level shape`,
    );
    const stable = tables.get(tableName).fields.find((field) =>
      field.name === "stable_key");
    assert(
      stable?.ty === "String",
      `${tableName}.stable_key is not typed as string`,
    );
  }
  for (const [tableName, fieldName, target] of [
    ["UnknowableDomainArea", "default_alignment_id",
      "UnknowableDomainAlignment"],
    ["UnknowableDomainLayerRoom", "layer_id", "UnknowableDomainLayer"],
    ["UnknowableDomainStageFlow", "area_id", "UnknowableDomainArea"],
  ]) {
    const field = tables.get(tableName).fields.find((candidate) =>
      candidate.name === fieldName);
    const type = field?.ty?.Optional ?? field?.ty;
    assert(
      type?.Ref?.table === target && type.Ref.field === "id",
      `${tableName}.${fieldName} is not ref<${target}.id>`,
    );
  }
  const alignments = json(
    "content-reference/unknowable-domain-v1/alignments.json",
  );
  const areas = json("content-reference/unknowable-domain-v1/areas.json");
  const layers = json("content-reference/unknowable-domain-v1/layers.json");
  const layerRooms = json(
    "content-reference/unknowable-domain-v1/layer-rooms.json",
  );
  const stageFlow = json(
    "content-reference/unknowable-domain-v1/stage-flow.json",
  );
  const alignmentSourceIds = new Set(alignments.map(({ source_id: id }) => id));
  const areaIds = new Set(areas.map(({ id }) => id));
  const layerIds = new Set(layers.map(({ id }) => id));
  assert(
    areas.every((row) => alignmentSourceIds.has(row.default_alignment)),
    "Area Alignment normalized reference drift",
  );
  assert(
    layerRooms.every(({ layer_id: id }) => layerIds.has(id)),
    "LayerRoom Layer normalized reference drift",
  );
  assert(
    stageFlow.every(({ area_id: id }) => !id || areaIds.has(id)),
    "StageFlow Area normalized reference drift",
  );
  const digest = crypto.createHash("sha256")
    .update(fs.readFileSync(schemaFile))
    .digest("hex");
  console.log(
    `Unknowable Domain Sora core schema verified (${tables.size} isolated ` +
    `tables; typed Alignment/Area/Layer references; core ${digest}; ` +
    "pinned Sora 0.3.0).",
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function locateSora() {
  const executable = process.platform === "win32" ? "sora.exe" : "sora";
  const local = path.join(root, policy.install_root, "bin", executable);
  if (fs.existsSync(local)) return local;
  const worktrees = spawnSync(
    "git",
    ["worktree", "list", "--porcelain"],
    { cwd: root, encoding: "utf8" },
  );
  if (worktrees.status === 0)
    for (const line of worktrees.stdout.split(/\r?\n/u))
      if (line.startsWith("worktree ")) {
        const candidate = path.join(
          line.slice("worktree ".length),
          policy.install_root,
          "bin",
          executable,
        );
        if (fs.existsSync(candidate)) return candidate;
      }
  return local;
}

function run(command, arguments_) {
  const result = spawnSync(command, arguments_, {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0)
    throw new Error(
      `${command} ${arguments_.join(" ")} failed\n` +
      `${result.stdout}\n${result.stderr}`,
    );
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
