import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-entry.json");
assert(policy.schema_revision === "starclock.goal04-universe-entry.v1", "unexpected Universe-entry policy revision");
const source = text("crates/starclock-mode-universe/src/entry.rs");
const tests = text("crates/starclock-mode-universe/tests/entry_compilation.rs");
const manifest = text("crates/starclock-mode-universe/Cargo.toml");

for (const marker of [
  "pub struct StandardUniverseEntry", "pub struct StandardUniverseProfile",
  "pub struct CompiledActivity", "pub enum StandardUniverseCompileError",
  "DifficultyWorldMismatch", "ParticipantPolicyMismatch", "DuplicateAbilityTreeNode",
  "MissingAbilityTreePrerequisite", "ActivityValue::OptionalId(None)",
  "ActivityValue::OrderedIdSet", "SlotCarryPolicy::CarryExact",
  "ActivityStateVisibility::Player", "participant_digest"
]) assert(source.includes(marker), `Universe entry compiler omits ${marker}`);
assert(source.includes(`pub const STANDARD_UNIVERSE_ENTRY_REVISION: &str = "${policy.entry_revision}";`), "entry revision differs");
assert(manifest.includes('starclock-activity = { path = "../starclock-activity" }'), "Universe profile does not compile to generic Activity types");
assert(!/f32|f64|HashMap|serde_json::/.test(source), "Universe entry compiler uses floats, unordered state or transport JSON");
assert(!source.includes("pub fn start("), "P3-B1 must not claim runtime start before P3-B2 topology compilation");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "Universe-entry focused test count differs");
for (const marker of [
  "every_world_and_difficulty_compiles_the_same_generic_entry_contract",
  "ability_tree_input_is_canonical_and_prerequisite_closed",
  "cross_world_difficulty_and_nonstandard_roster_policy_fail_closed",
  "world_difficulty_roster_and_ability_input_are_definition_identity"
]) assert(tests.includes(marker), `Universe-entry tests omit ${marker}`);
assert(tests.includes("assert_eq!(compiled, 33)"), "all 33 World/difficulty profiles are not exercised");
const golden = arrayGolden(tests, /definition_digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(golden === policy.definition_golden_sha256, "Universe-entry definition golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B1` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B1 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-entry-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "world-difficulty-roster-path-choice-and-ability-tree-compile-to-generic-activity-state",
  catalog_counts: policy.catalog_counts,
  activity_state: policy.activity_state,
  participant_policy: policy.participant_policy,
  definition_golden_sha256: policy.definition_golden_sha256,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    authoritative_float: false,
    catalog_cloned_per_run: false
  },
  source_sha256: {
    compiler: sha256("crates/starclock-mode-universe/src/entry.rs"),
    tests: sha256("crates/starclock-mode-universe/tests/entry_compilation.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-entry.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe-entry evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe-entry evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe entry verified (${policy.catalog_counts.worlds} Worlds, ${policy.catalog_counts.difficulties} difficulties, ${policy.catalog_counts.paths} Path choices).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
