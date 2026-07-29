#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  canonical,
  sha256,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const packRoot = path.join(root, "content-reference", "swarm-disaster-v1");
const evidencePath = path.join(
  root,
  "evidence",
  "swarm-disaster-reference-v1",
  "release-audit.json",
);
const schema = json(
  path.join(root, "content-manifests/swarm-disaster-v1/normalized-schema.json"),
);
const sourceManifest = json(
  path.join(root, "content-manifests/swarm-disaster-v1/content-manifest.json"),
);
const pack = new Map(schema.files.map(({ file }) => [
  file,
  json(path.join(packRoot, file)),
]));
const sourceRows = rows("sources.json");
const sourceById = uniqueIndex(sourceRows, "source registry");
const globalRows = [];
const rowById = new Map();
const rowByFile = new Map();
const commonRows = [];
const usedSources = new Set();
const qualityCounts = {};
const ownershipCounts = {};
let sourceReferenceCount = 0;

assert(schema.files.length === 64, "normalized file denominator differs");
assert(
  JSON.stringify([...pack.keys()])
    === JSON.stringify(schema.files.map(({ file }) => file)),
  "normalized file order differs",
);

for (const contract of schema.files) {
  const value = pack.get(contract.file);
  const values = Array.isArray(value) ? value : [value];
  rowByFile.set(contract.file, new Map());
  for (const row of values) {
    assert(row && typeof row === "object", `${contract.file} has non-row data`);
    assert(typeof row.id === "string", `${contract.file} row lacks stable ID`);
    assert(!rowById.has(row.id), `duplicate global stable ID ${row.id}`);
    rowById.set(row.id, { file: contract.file, row });
    rowByFile.get(contract.file).set(row.id, row);
    globalRows.push(row);
    if (row.schema_revision !== schema.common_envelope.schema_revision?.value
      && row.schema_revision !== "starclock.swarm-disaster-row.v1")
      continue;
    commonRows.push({ file: contract.file, row });
    auditCommonRow(contract, row);
  }
}

auditSources();
auditCoverage();
const referenceCounts = auditStableReferences();
const boundaryCounts = auditModeBoundaries();

assert(
  usedSources.size === sourceRows.length,
  `orphan provenance rows: ${sourceRows.length - usedSources.size}`,
);
assert(
  globalRows.length === 27_820
    && rowById.size === globalRows.length
    && commonRows.length === 19_617,
  "normalized/global row denominator differs",
);
assert(
  ownershipCounts.SwarmDisaster === 17_688
    && ownershipCounts.Shared === 1_929
    && Object.keys(ownershipCounts).length === 2,
  "ownership denominator or label differs",
);
assert(
  qualityCounts.ExactStructured === 15_503
    && qualityCounts.ProjectPolicy === 4_113
    && qualityCounts.ApproximateFromReleasedText === 1
    && Object.keys(qualityCounts).length === 3,
  "row evidence-quality denominator or label differs",
);

const report = {
  schema_revision: "starclock.swarm-disaster-release-audit.v1",
  goal_id: "swarm-disaster-reference-v1",
  snapshot: "Version 4.4",
  normalized_schema_sha256: sha256(fs.readFileSync(
    path.join(root, "content-manifests/swarm-disaster-v1/normalized-schema.json"),
  )),
  content_manifest_sha256: sha256(fs.readFileSync(
    path.join(root, "content-manifests/swarm-disaster-v1/content-manifest.json"),
  )),
  denominators: {
    normalized_files: schema.files.length,
    normalized_records: globalRows.length,
    common_envelope_records: commonRows.length,
    frozen_manifest_obligations: 6_963,
    coverage_records: rows("coverage.json").length,
    source_records: sourceRows.length,
    source_reference_occurrences: sourceReferenceCount,
    bilingual_records: commonRows.length,
  },
  ownership: sortedObject(ownershipCounts),
  evidence_quality: sortedObject(qualityCounts),
  references: referenceCounts,
  fail_closed_boundaries: boundaryCounts,
  checks: {
    exact_once_coverage: true,
    global_ids_unique: true,
    local_references_resolve_or_are_declared_boundaries: true,
    inherited_references_resolve_or_are_declared_locators: true,
    source_references_resolve: true,
    orphan_sources: 0,
    bilingual_fields_complete: true,
    ownership_labels_closed: true,
    evidence_quality_labels_closed: true,
    approximation_sources_replaceable: true,
    gold_rows_enabled_for_swarm: 0,
    unresolved_shared_dlc_rows_enabled_for_swarm: 0,
    gold_topology_rows: 0,
    gold_encounter_namespaces: 0,
    erudition_references: 0,
    unknowable_or_divergent_rows: 0,
  },
};
writeOrCheckEvidence(report);
console.log(
  `Swarm Disaster release audit ${write ? "written" : "verified"}: ` +
  `${globalRows.length} normalized records, ${commonRows.length} bilingual ` +
  `rows, 6,963 exact-once obligations, ${sourceRows.length} provenance rows.`,
);

