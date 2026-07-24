#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: generate-partitions.mjs [--check]");
const policyPath = "policy/goal07-partitions.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal07-partition-policy.v1",
  "unsupported Goal 07 partition policy");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `partition input drift: ${input.path}`);
const audit = json(policy.inputs.retained_audit.path);
const partitions = [];
const assigned = {
  records: new Map(),
  rules: new Map(),
  fixtures: new Map(),
  enemy_variants: new Map(),
  encounter_members: new Map(),
};

for (const milestone of [
  "G07-P2-M01", "G07-P2-M02", "G07-P2-M03", "G07-P2-M04", "G07-P2-M05",
  "G07-P2-M06", "G07-P2-M07", "G07-P2-M08", "G07-P2-M09", "G07-P2-M10",
  "G07-P3-M11", "G07-P3-M12", "G07-P4-M14",
])
  partitionRuleMilestone(milestone);
partitionNoncombatMilestone();
partitionEncounterMilestone();

for (const [index, partition] of partitions.entries()) {
  partition.ordinal = index;
  partition.dependencies = index === 0 ? [] : [partitions[index - 1].id];
  partition.focused_commands = policy.focused_commands
    .map((command) => command.replace("{batch_id}", partition.id));
  partition.expected = {
    records: partition.record_ids.length,
    rules: partition.rule_ids.length,
    fixtures: partition.fixture_ids.length,
    enemy_variants: partition.enemy_variant_ids.length,
    encounter_members: partition.encounter_member_ids.length,
    native_handler_admissions: partition.admitted_native_handler_ids.length,
  };
  partition.workbook_families = workbookFamilies(partition);
}

assert(partitions.length === policy.expected_generated_batches,
  `expected ${policy.expected_generated_batches} partitions, got ${partitions.length}`);
const countsByMilestone = countBy(partitions, "milestone");
assert(equal(countsByMilestone, policy.expected_by_milestone),
  "generated milestone partition denominator drift");
verifyAssignment("records", audit.records);
verifyAssignment("rules", audit.rules);
verifyAssignment("fixtures", audit.fixtures);
verifyAssignment("enemy_variants", audit.enemy_variants);
verifyAssignment("encounter_members", audit.encounter_members);
verifyCaps();

const manifest = {
  schema_revision: "starclock.goal07-content-partitions.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-25",
  policy_sha256: sha256(policyPath),
  source_audit_sha256: policy.inputs.retained_audit.sha256,
  summary: {
    generated_batches: partitions.length,
    fixed_batches: 17,
    total_batches: partitions.length + 17,
    by_milestone: countsByMilestone,
    assigned: {
      records: assigned.records.size,
      rules: assigned.rules.size,
      fixtures: assigned.fixtures.size,
      enemy_variants: assigned.enemy_variants.size,
      encounter_members: assigned.encounter_members.size,
    },
    native_review_candidate_rules: audit.summary.native_review_candidate_rules,
    admitted_native_handlers: 0,
  },
  caps: policy.caps,
  partitions,
};
const manifestRelative =
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json";
const manifestText = encode(manifest);
const ledgerRelative =
  "docs/goals/07-standard-universe-mechanics-content-ledger.md";
const ledgerText = ledger(manifest);
const evidence = {
  schema_revision: "starclock.goal07-partition-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "complete",
  generated_batches: partitions.length,
  total_batches: partitions.length + 17,
  by_milestone: countsByMilestone,
  assigned: manifest.summary.assigned,
  caps: policy.caps,
  manifest_sha256: digest(manifestText),
  ledger_sha256: digest(ledgerText),
  policy_sha256: manifest.policy_sha256,
};
const evidenceRelative =
  "evidence/standard-universe-mechanics-complete-v1/phase0/partition-summary.json";
