#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  canonical,
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const GOAL08_CHECKPOINT = "457d05f0e3a7b6fe3abb7e8f142f96fa271f5ecd";
const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const schema = await json(
  "content-manifests/swarm-disaster-v1/normalized-schema.json",
);
const sourceManifest = await json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const sourceInventory = await json(
  "content-manifests/swarm-disaster-v1/source-inventory.json",
);
const fixtureContract = await json(
  "content-manifests/swarm-disaster-v1/fixture-contract.json",
);
const finalFiles = new Set([
  "mechanic-rules.json",
  "sources.json",
  "coverage.json",
  "research-gaps.json",
  "review-fixtures.json",
  "reconciliation-receipts.json",
  "manifest.json",
  "pack-index.json",
]);
const outputs = new Map();
for (const contract of schema.files) {
  if (finalFiles.has(contract.file)) continue;
  outputs.set(
    contract.file,
    await json(`content-reference/swarm-disaster-v1/${contract.file}`),
  );
}

async function json(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}

function rows(file) {
  const value = outputs.get(file);
  if (!Array.isArray(value)) throw new Error(`${file} is not a row array`);
  return value;
}

function pick(file, predicate = () => true, label = file) {
  const row = rows(file).find(predicate);
  if (!row) throw new Error(`missing ${label} in ${file}`);
  return row;
}

function uniqueBy(values, key) {
  const seen = new Map();
  for (const value of values) {
    const id = key(value);
    const prior = seen.get(id);
    if (prior && canonical(prior) !== canonical(value))
      throw new Error(`conflicting duplicate ${id}`);
    seen.set(id, value);
  }
  return [...seen.values()];
}

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function sourceRefs(records) {
  return uniqueBy(
    records.flatMap(({ source_refs: refs }) => refs ?? []),
    ({ source_id: id }) => id,
  );
}

function evidenceQuality(records) {
  const refs = sourceRefs(records);
  if (refs.some(({ evidence_quality: quality }) => quality === "ProjectPolicy"))
    return "ProjectPolicy";
  if (refs.some(({ evidence_quality: quality }) =>
    quality === "ApproximateFromReleasedText"))
    return "ApproximateFromReleasedText";
  if (refs.some(({ evidence_quality: quality }) =>
    quality === "ExactPublicText"))
    return "ExactPublicText";
  return "ExactStructured";
}

