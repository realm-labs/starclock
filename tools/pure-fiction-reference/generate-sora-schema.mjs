import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const check = process.argv.includes("--check");
const batch = process.argv.find((arg) => arg.startsWith("--batch="))?.slice(8);
if (!batch) throw new Error("use --batch=G15-P3-B1|B2|B3|B4");
const groups = {
  "G15-P3-B1": { file: "system-core", workbook: "PureFiction.xlsx", tables: ["profiles", "seasons", "stages", "nodes", "tierce-starward", "participant-policies", "attempt-policies"] },
  "G15-P3-B2": { file: "system-mechanics", workbook: "PureFictionBindings.xlsx", tables: ["clocks", "spawn-programs", "score-programs", "objectives", "seasonal-mechanics", "cacophonies", "initial-resources"] },
  "G15-P3-B3": { file: "bindings", workbook: "PureFictionBindings.xlsx", tables: ["pool-proofs", "themes", "maze-buffs", "battle-events", "ability-programs", "encounters", "waves", "enemy-slots", "enemy-variants", "enemy-templates", "enemy-skills", "enemy-character-configs", "enemy-ai", "enemy-abilities", "enemy-statuses", "mechanic-rules"] },
  "G15-P3-B4": { file: "review", workbook: "PureFictionReview.xlsx", tables: ["sources", "content-audit", "coverage", "research-gaps", "reconciliation", "semantic-fixtures", "pack-index"] }
};
const selected = groups[batch];
if (!selected) throw new Error(`unknown schema batch ${batch}`);
function pascal(value) { return value.split("-").map((part) => part[0].toUpperCase() + part.slice(1)).join(""); }
const names = {
  profiles: "Profile", seasons: "Season", stages: "Stage", nodes: "Node",
  "tierce-starward": "TierceStarward", "participant-policies": "ParticipantPolicy",
  "attempt-policies": "AttemptPolicy", clocks: "Clock", "spawn-programs": "SpawnProgram",
  "score-programs": "ScoreProgram", objectives: "Objective",
  "seasonal-mechanics": "SeasonalMechanic", cacophonies: "Cacophony",
  "initial-resources": "InitialResource", "pool-proofs": "PoolProof", themes: "Theme",
  "maze-buffs": "MazeBuff", "battle-events": "BattleEvent",
  "ability-programs": "AbilityProgram", encounters: "Encounter", waves: "Wave",
  "enemy-slots": "EnemySlot", "enemy-variants": "EnemyVariant",
  "enemy-templates": "EnemyTemplate", "enemy-skills": "EnemySkill",
  "enemy-character-configs": "EnemyCharacterConfig", "enemy-ai": "EnemyAI",
  "enemy-abilities": "EnemyAbility", "enemy-statuses": "EnemyStatus",
  "mechanic-rules": "MechanicRule", sources: "SourceRecord",
  "content-audit": "ContentAudit", coverage: "Coverage", "research-gaps": "ResearchGap",
  reconciliation: "Reconciliation", "semantic-fixtures": "SemanticFixture",
  "pack-index": "PackFile",
};
function field(name, type, extra = "") { return ["[[tables.fields]]", `name = ${JSON.stringify(name)}`, `type = ${JSON.stringify(type)}`, extra].filter(Boolean).join("\n"); }
const common = [
  field("id", "i32", "range = [1, 2147483647]"), field("stable_key", "string", "length = [1, 1600]"), field("row_order", "i32", "range = [1, 1000000]"),
  field("name_en", "string", "length = [1, 3000]"), field("name_zh_cn", "string", "length = [1, 3000]"), field("summary_en", "string", "length = [1, 8000]"), field("summary_zh_cn", "string", "length = [1, 8000]"),
  field("ownership", "enum<PfOwnership>"), field("coverage_state", "enum<PfCoverageState>"), field("evidence_quality", "enum<PfEvidenceQuality>"), field("mechanism_quality", "enum<PfMechanismQuality>"),
  field("manifest_record_ids", "list<string>", 'parser = { kind = "split", separator = "|" }\nlength = [1, 4096]'), field("source_record_ids", "list<string>", 'parser = { kind = "split", separator = "|" }\nlength = [1, 4096]'),
  field("payload_json", "string", "length = [2, 1000000]"), field("runtime_executable", "bool")
];
const enums = `[[enums]]
name = "PfOwnership"
values = ["PureFiction", "Shared", "EvidenceOnly"]

[[enums]]
name = "PfCoverageState"
values = ["DataReady", "ResearchGap", "Excluded"]

[[enums]]
name = "PfEvidenceQuality"
values = ["ExactStructured", "ExactPublicText", "Observed", "ApproximateFromReleasedText", "ProjectPolicy"]

[[enums]]
name = "PfMechanismQuality"
values = ["Exact", "ReleasedText", "Observed", "Approximate", "DeterministicProjectPolicyNotObservedParity"]
`;
const tables = selected.tables.map((file) => [
  "[[tables]]", `name = "Pf${names[file] ?? pascal(file)}"`, 'mode = "map"', 'key = "id"', "[tables.source]", 'format = "xlsx"', `file = "${selected.workbook}"`, `sheet = "${names[file] ?? pascal(file)}"`, ...common,
  "[[tables.indexes]]", 'name = "by_stable_key"', 'fields = ["stable_key"]', "unique = true"
].join("\n")).join("\n\n");
const schemaBytes = `${batch === "G15-P3-B1" ? enums : ""}\n${tables}\n`;
const schemaPath = path.join(root, `config/pure-fiction/schema/${selected.file}.toml`);
async function emit(file, bytes) {
  if (check) { if (await readFile(file, "utf8").catch(() => "") !== bytes) throw new Error(`schema drift ${file}`); }
  else { await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, bytes); }
}
await emit(schemaPath, schemaBytes);
if (batch === "G15-P3-B4") {
  const project = `package = "starclock_pure_fiction_reference_config"
includes = [
  "schema/system-core.toml",
  "schema/system-mechanics.toml",
  "schema/bindings.toml",
  "schema/review.toml",
]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "../pure-fiction-generated/schema.lock"
excel_templates = "../pure-fiction-generated/templates"

[[build.codegen]]
target = "rust"
out = "../pure-fiction-generated/readers/rust"
format = "never"

[[build.exports]]
format = "binary"
out = "../pure-fiction-generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "../pure-fiction-generated/debug-json"

[codegen.rust]
runtime_format = "sora"
`;
  await emit(path.join(root, "config/pure-fiction/project.toml"), project);
}
console.log(`Pure Fiction ${selected.file} schema verified: ${selected.tables.length} tables`);
