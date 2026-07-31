#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import path from "node:path";
import { assert, compareText, digest, manifest, root, writeText } from "./lib.mjs";

const check = process.argv.includes("--check");
const packRoot = path.join(root, "content-reference/memory-of-chaos-v1");
const contract = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/memory-of-chaos-v1/authoring-contract.json",
), "utf8"));
const files = contract.normalized_pack.files;
assert(files.length === 27, `normalized file denominator drift: ${files.length}`);

const loaded = new Map();
for (const file of files) {
  const value = JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
  assert(value.file === file, `${file} self-identity drift`);
  assert(value.goal_id === "memory-of-chaos-reference-v1", `${file} goal identity drift`);
  assert(value.profile === "memory-of-chaos-v1", `${file} profile identity drift`);
  loaded.set(file, value);
}

const primaryFiles = files.slice(0, 21);
const primaryRows = primaryFiles.flatMap((file) => loaded.get(file).records.map((row) => ({ file, row })));
const allRows = files.flatMap((file) => loaded.get(file).records.map((row) => ({ file, row })));
const manifestRows = Object.entries(manifest.categories).flatMap(([category, value]) =>
  value.records.map((row) => ({ category, claim_id: `${category}:${row.id}`, ...row })));
const manifestByClaim = new Map(manifestRows.map((row) => [row.claim_id, row]));
const compatibleOwnershipProjections = new Set([
  "family_and_season:memory-family",
  "clock_and_resource_contracts:battle-entry-operations",
]);

const qualityLabels = new Set(["ExactStructured", "ExactPublicText", "ReproducibleObservation", "ProjectPolicy"]);
const mechanismLabels = new Set([
  "ExactRelationship",
  "ExactProgramProjection",
  "ExactSelectorClosure",
  "ExactInheritedDefinition",
  "IdentityCrossCheck",
  "PolicyBoundary",
]);
const evidenceGameVersions = new Set([
  "4.4",
  "1.1 stable-family cross-check",
  "stable-family cross-check accessed during Version 4.4",
  "1.0 stable-family cross-check",
  "stable-family cross-check",
]);
const evidenceIds = new Set();
const claims = [];
for (const { file, row } of primaryRows) {
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"]) {
    assert(typeof row[field] === "string" && row[field].trim() !== "", `${file}:${row.id} missing ${field}`);
  }
  assert(row.game_version === "4.4", `${file}:${row.id} version leak`);
  assert(row.coverage_state === "DataReady", `${file}:${row.id} is not DataReady`);
  assert(row.runtime_executable === false, `${file}:${row.id} is runtime executable`);
  assert(["MemoryOfChaos", "Shared"].includes(row.ownership), `${file}:${row.id} invalid ownership`);
  assert(!row.tags.some((tag) => ["preview", "future", "beta", "unreleased"].includes(tag)),
    `${file}:${row.id} contains unavailable-content tag`);

  for (const evidence of row.evidence_refs) {
    for (const field of [
      "id", "repository_or_url", "revision_or_access_date", "game_version", "path_or_page",
      "row_locator", "evidence_sha256", "quality", "mechanism_quality", "note",
    ]) {
      assert(typeof evidence[field] === "string" && evidence[field].trim() !== "",
        `${file}:${row.id} evidence ${evidence.id ?? "<unknown>"} missing ${field}`);
    }
    assert(evidenceGameVersions.has(evidence.game_version),
      `${file}:${row.id} evidence game-version drift`);
    assert(/^[0-9a-f]{64}$/u.test(evidence.evidence_sha256), `${file}:${row.id} invalid evidence digest`);
    assert(qualityLabels.has(evidence.quality), `${file}:${row.id} invalid evidence quality ${evidence.quality}`);
    assert(mechanismLabels.has(evidence.mechanism_quality),
      `${file}:${row.id} invalid mechanism quality ${evidence.mechanism_quality}`);
    evidenceIds.add(evidence.id);
  }

  for (const claimId of row.source_record_ids) {
    const expected = manifestByClaim.get(claimId);
    assert(expected !== undefined, `${file}:${row.id} claims unknown manifest row ${claimId}`);
    assert(expected.ownership === row.ownership || compatibleOwnershipProjections.has(claimId),
      `${file}:${row.id} ownership conflicts with ${claimId}`);
    assert(row.evidence_refs.some((evidence) =>
      evidence.path_or_page === expected.source_path
      && evidence.row_locator === expected.row_locator
      && evidence.evidence_sha256 === expected.evidence_sha256),
    `${file}:${row.id} lacks exact manifest provenance for ${claimId}`);
    claims.push({
      claim_id: claimId,
      file,
      record_id: row.id,
      manifest_ownership: expected.ownership,
      normalized_ownership: row.ownership,
    });
  }
}

