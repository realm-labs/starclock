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
const schemaRoot = path.join(
  root,
  "config",
  "unknowable-domain",
  "schema",
);
const schemaFiles = ["core.toml", "systems.toml"].map((name) =>
  path.join(schemaRoot, name));
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
  ["UnknowableDomainScepter", "scepters.json"],
  ["UnknowableDomainScepterLevel", "scepter-levels.json"],
  ["UnknowableDomainScepterActivationRule", "scepter-activation-rules.json"],
  ["UnknowableDomainScepterStateTransition",
    "scepter-state-transitions.json"],
  ["UnknowableDomainComponent", "components.json"],
  ["UnknowableDomainComponentLevel", "component-levels.json"],
  ["UnknowableDomainComponentSlotCompatibility",
    "component-slot-compatibility.json"],
  ["UnknowableDomainSlotLayout", "slot-layouts.json"],
  ["UnknowableDomainLoadout", "loadouts.json"],
  ["UnknowableDomainLoadoutTransitionRule",
    "loadout-transition-rules.json"],
  ["UnknowableDomainDecisionComponent", "decision-components.json"],
  ["UnknowableDomainComponentChoiceProgram",
    "component-choice-programs.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(
    fs.existsSync(project) && schemaFiles.every((file) => fs.existsSync(file)),
    "isolated Unknowable Domain schema is missing",
  );
  const projectText = fs.readFileSync(project, "utf8");
  for (const include of ["schema/core.toml", "schema/systems.toml"])
    assert(projectText.includes(include), `project lacks ${include}`);
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

  const before = new Map(schemaFiles.map((file) => [
    file,
    fs.readFileSync(file),
  ]));
  run(process.execPath, [
    "tools/unknowable-domain-reference/generate-sora-schema.mjs",
    root,
  ]);
  for (const file of schemaFiles)
    assert(
      before.get(file).equals(fs.readFileSync(file)),
      `${path.basename(file)} generation drifted`,
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
    ["UnknowableDomainScepter", "alignment_id",
      "UnknowableDomainAlignment"],
    ["UnknowableDomainScepterLevel", "scepter_id",
      "UnknowableDomainScepter"],
    ["UnknowableDomainScepterLevel", "slot_layout_id",
      "UnknowableDomainSlotLayout"],
    ["UnknowableDomainScepterActivationRule", "scepter_id",
      "UnknowableDomainScepter"],
    ["UnknowableDomainScepterActivationRule", "scepter_level_id",
      "UnknowableDomainScepterLevel"],
    ["UnknowableDomainScepterStateTransition", "scepter_id",
      "UnknowableDomainScepter"],
    ["UnknowableDomainScepterStateTransition", "scepter_level_id",
      "UnknowableDomainScepterLevel"],
    ["UnknowableDomainScepterStateTransition", "activation_rule_id",
      "UnknowableDomainScepterActivationRule"],
    ["UnknowableDomainComponentLevel", "component_id",
      "UnknowableDomainComponent"],
    ["UnknowableDomainComponentSlotCompatibility", "component_id",
      "UnknowableDomainComponent"],
    ["UnknowableDomainComponentSlotCompatibility", "component_level_id",
      "UnknowableDomainComponentLevel"],
    ["UnknowableDomainLoadout", "scepter_id",
      "UnknowableDomainScepter"],
    ["UnknowableDomainLoadout", "scepter_level_id",
      "UnknowableDomainScepterLevel"],
    ["UnknowableDomainLoadout", "slot_layout_id",
      "UnknowableDomainSlotLayout"],
    ["UnknowableDomainDecisionComponent", "component_id",
      "UnknowableDomainComponent"],
    ["UnknowableDomainDecisionComponent", "effect_program_id",
      "UnknowableDomainComponentLevel"],
    ["UnknowableDomainComponentChoiceProgram", "decision_component_id",
      "UnknowableDomainDecisionComponent"],
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
  const alignmentIds = new Set(alignments.map(({ id }) => id));
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
  const scepters = json(
    "content-reference/unknowable-domain-v1/scepters.json",
  );
  const scepterLevels = json(
    "content-reference/unknowable-domain-v1/scepter-levels.json",
  );
  const activationRules = json(
    "content-reference/unknowable-domain-v1/scepter-activation-rules.json",
  );
  const transitions = json(
    "content-reference/unknowable-domain-v1/scepter-state-transitions.json",
  );
  const components = json(
    "content-reference/unknowable-domain-v1/components.json",
  );
  const componentLevels = json(
    "content-reference/unknowable-domain-v1/component-levels.json",
  );
  const compatibility = json(
    "content-reference/unknowable-domain-v1/" +
      "component-slot-compatibility.json",
  );
  const layouts = json(
    "content-reference/unknowable-domain-v1/slot-layouts.json",
  );
  const loadouts = json(
    "content-reference/unknowable-domain-v1/loadouts.json",
  );
  const decisions = json(
    "content-reference/unknowable-domain-v1/decision-components.json",
  );
  const choices = json(
    "content-reference/unknowable-domain-v1/component-choice-programs.json",
  );
  const scepterIds = new Set(scepters.map(({ id }) => id));
  const scepterLevelIds = new Set(scepterLevels.map(({ id }) => id));
  const activationIds = new Set(activationRules.map(({ id }) => id));
  const componentIds = new Set(components.map(({ id }) => id));
  const componentLevelIds = new Set(componentLevels.map(({ id }) => id));
  const layoutIds = new Set(layouts.map(({ id }) => id));
  const decisionIds = new Set(decisions.map(({ id }) => id));
  assert(
    scepters.every(({ alignment_id: id }) => alignmentIds.has(id)),
    "Scepter Alignment normalized reference drift",
  );
  assert(
    scepterLevels.every((row) =>
      scepterIds.has(row.scepter_id) && layoutIds.has(row.slot_layout_id)),
    "ScepterLevel parent/layout normalized reference drift",
  );
  assert(
    activationRules.every((row) =>
      scepterIds.has(row.scepter_id)
        && scepterLevelIds.has(row.scepter_level_id)),
    "ScepterActivationRule normalized reference drift",
  );
  assert(
    transitions.every((row) =>
      scepterIds.has(row.scepter_id)
        && scepterLevelIds.has(row.scepter_level_id)
        && activationIds.has(row.activation_rule_id)),
    "ScepterStateTransition normalized reference drift",
  );
  assert(
    componentLevels.every(({ component_id: id }) => componentIds.has(id))
      && compatibility.every((row) =>
        componentIds.has(row.component_id)
          && componentLevelIds.has(row.component_level_id)),
    "Component level/compatibility normalized reference drift",
  );
  assert(
    loadouts.every((row) =>
      scepterIds.has(row.scepter_id)
        && scepterLevelIds.has(row.scepter_level_id)
        && layoutIds.has(row.slot_layout_id)),
    "Loadout normalized reference drift",
  );
  assert(
    decisions.every((row) =>
      componentIds.has(row.component_id)
        && componentLevelIds.has(row.effect_program_id))
      && choices.every(({ decision_component_id: id }) =>
        decisionIds.has(id)),
    "Decision Component normalized reference drift",
  );
  const digest = crypto.createHash("sha256")
    .update(schemaFiles.map((file) => fs.readFileSync(file)).join("\n"))
    .digest("hex");
  console.log(
    `Unknowable Domain Sora schema verified (${tables.size} isolated core/` +
    `system tables; typed local references; schemas ${digest}; ` +
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