writeOrCheck(manifestRelative, manifestText);
writeOrCheck(ledgerRelative, ledgerText);
writeOrCheck(evidenceRelative, encode(evidence));
console.log(
  `Goal 07 partitions ${check ? "verified" : "generated"} ` +
  `(${partitions.length} content batches, ${partitions.length + 17} total batches).`,
);

function partitionRuleMilestone(milestone) {
  const rules = audit.rules.filter((entry) => entry.milestone === milestone);
  const records = audit.records.filter((entry) => entry.milestone === milestone);
  const fixtures = audit.fixtures.filter((entry) => entry.milestone === milestone);
  const batches = chunks(rules, policy.caps.mechanic_rules_per_batch)
    .map((batchRules, index) => createPartition(
      milestone,
      index + 1,
      milestone === "G07-P4-M14" ? "noncombat-service" : "combat-mechanic",
      records[0]?.mechanic_family ?? rules[0]?.mechanic_family,
      [],
      batchRules,
      [],
      [],
      [],
      [],
    ));
  assert(batches.length > 0, `${milestone}: no rule batches`);
  const ruleOwner = new Map();
  for (const batch of batches)
    for (const rule of batch.rule_entries) ruleOwner.set(rule.id, batch);
  for (const record of records) {
    const owner = record.linked_rule_ids
      .map((id) => ruleOwner.get(id))
      .find(Boolean) ?? batches[0];
    owner.record_entries.push(record);
  }
  for (const fixture of fixtures) {
    const owner = fixture.input_ids
      .map((id) => batches.find((batch) =>
        batch.record_entries.some((record) => record.id === id)))
      .find(Boolean) ?? batches[0];
    owner.fixture_entries.push(fixture);
  }
  finalize(batches);
}

function partitionNoncombatMilestone() {
  const milestone = "G07-P4-M13";
  const records = audit.records.filter((entry) => entry.milestone === milestone);
  const fixtures = audit.fixtures.filter((entry) => entry.milestone === milestone);
  const batches = chunks(records, policy.caps.effect_bearing_noncombat_records_per_batch)
    .map((batchRecords, index) => createPartition(
      milestone,
      index + 1,
      "noncombat-occurrence",
      "occurrences-and-choices",
      batchRecords,
      [],
      [],
      [],
      [],
      [],
    ));
  for (const fixture of fixtures) {
    const owner = fixture.input_ids
      .map((id) => batches.find((batch) =>
        batch.record_entries.some((record) => record.id === id)))
      .find(Boolean) ?? batches[0];
    owner.fixture_entries.push(fixture);
  }
  finalize(batches);
}