const familySpecs = new Map([
  ["profile-entry", {
    domain: "Activity",
    triggers: ["RunEntryRequested"],
    stateSlots: ["profile", "difficulty", "entry-bonus"],
    records: () => [
      pick("profiles.json", ({ kind }) => kind === "SwarmProfile"),
      ...rows("areas.json").filter(({ area_kind: kind }) => kind === "Formal"),
      ...rows("bonuses.json"),
    ],
  }],
  ["topology-generation", {
    domain: "Activity",
    triggers: ["PlaneGraphRequested"],
    stateSlots: ["plane-graph", "current-node"],
    records: () => [
      rows("chessboards.json")[0],
      rows("map-columns.json")[0],
      rows("map-nodes.json")[0],
      rows("map-edges.json")[0],
    ],
  }],
  ["topology-event-order", {
    domain: "Activity",
    triggers: ["TopologyEventMatched"],
    stateSlots: ["plane-graph", "topology-event-queue"],
    records: () => [rows("map-events.json")[0], rows("block-create-rules.json")[0]],
  }],
  ["domain-replacement", {
    domain: "Activity",
    triggers: ["DiceFaceAccepted"],
    stateSlots: ["plane-graph", "domain-kind"],
    records: () => [rows("domains.json")[0], rows("topology-consequences.json")[0]],
  }],
  ["beacon-copy-and-blanking", {
    domain: "Activity",
    triggers: ["DiceFaceAccepted"],
    stateSlots: ["plane-graph", "beacon-set"],
    records: () => [
      ...rows("beacons.json"),
      ...rows("topology-consequences.json").slice(0, 3),
    ],
  }],
  ["boss-choice-consequence", {
    domain: "CrossBattle",
    triggers: ["PlaneBossChoiceAccepted"],
    stateSlots: ["boss-choice-set", "final-boss-contributions"],
    records: () => {
      const choice = rows("boss-choices.json")[0];
      return [
        choice,
        pick("boss-pools.json", ({ choice_consequences: values }) =>
          values.some(({ boss_choice_id: id }) => id === choice.id)),
      ];
    },
  }],
  ["countdown-lifecycle", {
    domain: "Activity",
    triggers: ["RunStarted", "MapMoveAccepted", "PlaneTransitioned"],
    stateSlots: ["countdown"],
    records: () => [rows("countdown-and-disarray.json")[0]],
  }],
  ["planar-disarray-transition", {
    domain: "CrossBattle",
    triggers: ["MapMoveAccepted", "BattleSpecRequested"],
    stateSlots: ["countdown", "planar-disarray-tier"],
    records: () => [rows("countdown-and-disarray.json")[0]],
  }],
  ["boss-decay-stack", {
    domain: "CrossBattle",
    triggers: ["PlaneBossChoiceAccepted", "FinalBossBattleSpecRequested"],
    stateSlots: ["boss-decay-selection-set"],
    records: () => rows("boss-decay-levels.json")
      .filter(({ swarm_applicability: value }) =>
        value === "EnabledByReleasedSwarmText").slice(0, 3),
  }],
  ["audience-die-passive", {
    domain: "Activity",
    triggers: ["PathSelected", "RunStarted", "MapMoveAccepted"],
    stateSlots: ["selected-path", "audience-die", "plane-graph"],
    records: () => [rows("audience-paths.json")[0], rows("audience-dice.json")[0]],
  }],
  ["dice-face-targeting", {
    domain: "Activity",
    triggers: ["DiceFaceAccepted"],
    stateSlots: ["plane-graph", "dice-result"],
    records: () => [rows("dice-faces.json")[0], rows("dice-target-rules.json")[0]],
  }],
  ["dice-roll-reroll-cheat", {
    domain: "Activity",
    triggers: ["DiceRolled", "DiceRerolled", "DiceCheatRequested"],
    stateSlots: ["dice-result", "reroll-charges", "cheat-charges"],
    records: () => rows("dice-roll-controls.json"),
  }],
  ["communing-choice", {
    domain: "Activity",
    triggers: ["CommuningChoiceAccepted"],
    stateSlots: ["aeon-choice-counters", "communing-cabinet"],
    records: () => [rows("communing-choices.json")[0], rows("pathstrider-cabinets.json")[0]],
  }],
  ["communing-dimension-points", {
    domain: "Activity",
    triggers: ["CommuningPointAdjustmentAccepted"],
    stateSlots: ["communing-dimension-points"],
    records: () => [
      rows("communing-dimensions.json")[0],
      rows("communing-point-adjustments.json")[0],
    ],
  }],
  ["communing-trail-effect", {
    domain: "CrossBattle",
    triggers: ["CommuningTrailNodeUnlocked", "BattleSpecRequested"],
    stateSlots: ["communing-trail-unlocks", "battle-contributions"],
    records: () => {
      const effect = pick("communing-trail-effects.json",
        ({ battle_projection: projection }) => projection?.enabled,
        "battle-projecting Communing Trail effect");
      return [
        pick("communing-trail-nodes.json", ({ id }) => id === effect.node_id),
        ...rows("communing-trail-prerequisites.json")
          .filter(({ node_id: id }) => id === effect.node_id).slice(0, 1),
        effect,
      ];
    },
  }],
  ["pathstrider-progress", {
    domain: "Activity",
    triggers: ["ActivityOperationCommitted"],
    stateSlots: ["pathstrider-progress", "pathstrider-unlocks"],
    records: () => [
      rows("pathstrider-objectives.json")[0],
      rows("pathstrider-finish-conditions.json")[0],
      rows("pathstrider-unlocks.json")[0],
    ],
  }],
  ["path-and-propagation-unlock", {
    domain: "Activity",
    triggers: ["MechanicalChapterRequirementSatisfied"],
    stateSlots: ["available-paths", "mechanical-chapters"],
    records: () => [
      pick("paths.json", (row) => canonical(row).includes("propagation"),
        "Propagation path"),
      pick("pathstrider-unlocks.json",
        (row) => canonical(row).includes("1000008"), "Propagation unlock"),
      rows("mechanical-chapter-locators.json")[0],
    ],
  }],
  ["resonance-interplay", {
    domain: "CrossBattle",
    triggers: ["BlessingInventoryMutationCommitted", "BattleSpecRequested"],
    stateSlots: ["blessing-inventory", "active-interplays"],
    records: () => [rows("resonance-interplays.json")[0], rows("resonances.json")[0]],
  }],
  ["curio-lifecycle", {
    domain: "CrossBattle",
    triggers: ["CurioGranted", "BattleCompleted", "CurioRepairRequested"],
    stateSlots: ["curio-inventory", "curio-state"],
    records: () => {
      const state = pick("curio-states.json",
        (row) => canonical(row).includes("repair"), "repairable Curio state");
      return [
        pick("curios.json", ({ id }) => id === state.curio_id),
        state,
        pick("curio-rules.json", ({ curio_id: id }) => id === state.curio_id),
      ];
    },
  }],
  ["occurrence-choice", {
    domain: "Activity",
    triggers: ["OccurrenceChoiceAccepted"],
    stateSlots: ["occurrence-graph", "run-inventory"],
    records: () => {
      const choice = pick("occurrence-choices.json",
        (row) => (row.costs ?? []).length > 0, "costed Occurrence choice");
      return [
        choice,
        pick("occurrence-variants.json",
          ({ id }) => id === choice.variant_id),
      ];
    },
  }],
  ["service-and-adventure", {
    domain: "Activity",
    triggers: ["ServicePurchaseAccepted", "AdventureOutcomeOffered"],
    stateSlots: ["cosmic-fragments", "run-inventory", "external-outcome"],
    records: () => [
      rows("services.json")[0],
      rows("service-rules.json")[0],
      rows("adventure-outcomes.json")[0],
    ],
  }],
  ["encounter-selection", {
    domain: "CrossBattle",
    triggers: ["BattleSpecRequested"],
    stateSlots: ["resolved-domain", "difficulty-segment", "encounter-selection"],
    records: () => {
      const group = pick("encounter-groups.json",
        ({ encounter_role: role }) => role === "FirstPlaneBossAlternative");
      const wave = pick("encounter-waves.json",
        ({ encounter_group_id: id }) => id === group.id);
      return [
        group,
        wave,
        ...rows("enemy-slots.json").filter(({ wave_id: id }) => id === wave.id),
        pick("boss-pools.json",
          ({ candidate_ids: ids }) => ids.includes(group.id)),
      ];
    },
  }],
  ["final-boss-consequence", {
    domain: "CrossBattle",
    triggers: ["FinalBossBattleSpecRequested"],
    stateSlots: [
      "final-boss-selection",
      "boss-decay-selection-set",
      "active-interplays",
      "planar-disarray-tier",
    ],
    records: () => [
      pick("encounter-groups.json",
        ({ encounter_role: role }) => role === "FinalBoss"),
      pick("boss-pools.json", ({ pool_tier: tier }) => tier === "FinalBoss"),
      pick("paths.json", (row) => canonical(row).includes("propagation"),
        "Propagation path"),
      rows("resonance-interplays.json")[0],
      rows("countdown-and-disarray.json")[0],
      pick("boss-decay-levels.json",
        ({ swarm_applicability: value }) =>
          value === "EnabledByReleasedSwarmText"),
    ],
  }],
]);