function auditCommonRow(contract, row) {
  for (const field of schema.common_envelope.required_fields)
    assert(Object.hasOwn(row, field), `${contract.file}/${row.id} lacks ${field}`);
  if (contract.file !== "profiles.json" || row.kind === "SwarmProfile")
    for (const field of contract.required_domain_fields)
      assert(Object.hasOwn(row, field),
        `${contract.file}/${row.id} lacks domain field ${field}`);
  assert(
    row.schema_revision === "starclock.swarm-disaster-row.v1",
    `${contract.file}/${row.id} row revision differs`,
  );
  const allowedKinds = contract.file === "profiles.json"
    ? ["EntryPoint", "SwarmProfile"]
    : contract.file === "resonances.json"
      ? ["Formation", "Resonance"]
      : [contract.record_kind];
  assert(allowedKinds.includes(row.kind),
    `${contract.file}/${row.id} kind differs`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim(),
      `${contract.file}/${row.id} lacks bilingual ${field}`);
  assert(["SwarmDisaster", "Shared"].includes(row.ownership),
    `${contract.file}/${row.id} has forbidden ownership ${row.ownership}`);
  assert(row.coverage_state === "DataReady",
    `${contract.file}/${row.id} is not DataReady`);
  assert(schema.common_envelope.evidence_quality.enum.includes(
    row.evidence_quality,
  ), `${contract.file}/${row.id} has invalid evidence quality`);
  assert(
    Array.isArray(row.tags)
      && new Set(row.tags).size === row.tags.length
      && JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${contract.file}/${row.id} tags are not canonical`,
  );
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${contract.file}/${row.id} lacks provenance`);
  let matchingQuality = false;
  for (const ref of row.source_refs) {
    auditSourceRef(contract.file, row.id, ref);
    sourceReferenceCount += 1;
    usedSources.add(ref.source_id);
    if (ref.evidence_quality === row.evidence_quality) matchingQuality = true;
  }
  assert(matchingQuality,
    `${contract.file}/${row.id} row quality lacks matching evidence`);
  ownershipCounts[row.ownership] = (ownershipCounts[row.ownership] ?? 0) + 1;
  qualityCounts[row.evidence_quality] =
    (qualityCounts[row.evidence_quality] ?? 0) + 1;
}

function auditSourceRef(file, id, ref) {
  const source = sourceById.get(ref.source_id);
  assert(source, `${file}/${id} references unknown source ${ref.source_id}`);
  for (const field of [
    "repository",
    "revision",
    "path",
    "locator",
    "sha256",
    "access_date",
    "evidence_quality",
  ])
    assert(ref[field] === source[field],
      `${file}/${id} source ${ref.source_id} differs at ${field}`);
  if (["ApproximateFromReleasedText", "ProjectPolicy"].includes(
    ref.evidence_quality,
  ))
    assert(
      ref.note?.trim()
        && ref.replacement_condition?.trim()
        && ref.note === source.note
        && ref.replacement_condition === source.replacement_condition,
      `${file}/${id} source ${ref.source_id} is not replaceable`,
    );
}

