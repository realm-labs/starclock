#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const args = process.argv.slice(2);
const check = args.includes("--check");
const output = path.join(root,
  "content-manifests/memory-of-chaos-v1/authoring-contract.json");
const auditOutput = path.join(root,
  "evidence/memory-of-chaos-reference-v1/authoring-contract-audit.md");
const manifest = JSON.parse(await readFile(path.join(root,
  "content-manifests/memory-of-chaos-v1/content-manifest.json"), "utf8"));
assert(manifest.counts.required === 477, "frozen manifest denominator drift");

const normalizedFiles = [
  "profile.json", "seasons.json", "entries.json", "stages.json",
  "nodes.json", "tierce.json", "participant-policies.json",
  "attempt-rules.json", "clock-rules.json", "resource-rules.json",
  "objectives.json", "turbulence.json", "battle-events.json",
  "rule-contributions.json", "pool-audits.json", "encounters.json",
  "waves.json", "enemy-slots.json", "enemy-variants.json",
  "enemy-templates.json", "enemy-abilities.json", "sources.json",
  "reconciliation-receipts.json", "research-gaps.json", "coverage.json",
  "semantic-fixtures.json", "pack-index.json",
];
const fixtureFamilies = [
  "active-season-selection", "ordinary-stage-order",
  "tierce-selected-extension", "participant-uniqueness",
  "loadout-lock-retry", "cycle-first-av-window", "cycle-node-wave-carry",
  "objective-star-aggregation", "turbulence-hit-accumulation",
  "turbulence-cap-cycle-start", "turbulence-target-true-damage",
  "initial-resources", "battle-entry-operations", "encounter-wave-order",
  "enemy-transitive-closure", "empty-pool-selector-closure",
  "future-season-exclusion", "shared-row-reconciliation",
];
const workbooks = [
  { name: "MemoryOfChaos.xlsx", tables: [
    "MoCProfile", "MoCSeason", "MoCEntry", "MoCStage", "MoCNode",
    "MoCTierce", "MoCParticipantPolicy", "MoCAttemptRule",
    "MoCClockRule", "MoCResourceRule",
  ] },
  { name: "MemoryOfChaosBindings.xlsx", tables: [
    "MoCObjective", "MoCTurbulence", "MoCBattleEvent", "MoCRuleBinding",
    "MoCPoolAudit", "MoCEncounter", "MoCWave", "MoCEnemySlot",
    "MoCEnemyVariant", "MoCEnemyAbility",
  ] },
  { name: "MemoryOfChaosReview.xlsx", tables: [
    "MoCSource", "MoCReconciliation", "MoCResearchGap", "MoCCoverage",
    "MoCSemanticFixture", "MoCFixtureFact", "MoCPackFile",
  ] },
];
const payload = {
  schema_revision: "starclock.memory-of-chaos-authoring-contract.v1",
  goal_id: "memory-of-chaos-reference-v1",
  snapshot: "4.4",
  manifest: { schema_revision: manifest.schema_revision,
    required_obligations: manifest.counts.required,
    sha256: sha256(await readFile(path.join(root,
      "content-manifests/memory-of-chaos-v1/content-manifest.json"))) },
  lane: { initial: "Experimental", terminal: "Candidate",
    runtime_profile: "Unreleased", runtime_rows_allowed: false },
  normalized_pack: {
    root: "content-reference/memory-of-chaos-v1",
    files: normalizedFiles,
    canonical_encoding: "UTF-8; LF; JSON.stringify with two-space indentation and trailing LF",
    canonical_decimals: "strings only",
    stable_id_order: "lexicographic UTF-8 code-unit order",
    semantic_sequences: "preserve explicit stage/node/wave/slot/effect order",
    set_sequences: "sort by stable Starclock ID",
    source_ids_are_runtime_ids: false,
  },
  common_record: {
    required_fields: ["id", "name_en", "name_zh_cn", "summary_en",
      "summary_zh_cn", "game_version", "ownership", "coverage_state",
      "source_record_ids"],
    coverage_state: "DataReady",
    ownership_values: ["MemoryOfChaos", "Shared", "EvidenceOnly"],
  },
  evidence: {
    required_fields: ["id", "repository_or_url", "revision_or_access_date",
      "game_version", "path_or_page", "row_locator", "evidence_sha256",
      "quality", "mechanism_quality", "note"],
    quality_values: ["ExactStructured", "ExactPublicText", "Observed",
      "ApproximateFromReleasedText", "ProjectPolicy"],
    approximation_required_fields: ["known_facts", "selected_behavior",
      "rejected_alternatives", "rationale", "affected_fixture_ids",
      "confidence", "replacement_condition"],
    source_prose_committed: false,
  },
  reconciliation: {
    key: ["source_path", "stable_row_locator", "evidence_sha256"],
    fields: ["id", "peer_goal", "peer_revision", "source_path",
      "stable_row_locator", "evidence_sha256", "ownership_local",
      "ownership_peer", "semantic_result", "note"],
    allowed_results: ["Match", "CompatibleProjection", "Conflict"],
    conflict_policy: "record-and-stop-merge-coordination-no-peer-overwrite",
  },
  semantic_fixtures: {
    families: fixtureFamilies,
    count: fixtureFamilies.length,
    may_shrink: false,
    required_fields: ["id", "family", "mechanism_quality_floor",
      "canonical_seed", "initial_state", "commands", "expected_facts",
      "source_record_ids"],
    execution_scope: "reference-review-only-no-runtime-parity-claim",
  },
  excel_sora: {
    format: "xlsx", adapter: "python-openpyxl", adapter_version: "3.1.5",
    schema_authority: "sora-cli-0.3.0", project:
      "config/memory-of-chaos/project.toml", generated_root:
      "config/memory-of-chaos-generated", workbooks,
    workbook_generation: "complete-clean-target-no-overwrite",
    forbidden_outputs: ["config/generated/", "config/universe-generated/",
      "config/gold-and-gears-generated/", "config/pure-fiction-generated/",
      "config/apocalyptic-shadow-generated/"],
  },
  normalized_family_bindings: Object.fromEntries(normalizedFiles.map(
    (file, index) => [file, { order: index + 1,
      workbook: workbookFor(file, workbooks) } ])),
};
const bytes = Buffer.from(`${JSON.stringify(payload, null, 2)}\n`, "utf8");
const audit = `# Goal 17 Authoring Contract Audit\n\n` +
  `- Result: passed\n- Frozen manifest obligations: 477\n` +
  `- Normalized files: ${normalizedFiles.length}\n` +
  `- Workbooks: ${workbooks.length}; primary Sora tables: ` +
  `${workbooks.reduce((sum, row) => sum + row.tables.length, 0)}\n` +
  `- Non-shrinking semantic fixture families: ${fixtureFamilies.length}\n` +
  `- Reconciliation identity: source path + stable row locator + evidence digest\n` +
  `- Excel adapter: Python openpyxl 3.1.5; authority: Sora 0.3.0\n` +
  `- Runtime rows/profile: forbidden/unreleased\n` +
  `- Contract digest: \`${sha256(bytes)}\`\n`;

if (check) {
  assert((await readFile(output)).equals(bytes), "authoring contract drift");
  assert((await readFile(auditOutput, "utf8")) === audit,
    "authoring contract audit drift");
  console.log("Goal 17 authoring contracts verified (27 files; 27 tables; 18 fixtures).");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await mkdir(path.dirname(auditOutput), { recursive: true });
  await writeFile(output, bytes);
  await writeFile(auditOutput, audit);
  console.log("Goal 17 authoring contracts generated (27 files; 27 tables; 18 fixtures).");
}

function workbookFor(file, workbookRows) {
  if (["sources.json", "reconciliation-receipts.json", "research-gaps.json",
    "coverage.json", "semantic-fixtures.json", "pack-index.json"].includes(file))
    return workbookRows[2].name;
  if (["objectives.json", "turbulence.json", "battle-events.json",
    "rule-contributions.json", "pool-audits.json", "encounters.json",
    "waves.json", "enemy-slots.json", "enemy-variants.json",
    "enemy-templates.json", "enemy-abilities.json"].includes(file))
    return workbookRows[1].name;
  return workbookRows[0].name;
}
function sha256(value) { return createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
