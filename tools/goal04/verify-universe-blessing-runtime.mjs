import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-universe-blessing-runtime.json");
assert(policy.schema_revision === "starclock.goal04-universe-blessing-runtime.v1", "unexpected Blessing-runtime policy revision");
const blessing = text("crates/starclock-mode-universe/src/blessing_runtime.rs");
const topology = text("crates/starclock-mode-universe/src/topology.rs");
const activity = text("crates/starclock-activity/src/graph_activity.rs");
const program = text("crates/starclock-activity/src/program.rs");
const rng = text("crates/starclock-activity/src/activity_rng.rs");
const tests = text("crates/starclock-mode-universe/tests/blessing_runtime.rs");
const encounterTests = text("crates/starclock-mode-universe/tests/encounter_runtime.rs");

for (const marker of [
  "BlessingOfferEligibility", "BlessingRuntimeCatalog", "enhancement_operations",
  "replacement_operations", "BlessingContributionSet", "source_binding_key", "rule_key"
]) assert(blessing.includes(marker), `Blessing runtime omits ${marker}`);
for (const marker of [
  "ActivityRandomOffer", "ActivityRngLabel::Reward", "BLESSING_DRAW_PURPOSE",
  "blessing_inventory", "blessing_reroll_slot"
]) assert(topology.includes(marker), `Blessing topology integration omits ${marker}`);
assert(activity.includes("reroll_random_offer") && activity.includes("restrict_random_offer"), "generic random-offer execution is incomplete");
assert(program.includes("InventoryCount"), "generic inventory predicate is missing");
assert(rng.includes("choose_weighted_without_replacement"), "weighted no-replacement RNG is missing");
assert(!/f32|f64|HashMap/.test(blessing + topology + activity), "Blessing runtime contains floats or unordered state");
assert((tests.match(/^#\[test\]$/gm) ?? []).length === 2, "Blessing focused test count differs");
assert(encounterTests.includes("reroll_blessing_offer"), "end-to-end Blessing reset test is missing");

const catalogGolden = arrayGolden(tests, /runtime\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
const initialGolden = arrayGolden(text("crates/starclock-mode-universe/tests/topology_runtime.rs"), /view\.state_hash\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const rewardGolden = arrayGolden(encounterTests, /settled\.state_hash\(\)\.bytes\(\),\s*\[([\s\S]*?)\]\s*\)/);
const contributionGolden = arrayGolden(encounterTests, /contributions\.digest\(\),\s*\[([\s\S]*?)\]\s*\)/);
assert(catalogGolden === policy.goldens.runtime_catalog_sha256, "Blessing runtime-catalog golden differs");
assert(initialGolden === policy.goldens.seed7_initial_state_sha256, "Blessing-aware initial-state golden differs");
assert(rewardGolden === policy.goldens.won_reward_offer_state_sha256, "Blessing reward-offer golden differs");
assert(contributionGolden === policy.goldens.selected_contribution_set_sha256, "Blessing contribution-set golden differs");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P3-B4` \| `(InProgress|Complete)` \|/m.test(status), "G04-P3-B4 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-universe-blessing-runtime-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "blessing-offer-reroll-inventory-level-and-typed-contribution-runtime-is-deterministic",
  counts: policy.counts,
  policies: policy.policies,
  goldens: policy.goldens,
  deferred_scope: policy.scope,
  compatibility: {
    generated_rows_public: false,
    mutable_mode_engine_added: false,
    invalid_reroll_mutates_state_or_rng: false,
    authoritative_float: false,
    complete_catalog_cloned_per_run: false
  },
  source_sha256: Object.fromEntries([
    "crates/starclock-activity/src/activity_rng.rs",
    "crates/starclock-activity/src/program.rs",
    "crates/starclock-activity/src/transaction.rs",
    "crates/starclock-activity/src/transaction/decision.rs",
    "crates/starclock-activity/src/graph_activity.rs",
    "crates/starclock-mode-universe/src/blessing_runtime.rs",
    "crates/starclock-mode-universe/src/topology.rs",
    "crates/starclock-mode-universe/src/runtime.rs",
    "crates/starclock-mode-universe/tests/blessing_runtime.rs",
    "crates/starclock-mode-universe/tests/encounter_runtime.rs"
  ].map((relative) => [relative, sha256(relative)])),
  focused_tests: policy.focused_tests,
  new_registry_packages: policy.new_registry_packages
};
const relative = "evidence/standard-universe-runtime-v1/profile/universe-blessing-runtime.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "Universe Blessing-runtime evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "Universe Blessing-runtime evidence is stale; run with --bless");
}
console.log(`Goal 04 Universe Blessing runtime verified (${policy.counts.blessings} Blessings, ${policy.counts.blessing_levels} levels, ${policy.counts.reward_hubs} reward hubs).`);

function arrayGolden(sourceText, pattern) {
  const match = sourceText.match(pattern);
  assert(match !== null, "expected Rust byte-array golden is missing");
  return match[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => Number.parseInt(value, 10).toString(16).padStart(2, "0")).join("");
}
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
