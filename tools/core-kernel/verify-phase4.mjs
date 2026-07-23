import path from "node:path";
import { spawnSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const artifactOnly = process.argv.slice(2).includes("--artifacts-only");
if (process.argv.slice(2).some((argument) => argument !== "--artifacts-only"))
  throw new Error("usage: verify-phase4.mjs [--artifacts-only]");
const artifactCommands = [
  ["node", "tools/config-probes/verify-asta-modifier.mjs"],
  ["node", "tools/config-probes/verify-firefly-damage.mjs"],
  ["node", "tools/config-probes/verify-firefly-transform.mjs"],
  ["node", "tools/config-probes/verify-kafka-dot.mjs"],
  ["node", "tools/config-probes/verify-clara-counter.mjs"],
  ["node", "tools/config-probes/verify-aglaea-memosprite.mjs"],
  ["node", "tools/config-probes/verify-elation-probes.mjs"],
  ["node", "tools/goal-research/generate.mjs", "--check"],
  ["node", "tools/goal-research/verify.mjs"],
  ["node", "tools/benchmark/verify.mjs"],
  ["node", "tools/benchmark/review-phase4.mjs", "--check"],
];
const testCommands = [
  ["cargo", "test", "-p", "starclock-combat", "--all-features", "--test", "numeric_formula_oracle", "--test", "damage_sustain_pipeline", "--test", "toughness_formula", "--test", "damage_lifecycle", "--test", "effect_resource_pipeline", "--test", "reaction_scheduler", "--test", "linked_lifecycle", "--test", "elation_subsystem", "--test", "enemy_orchestration", "--test", "rule_ir_contract", "--test", "catalog_contract"],
  ["cargo", "test", "-p", "starclock-rules", "--all-features"],
  ["cargo", "test", "-p", "starclock-ai", "--all-features"],
  ["cargo", "test", "-p", "starclock-data", "probe_tests", "--all-features"],
];
const commands = artifactOnly ? artifactCommands : [...artifactCommands, ...testCommands];

for (const command of commands) {
  console.log(`\n==> ${command.join(" ")}`);
  const result = spawnSync(command[0], command.slice(1), { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

console.log(artifactOnly
  ? "\nPhase 4 artifact/probe golden suite passed; workspace Rust tests are owned by the repository test runner."
  : "\nPhase 4 core formula/lifecycle/rule/probe golden suite passed.");