function fixture(familyContract) {
  const family = familyContract.id;
  const spec = familySpecs.get(family);
  if (!spec) throw new Error(`missing fixture specification ${family}`);
  const records = uniqueBy(spec.records(), ({ id }) => id);
  const refs = sourceRefs(records);
  const quality = evidenceQuality(records);
  const policyRefs = refs.filter(({ evidence_quality: evidence }) =>
    evidence === "ProjectPolicy"
    || evidence === "ApproximateFromReleasedText");
  return {
    ...context.envelope({
      id: `swarm-disaster.fixture.${family}`,
      kind: "SemanticReviewFixture",
      nameEn: `${family} Semantic Review Fixture`,
      nameZh: `${family} 语义审查夹具`,
      summaryEn:
        `Reference-only semantic fixture for the ${family} family; it reviews every frozen must-cover fact without claiming runtime executability.`,
      summaryZh:
        `${family} 机制族的仅资料语义夹具；审查全部冻结必覆盖事实，不声称运行时可执行性。`,
      evidenceQuality: quality,
      sourceRefs: refs,
      tags: ["review-fixture", family],
    }),
    family_id: family,
    source_record_ids: uniqueSorted(records.map(({ id }) => id)),
    preconditions: [
      { fact: "runtime_loading", value: "ForbiddenReferenceOnly" },
      { fact: "source_records_data_ready", value: true },
    ],
    input: {
      family_id: family,
      deterministic_seed: "0",
      selected_record_ids: uniqueSorted(records.map(({ id }) => id)),
      external_outcome_only: family === "service-and-adventure",
    },
    ordered_operations: familyContract.must_cover.map((fact, index) => ({
      sequence: index + 1,
      operation: `Review${slug(fact).split("-").map((word) =>
        `${word[0].toUpperCase()}${word.slice(1)}`).join("")}`,
      fact,
      unresolved_behavior: policyRefs.length > 0
        ? "FailClosed"
        : "NotApplicable",
    })),
    expected_facts: [
      ...familyContract.must_cover.map((fact) => ({
        path: `must_cover.${slug(fact)}`,
        operator: "reviewed",
        value: true,
      })),
      {
        path: "source_record_count",
        operator: "equals",
        value: String(records.length),
      },
    ],
    evidence_refs: refs.map(({ source_id: id }) => id),
    fixture_evidence_quality: quality,
    ...(policyRefs.length > 0 ? {
      note: uniqueSorted(policyRefs.map(({ note }) => note).filter(Boolean))
        .join(" "),
      replacement_condition: uniqueSorted(
        policyRefs.map(({ replacement_condition: condition }) => condition)
          .filter(Boolean),
      ).join(" "),
    } : {}),
  };
}

