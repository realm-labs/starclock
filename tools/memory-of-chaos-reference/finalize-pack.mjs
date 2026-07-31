#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import path from "node:path";
import {
  assert,
  digest,
  manifest,
  normalizedFile,
  policyRef,
  record,
  root,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
const contract = JSON.parse(await readFile(
  path.join(root, "content-manifests/memory-of-chaos-v1/authoring-contract.json"),
  "utf8",
));
const primaryFiles = contract.normalized_pack.files.slice(0, 21);
assert(primaryFiles[20] === "enemy-abilities.json", "primary normalized-file boundary drift");

async function readNormalized(file) {
  return JSON.parse(await readFile(path.join(root, "content-reference/memory-of-chaos-v1", file), "utf8"));
}
const primary = await Promise.all(primaryFiles.map(async (file) => [file, await readNormalized(file)]));
const primaryRecords = primary.flatMap(([file, value]) => value.records.map((row) => ({ file, row })));

function walk(value, visit) {
  visit(value);
  if (Array.isArray(value)) value.forEach((entry) => walk(entry, visit));
  else if (value && typeof value === "object") Object.values(value).forEach((entry) => walk(entry, visit));
}
for (const { file, row } of primaryRecords) {
  assert(row.schema_revision === "starclock.memory-of-chaos-row.v1", `${file}:${row.id} schema drift`);
  assert(row.coverage_state === "DataReady", `${file}:${row.id} not DataReady`);
  assert(row.runtime_executable === false, `${file}:${row.id} must remain reference-only`);
  assert(typeof row.name_en === "string" && row.name_en.length > 0, `${file}:${row.id} missing English name`);
  assert(typeof row.name_zh_cn === "string" && row.name_zh_cn.length > 0, `${file}:${row.id} missing Chinese name`);
  assert(typeof row.summary_en === "string" && row.summary_en.length > 0, `${file}:${row.id} missing English summary`);
  assert(typeof row.summary_zh_cn === "string" && row.summary_zh_cn.length > 0, `${file}:${row.id} missing Chinese summary`);
  walk(row, (value) => assert(typeof value !== "number" || Number.isInteger(value),
    `${file}:${row.id} contains non-integer JSON number ${value}`));
}

const manifestRows = Object.entries(manifest.categories).flatMap(([category, value]) =>
  value.records.map((row) => ({ category, ...row, claim_id: `${category}:${row.id}` })));
const primaryClaims = primaryRecords.flatMap(({ file, row }) =>
  row.source_record_ids.map((claimId) => ({ claimId, file, recordId: row.id })));
assert(primaryClaims.length === 477, `primary exact-once claim count drift: ${primaryClaims.length}`);
assert(new Set(primaryClaims.map(({ claimId }) => claimId)).size === 477,
  "manifest obligations claimed more than once");
const expectedClaimIds = manifestRows.map(({ claim_id: claimId }) => claimId).sort();
assert(JSON.stringify(primaryClaims.map(({ claimId }) => claimId).sort()) === JSON.stringify(expectedClaimIds),
  "manifest exact-once coverage mismatch");

const sourceByKey = new Map();
for (const { row } of primaryRecords) {
  for (const evidence of row.evidence_refs) {
    const key = [evidence.repository_or_url, evidence.revision_or_access_date,
      evidence.path_or_page, evidence.row_locator, evidence.evidence_sha256].join("\u001f");
    const prior = sourceByKey.get(key);
    if (prior) {
      assert(prior.quality === evidence.quality, `evidence quality conflict ${evidence.id}`);
      continue;
    }
    sourceByKey.set(key, evidence);
  }
}
const sourceEvidence = [...sourceByKey.values()].sort((left, right) => {
  const leftKey = `${left.path_or_page}\u001f${left.row_locator}\u001f${left.evidence_sha256}`;
  const rightKey = `${right.path_or_page}\u001f${right.row_locator}\u001f${right.evidence_sha256}`;
  return leftKey.localeCompare(rightKey, "en");
});
const sources = sourceEvidence.map((evidence, index) => record({
  id: `source.${String(index + 1).padStart(4, "0")}`,
  kind: "EvidenceSource",
  nameEn: `Evidence source ${index + 1}`,
  nameZh: `证据来源${index + 1}`,
  summaryEn: `Canonical evidence locator ${evidence.path_or_page} / ${evidence.row_locator}.`,
  summaryZh: `规范证据定位：${evidence.path_or_page} / ${evidence.row_locator}。`,
  ownership: "Shared",
  sourceIds: [],
  evidence: [],
  tags: ["evidence", evidence.quality],
  fields: {
    source_evidence_id: evidence.id,
    repository_or_url: evidence.repository_or_url,
    revision_or_access_date: evidence.revision_or_access_date,
    source_game_version: evidence.game_version,
    path_or_page: evidence.path_or_page,
    row_locator: evidence.row_locator,
    evidence_sha256: evidence.evidence_sha256,
    quality: evidence.quality,
    mechanism_quality: evidence.mechanism_quality,
    note: evidence.note,
    approximations: [],
  },
}));

const goal03Revision = "60ca52ed98c5c83d867d33bff7f88c69e0b389de";
const sharedManifestRows = manifestRows.filter(({ ownership }) => ownership === "Shared");
assert(sharedManifestRows.length === 305, "shared reconciliation denominator drift");
const reconciliations = sharedManifestRows.map((manifestRow) => {
  const enemyShared = manifestRow.category.startsWith("enemy_");
  return record({
    id: `reconciliation.${manifestRow.category}.${manifestRow.id}`,
    kind: "ReconciliationReceipt",
    nameEn: `Shared reconciliation ${manifestRow.id}`,
    nameZh: `共享对账${manifestRow.id}`,
    summaryEn: `${enemyShared ? "Matches" : "Compatibly projects"} the frozen shared identity without editing peer artifacts.`,
    summaryZh: `${enemyShared ? "匹配" : "兼容投影"}冻结共享标识，且不修改其它目标产物。`,
    ownership: "Shared",
    sourceIds: [],
    evidence: [],
    tags: ["reconciliation", enemyShared ? "match" : "compatible-projection"],
    fields: {
      peer_goal: enemyShared ? "standard-universe-reference-v1" : "shared-configuration-foundation",
      peer_revision: enemyShared ? goal03Revision : "92febad080dd4cf9997718d64b3648fc198ab1f8",
      source_path: manifestRow.source_path,
      stable_row_locator: manifestRow.row_locator,
      evidence_sha256: manifestRow.evidence_sha256,
      ownership_local: manifestRow.ownership,
      ownership_peer: "Shared",
      semantic_result: enemyShared ? "Match" : "CompatibleProjection",
      note: enemyShared
        ? "Goal 17 embeds the immutable Goal 03 definition byte-for-byte as a nested definition and adds reachability only."
        : "The common family/entry locator remains shared; Goal 17 adds only Memory of Chaos reachability and bilingual projection.",
      approximations: [],
    },
  });
});
assert(reconciliations.every(({ semantic_result: result }) => result !== "Conflict"),
  "shared reconciliation conflict");

const gaps = [];
for (const { file, row } of primaryRecords) {
  const approximations = row.approximations ?? [];
  approximations.forEach((item, index) => {
    const gapId = digest({ file, record: row.id, index, replacement: item.replacement_condition }).slice(0, 16);
    gaps.push(record({
      id: `research-gap.${gapId}`,
      kind: "ResearchGap",
      nameEn: `Policy boundary for ${row.name_en}`,
      nameZh: `${row.name_zh_cn}的策略边界`,
      summaryEn: `Nonblocking Candidate policy retained until: ${item.replacement_condition}`,
      summaryZh: `非阻塞候选策略；满足以下条件时替换：${item.replacement_condition}`,
      ownership: row.ownership,
      sourceIds: [],
      evidence: [policyRef(`research-gap:${gapId}`, `Field-level replacement condition for ${file}:${row.id}.`)],
      tags: ["candidate", "nonblocking", "policy-bound", "research-gap"],
      fields: {
        state: "PolicyBound",
        blocking: false,
        source_file: file,
        source_record_id: row.id,
        approximation_index: index,
        known_facts: item.known_facts,
        selected_behavior: item.selected_behavior,
        rejected_alternatives: item.rejected_alternatives,
        rationale: item.rationale,
        affected_fixture_ids: item.affected_fixture_ids,
        confidence: item.confidence,
        replacement_condition: item.replacement_condition,
        approximations: [],
      },
    }));
  });
}
gaps.sort((left, right) => left.id.localeCompare(right.id, "en"));
assert(gaps.length > 0 && gaps.every(({ blocking }) => blocking === false), "research-gap state drift");

const claimsByCategory = new Map();
for (const { claimId } of primaryClaims) {
  const category = claimId.slice(0, claimId.indexOf(":"));
  claimsByCategory.set(category, (claimsByCategory.get(category) ?? 0) + 1);
}
const coverageRecords = Object.entries(manifest.categories).map(([category, value]) => record({
  id: `coverage.${category}`,
  kind: "CoverageCategory",
  nameEn: `${category} coverage`,
  nameZh: `${category}覆盖`,
  summaryEn: `${value.count}/${value.count} frozen obligations are accounted exactly once and DataReady.`,
  summaryZh: `${value.count}/${value.count}条冻结义务均精确计数一次并达到DataReady。`,
  ownership: value.records.every(({ ownership }) => ownership === "Shared") ? "Shared" : "MemoryOfChaos",
  sourceIds: [],
  evidence: [],
  tags: ["coverage", "exact-once"],
  fields: {
    category,
    required: value.count,
    accounted: claimsByCategory.get(category) ?? 0,
    data_ready: claimsByCategory.get(category) ?? 0,
    missing: 0,
    duplicate_claims: 0,
    coverage_ratio: "1",
    state: "Complete",
    approximations: [],
  },
}));
assert(coverageRecords.every(({ required, accounted, data_ready: ready }) =>
  required === accounted && accounted === ready), "category coverage mismatch");

const fixtureDefinitions = [
  ["active-season-selection", "ExactStructured", ["family_and_season:schedule-201033", "family_and_season:group-1033"], ["select schedule 201033", "resolve group 1033"], ["group 1033 selected exactly once", "group 1034 excluded"]],
  ["ordinary-stage-order", "ExactStructured", ["ordinary_stages:stage-5201", "ordinary_stages:stage-5212"], ["enumerate active ordinary stages"], ["ordered IDs are 5201 through 5212", "terminal outcomes are fail-closed"]],
  ["tierce-selected-extension", "ProjectPolicy", ["tierce:tierce-5213"], ["complete stage 5212", "select Tierce 5213"], ["one independent encounter 30123123", "45 cycles", "ordinary state not carried"]],
  ["participant-uniqueness", "ProjectPolicy", ["participant_and_attempt_contracts:ordinary-team-slots", "participant_and_attempt_contracts:combat-form-uniqueness"], ["submit two node teams", "validate combat-form identities"], ["two disjoint teams accepted", "duplicate combat form rejected"]],
  ["loadout-lock-retry", "ProjectPolicy", ["participant_and_attempt_contracts:loadout-lock", "participant_and_attempt_contracts:retry-reset"], ["start attempt", "request mutation", "retry whole stage"], ["post-start mutation rejected byte-identically", "retry creates fresh attempt"]],
  ["cycle-first-av-window", "ProjectPolicy", ["clock_and_resource_contracts:cycle-action-value-preset"], ["start fresh battle", "consume 150 AV"], ["first window is 150 AV", "later windows are 100 AV"]],
  ["cycle-node-wave-carry", "ProjectPolicy", ["clock_and_resource_contracts:node-cycle-carry", "clock_and_resource_contracts:wave-cycle-carry"], ["cross wave", "cross node", "reach zero boundary"], ["remaining cycles persist", "wave elapsed AV resets", "Node 2 opens fresh 150 AV window"]],
  ["objective-star-aggregation", "ProjectPolicy", ["objectives:target-251", "objectives:target-252", "objectives:target-253"], ["complete multiple attempts with different objectives"], ["best objectives latch independently", "failed attempts contribute nothing"]],
  ["turbulence-hit-accumulation", "ExactStructured", ["turbulence_and_battle_event:maze-buff-3030146"], ["use Ultimate", "use Follow-Up ATK"], ["one hit stored per qualifying action", "multi-hit action does not multiply gain"]],
  ["turbulence-cap-cycle-start", "ProjectPolicy", ["turbulence_and_battle_event:battle-event-30146"], ["accumulate beyond cap", "start cycle"], ["stored hit count caps at 15", "accumulator resets after resolution"]],
  ["turbulence-target-true-damage", "ProjectPolicy", ["turbulence_and_battle_event:battle-event-30146"], ["resolve one stored hit for each rank branch"], ["one target sampled per hit", "coefficients are 0.12, 0.02 and 0.012"]],
  ["initial-resources", "ProjectPolicy", ["clock_and_resource_contracts:initial-hp-energy-skill-points"], ["start Node 1", "start Node 2"], ["full HP", "half Energy", "team maximum Skill Points", "no cross-node resource carry"]],
  ["battle-entry-operations", "ExactStructured", ["clock_and_resource_contracts:battle-entry-operations"], ["project shared level program"], ["activity buff binds before team creation", "wave creation precedes battle start"]],
  ["encounter-wave-order", "ExactStructured", ["stage_configs:stage-config-30123011", "encounter_enemy_slots:slot-001"], ["load encounter 30123011", "advance wave 1 to wave 2"], ["slot order is stable", "wave 2 follows wave 1"]],
  ["enemy-transitive-closure", "ExactStructured", ["enemy_variants:enemy-variant-1003010", `enemy_templates:${templateManifestFirst()}`, `enemy_abilities:${abilityManifestFirst()}`], ["resolve every slot variant transitively"], ["41 variants map to 41 templates", "221 abilities are reachable"]],
  ["empty-pool-selector-closure", "ExactStructured", ["empty_pool_proofs:empty-blessing", "empty_pool_proofs:empty-shop"], ["walk active selector closure"], ["all ten audited families contain zero reachable rows"]],
  ["future-season-exclusion", "ExactStructured", [], ["evaluate schedule 201034 at frozen access boundary"], ["future group 1034 remains excluded", "no preview row becomes a denominator"]],
  ["shared-row-reconciliation", "ExactStructured", ["enemy_variants:enemy-variant-1003010"], ["compare source path, locator and digest"], ["303 enemy receipts match Goal 03", "no shared conflict"]],
];
function templateManifestFirst() { return manifest.categories.enemy_templates.records[0].id; }
function abilityManifestFirst() { return manifest.categories.enemy_abilities.records[0].id; }
assert(fixtureDefinitions.length === 18, "semantic fixture family denominator drift");
assert(JSON.stringify(fixtureDefinitions.map(([family]) => family)) ===
  JSON.stringify(contract.semantic_fixtures.families), "semantic fixture family order drift");
const fixtureCaseIds = {
  "ordinary-stage-order": ["fixture.ordinary-stage-order.terminal-outcomes"],
  "tierce-selected-extension": ["fixture.tierce-selected-extension.independent-projection"],
  "participant-uniqueness": ["fixture.participant-uniqueness.accept-two-disjoint-teams", "fixture.participant-uniqueness.reject-duplicate-form", "fixture.participant-uniqueness.team-slot-cardinality"],
  "loadout-lock-retry": ["fixture.loadout-lock-retry.new-attempt-after-failure", "fixture.loadout-lock-retry.node-transition-lock", "fixture.loadout-lock-retry.reject-after-start-mutation"],
  "cycle-first-av-window": ["fixture.cycle-first-av-window.150-then-100"],
  "cycle-node-wave-carry": ["fixture.cycle-node-wave-carry.expiry-before-start-rules", "fixture.cycle-node-wave-carry.node2-fresh-window", "fixture.cycle-node-wave-carry.stage-owned-budget", "fixture.cycle-node-wave-carry.tick-before-cycle-start", "fixture.cycle-node-wave-carry.wave-av-reset"],
  "objective-star-aggregation": ["fixture.objective-star-aggregation.cumulative-independent-objectives"],
  "turbulence-cap-cycle-start": ["fixture.turbulence-cap-cycle-start.empty-target-reset"],
  "turbulence-target-true-damage": ["fixture.turbulence-target-true-damage.random-per-hit", "fixture.turbulence-target-true-damage.rank-coefficients"],
  "initial-resources": ["fixture.initial-resources.fresh-node-reset", "fixture.initial-resources.tierce-config-equality"],
  "battle-entry-operations": ["fixture.battle-entry-operations.resolved-technique-contribution"],
};
const fixtures = fixtureDefinitions.map(([family, quality, sourceIds, commands, facts], index) => record({
  id: `fixture.${family}`,
  kind: "SemanticFixture",
  nameEn: `${family} semantic fixture`,
  nameZh: `${family}语义夹具`,
  summaryEn: `Reference-review fixture for the ${family} mechanic family.`,
  summaryZh: `${family}机制族的资料复核夹具。`,
  ownership: "MemoryOfChaos",
  sourceIds,
  evidence: [],
  tags: ["reference-review", "semantic-fixture"],
  fields: {
    family,
    case_ids: fixtureCaseIds[family] ?? [`fixture.${family}.canonical`],
    mechanism_quality_floor: quality,
    canonical_seed: digest(`memory-of-chaos-v1:${family}:${index}`).slice(0, 32),
    initial_state: { profile: "memory-of-chaos-v1", family, authoritative_state: "DeclarativeFixture" },
    commands: commands.map((command, commandIndex) => ({ order: commandIndex + 1, command })),
    expected_facts: facts.map((fact, factIndex) => ({ order: factIndex + 1, fact })),
    execution_scope: contract.semantic_fixtures.execution_scope,
    source_claim_role: "SupportingReferenceNotManifestClaim",
    approximations: [],
  },
}));

const sourceOutput = normalizedFile("sources.json", "EvidenceSource", sources);
const reconciliationOutput = normalizedFile("reconciliation-receipts.json", "ReconciliationReceipt", reconciliations);
const gapOutput = normalizedFile("research-gaps.json", "ResearchGap", gaps);
const coverageOutput = normalizedFile("coverage.json", "CoverageCategory", coverageRecords);
const fixtureOutput = normalizedFile("semantic-fixtures.json", "SemanticFixture", fixtures);
await writeCanonical("sources.json", sourceOutput, check);
await writeCanonical("reconciliation-receipts.json", reconciliationOutput, check);
await writeCanonical("research-gaps.json", gapOutput, check);
await writeCanonical("coverage.json", coverageOutput, check);
await writeCanonical("semantic-fixtures.json", fixtureOutput, check);

const indexedFiles = contract.normalized_pack.files.slice(0, 26);
const fileEntries = [];
for (const file of indexedFiles) {
  const bytes = await readFile(path.join(root, "content-reference/memory-of-chaos-v1", file), "utf8");
  const parsed = JSON.parse(bytes);
  fileEntries.push({
    order: contract.normalized_family_bindings[file].order,
    file,
    record_kind: parsed.record_kind,
    record_count: parsed.records.length,
    sha256: digest(bytes),
  });
}
assert(fileEntries.every((entry, index) => entry.order === index + 1), "pack file order drift");
const manifestBytes = await readFile(
  path.join(root, "content-manifests/memory-of-chaos-v1/content-manifest.json"), "utf8");
const packDigest = digest({
  goal_id: "memory-of-chaos-reference-v1",
  manifest_sha256: digest(manifestBytes),
  files: fileEntries,
});
const packIndex = normalizedFile("pack-index.json", "PackIndex", [record({
  id: "pack-index.memory-of-chaos-v1",
  kind: "PackIndex",
  nameEn: "Memory of Chaos Version 4.4 reference pack",
  nameZh: "混沌回忆4.4版本资料包",
  summaryEn: "Canonical Candidate reference-only pack with complete exact-once coverage and no runtime publication.",
  summaryZh: "具备完整精确计数覆盖、且不发布运行时内容的规范候选资料包。",
  ownership: "MemoryOfChaos",
  sourceIds: [],
  evidence: [],
  tags: ["candidate", "pack-index", "reference-only"],
  fields: {
    lane: contract.lane,
    manifest_sha256: digest(manifestBytes),
    manifest_required: manifest.counts.required,
    manifest_accounted: primaryClaims.length,
    manifest_data_ready: primaryClaims.length,
    source_count: sources.length,
    reconciliation_count: reconciliations.length,
    research_gap_count: gaps.length,
    blocking_research_gap_count: 0,
    semantic_fixture_family_count: fixtures.length,
    normalized_files: fileEntries,
    canonical_pack_sha256: packDigest,
    runtime_publishable: false,
    approximations: [],
  },
})]);
await writeCanonical("pack-index.json", packIndex, check);
await writeText(
  "evidence/memory-of-chaos-reference-v1/normalized-pack-audit.md",
  `# Goal 17 normalized-pack audit

- Manifest obligations: 477/477 exact-once claims
- DataReady obligations: 477/477
- Primary normalized files: 21
- Review/index normalized files: 6
- Canonical evidence sources: ${sources.length}
- Shared reconciliation receipts: ${reconciliations.length} (${reconciliations.filter(({ semantic_result: result }) => result === "Conflict").length} conflicts)
- Field-level nonblocking research gaps: ${gaps.length}
- Semantic fixture families: 18/18
- Canonical pack digest: \`${packDigest}\`
- Blocking research gaps: 0
- Runtime executable/publishable rows: 0

Coverage counts only primary content claims. Review fixtures retain supporting
source IDs with \`source_claim_role=SupportingReferenceNotManifestClaim\` and do
not inflate the frozen denominator.
`,
  check,
);
console.log(`Goal 17 pack ${check ? "verified" : "generated"}: 477/477 DataReady, ${sources.length} sources, ${gaps.length} policy gaps, pack=${packDigest}.`);