function partitionEncounterMilestone() {
  const milestone = "G07-P5-M15";
  const family = "enemies-encounters-worlds-difficulty-carry";
  const waveById = new Map(tableRows(policy.inputs.encounter_waves.path)
    .map((values) => [integer(values.id), integer(values.member_id)]));
  const memberEnemies = new Map();
  for (const values of tableRows("config/universe-generated/debug-json/UniverseEncounterWaveEnemy.json")) {
    const member = required(waveById, integer(values.wave_id), "encounter wave");
    if (!memberEnemies.has(member)) memberEnemies.set(member, new Set());
    memberEnemies.get(member).add(string(values.enemy_variant_stable_key));
  }
  const bossKeys = new Set(tableRows(
    "config/universe-generated/debug-json/UniverseDifficultyEnemy.json",
  ).filter((values) => string(values.role) === "Boss")
    .map((values) => string(values.enemy_variant_stable_key)));
  const bossEntries = audit.enemy_variants.filter(({ id }) => bossKeys.has(id));
  const ordinaryEntries = audit.enemy_variants.filter(({ id }) => !bossKeys.has(id));
  const enemyBatches = [
    ...bossEntries.map((entry) => [entry]),
    ...chunks(ordinaryEntries, policy.caps.ordinary_enemy_variants_per_batch),
  ].map((entries, index) => createPartition(
    milestone,
    index + 1,
    entries.length === 1 && bossKeys.has(entries[0].id)
      ? "enemy-boss"
      : "enemy-ordinary",
    family,
    [],
    [],
    [],
    entries,
    [],
    [],
  ));
  const enemyOwner = new Map();
  for (const batch of enemyBatches)
    for (const enemy of batch.enemy_entries) enemyOwner.set(enemy.id, batch);
  for (const member of audit.encounter_members) {
    const id = Number(member.id.split(".").at(-1));
    const owners = [...required(memberEnemies, id, "member enemy set")]
      .map((enemy) => required(enemyOwner, enemy, "enemy partition"));
    owners.sort((left, right) => left.id.localeCompare(right.id));
    owners[0].encounter_member_entries.push(member);
    owners[0].logical_groups.push(`member:${member.id}`);
  }
  const encounterGroups = byId(json(
    "content-reference/standard-universe-v1/encounter-groups.json",
  ));
  for (const record of audit.records.filter((entry) =>
    entry.milestone === milestone && entry.source_category === "encounter-groups")) {
    const group = required(encounterGroups, record.id, "encounter group");
    const owners = new Set();
    for (const member of group.weighted_member_ids)
      for (const wave of member.waves)
        for (const enemy of wave.enemy_variant_ids)
          owners.add(required(enemyOwner, enemy.enemy_variant_id, "group enemy partition"));
    const ordered = [...owners].sort((left, right) => left.id.localeCompare(right.id));
    ordered[0].record_entries.push(record);
    ordered[0].logical_groups.push(`encounter-group:${record.id}`);
  }

  const structuralRecords = audit.records.filter((entry) =>
    entry.milestone === milestone && entry.source_category !== "encounter-groups");
  const nextSequence = () => enemyBatches.length + structuralBatches.length + 1;
  const structuralBatches = [];
  structuralBatches.push(createPartition(
    milestone, nextSequence(), "domain-graph", family,
    structuralRecords.filter(({ source_category: category }) => category === "domains"),
    [], [], [], [], ["domains"],
  ));
  for (const entries of chunks(
    structuralRecords.filter(({ source_category: category }) =>
      category === "encounter-pools"),
    policy.caps.effect_bearing_noncombat_records_per_batch,
  ))
    structuralBatches.push(createPartition(
      milestone, nextSequence(), "encounter-selection", family,
      entries, [], [], [], [], ["encounter-pools"],
    ));

  const maps = json("content-reference/standard-universe-v1/maps.json");
  const mapsById = groupBy(maps, "map_id");
  let mapPack = [];
  let mapRows = 0;
  for (const [mapId, entries] of [...mapsById].sort(([left], [right]) =>
    left.localeCompare(right))) {
    if (mapRows > 0
      && mapRows + entries.length > policy.caps.topology_metadata_rows_per_batch) {
      structuralBatches.push(mapPartition(mapPack, nextSequence(), family));
      mapPack = [];
      mapRows = 0;
    }
    mapPack.push([mapId, entries]);
    mapRows += entries.length;
  }
  if (mapPack.length > 0)
    structuralBatches.push(mapPartition(mapPack, nextSequence(), family));

  for (const entries of chunks(
    structuralRecords.filter(({ source_category: category }) => category === "rooms"),
    policy.caps.effect_bearing_noncombat_records_per_batch,
  ))
    structuralBatches.push(createPartition(
      milestone, nextSequence(), "room-content", family,
      entries, [], [], [], [], ["rooms"],
    ));
  structuralBatches.push(createPartition(
    milestone, nextSequence(), "world-difficulty", family,
    structuralRecords.filter(({ source_category: category }) =>
      category === "worlds" || category === "world-difficulties"),
    [], [], [], [], ["worlds-and-difficulties"],
  ));

  const all = [...enemyBatches, ...structuralBatches];
  const recordOwner = new Map();
  for (const batch of all)
    for (const record of batch.record_entries) recordOwner.set(record.id, batch);
  for (const fixture of audit.fixtures.filter((entry) => entry.milestone === milestone)) {
    const owner = fixture.input_ids.map((id) => recordOwner.get(id)).find(Boolean) ?? all[0];
    owner.fixture_entries.push(fixture);
  }
  finalize(all);
}