const fixtures = fixtureContract.required_families
  .map(fixture)
  .sort((left, right) =>
    left.family_id.localeCompare(right.family_id)
    || left.id.localeCompare(right.id));
outputs.set("review-fixtures.json", fixtures);

const rules = fixtures.map((fixtureRow) => {
  const spec = familySpecs.get(fixtureRow.family_id);
  return {
    ...context.envelope({
      id: `swarm-disaster.mechanic-rule.${fixtureRow.family_id}`,
      kind: "SwarmMechanicRule",
      nameEn: `${fixtureRow.family_id} Mechanic Rule`,
      nameZh: `${fixtureRow.family_id} 机制规则`,
      summaryEn:
        `${fixtureRow.family_id} preserves implementation-facing triggers, state ownership and ordered reference operations while remaining runtime-disabled.`,
      summaryZh:
        `${fixtureRow.family_id} 保留面向实现的触发器、状态归属与有序资料操作，同时保持运行时禁用。`,
      evidenceQuality: fixtureRow.fixture_evidence_quality,
      sourceRefs: fixtureRow.source_refs,
      tags: ["mechanic-rule", fixtureRow.family_id],
    }),
    family_id: fixtureRow.family_id,
    domain: spec.domain,
    triggers: spec.triggers,
    state_slots: spec.stateSlots.map((id) => ({
      id,
      owner: id.includes("battle") || spec.domain === "Battle"
        ? "Battle"
        : "Activity",
    })),
    program: fixtureRow.ordered_operations.map((operation) => ({
      sequence: operation.sequence,
      operation: operation.operation,
      source_fact: operation.fact,
      unresolved_behavior: operation.unresolved_behavior,
    })),
    fixture_ids: [fixtureRow.id],
    execution_disposition: "ReferenceOnly",
    runtime_handler_id: "",
  };
}).sort((left, right) =>
  left.domain.localeCompare(right.domain) || left.id.localeCompare(right.id));
outputs.set("mechanic-rules.json", rules);

