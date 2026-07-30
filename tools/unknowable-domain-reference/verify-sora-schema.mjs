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
const schemaFiles = [
  "core.toml",
  "systems.toml",
  "progression.toml",
  "mechanics.toml",
  "content.toml",
  "evidence.toml",
].map((name) =>
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
  ["UnknowableDomainSynthesisRule", "synthesis-rules.json"],
  ["UnknowableDomainUpgradeRule", "upgrade-rules.json"],
  ["UnknowableDomainReforgeRule", "reforge-rules.json"],
  ["UnknowableDomainWorkbench", "workbenches.json"],
  ["UnknowableDomainWorkbenchFunction", "workbench-functions.json"],
  ["UnknowableDomainGambleGroup", "gamble-groups.json"],
  ["UnknowableDomainGambleUnit", "gamble-units.json"],
  ["UnknowableDomainServiceOfferRule", "service-offer-rules.json"],
  ["UnknowableDomainModeConstant", "mode-constants.json"],
  ["UnknowableDomainTalent", "talents.json"],
  ["UnknowableDomainUnlock", "unlocks.json"],
  ["UnknowableDomainLayerEffect", "layer-effects.json"],
  ["UnknowableDomainMazeBuff", "maze-buffs.json"],
  ["UnknowableDomainScoreInput", "score-inputs.json"],
  ["UnknowableDomainProgressionEffect", "progression-effects.json"],
  ["UnknowableDomainMechanicSourceFile", "mechanic-source-files.json"],
  ["UnknowableDomainMechanicRule", "mechanic-rules.json"],
  ["UnknowableDomainBlessing", "blessings.json"],
  ["UnknowableDomainPoolMembership", "pool-membership.json"],
  ["UnknowableDomainCurio", "curios.json"],
  ["UnknowableDomainCurioState", "curio-states.json"],
  ["UnknowableDomainCurioGroup", "curio-groups.json"],
  ["UnknowableDomainCurioRule", "curio-rules.json"],
  ["UnknowableDomainOccurrence", "occurrences.json"],
  ["UnknowableDomainOccurrenceVariant", "occurrence-variants.json"],
  ["UnknowableDomainOccurrenceChoice", "occurrence-choices.json"],
  ["UnknowableDomainModeServiceNpc", "mode-service-npcs.json"],
  ["UnknowableDomainAdventureOutcome", "adventure-outcomes.json"],
  ["UnknowableDomainCurrency", "currencies.json"],
  ["UnknowableDomainServiceRule", "service-rules.json"],
  ["UnknowableDomainBossChoice", "boss-choices.json"],
  ["UnknowableDomainEncounterSourceObligation",
    "encounter-source-obligations.json"],
  ["UnknowableDomainEncounterGroup", "encounter-groups.json"],
  ["UnknowableDomainEncounterWave", "encounter-waves.json"],
  ["UnknowableDomainEnemySlot", "enemy-slots.json"],
  ["UnknowableDomainBossPool", "boss-pools.json"],
  ["UnknowableDomainSourceEvidence", "sources.json"],
  ["UnknowableDomainCoverage", "coverage.json"],
  ["UnknowableDomainResearchGap", "research-gaps.json"],
  ["UnknowableDomainSemanticFixtureFamily",
    "semantic-fixture-families.json"],
  ["UnknowableDomainReviewFixture", "review-fixtures.json"],
  ["UnknowableDomainReconciliationReceipt",
    "reconciliation-receipts.json"],
  ["UnknowableDomainManifest", "manifest.json"],
  ["UnknowableDomainPackIndex", "pack-index.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(
    fs.existsSync(project) && schemaFiles.every((file) => fs.existsSync(file)),
    "isolated Unknowable Domain schema is missing",
  );
  const projectText = fs.readFileSync(project, "utf8");
  for (const include of [
    "schema/core.toml",
    "schema/systems.toml",
    "schema/progression.toml",
    "schema/mechanics.toml",
    "schema/content.toml",
    "schema/evidence.toml",
  ])
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
    ["UnknowableDomainWorkbenchFunction", "offer_policy_id",
      "UnknowableDomainServiceOfferRule"],
    ["UnknowableDomainGambleGroup", "offer_policy_id",
      "UnknowableDomainServiceOfferRule"],
    ["UnknowableDomainUnlock", "finish_condition_id",
      "UnknowableDomainFinishCondition"],
    ["UnknowableDomainMechanicRule", "source_file_id",
      "UnknowableDomainMechanicSourceFile"],
    ["UnknowableDomainCurioState", "curio_id",
      "UnknowableDomainCurio"],
    ["UnknowableDomainCurioRule", "curio_id",
      "UnknowableDomainCurio"],
    ["UnknowableDomainCurioRule", "curio_group_id",
      "UnknowableDomainCurioGroup"],
    ["UnknowableDomainOccurrenceVariant", "occurrence_id",
      "UnknowableDomainOccurrence"],
    ["UnknowableDomainOccurrenceChoice", "variant_id",
      "UnknowableDomainOccurrenceVariant"],
    ["UnknowableDomainEncounterWave", "encounter_group_id",
      "UnknowableDomainEncounterGroup"],
    ["UnknowableDomainEnemySlot", "wave_id",
      "UnknowableDomainEncounterWave"],
    ["UnknowableDomainBossPool", "area_id",
      "UnknowableDomainArea"],
    ["UnknowableDomainReviewFixture", "family_id",
      "UnknowableDomainSemanticFixtureFamily"],
    ["UnknowableDomainManifest", "profile_id",
      "UnknowableDomainProfile"],
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
  const offerRules = json(
    "content-reference/unknowable-domain-v1/service-offer-rules.json",
  );
  const workbenchFunctions = json(
    "content-reference/unknowable-domain-v1/workbench-functions.json",
  );
  const gambleGroups = json(
    "content-reference/unknowable-domain-v1/gamble-groups.json",
  );
  const unlocks = json(
    "content-reference/unknowable-domain-v1/unlocks.json",
  );
  const finishConditions = json(
    "content-reference/unknowable-domain-v1/finish-conditions.json",
  );
  const mechanicSources = json(
    "content-reference/unknowable-domain-v1/mechanic-source-files.json",
  );
  const mechanicRules = json(
    "content-reference/unknowable-domain-v1/mechanic-rules.json",
  );
  const offerIds = new Set(offerRules.map(({ id }) => id));
  const finishIds = new Set(finishConditions.map(({ id }) => id));
  const mechanicSourceIds = new Set(mechanicSources.map(({ id }) => id));
  assert(
    workbenchFunctions.every(({ offer_policy_id: id }) => offerIds.has(id))
      && gambleGroups.every(({ offer_policy_id: id }) => offerIds.has(id)),
    "service offer normalized reference drift",
  );
  assert(
    unlocks.every(({ finish_condition_id: id }) => finishIds.has(id)),
    "Unlock finish-condition normalized reference drift",
  );
  assert(
    mechanicRules.every(({ source_file_id: id }) =>
      mechanicSourceIds.has(id)),
    "MechanicRule source-file normalized reference drift",
  );
  const curios = json(
    "content-reference/unknowable-domain-v1/curios.json",
  );
  const curioStates = json(
    "content-reference/unknowable-domain-v1/curio-states.json",
  );
  const curioGroups = json(
    "content-reference/unknowable-domain-v1/curio-groups.json",
  );
  const curioRules = json(
    "content-reference/unknowable-domain-v1/curio-rules.json",
  );
  const occurrences = json(
    "content-reference/unknowable-domain-v1/occurrences.json",
  );
  const occurrenceVariants = json(
    "content-reference/unknowable-domain-v1/occurrence-variants.json",
  );
  const occurrenceChoices = json(
    "content-reference/unknowable-domain-v1/occurrence-choices.json",
  );
  const bossPools = json(
    "content-reference/unknowable-domain-v1/boss-pools.json",
  );
  const fixtureFamilies = json(
    "content-reference/unknowable-domain-v1/" +
      "semantic-fixture-families.json",
  );
  const reviewFixtures = json(
    "content-reference/unknowable-domain-v1/review-fixtures.json",
  );
  const manifestRows = json(
    "content-reference/unknowable-domain-v1/manifest.json",
  );
  const curioIds = new Set(curios.map(({ id }) => id));
  const curioGroupIds = new Set(curioGroups.map(({ id }) => id));
  const occurrenceIds = new Set(occurrences.map(({ id }) => id));
  const occurrenceVariantIds = new Set(occurrenceVariants.map(({ id }) => id));
  const familySourceIds = new Set(fixtureFamilies.map(
    ({ source_id: id }) => id,
  ));
  const profileIds = new Set(json(
    "content-reference/unknowable-domain-v1/profiles.json",
  ).map(({ id }) => id));
  assert(
    curioStates.every(({ curio_id: id }) => curioIds.has(id))
      && curioRules.every((row) =>
        (row.curio_id === "NotApplicable" || curioIds.has(row.curio_id))
          && (!row.curio_group_id
            || row.curio_group_id === "NotApplicable"
            || curioGroupIds.has(row.curio_group_id))),
    "Curio state/rule normalized reference drift",
  );
  assert(
    occurrenceVariants.every(({ occurrence_id: id }) =>
      occurrenceIds.has(id))
      && occurrenceChoices.every(({ variant_id: id }) =>
        occurrenceVariantIds.has(id)),
    "Occurrence variant/choice normalized reference drift",
  );
  assert(
    bossPools.every(({ area_id: id }) => areaIds.has(id)),
    "BossPool Area normalized reference drift",
  );
  assert(
    reviewFixtures.every(({ family_id: id }) => familySourceIds.has(id)),
    "ReviewFixture family normalized reference drift",
  );
  assert(
    manifestRows.length === 1 && profileIds.has(manifestRows[0].profile_id),
    "Manifest Profile normalized reference drift",
  );
  const committed = path.join(root, "config", "unknowable-domain-generated");
  const committedLock = path.join(committed, "schema.lock");
  assert(fs.existsSync(committedLock), "committed schema lock is missing");
  assert(
    fs.readFileSync(lock).equals(fs.readFileSync(committedLock)),
    "committed schema lock drifted",
  );
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
  for (const workbook of [
    "UnknowableDomain.xlsx",
    "UnknowableDomainBindings.xlsx",
    "UnknowableDomainReview.xlsx",
  ]) {
    assert(
      fs.statSync(path.join(directTemplates, workbook)).size > 1000,
      `${workbook} direct template is missing`,
    );
    assert(
      fs.statSync(path.join(committed, "templates", workbook)).size > 1000,
      `${workbook} committed template is missing`,
    );
  }
  const committedRust = path.join(committed, "rust");
  const directRustFiles = fs.readdirSync(directRust)
    .filter((name) => name.endsWith(".rs") && name !== "mod.rs")
    .sort();
  const committedRustFiles = fs.readdirSync(committedRust)
    .filter((name) => name.endsWith(".rs"))
    .sort();
  assert(
    !fs.existsSync(path.join(committedRust, "mod.rs")),
    "oversized generated registry facade must remain uncommitted",
  );
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
  const digest = crypto.createHash("sha256")
    .update(schemaFiles.map((file) => fs.readFileSync(file)).join("\n"))
    .digest("hex");
  console.log(
    `Unknowable Domain Sora schema verified (${tables.size} isolated ` +
    `tables; three templates; typed local references; schemas ${digest}; ` +
    "generated lock/readers stable; pinned Sora 0.3.0).",
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