function auditSources() {
  const qualities = new Set(schema.common_envelope.evidence_quality.enum);
  for (const source of sourceRows) {
    assert(
      source.schema_revision === "starclock.swarm-disaster-source.v1"
        && source.kind === "SourceRecord"
        && source.id === source.source_id,
      `${source.id} source identity differs`,
    );
    for (const field of [
      "repository",
      "revision",
      "game_version",
      "path",
      "locator",
      "sha256",
      "access_date",
    ])
      assert(typeof source[field] === "string" && source[field].trim(),
        `${source.id} lacks ${field}`);
    assert(source.game_version === "4.4", `${source.id} game version differs`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${source.id} digest is invalid`);
    assert(qualities.has(source.evidence_quality),
      `${source.id} evidence quality is invalid`);
    if (["ApproximateFromReleasedText", "ProjectPolicy"].includes(
      source.evidence_quality,
    ))
      assert(source.note?.trim() && source.replacement_condition?.trim(),
        `${source.id} approximation is not replaceable`);
  }
}

function auditCoverage() {
  const coverage = rows("coverage.json");
  const byObligation = new Map();
  for (const row of coverage) {
    const key = `${row.manifest_category}\0${row.manifest_record_id}`;
    assert(!byObligation.has(key), `duplicate coverage assignment ${key}`);
    byObligation.set(key, row);
    assert(
      row.coverage_state === "DataReady"
        && row.blocking_gap_ids.length === 0
        && row.normalized_refs.length > 0,
      `${row.id} does not close DataReady`,
    );
    for (const ref of row.normalized_refs)
      assert(rowByFile.get(ref.file)?.has(ref.id),
        `${row.id} has unresolved normalized ref ${ref.file}/${ref.id}`);
  }
  let manifestCount = 0;
  for (const [categoryId, category] of Object.entries(sourceManifest.categories))
    for (const record of category.records) {
      manifestCount += 1;
      assert(["SwarmDisaster", "Shared"].includes(record.ownership),
        `${categoryId}/${record.id} has forbidden manifest ownership`);
      assert([
        "Direct",
        "Referenced",
        "InheritedSharedPool",
      ].includes(record.reachability),
      `${categoryId}/${record.id} has forbidden reachability`);
      const row = byObligation.get(`${categoryId}\0${record.id}`);
      assert(
        row
          && row.source_locator === record.source
          && row.source_evidence_sha256 === record.evidence_sha256
          && row.ownership === record.ownership,
        `${categoryId}/${record.id} coverage evidence differs`,
      );
    }
  assert(
    manifestCount === 6_963
      && coverage.length === manifestCount
      && byObligation.size === manifestCount,
    "manifest coverage is not exact-once",
  );
}

function auditStableReferences() {
  const standardIds = collectIds(
    path.join(root, "content-reference", "standard-universe-v1"),
  );
  const enemyIds = new Set(json(
    path.join(root, "content-reference/v4.4/enemy-variants.json"),
  ).map(({ id }) => id));
  const counts = {
    local_resolved_occurrences: 0,
    local_declared_boundary_occurrences: 0,
    inherited_resolved_occurrences: 0,
    inherited_declared_locator_occurrences: 0,
    enemy_resolved_occurrences: 0,
  };
  for (const [file, value] of pack)
    walkStrings(value, [], (candidate, segments) => {
      if (candidate.startsWith("swarm-disaster.")) {
        if (rowById.has(candidate)) counts.local_resolved_occurrences += 1;
        else {
          assert(isDeclaredLocalBoundary(file, segments, candidate),
            `${file}/${segments.join(".")} has dangling local ref ${candidate}`);
          counts.local_declared_boundary_occurrences += 1;
        }
      } else if (candidate.startsWith("universe.")) {
        if (standardIds.has(candidate))
          counts.inherited_resolved_occurrences += 1;
        else {
          assert(isDeclaredInheritedLocator(file, segments, candidate),
            `${file}/${segments.join(".")} has unknown inherited ref ${candidate}`);
          counts.inherited_declared_locator_occurrences += 1;
        }
      } else if (candidate.startsWith("enemy.")) {
        assert(enemyIds.has(candidate),
          `${file}/${segments.join(".")} has unknown enemy ${candidate}`);
        counts.enemy_resolved_occurrences += 1;
      }
    });
  return counts;
}

function isDeclaredLocalBoundary(file, segments, value) {
  const field = segments.at(-1);
  if (file === "adventure-outcomes.json")
    return (field === "payload_schema"
        && value === "swarm-disaster.external-adventure-reward.v1")
      || (field === "blessing_pool_id"
        && value === "swarm-disaster.pool.blessings")
      || (field === "curio_pool_prefix"
        && value === "swarm-disaster.curio-pool.");
  if (field === "pool_id")
    return /^swarm-disaster\.(?:pool|curio-pool|occurrence-pool)\./u
      .test(value);
  if (file === "communing-choices.json" && field === "counter_id")
    return /^swarm-disaster\.aeon-choice-counter\.[1-7]$/u.test(value);
  if (file === "curio-rules.json" && field === "random_stream")
    return /^swarm-disaster\.curio-replacement\.[0-9]+$/u.test(value);
  if (file === "pathstrider-objectives.json"
    && field === "finish_condition_id")
    return /^swarm-disaster\.external-quest-condition\.[0-9]+$/u.test(value);
  if (file === "pathstrider-unlocks.json" && field === "unlock_flag_id")
    return /^swarm-disaster\.dlc-unlock-flag\.[0-9]+$/u.test(value);
  return false;
}

function isDeclaredInheritedLocator(file, segments, value) {
  const field = segments.at(-1);
  return (file === "blessings.json"
      && field === "prerequisite_ids"
      && /^universe\.unlock\.source-[0-9]+$/u.test(value))
    || (file === "services.json"
      && ["inherited_price_formula_id", "inherited_offer_pool_id"]
        .includes(field)
      && /^universe\.(?:price|service|pool)\./u.test(value));
}

function auditModeBoundaries() {
  const finish = rows("pathstrider-finish-conditions.json");
  const unlocks = rows("pathstrider-unlocks.json");
  for (const row of finish) {
    const enabled = row.mode_hint === "SwarmDisaster";
    assert(row.enabled_for_swarm_compilation === enabled,
      `${row.id} finish-condition applicability leaks`);
  }
  for (const row of unlocks) {
    const enabled = row.mode_hint === "SwarmDisaster";
    assert(row.unlock_consequence.enabled_for_swarm_compilation === enabled,
      `${row.id} unlock applicability leaks`);
  }
  const goldFinish = finish.filter(({ mode_hint: hint }) =>
    hint === "GoldAndGears");
  const goldUnlock = unlocks.filter(({ mode_hint: hint }) =>
    hint === "GoldAndGears");
  const unresolvedFinish = finish.filter(({ mode_hint: hint }) =>
    hint === "UnresolvedSharedDlc");
  const unresolvedUnlock = unlocks.filter(({ mode_hint: hint }) =>
    hint === "UnresolvedSharedDlc");
  assert(
    goldFinish.length === 45
      && goldUnlock.length === 51
      && unresolvedFinish.length === 42
      && unresolvedUnlock.length === 44,
    "shared-DLC fail-closed denominator differs",
  );
  const disabledDecay = rows("boss-decay-levels.json").filter(
    ({ swarm_applicability: state }) =>
      state === "DisabledUnprovenSharedDlcRow",
  );
  assert(disabledDecay.length === 27,
    "unproven boss-decay boundary differs");
  assert(rows("chessboards.json").every(({ source_config_path: value }) =>
    !value.includes("MapRepo160") && !value.includes("ChessRogueNous")),
  "Gold topology leaked into Swarm pack");
  assert(rows("encounter-groups.json").every(
    ({ source_namespace: namespace }) =>
      namespace === "SwarmDisaster81Series",
  ), "non-Swarm encounter namespace leaked");
  assert(
    !canonical([...pack.values()]).includes("universe.path.erudition"),
    "Erudition leaked into Swarm pack",
  );
  assert(rows("reconciliation-receipts.json").every(
    ({ outcome }) => outcome === "MatchedSharedFact",
  ), "divergent reconciliation receipt remains");
  let forbiddenClassifications = 0;
  walkStrings([...pack.values()], [], (value) => {
    if ([
      "Unknowable",
      "DivergentRepresentation",
      "Conflict",
      "GoldOnly",
    ].includes(value))
      forbiddenClassifications += 1;
  });
  assert(forbiddenClassifications === 0,
    "unknowable, divergent or conflicting classification remains");
  return {
    gold_finish_conditions_disabled: goldFinish.length,
    gold_unlocks_disabled: goldUnlock.length,
    unresolved_shared_finish_conditions_disabled: unresolvedFinish.length,
    unresolved_shared_unlocks_disabled: unresolvedUnlock.length,
    unproven_shared_boss_decay_rows_disabled: disabledDecay.length,
  };
}

function collectIds(directory) {
  const ids = new Set();
  for (const file of fs.readdirSync(directory).filter(
    (name) => name.endsWith(".json"),
  )) {
    const value = json(path.join(directory, file));
    for (const row of Array.isArray(value) ? value : [value])
      if (typeof row?.id === "string") ids.add(row.id);
  }
  return ids;
}

function walkStrings(value, segments, visit) {
  if (typeof value === "string") {
    visit(value, segments);
    return;
  }
  if (Array.isArray(value)) {
    for (const item of value) walkStrings(item, segments, visit);
    return;
  }
  if (value && typeof value === "object")
    for (const [key, item] of Object.entries(value))
      walkStrings(item, [...segments, key], visit);
}

function uniqueIndex(values, label) {
  const result = new Map();
  for (const value of values) {
    assert(!result.has(value.id), `${label} contains duplicate ${value.id}`);
    result.set(value.id, value);
  }
  return result;
}

function sortedObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function writeOrCheckEvidence(value) {
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (write) {
    fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
    fs.writeFileSync(evidencePath, encoded);
    return;
  }
  assert(fs.readFileSync(evidencePath, "utf8") === encoded,
    "release-audit evidence has generated drift");
}

function rows(file) {
  const value = pack.get(file);
  assert(Array.isArray(value), `${file} is not a row array`);
  return value;
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
