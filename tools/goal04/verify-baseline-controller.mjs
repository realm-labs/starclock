import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-baseline-controller.json");
assert(policy.schema_revision === "starclock.goal04-baseline-controller.v1", "unexpected baseline policy revision");
const controller = text("crates/starclock-mode-universe/src/baseline_controller.rs");
const runner = text("crates/starclock-mode-universe/src/baseline_runner.rs");
const preparation = text("crates/starclock-activity/src/battle_preparation.rs");
const goldenTest = text("crates/starclock-mode-universe/tests/encounter_runtime.rs");

for (const marker of [
  `REVISION: &'static str = "${policy.controller_revision}"`,
  ".options()",
  "sort_by_key(|(id, _)| *id)",
  "i64::from(authored_priority) + hint_total"
]) assert(controller.includes(marker), `Activity controller omits ${marker}`);
for (const marker of [
  `REVISION: &'static str = "${policy.runner_revision}"`,
  "start_pending_battle(view.state_hash())",
  "executor.execute(&handoff)",
  "submit_pending_battle_result(activity.view().state_hash(), result)",
  "ActivityExternalOutcomeId::new(selected.option().get())"
]) assert(runner.includes(marker), `Standard Universe runner omits ${marker}`);
assert(preparation.includes("!self.settled && self.pending.is_none()"), "settled preparation attempts remain player-visible");
for (const marker of [
  "baseline_runner_uses_offered_options_and_executes_nested_battles_to_terminal",
  String(policy.golden.accepted_steps),
  "ActivityTerminalOutcome::Completed"
]) assert(goldenTest.includes(marker), `baseline golden test omits ${marker}`);

execFileSync("cargo", ["test", "-p", "starclock-mode-universe", "--lib", "baseline_controller::tests"], { cwd: root, stdio: "inherit" });
execFileSync("cargo", ["test", "-p", "starclock-mode-universe", "--test", "encounter_runtime", "baseline_runner_uses_offered_options_and_executes_nested_battles_to_terminal"], { cwd: root, stdio: "inherit" });

const sources = [
  "crates/starclock-activity/src/battle_preparation.rs",
  "crates/starclock-mode-universe/src/baseline_controller.rs",
  "crates/starclock-mode-universe/src/baseline_runner.rs",
  "crates/starclock-mode-universe/tests/encounter_runtime.rs"
];
const evidence = {
  schema_revision: "starclock.goal04-baseline-controller-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "ordered-legal-option-controller-completes-a-seeded-activity-through-verified-nested-battles",
  revisions: {
    controller: policy.controller_revision,
    runner: policy.runner_revision
  },
  golden: policy.golden,
  contracts: policy.contracts,
  source_sha256: Object.fromEntries(sources.map((relative) => [relative, sha256(relative)])),
  policy_sha256: sha256("policy/goal04-baseline-controller.json"),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/interfaces/baseline-controller.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "baseline controller evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "baseline controller evidence is stale; run with --bless");
}
console.log(`Goal 04 baseline controller verified (${policy.golden.accepted_steps} steps, ${policy.golden.nested_battles} nested battles).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
