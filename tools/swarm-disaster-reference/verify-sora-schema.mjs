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
const schemaRoot = path.join(
  root,
  "config",
  "swarm-disaster",
  "schema",
);
const schemaFiles = ["core.toml", "progression.toml"].map((name) =>
  path.join(schemaRoot, name));
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
  ["SwarmDisasterAudiencePath", "audience-paths.json"],
  ["SwarmDisasterAudienceDie", "audience-dice.json"],
  ["SwarmDisasterDiceFace", "dice-faces.json"],
  ["SwarmDisasterDiceRarity", "dice-rarities.json"],
  ["SwarmDisasterDiceTargetRule", "dice-target-rules.json"],
  ["SwarmDisasterDiceRollControl", "dice-roll-controls.json"],
  ["SwarmDisasterCommuningChoice", "communing-choices.json"],
  ["SwarmDisasterPathstriderCabinet", "pathstrider-cabinets.json"],
  ["SwarmDisasterCommuningDimension", "communing-dimensions.json"],
  ["SwarmDisasterCommuningPointAdjustment",
    "communing-point-adjustments.json"],
  ["SwarmDisasterCommuningTrailNode", "communing-trail-nodes.json"],
  ["SwarmDisasterCommuningTrailPrerequisite",
    "communing-trail-prerequisites.json"],
  ["SwarmDisasterCommuningTrailEffect", "communing-trail-effects.json"],
  ["SwarmDisasterPathstriderObjective", "pathstrider-objectives.json"],
  ["SwarmDisasterPathstriderFinishCondition",
    "pathstrider-finish-conditions.json"],
  ["SwarmDisasterPathstriderUnlock", "pathstrider-unlocks.json"],
  ["SwarmDisasterMechanicalChapterLocator",
    "mechanical-chapter-locators.json"],
  ["SwarmDisasterPath", "paths.json"],
  ["SwarmDisasterResonance", "resonances.json"],
  ["SwarmDisasterPathBoost", "path-boosts.json"],
  ["SwarmDisasterResonanceInterplay", "resonance-interplays.json"],
  ["SwarmDisasterTrailblazeBonus", "bonuses.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(fs.existsSync(project) && schemaFiles.every((file) =>
    fs.existsSync(file)),
    "isolated Swarm Disaster schema is missing");
  const projectText = fs.readFileSync(project, "utf8");
  for (const include of ["schema/core.toml", "schema/progression.toml"])
    assert(projectText.includes(include),
      `project lacks ${include}`);
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

  const before = new Map(schemaFiles.map((file) => [
    file,
    fs.readFileSync(file),
  ]));
  run(process.execPath, [
    "tools/swarm-disaster-reference/generate-sora-schema.mjs",
    root,
  ]);
  for (const file of schemaFiles)
    assert(before.get(file).equals(fs.readFileSync(file)),
      `${path.basename(file)} generation drifted`);
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
    ["SwarmDisasterAudiencePath", "audience_die_id",
      "SwarmDisasterAudienceDie"],
    ["SwarmDisasterAudienceDie", "audience_path_id",
      "SwarmDisasterAudiencePath"],
    ["SwarmDisasterDiceFace", "audience_die_id",
      "SwarmDisasterAudienceDie"],
    ["SwarmDisasterDiceFace", "rarity_id", "SwarmDisasterDiceRarity"],
    ["SwarmDisasterDiceFace", "target_rule_id",
      "SwarmDisasterDiceTargetRule"],
    ["SwarmDisasterPathstriderCabinet", "objective_id",
      "SwarmDisasterPathstriderObjective"],
    ["SwarmDisasterCommuningPointAdjustment", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterCommuningTrailNode", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterCommuningTrailPrerequisite", "node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterCommuningTrailPrerequisite", "required_node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterCommuningTrailEffect", "node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterPathstriderObjective", "cabinet_id",
      "SwarmDisasterPathstriderCabinet"],
    ["SwarmDisasterPathstriderObjective", "finish_condition_id",
      "SwarmDisasterPathstriderFinishCondition"],
    ["SwarmDisasterPathstriderUnlock", "finish_condition_id",
      "SwarmDisasterPathstriderFinishCondition"],
    ["SwarmDisasterMechanicalChapterLocator", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterPath", "audience_die_id", "SwarmDisasterAudienceDie"],
    ["SwarmDisasterPath", "resonance_id", "SwarmDisasterResonance"],
    ["SwarmDisasterResonance", "path_id", "SwarmDisasterPath"],
    ["SwarmDisasterPathBoost", "path_id", "SwarmDisasterPath"],
    ["SwarmDisasterResonanceInterplay", "main_path_id",
      "SwarmDisasterPath"],
    ["SwarmDisasterResonanceInterplay", "sub_path_id",
      "SwarmDisasterPath"],
  ]) {
    const field = tables.get(tableName).fields.find((candidate) =>
      candidate.name === fieldName);
    const type = field?.ty?.Optional ?? field?.ty;
    assert(type?.Ref?.table === target && type.Ref.field === "id",
      `${tableName}.${fieldName} is not ref<${target}.id>`);
  }
  console.log(
    `Swarm Disaster Sora schema verified (${tables.size} isolated core/` +
    "progression tables; typed local references; pinned Sora 0.3.0).",
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
