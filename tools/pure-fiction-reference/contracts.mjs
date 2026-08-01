import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";

const check = process.argv.includes("--check");
const schemaPath = "content-reference/pure-fiction-v1/schema.json";
const evidencePath = "evidence/pure-fiction-reference-v1/phase-0/contracts.json";
const normalizedFiles = [
  "profiles.json", "seasons.json", "stages.json", "nodes.json", "tierce-starward.json",
  "participant-policies.json", "attempt-policies.json", "clocks.json", "spawn-programs.json",
  "score-programs.json", "objectives.json", "seasonal-mechanics.json", "cacophonies.json",
  "initial-resources.json", "pool-proofs.json", "themes.json", "maze-buffs.json",
  "battle-events.json", "ability-programs.json", "encounters.json", "waves.json",
  "enemy-slots.json", "enemy-variants.json", "enemy-templates.json", "enemy-skills.json",
  "enemy-character-configs.json", "enemy-ai.json", "enemy-abilities.json", "enemy-statuses.json",
  "mechanic-rules.json", "sources.json", "coverage.json", "research-gaps.json",
  "reconciliation.json", "semantic-fixtures.json", "pack-index.json"
];
const workbookTables = {
  "PureFiction.xlsx": [
    "Profile", "Season", "Stage", "Node", "TierceStarward", "ParticipantPolicy", "AttemptPolicy"
  ],
  "PureFictionBindings.xlsx": [
    "Clock", "SpawnProgram", "ScoreProgram", "Objective", "SeasonalMechanic", "Cacophony",
    "InitialResource", "PoolProof", "Theme", "MazeBuff", "BattleEvent", "AbilityProgram",
    "Encounter", "Wave", "EnemySlot", "EnemyVariant", "EnemyTemplate", "EnemySkill",
    "EnemyCharacterConfig", "EnemyAI", "EnemyAbility", "EnemyStatus", "MechanicRule"
  ],
  "PureFictionReview.xlsx": [
    "SourceRecord", "ContentAudit", "Coverage", "ResearchGap", "Reconciliation", "SemanticFixture", "PackFile"
  ]
};
const schema = {
  schema_revision: "pure-fiction-normalized-contract-v1",
  common_envelope: [
    "id", "name_en", "name_zh_cn", "summary_en", "summary_zh_cn", "game_version_snapshot",
    "owner", "release_state", "enabled", "coverage_state", "source_record_ids"
  ],
  evidence_fields: [
    "publisher", "url_or_repository", "revision_or_access_date", "game_version", "path_or_page",
    "row_locator", "evidence_digest", "quality", "mechanism_quality", "note"
  ],
  qualities: ["ExactStructured", "ExactPublicText", "Observed", "ApproximateFromReleasedText", "ProjectPolicy"],
  mechanism_qualities: ["Exact", "ReleasedText", "Observed", "Approximate", "DeterministicProjectPolicyNotObservedParity"],
  canonical_encoding: {
    "json": "UTF-8, two-space indentation, LF, one terminal newline",
    "decimal": "canonical base-10 string without exponent",
    "set_order": "ascending stable ID or Unicode stable-key order",
    "semantic_order": "preserve authored wave, slot, parameter and operation order",
    "hash": "SHA-256 over exact committed bytes"
  },
  normalized_files: normalizedFiles,
  workbook_tables: workbookTables,
  reconciliation_key: ["source_path", "source_locator", "evidence_digest"],
  fixture_contract: {
    required_fields: ["id", "family", "input_ids", "initial_state", "commands", "expected_facts", "evidence_quality", "replacement_condition"],
    fact_language: ["equals", "contains", "ordered_equals", "absent"],
    execution_status: "reference_review_only"
  },
  runtime_status: "Unreleased"
};
schema.contract_digest = createHash("sha256").update(JSON.stringify(schema)).digest("hex");
const evidence = {
  schema_revision: "pure-fiction-contract-evidence-v1",
  normalized_file_count: normalizedFiles.length,
  workbook_count: Object.keys(workbookTables).length,
  table_count: Object.values(workbookTables).flat().length,
  contract_digest: schema.contract_digest,
  workbook_ownership: "isolated_goal15_only",
  json_disposition: "research_bootstrap_debug_only",
  sora_version: "0.3.0",
  authoring_adapter: "openpyxl",
  runtime_lowering: "forbidden_in_goal15"
};
function emit(path, value) {
  const canonical = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    if (readFileSync(path, "utf8") !== canonical) throw new Error(`Contract drift: ${path}`);
  } else writeFileSync(path, canonical);
}
emit(schemaPath, schema);
emit(evidencePath, evidence);
console.log(`Pure Fiction contracts verified: ${normalizedFiles.length} files, ${evidence.table_count} tables`);

