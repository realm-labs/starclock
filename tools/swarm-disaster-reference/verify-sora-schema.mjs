#!/usr/bin/env node

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
const project = path.join(root, "config", "swarm-disaster", "project.toml");
const schemaFile = path.join(
  root,
  "config",
  "swarm-disaster",
  "schema",
  "core.toml",
);
const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-swarm-disaster-schema-"),
);
const expected = new Map([
  ["SwarmDisasterProfile", "profiles.json"],
  ["SwarmDisasterArea", "areas.json"],
  ["SwarmDisasterDifficultySegment", "difficulty-segments.json"],
  ["SwarmDisasterPlane", "planes.json"],
  ["SwarmDisasterChessboard", "chessboards.json"],
  ["SwarmDisasterMapColumn", "map-columns.json"],
  ["SwarmDisasterMapNode", "map-nodes.json"],
  ["SwarmDisasterMapEdge", "map-edges.json"],
  ["SwarmDisasterMapEvent", "map-events.json"],
  ["SwarmDisasterBlockCreateRule", "block-create-rules.json"],
  ["SwarmDisasterRoom", "rooms.json"],
  ["SwarmDisasterDomain", "domains.json"],
  ["SwarmDisasterBeacon", "beacons.json"],
  ["SwarmDisasterBossChoice", "boss-choices.json"],
  ["SwarmDisasterTopologyConsequence", "topology-consequences.json"],
  ["SwarmDisasterCountdownAndDisarray", "countdown-and-disarray.json"],
  ["SwarmDisasterBossDecayLevel", "boss-decay-levels.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(fs.existsSync(project) && fs.existsSync(schemaFile),
    "isolated Swarm Disaster schema is missing");
  const projectText = fs.readFileSync(project, "utf8");
  assert(projectText.includes('includes = [\n  "schema/core.toml",\n]'),
    "P3-B1 project include set drift");
  for (const forbidden of [
    "config/data",
    "config/generated",
    "config/universe-generated",
    "config/gold-and-gears",
    "config/gold-and-gears-generated",
    "config/unknowable-domain",
    "config/unknowable-domain-generated",
  ])
    assert(!projectText.includes(forbidden),
      `isolated project references forbidden output ${forbidden}`);

  const before = fs.readFileSync(schemaFile);
  run(process.execPath, [
    "tools/swarm-disaster-reference/generate-sora-schema.mjs",
    root,
  ]);
  assert(before.equals(fs.readFileSync(schemaFile)),
    "core.toml generation drifted");
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
  assert(parsed.package === "starclock_swarm_disaster_reference_config",
    "schema package differs");
  const tables = new Map(parsed.tables.map((table) => [table.name, table]));
  assert(tables.size === expected.size,
    `expected ${expected.size} core tables, found ${tables.size}`);
  for (const [tableName, normalized] of expected) {
    assert(tables.has(tableName), `missing table ${tableName}`);
    assert(Array.isArray(json(
      `content-reference/swarm-disaster-v1/${normalized}`,
    )), `${normalized} has the wrong top-level shape`);
    const stable = tables.get(tableName).fields.find((field) =>
      field.name === "stable_key");
    assert(stable?.ty === "String",
      `${tableName}.stable_key is not typed as string`);
  }
  for (const [tableName, fieldName, target] of [
    ["SwarmDisasterChessboard", "start_node_id", "SwarmDisasterMapNode"],
    ["SwarmDisasterChessboard", "end_node_id", "SwarmDisasterMapNode"],
    ["SwarmDisasterMapColumn", "chessboard_id", "SwarmDisasterChessboard"],
    ["SwarmDisasterMapNode", "chessboard_id", "SwarmDisasterChessboard"],
    ["SwarmDisasterMapNode", "column_id", "SwarmDisasterMapColumn"],
    ["SwarmDisasterMapEdge", "chessboard_id", "SwarmDisasterChessboard"],
    ["SwarmDisasterMapEdge", "from_node_id", "SwarmDisasterMapNode"],
    ["SwarmDisasterMapEdge", "to_node_id", "SwarmDisasterMapNode"],
    ["SwarmDisasterMapEvent", "chessboard_id", "SwarmDisasterChessboard"],
    ["SwarmDisasterBlockCreateRule", "chessboard_id",
      "SwarmDisasterChessboard"],
    ["SwarmDisasterBlockCreateRule", "domain_id", "SwarmDisasterDomain"],
    ["SwarmDisasterRoom", "domain_id", "SwarmDisasterDomain"],
  ]) {
    const field = tables.get(tableName).fields.find((candidate) =>
      candidate.name === fieldName);
    const type = field?.ty?.Optional ?? field?.ty;
    assert(type?.Ref?.table === target && type.Ref.field === "id",
      `${tableName}.${fieldName} is not ref<${target}.id>`);
  }
  console.log(
    `Swarm Disaster Sora core schema verified (${tables.size} isolated ` +
    "tables; typed topology/domain references; pinned Sora 0.3.0).",
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