function manifestCategoryRef(categoryId, category) {
  return {
    source_id: `source.goal09.manifest-category.${slug(categoryId)}`,
    repository: "starclock",
    revision: "starclock.swarm-disaster-content-manifest.v1",
    path: "content-manifests/swarm-disaster-v1/content-manifest.json",
    locator: `categories/${categoryId}`,
    sha256: sha256(canonical(category)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}

function addIndex(index, key, value) {
  if (key === undefined || key === null || key === "") return;
  const text = String(key);
  if (!index.has(text)) index.set(text, []);
  index.get(text).push(value);
}

function collectIdentityValues(value, key = "") {
  if (Array.isArray(value))
    return value.flatMap((item) => collectIdentityValues(item, key));
  if (value && typeof value === "object")
    return Object.entries(value).flatMap(([childKey, child]) =>
      collectIdentityValues(child, childKey));
  if ((key === "id" || key.endsWith("_id") || key.endsWith("_ids"))
    && (typeof value === "string" || typeof value === "number"))
    return [String(value)];
  return [];
}

function normalizedIndexes() {
  const source = new Map();
  const identity = new Map();
  for (const [file, value] of outputs) {
    if (!Array.isArray(value) || finalFiles.has(file)) continue;
    for (const row of value) {
      const ref = { file, id: row.id };
      for (const sourceRef of row.source_refs ?? [])
        addIndex(source, `${sourceRef.path}#${sourceRef.locator}`, ref);
      for (const id of collectIdentityValues(row)) addIndex(identity, id, ref);
    }
  }
  return { source, identity };
}

function refKey(ref) {
  return `${ref.file}\0${ref.id}`;
}

function coverageRows() {
  const indexes = normalizedIndexes();
  const result = [];
  for (const [categoryId, category] of Object.entries(sourceManifest.categories)
    .sort(([left], [right]) => left.localeCompare(right))) {
    const allowedFiles = new Set(schema.files
      .filter(({ manifest_category_inputs: ids }) => ids.includes(categoryId))
      .map(({ file }) => file));
    for (const record of category.records) {
      let normalizedRefs = [
        ...(indexes.source.get(record.source) ?? []),
        ...(indexes.identity.get(String(record.id)) ?? []),
      ].filter(({ file }) => allowedFiles.has(file));
      if (categoryId === "map_columns") {
        const [boardId, positionX] = String(record.id).split(":");
        normalizedRefs.push(...rows("map-columns.json")
          .filter(({ chessboard_id: chessboardId, position_x: x }) =>
            chessboardId.endsWith(`.${boardId}`) && String(x) === positionX)
          .map(({ id }) => ({ file: "map-columns.json", id })));
      }
      if (categoryId === "profiles")
        normalizedRefs.push({
          file: "profiles.json",
          id: pick("profiles.json", ({ kind }) => kind === "SwarmProfile").id,
        });
      if (categoryId === "semantic_fixture_families")
        normalizedRefs.push(
          {
            file: "review-fixtures.json",
            id: `swarm-disaster.fixture.${record.id}`,
          },
          {
            file: "mechanic-rules.json",
            id: `swarm-disaster.mechanic-rule.${record.id}`,
          },
        );
      normalizedRefs = uniqueBy(normalizedRefs, refKey)
        .sort((left, right) =>
          left.file.localeCompare(right.file) || left.id.localeCompare(right.id));
      if (normalizedRefs.length === 0)
        throw new Error(
          `manifest obligation ${categoryId}/${record.id} has no normalized reference`,
        );
      const sourceRef = manifestCategoryRef(categoryId, category);
      result.push({
        ...context.envelope({
          id: `swarm-disaster.coverage.${slug(categoryId)}.${slug(record.id)}`,
          kind: "CoverageRecord",
          nameEn: `${categoryId}/${record.id} Coverage`,
          nameZh: `${categoryId}/${record.id} 覆盖`,
          summaryEn:
            `Frozen ${categoryId} obligation ${record.id} resolves to ${normalizedRefs.length} DataReady normalized reference(s).`,
          summaryZh:
            `冻结的 ${categoryId} 义务 ${record.id} 解析到 ${normalizedRefs.length} 个 DataReady 规范化引用。`,
          ownership: record.ownership,
          sourceRefs: [sourceRef],
          tags: ["coverage", categoryId],
        }),
        manifest_category: categoryId,
        manifest_record_id: String(record.id),
        source_locator: record.source,
        source_evidence_sha256: record.evidence_sha256,
        coverage_state: "DataReady",
        normalized_refs: normalizedRefs,
        blocking_gap_ids: [],
      });
    }
  }
  return result.sort((left, right) =>
    left.manifest_category.localeCompare(right.manifest_category)
    || left.manifest_record_id.localeCompare(right.manifest_record_id));
}

const coverage = coverageRows();
outputs.set("coverage.json", coverage);

function goal08Manifest() {
  const object = `${GOAL08_CHECKPOINT}:content-manifests/gold-and-gears-v1/content-manifest.json`;
  const result = spawnSync("git", ["cat-file", "blob", object], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  if (result.status !== 0)
    throw new Error(
      `cannot read Goal 08 checkpoint manifest: ${result.stderr.trim()}`,
    );
  return { raw: result.stdout, value: JSON.parse(result.stdout) };
}

function reconciliationReceipts() {
  const checkpoint = goal08Manifest();
  const goldByIdentity = new Map();
  for (const [categoryId, category] of Object.entries(
    checkpoint.value.categories,
  ))
    for (const record of category.records)
      goldByIdentity.set(`${record.source}\0${record.id}`, {
        categoryId,
        record,
      });
  const checkpointRef = {
    source_id: "source.goal09.goal08-checkpoint.content-manifest",
    repository: "starclock",
    revision: GOAL08_CHECKPOINT,
    path: "content-manifests/gold-and-gears-v1/content-manifest.json",
    locator: "root",
    sha256: sha256(checkpoint.raw),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
  const receipts = [];
  for (const [categoryId, category] of Object.entries(sourceManifest.categories))
    for (const record of category.records) {
      const gold = goldByIdentity.get(`${record.source}\0${record.id}`);
      if (!gold) continue;
      if (record.evidence_sha256 !== gold.record.evidence_sha256)
        throw new Error(
          `Goal 08 reconciliation conflict ${record.source}/${record.id}`,
        );
      const [sourcePath, explicitLocator] = record.source.split("#", 2);
      receipts.push({
        ...context.envelope({
          id:
            `swarm-disaster.reconciliation.${slug(sourcePath)}.${slug(explicitLocator ?? record.id)}.${slug(record.id)}`,
          kind: "Goal08ReconciliationReceipt",
          nameEn: `Goal 08 Reconciliation ${record.id}`,
          nameZh: `Goal 08 对账 ${record.id}`,
          summaryEn:
            `${categoryId}/${record.id} matches the committed Goal 08 fact at the same source identity and evidence digest.`,
          summaryZh:
            `${categoryId}/${record.id} 与已提交 Goal 08 中同一来源身份及证据摘要完全一致。`,
          ownership: record.ownership,
          sourceRefs: [checkpointRef],
          tags: ["goal08-reconciliation", "matched"],
        }),
        source_path: sourcePath,
        row_locator: explicitLocator ?? String(record.id),
        evidence_sha256: record.evidence_sha256,
        swarm_category: categoryId,
        swarm_record_id: String(record.id),
        goal08_category: gold.categoryId,
        goal08_record_id: String(gold.record.id),
        goal08_commit: GOAL08_CHECKPOINT,
        outcome: "MatchedSharedFact",
      });
    }
  return receipts.sort((left, right) =>
    left.source_path.localeCompare(right.source_path)
    || left.row_locator.localeCompare(right.row_locator)
    || left.id.localeCompare(right.id));
}

const receipts = reconciliationReceipts();
outputs.set("reconciliation-receipts.json", receipts);

function collectSourceRefs() {
  const refs = [];
  for (const value of outputs.values()) {
    if (!Array.isArray(value)) continue;
    for (const row of value) refs.push(...(row.source_refs ?? []));
  }
  return uniqueBy(refs, ({ source_id: id }) => id)
    .sort((left, right) => left.source_id.localeCompare(right.source_id));
}

function researchGaps(allSourceRefs) {
  const affected = new Map();
  for (const [file, value] of outputs) {
    if (!Array.isArray(value)) continue;
    for (const row of value)
      for (const ref of row.source_refs ?? []) {
        if (!["ProjectPolicy", "ApproximateFromReleasedText"]
          .includes(ref.evidence_quality))
          continue;
        if (!affected.has(ref.source_id)) affected.set(ref.source_id, []);
        affected.get(ref.source_id).push({ file, id: row.id });
      }
  }
  const fixtureOverrides = new Map([
    [
      "source.goal09.project-policy.communing-trail-prerequisites",
      ["communing-trail-effect"],
    ],
    [
      "source.goal09.project-policy.occurrence-pool-selection",
      ["occurrence-choice"],
    ],
    [
      "source.goal09.project-policy.occurrence-random-outcome",
      ["occurrence-choice"],
    ],
    [
      "source.goal09.project-policy.rooms",
      ["encounter-selection"],
    ],
    [
      "source.goal09.project-policy.shared-content-pool-weight",
      ["resonance-interplay", "service-and-adventure"],
    ],
  ]);
  return allSourceRefs.filter(({ evidence_quality: quality }) =>
    ["ProjectPolicy", "ApproximateFromReleasedText"].includes(quality))
    .map((ref) => {
      const affectedRecords = uniqueBy(
        affected.get(ref.source_id) ?? [],
        ({ file, id }) => `${file}\0${id}`,
      ).sort((left, right) =>
        left.file.localeCompare(right.file) || left.id.localeCompare(right.id));
      const affectedIds = new Set(affectedRecords.map(({ id }) => id));
      const affectedFixtureIds = uniqueSorted([
        ...fixtures.filter((fixture) =>
          affectedIds.has(fixture.id)
          || fixture.source_record_ids.some((id) => affectedIds.has(id)))
          .map(({ id }) => id),
        ...(fixtureOverrides.get(ref.source_id) ?? []).map((family) =>
          `swarm-disaster.fixture.${family}`),
      ]);
      if (affectedFixtureIds.length === 0)
        throw new Error(`${ref.source_id} lacks an affected fixture`);
      const confidence = ref.evidence_quality === "ProjectPolicy"
        ? "DeterministicPolicyNotObservedParity"
        : "ReleasedTextCrossCheck";
      return ({
      ...context.envelope({
        id: `swarm-disaster.research-gap.${slug(ref.source_id)}`,
        kind: "ResearchGap",
        nameEn: `${ref.locator} Evidence Boundary`,
        nameZh: `${ref.locator} 证据边界`,
        summaryEn:
          `Nonblocking ${ref.evidence_quality} boundary with an explicit deterministic policy and replacement condition.`,
        summaryZh:
          `非阻塞的 ${ref.evidence_quality} 证据边界，具有显式确定性策略与替换条件。`,
        evidenceQuality: ref.evidence_quality,
        sourceRefs: [ref],
        tags: ["research-gap", "nonblocking"],
      }),
      state: "PolicyBound",
      gap_state: "PolicyBound",
      blocking: false,
      field: ref.locator,
      policy_source_id: ref.source_id,
      known_facts: ref.note ?? "",
      selected_policy: ref.note ?? "",
      rejected_alternatives: [
        `Treat ${ref.locator} as exact without released evidence.`,
        `Borrow ${ref.locator} from an adjacent mode without a proven Swarm consumer.`,
      ],
      rationale:
        `Keep ${ref.locator} deterministic and fail closed while preserving ` +
        "released facts separately; never claim the selected policy as " +
        "observed parity.",
      affected_fixture_ids: affectedFixtureIds,
      confidence,
      affected_records: affectedRecords,
      note: ref.note ?? "",
      replacement_condition: ref.replacement_condition ?? "",
    });
    })
    .sort((left, right) =>
      left.state.localeCompare(right.state) || left.id.localeCompare(right.id));
}

const gaps = researchGaps(collectSourceRefs());
for (const gap of gaps)
  if (!gap.replacement_condition)
    throw new Error(`${gap.id} lacks a replacement condition`);
outputs.set("research-gaps.json", gaps);

function sourceRegistry(allRefs) {
  return allRefs.map((ref) => ({
    id: ref.source_id,
    schema_revision: "starclock.swarm-disaster-source.v1",
    kind: "SourceRecord",
    source_id: ref.source_id,
    source_kind: ref.evidence_quality === "ProjectPolicy"
      ? "ProjectPolicy"
      : ref.repository === "starclock"
      ? "InheritedOrLocal"
      : ref.repository.startsWith("http")
      && !ref.repository.includes("gitlab.com")
      ? "PublicCrossCheck"
      : "PinnedStructured",
    repository: ref.repository,
    revision: ref.revision,
    game_version: GAME_VERSION,
    path: ref.path,
    locator: ref.locator,
    sha256: ref.sha256,
    evidence_quality: ref.evidence_quality,
    access_date: ref.access_date,
    note: ref.note ?? "",
    replacement_condition: ref.replacement_condition ?? "",
  })).sort((left, right) =>
    left.id.localeCompare(right.id) || left.locator.localeCompare(right.locator));
}

const sources = sourceRegistry(collectSourceRefs());
outputs.set("sources.json", sources);

const sourceInventoryBytes = await fs.readFile(
  path.join(root, "content-manifests/swarm-disaster-v1/source-inventory.json"),
);
const contentManifestBytes = await fs.readFile(
  path.join(root, "content-manifests/swarm-disaster-v1/content-manifest.json"),
);
outputs.set("manifest.json", {
  id: "swarm-disaster.pack-manifest.v1",
  schema_revision: "starclock.swarm-disaster-pack-manifest.v1",
  goal_id: "swarm-disaster-reference-v1",
  profile_id: "swarm-disaster.profile.v1",
  snapshot: sourceInventory.snapshot,
  source_manifest_sha256: sha256(sourceInventoryBytes),
  content_manifest_sha256: sha256(contentManifestBytes),
  structured_source_revision:
    "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  bilingual_index_revision:
    "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  frozen_source_obligations: sourceManifest.counts.records,
  data_ready_source_obligations: coverage.length,
  coverage_percent: "100",
  normalized_file_count: schema.files.length,
  mechanic_rule_count: rules.length,
  semantic_fixture_family_count: fixtures.length,
  research_gap_count: gaps.length,
  blocking_research_gap_count: gaps.filter(({ blocking }) => blocking).length,
  reconciliation_receipt_count: receipts.length,
  runtime_loading: "ForbiddenReferenceOnly",
  authoring_target: "ExcelOpenPyxlThenSora030",
  candidate_quality: true,
  files: schema.files.map(({ file }) => file).sort(),
});

if (outputs.size !== schema.files.length - 1)
  throw new Error(
    `expected ${schema.files.length - 1} pre-index files, got ${outputs.size}`,
  );

function packIndex() {
  const files = [...outputs.entries()]
    .filter(([file]) => file !== "pack-index.json")
    .map(([file, value]) => {
      const bytes = `${JSON.stringify(value, null, 2)}\n`;
      return {
        file,
        bytes: Buffer.byteLength(bytes),
        rows: Array.isArray(value) ? value.length : 1,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => left.file.localeCompare(right.file));
  const packSha256 = sha256(
    files.map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n"),
  );
  return files.map((entry) => ({
    id: `swarm-disaster.pack-index.${slug(entry.file)}`,
    schema_revision: "starclock.swarm-disaster-pack-index.v1",
    ...entry,
    pack_sha256: packSha256,
  }));
}

outputs.set("pack-index.json", packIndex());
const expectedFiles = schema.files.map(({ file }) => file).sort();
const actualFiles = [...outputs.keys()].sort();
if (canonical(expectedFiles) !== canonical(actualFiles))
  throw new Error("normalized output file set drift");

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster pack ${check ? "verified" : "finalized"}: ` +
  `${rules.length} rules, ${sources.length} sources, ${coverage.length} ` +
  `coverage rows, ${gaps.length} nonblocking gaps, ${fixtures.length} fixtures, ` +
  `${receipts.length} Goal 08 receipts, ${schema.files.length} files.`,
);