function mapPartition(pack, sequence, family) {
  const recordIds = new Set(pack.flatMap(([, entries]) =>
    entries.map(({ id }) => id)));
  return createPartition(
    "G07-P5-M15", sequence, "topology-map", family,
    audit.records.filter(({ id }) => recordIds.has(id)),
    [], [], [], [], pack.map(([mapId]) => `map:${mapId}`),
  );
}

function createPartition(
  milestone,
  sequence,
  lane,
  mechanicFamily,
  recordEntries,
  ruleEntries,
  fixtureEntries,
  enemyEntries,
  encounterMemberEntries,
  logicalGroups,
) {
  return {
    id: `${milestone}-S${String(sequence).padStart(2, "0")}`,
    milestone,
    lane,
    mechanic_family: mechanicFamily,
    ordinal: 0,
    dependencies: [],
    record_entries: [...recordEntries],
    rule_entries: [...ruleEntries],
    fixture_entries: [...fixtureEntries],
    enemy_entries: [...enemyEntries],
    encounter_member_entries: [...encounterMemberEntries],
    logical_groups: [...logicalGroups],
    record_ids: [],
    rule_ids: [],
    fixture_ids: [],
    enemy_variant_ids: [],
    encounter_member_ids: [],
    native_review_candidate_rule_ids: [],
    admitted_native_handler_ids: [],
    workbook_families: [],
    focused_commands: [],
    expected: {},
  };
}

function finalize(batches) {
  for (const batch of batches) {
    batch.record_entries.sort(byStableId);
    batch.rule_entries.sort(byStableId);
    batch.fixture_entries.sort(byStableId);
    batch.enemy_entries.sort(byStableId);
    batch.encounter_member_entries.sort(byStableId);
    batch.logical_groups.sort();
    batch.record_ids = assign("records", batch.record_entries, batch.id);
    batch.rule_ids = assign("rules", batch.rule_entries, batch.id);
    batch.fixture_ids = assign("fixtures", batch.fixture_entries, batch.id);
    batch.enemy_variant_ids = assign("enemy_variants", batch.enemy_entries, batch.id);
    batch.encounter_member_ids = assign(
      "encounter_members", batch.encounter_member_entries, batch.id,
    );
    batch.native_review_candidate_rule_ids = batch.rule_entries
      .filter(({ native_review_candidate: candidate }) => candidate)
      .map(({ id }) => id);
    delete batch.record_entries;
    delete batch.rule_entries;
    delete batch.fixture_entries;
    delete batch.enemy_entries;
    delete batch.encounter_member_entries;
    partitions.push(batch);
  }
}

