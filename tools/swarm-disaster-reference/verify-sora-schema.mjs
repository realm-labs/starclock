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
const schemaFiles = [
  "core.toml",
  "progression.toml",
  "content.toml",
  "evidence.toml",
].map((name) => path.join(schemaRoot, name));
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
  ["SwarmDisasterCountdownDisarray", "countdown-and-disarray.json"],
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
  ["SwarmDisasterPointAdjustment",
    "communing-point-adjustments.json"],
  ["SwarmDisasterCommuningTrailNode", "communing-trail-nodes.json"],
  ["SwarmDisasterTrailPrerequisite",
    "communing-trail-prerequisites.json"],
  ["SwarmDisasterTrailEffect", "communing-trail-effects.json"],
  ["SwarmDisasterPathObjective", "pathstrider-objectives.json"],
  ["SwarmDisasterPathstriderFinish",
    "pathstrider-finish-conditions.json"],
  ["SwarmDisasterPathstriderUnlock", "pathstrider-unlocks.json"],
  ["SwarmDisasterMechanicalChapter",
    "mechanical-chapter-locators.json"],
  ["SwarmDisasterPath", "paths.json"],
  ["SwarmDisasterResonance", "resonances.json"],
  ["SwarmDisasterPathBoost", "path-boosts.json"],
  ["SwarmDisasterResonanceInterplay", "resonance-interplays.json"],
  ["SwarmDisasterTrailblazeBonus", "bonuses.json"],
  ["SwarmDisasterBlessing", "blessings.json"],
  ["SwarmDisasterBlessingLevel", "blessing-levels.json"],
  ["SwarmDisasterPoolMembership", "pool-membership.json"],
  ["SwarmDisasterCurio", "curios.json"],
  ["SwarmDisasterCurioState", "curio-states.json"],
  ["SwarmDisasterCurioRule", "curio-rules.json"],
  ["SwarmDisasterOccurrence", "occurrences.json"],
  ["SwarmDisasterOccurrenceVariant", "occurrence-variants.json"],
  ["SwarmDisasterOccurrenceChoice", "occurrence-choices.json"],
  ["SwarmDisasterService", "services.json"],
  ["SwarmDisasterAdventureOutcome", "adventure-outcomes.json"],
  ["SwarmDisasterCurrency", "currencies.json"],
  ["SwarmDisasterServiceRule", "service-rules.json"],
  ["SwarmDisasterEncounterGroup", "encounter-groups.json"],
  ["SwarmDisasterEncounterWave", "encounter-waves.json"],
  ["SwarmDisasterEnemySlot", "enemy-slots.json"],
  ["SwarmDisasterBossPool", "boss-pools.json"],
  ["SwarmDisasterMechanicRule", "mechanic-rules.json"],
  ["SwarmDisasterSourceRecord", "sources.json"],
  ["SwarmDisasterCoverage", "coverage.json"],
  ["SwarmDisasterResearchGap", "research-gaps.json"],
  ["SwarmDisasterReviewFixture", "review-fixtures.json"],
  ["SwarmDisasterReconcileReceipt", "reconciliation-receipts.json"],
  ["SwarmDisasterManifest", "manifest.json"],
  ["SwarmDisasterPackIndex", "pack-index.json"],
]);
const childTables = new Set([
  "SwarmDisasterResearchGapAffected",
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(fs.existsSync(project) && schemaFiles.every((file) =>
    fs.existsSync(file)),
    "isolated Swarm Disaster schema is missing");
  const projectText = fs.readFileSync(project, "utf8");
  for (const include of [
    "schema/core.toml",
    "schema/progression.toml",
    "schema/content.toml",
    "schema/evidence.toml",
  ])
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
  assert(tables.size === expected.size + childTables.size,
    `expected ${expected.size} primary and ${childTables.size} child ` +
    `tables, found ${tables.size}`);
  for (const [tableName, normalized] of expected) {
    assert(tables.has(tableName), `missing table ${tableName}`);
    const normalizedValue = json(
      `content-reference/swarm-disaster-v1/${normalized}`,
    );
    assert(
      Array.isArray(normalizedValue) || normalized === "manifest.json",
      `${normalized} has the wrong top-level shape`,
    );
    const stable = tables.get(tableName).fields.find((field) =>
      field.name === "stable_key");
    assert(stable?.ty === "String",
      `${tableName}.stable_key is not typed as string`);
  }
  for (const tableName of childTables)
    assert(tables.has(tableName), `missing child table ${tableName}`);
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
    ["SwarmDisasterPointAdjustment", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterCommuningTrailNode", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterTrailPrerequisite", "node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterTrailPrerequisite", "required_node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterTrailEffect", "node_id",
      "SwarmDisasterCommuningTrailNode"],
    ["SwarmDisasterPathObjective", "cabinet_id",
      "SwarmDisasterPathstriderCabinet"],
    ["SwarmDisasterPathstriderUnlock", "finish_condition_id",
      "SwarmDisasterPathstriderFinish"],
    ["SwarmDisasterMechanicalChapter", "dimension_id",
      "SwarmDisasterCommuningDimension"],
    ["SwarmDisasterPath", "audience_die_id", "SwarmDisasterAudienceDie"],
    ["SwarmDisasterPath", "resonance_id", "SwarmDisasterResonance"],
    ["SwarmDisasterResonance", "path_id", "SwarmDisasterPath"],
    ["SwarmDisasterPathBoost", "path_id", "SwarmDisasterPath"],
    ["SwarmDisasterResonanceInterplay", "main_path_id",
      "SwarmDisasterPath"],
    ["SwarmDisasterResonanceInterplay", "sub_path_id",
      "SwarmDisasterPath"],
    ["SwarmDisasterBlessingLevel", "blessing_id",
      "SwarmDisasterBlessing"],
    ["SwarmDisasterCurio", "initial_state_id", "SwarmDisasterCurioState"],
    ["SwarmDisasterCurioState", "curio_id", "SwarmDisasterCurio"],
    ["SwarmDisasterCurioRule", "curio_id", "SwarmDisasterCurio"],
    ["SwarmDisasterCurioRule", "state_id", "SwarmDisasterCurioState"],
    ["SwarmDisasterOccurrenceChoice", "variant_id",
      "SwarmDisasterOccurrenceVariant"],
    ["SwarmDisasterEncounterWave", "encounter_group_id",
      "SwarmDisasterEncounterGroup"],
    ["SwarmDisasterEnemySlot", "wave_id", "SwarmDisasterEncounterWave"],
    ["SwarmDisasterBossPool", "area_id", "SwarmDisasterArea"],
    ["SwarmDisasterResearchGapAffected", "research_gap_id",
      "SwarmDisasterResearchGap"],
  ]) {
    const field = tables.get(tableName).fields.find((candidate) =>
      candidate.name === fieldName);
    const type = field?.ty?.Optional ?? field?.ty;
    assert(type?.Ref?.table === target && type.Ref.field === "id",
      `${tableName}.${fieldName} is not ref<${target}.id>`);
  }
  const committed = path.join(root, "config", "swarm-disaster-generated");
  assert(fs.existsSync(path.join(committed, "schema.lock")),
    "committed schema lock is missing");
  const directTemplates = path.join(temporary, "templates");
  const directRust = path.join(temporary, "rust");
  run(sora, [
    "--serial",
    "excel-template",
    "--project",
    project,
    "--out",
    directTemplates,
  ]);
  run(sora, [
    "--serial",
    "gen",
    "--target",
    "rust",
    "--project",
    project,
    "--out",
    directRust,
    "--format-code",
    "never",
  ]);
  formatRust(directRust);
  assert(
    fs.readFileSync(lock).equals(
      fs.readFileSync(path.join(committed, "schema.lock")),
    ),
    "committed schema lock drifted",
  );
  for (const workbook of [
    "SwarmDisaster.xlsx",
    "SwarmDisasterProgression.xlsx",
    "SwarmDisasterContent.xlsx",
    "SwarmDisasterEvidence.xlsx",
  ]) {
    assert(fs.statSync(path.join(directTemplates, workbook)).size > 1000,
      `${workbook} direct template is missing`);
    assert(fs.statSync(path.join(committed, "templates", workbook)).size > 1000,
      `${workbook} committed template is missing`);
  }
  const committedRust = path.join(committed, "rust");
  const directRustFiles = fs.readdirSync(directRust)
    .filter((name) => name.endsWith(".rs"))
    .sort();
  const committedRustFiles = fs.readdirSync(committedRust)
    .filter((name) => name.endsWith(".rs"))
    .sort();
  assert(
    JSON.stringify(committedRustFiles) === JSON.stringify(directRustFiles),
    "committed generated reader file set drifted",
  );
  for (const file of directRustFiles)
    assert(
      fs.readFileSync(path.join(directRust, file)).equals(
        fs.readFileSync(path.join(committedRust, file)),
      ),
      `${file} generated reader drifted`,
    );
  console.log(
    `Swarm Disaster Sora schema verified (${tables.size} isolated tables, ` +
    "four templates, generated lock/readers stable).",
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

function formatRust(directory) {
  const files = fs.readdirSync(directory)
    .filter((name) => name.endsWith(".rs"))
    .map((name) => path.join(directory, name));
  const result = spawnSync(
    "rustfmt",
    ["--edition", "2024", ...files],
    { cwd: root, encoding: "utf8" },
  );
  if (result.status !== 0)
    throw new Error(
      `rustfmt failed\n${result.stdout}\n${result.stderr}`,
    );
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
