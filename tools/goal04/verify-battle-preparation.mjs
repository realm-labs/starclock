import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-battle-preparation.json");
assert(policy.schema_revision === "starclock.goal04-battle-preparation.v1", "unexpected battle-preparation policy revision");
const source = text("crates/starclock-activity/src/battle_preparation.rs");
const participant = text("crates/starclock-activity/src/participant.rs");
const transaction = text("crates/starclock-activity/src/transaction.rs");
const view = text("crates/starclock-activity/src/view.rs");
const tests = text("crates/starclock-activity/tests/battle_preparation.rs");

for (const [constant, value] of [
  ["MAX_PREPARATION_TECHNIQUES", policy.bounds.maximum_techniques],
  ["MAX_PREPARED_BATTLE_VARIANTS", policy.bounds.maximum_variants]
]) assert(new RegExp(`const ${constant}: [^=]+ = ${numberLiteral(value)};`).test(source), `${constant} differs from policy`);
assert(participant.includes(`entries.len() > ${policy.bounds.maximum_locked_participants}`), "participant-lock bound differs from policy");
for (const marker of [
  "pub enum EncounterInitiativePolicy", "pub enum TechniqueEngagement",
  "pub struct ActivityRosterLock", "pub struct EncounterPreparationDefinition",
  "pub struct PendingBattleSpec", "pub struct ActivityBattlePreparationRequest",
  "MissingPrefixVariant", "SequenceAfterEngagement", "EnemyPreemptive",
  "ParticipantSource::Player", "resolved_spec_digest", "Arc<BattleBinding>",
  "pub fn begin_battle_preparation", "pub fn choose_preparation_option"
]) assert((source + participant).includes(marker), `battle preparation omits ${marker}`);
assert(source.includes("actual.combatant().form() != expected.character()"), "battle preparation does not verify locked form identity");
assert(source.includes("actual.formation().get() != expected.formation_index()"), "battle preparation does not verify locked formation");
assert(source.includes("actual.combatant().digest() != expected.build().resolved_spec_digest()"), "battle preparation does not verify opaque resolved-spec identity");
assert(!/CombatantBuildSpec|TraceDefinition|LightCone|Relic/.test(source), "battle preparation interprets peripheral build fields");
assert(!/BattleSpec::new|ResolvedCombatantSpec::new/.test(source), "Activity constructs combat/build internals instead of selecting a validated immutable BattleSpec");
assert(!/f32|f64|HashMap/.test(source + participant + transaction + view), "battle preparation uses float or unordered map");
assert(transaction.includes("writer.bool(self.attempt.is_some())") && transaction.includes("attempt.encode(&mut writer)"), "canonical state omits domain-attempt preparation");
assert(view.includes("pending_battle: Option<ActivityPendingBattleView>"), "player view omits bounded pending-battle identity");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === policy.focused_tests, "battle-preparation focused test count differs");
for (const marker of [
  "accumulated_and_attacking_techniques", "normal_engagement_uses_the_variant",
  "enemy_preemptive_policy_skips", "reject_mismatch_without_mutating",
  "requires_prefix_closed_reachable_sequences"
]) assert(tests.includes(marker), `battle-preparation tests omit ${marker}`);
const definitionGolden = arrayGolden(tests, /definition\.digest\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const pendingGolden = arrayGolden(tests, /pending_state_hash\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(definitionGolden === policy.goldens.preparation_definition_sha256, "preparation-definition golden differs");
assert(pendingGolden === policy.goldens.pending_activity_state_sha256, "pending Activity-state golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P2-B5` \| `(InProgress|Complete)` \|/m.test(status), "G04-P2-B5 is not active or complete");
assert(/^\| `G04-R05` \| `Resolved` \|/m.test(status), "G04-R05 technique/initiative policy is not resolved");

const evidence = {
  schema_revision: "starclock.goal04-battle-preparation-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "roster-locked-attempt-preparation-and-pending-battle-implemented",
  bounds: policy.bounds,
  shape: policy.shape,
  goldens: policy.goldens,
  compatibility: {
    goal01_one_battle_hash_goldens_unchanged: true,
    battle_result_projection_deferred: "G04-P2-B6",
    build_fields_public_or_interpreted: false,
    generated_rows_public: false,
    authoritative_float: false
  },
  source_sha256: {
    preparation: sha256("crates/starclock-activity/src/battle_preparation.rs"),
    participant: sha256("crates/starclock-activity/src/participant.rs"),
    transaction: sha256("crates/starclock-activity/src/transaction.rs"),
    view: sha256("crates/starclock-activity/src/view.rs"),
    tests: sha256("crates/starclock-activity/tests/battle_preparation.rs")
  },
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/activity/battle-preparation.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "battle-preparation evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "battle-preparation evidence is stale; run with --bless");
}
console.log(`Goal 04 battle preparation verified (${policy.bounds.maximum_techniques} techniques, ${policy.bounds.maximum_variants} variants, ${policy.focused_tests} tests).`);

function arrayGolden(source, pattern) {
  const match = source.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