function assign(kind, entries, partition) {
  for (const { id } of entries) {
    assert(!assigned[kind].has(id), `${kind} ${id} assigned twice`);
    assigned[kind].set(id, partition);
  }
  return entries.map(({ id }) => id);
}
function verifyAssignment(kind, entries) {
  assert(assigned[kind].size === entries.length,
    `${kind}: expected ${entries.length} assignments, got ${assigned[kind].size}`);
  for (const { id } of entries)
    assert(assigned[kind].has(id), `${kind} ${id} is unassigned`);
}
function verifyCaps() {
  for (const partition of partitions) {
    assert(partition.rule_ids.length <= policy.caps.mechanic_rules_per_batch,
      `${partition.id}: rule cap exceeded`);
    if (partition.lane === "noncombat-occurrence"
      || partition.lane === "noncombat-service")
      assert(partition.record_ids.length
        <= policy.caps.effect_bearing_noncombat_records_per_batch,
      `${partition.id}: noncombat record cap exceeded`);
    if (partition.lane === "enemy-ordinary")
      assert(partition.enemy_variant_ids.length
        <= policy.caps.ordinary_enemy_variants_per_batch,
      `${partition.id}: ordinary enemy cap exceeded`);
    if (partition.lane === "enemy-boss")
      assert(partition.enemy_variant_ids.length === policy.caps.boss_variants_per_batch,
        `${partition.id}: boss isolation drift`);
    if (partition.lane === "topology-map")
      assert(partition.record_ids.length <= policy.caps.topology_metadata_rows_per_batch,
        `${partition.id}: topology row cap exceeded`);
    assert(partition.admitted_native_handler_ids.length
      <= policy.caps.new_native_handler_admissions_per_batch,
    `${partition.id}: native-handler admission cap exceeded`);
  }
}
function workbookFamilies(partition) {
  const sourceByRecord = new Map(audit.records.map((entry) =>
    [entry.id, entry.source_category]));
  const families = new Set(partition.record_ids.map((id) => sourceByRecord.get(id)));
  if (partition.enemy_variant_ids.length > 0) families.add("enemy-definitions");
  if (partition.encounter_member_ids.length > 0) families.add("encounter-members");
  return [...families].filter(Boolean).sort();
}
function ledger(manifest) {
  const lines = [
    "# Goal 07 Expanded Content Batch Ledger",
    "",
    "This file is generated by `tools/goal07/generate-partitions.mjs`. Each row is an atomic commit.",
    "",
    "| Batch | State | Milestone | Lane | Records | Rules | Fixtures | Enemies | Members | Depends on |",
    "|---|---|---|---|---:|---:|---:|---:|---:|---|",
  ];
  for (const entry of manifest.partitions)
    lines.push(
      `| \`${entry.id}\` | \`Pending\` | \`${entry.milestone}\` | ` +
      `\`${entry.lane}\` | ${entry.expected.records} | ${entry.expected.rules} | ` +
      `${entry.expected.fixtures} | ${entry.expected.enemy_variants} | ` +
      `${entry.expected.encounter_members} | ` +
      `${entry.dependencies.map((id) => `\`${id}\``).join(", ") || "—"} |`,
    );
  lines.push("");
  return `${lines.join("\n")}\n`;
}
function chunks(entries, size) {
  const result = [];
  for (let index = 0; index < entries.length; index += size)
    result.push(entries.slice(index, index + size));
  return result;
}
function groupBy(entries, field) {
  const result = new Map();
  for (const entry of entries) {
    const values = result.get(entry[field]) ?? [];
    values.push(entry);
    result.set(entry[field], values);
  }
  return result;
}
function countBy(entries, field) {
  const result = {};
  for (const entry of entries)
    result[entry[field]] = (result[entry[field]] ?? 0) + 1;
  return Object.fromEntries(Object.entries(result).sort(([left], [right]) =>
    left.localeCompare(right)));
}
function tableRows(relative) {
  return json(relative).table.rows.map(({ values }) => values);
}
function byId(entries) {
  return new Map(entries.map((entry) => [entry.id, entry]));
}
function integer(value) {
  assert(value && Number.isInteger(value.Integer), "expected encoded integer");
  return value.Integer;
}
function string(value) {
  assert(value && typeof value.String === "string", "expected encoded string");
  return value.String;
}
function required(collection, key, label) {
  const value = collection instanceof Map ? collection.get(key) : collection[key];
  assert(value !== undefined, `${label} is missing ${key}`);
  return value;
}
function byStableId(left, right) {
  return left.id.localeCompare(right.id);
}
function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}
function writeOrCheck(relative, value) {
  const file = path.join(root, relative);
  if (check) {
    assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(),
      `${relative} is missing; run without --check`);
    assert(fs.readFileSync(file, "utf8") === value, `${relative} has generated drift`);
  } else {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, value);
  }
}
function encode(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return digest(fs.readFileSync(path.join(root, relative)));
}
function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