assert(claims.length === manifest.counts.required, `claim count drift: ${claims.length}`);
assert(new Set(claims.map(({ claim_id: claimId }) => claimId)).size === manifest.counts.required,
  "manifest claim is duplicated");
assert(manifestRows.every(({ claim_id: claimId }) => claims.some((claim) => claim.claim_id === claimId)),
  "manifest claim is missing");
const ownershipCounts = Object.fromEntries(["MemoryOfChaos", "Shared"].map((ownership) => [
  ownership,
  claims.filter((claim) => claim.manifest_ownership === ownership).length,
]));
assert(JSON.stringify(ownershipCounts) === JSON.stringify(manifest.counts.ownership),
  `ownership denominator drift: ${JSON.stringify(ownershipCounts)}`);

const sourceRows = loaded.get("sources.json").records;
const sourceEvidenceIds = new Set(sourceRows.map((row) => row.source_evidence_id));
assert(sourceRows.length === 594 && sourceEvidenceIds.size === 594, "canonical source denominator drift");
assert([...evidenceIds].every((id) => sourceEvidenceIds.has(id)), "primary evidence is absent from sources.json");
const reconciliationRows = loaded.get("reconciliation-receipts.json").records;
const compatibleProjectionLocators = new Set(reconciliationRows
  .filter(({ semantic_result: result }) => result === "CompatibleProjection")
  .map((row) => `${row.source_path}\u001f${row.stable_row_locator}\u001f${row.evidence_sha256}`));
assert(reconciliationRows.length === 305 && compatibleProjectionLocators.size === 2,
  "shared reconciliation/compatible-projection denominator drift");
for (const claimId of ["family_and_season:memory-family"]) {
  const row = manifestByClaim.get(claimId);
  assert(compatibleProjectionLocators.has(`${row.source_path}\u001f${row.row_locator}\u001f${row.evidence_sha256}`),
    `ownership projection lacks reconciliation receipt: ${claimId}`);
}

const allIds = new Set(allRows.map(({ row }) => row.id));
const referencePrefixes = new Set([
  "profile", "season", "entry", "stage", "node", "tierce", "participant-policy", "attempt-rule",
  "clock-rule", "resource-rule", "objective", "turbulence", "battle-event", "rule-contribution",
  "pool-audit", "encounter", "wave", "enemy-slot", "enemy-variant", "enemy-template", "enemy-ability",
]);
let referenceCount = 0;
function auditReferences(value, key = "") {
  if (Array.isArray(value)) {
    value.forEach((item) => auditReferences(item, key));
    return;
  }
  if (value && typeof value === "object") {
    Object.entries(value).forEach(([childKey, item]) => auditReferences(item, childKey));
    return;
  }
  if (typeof value !== "string" || !(key.endsWith("_id") || key.endsWith("_ids"))) return;
  const prefix = value.split(".", 1)[0];
  if (!referencePrefixes.has(prefix)) return;
  referenceCount += 1;
  assert(allIds.has(value), `unresolved normalized reference ${key}=${value}`);
}
primaryRows.forEach(({ row }) => auditReferences(row));
assert(referenceCount > 400, `normalized reference audit unexpectedly small: ${referenceCount}`);

const seasons = loaded.get("seasons.json").records;
const activeSchedule = seasons.find(({ id }) => id === "season.schedule-201033");
const activeSeason = seasons.find(({ id }) => id === "season.group-1033");
assert(seasons.length === 2 && activeSchedule?.upstream_schedule_id === 201033, "active schedule selection drift");
assert(activeSchedule.begins_at_server_time === "2026-07-06 04:00:00"
  && activeSchedule.ends_at_server_time === "2026-08-17 04:00:00", "active schedule boundary drift");
assert(activeSeason?.upstream_group_id === 1033 && activeSeason.schedule_id === activeSchedule.id,
  "active season binding drift");
assert(activeSeason.ordinary_stage_ids.length === 12
  && activeSeason.ordinary_stage_ids[0] === "stage.5201"
  && activeSeason.ordinary_stage_ids.at(-1) === "stage.5212", "ordinary stage selector drift");
assert(activeSeason.tierce_id === "tierce.5213" && activeSeason.future_group_1034_included === false,
  "Tierce/future selection drift");
assert(manifest.exclusions.some(({ id }) => id === "future-schedule-201034")
  && manifest.exclusions.some(({ id }) => id === "future-group-1034"), "future exclusion proof drift");

const tierceRows = loaded.get("tierce.json").records;
assert(tierceRows.length === 1, "Tierce cardinality drift");
const tierce = tierceRows[0];
assert(tierce.id === "tierce.5213" && tierce.predecessor_stage_id === "stage.5212", "Tierce predecessor drift");
assert(JSON.stringify(tierce.stage_config_ids) === JSON.stringify(["encounter.30123123"]),
  "Tierce encounter projection drift");
assert(tierce.challenge_countdown === 45
  && JSON.stringify(tierce.objective_ids) === JSON.stringify(["objective.601", "objective.602", "objective.603"]),
"Tierce objective/clock drift");
assert(tierce.activity_projection.carries_ordinary_stage_state === false
  && tierce.activity_projection.participant_policy === "UnresolvedFailClosed"
  && tierce.activity_projection.runtime_publishable === false, "Tierce fail-closed policy drift");

const coverageRows = loaded.get("coverage.json").records;
assert(coverageRows.length === Object.keys(manifest.categories).length, "coverage category cardinality drift");
assert(coverageRows.every((row) => row.required === row.accounted
  && row.accounted === row.data_ready && row.missing === 0 && row.duplicate_claims === 0),
"coverage category is incomplete");
const packIndex = loaded.get("pack-index.json").records[0];
assert(packIndex.manifest_required === 477 && packIndex.manifest_accounted === 477
  && packIndex.manifest_data_ready === 477 && packIndex.runtime_publishable === false,
"pack-index release disposition drift");

const audit = {
  schema_revision: "starclock.memory-of-chaos-release-audit.v1",
  goal_id: "memory-of-chaos-reference-v1",
  lane: "Candidate",
  result: "Pass",
  normalized_files: files.length,
  primary_files: primaryFiles.length,
  primary_records: primaryRows.length,
  manifest_required: manifest.counts.required,
  exact_once_claims: claims.length,
  data_ready_claims: claims.length,
  ownership_counts: ownershipCounts,
  compatible_ownership_projections: [...compatibleOwnershipProjections].sort(compareText),
  bilingual_rows: primaryRows.length,
  provenance_rows: primaryRows.filter(({ row }) => row.evidence_refs.length > 0).length,
  canonical_sources: sourceRows.length,
  resolved_normalized_references: referenceCount,
  active_schedule_id: 201033,
  active_group_id: 1033,
  ordinary_stage_ids: activeSeason.ordinary_stage_ids,
  tierce_id: 5213,
  tierce_policy_boundary: "UnresolvedFailClosed",
  future_exclusions: [201034, 1034],
  runtime_publishable: false,
  audited_claims_sha256: digest(claims.sort((left, right) => compareText(left.claim_id, right.claim_id))),
};
const auditBytes = `${JSON.stringify(audit, null, 2)}\n`;
await writeText("evidence/memory-of-chaos-reference-v1/release-audits/coverage-ownership-audit.json", auditBytes, check);
console.log(`Goal 17 release audit ${check ? "verified" : "generated"}: 477/477 exact-once, 477 DataReady, ${referenceCount} references, no release leaks.`);
