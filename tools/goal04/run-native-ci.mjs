import process from "node:process";
import { execFileSync } from "node:child_process";

if (process.argv.length !== 3 || !["--foundation", "--hardening"].includes(process.argv[2]))
  throw new Error("usage: run-native-ci.mjs --foundation|--hardening");

const hardening = [
  ["node", ["tools/goal04/verify-seeded-matrix.mjs", "."]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "activity_suite", "activity_hardening", "--all-features"]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "exhaustive_suite", "replay", "--all-features"]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "adapter_suite", "activity_session_loop", "--all-features"]],
  ["node", ["tools/repository-check/verify-generated-drift.mjs"]],
  ["node", ["tools/goal-hardening/verify-release-contract.mjs"]],
  ["node", ["tools/agent-control/verify-goal02-release-contract.mjs"]],
  ["node", ["tools/universe-reference/verify-release.mjs", "."]],
  ["node", ["tools/goal04/verify-determinism-hardening.mjs", "."]]
];

if (process.argv[2] === "--hardening") {
  for (const [command, args] of hardening) execFileSync(command, args, { stdio: "inherit" });
  console.log("Goal 04 native determinism hardening gate passed.");
  process.exit(0);
}

for (const [command, args] of [
  ["node", ["tools/goal04/verify-surface-audit.mjs", "."]],
  ["node", ["tools/goal04/verify-interface-contract.mjs", "."]],
  ["node", ["tools/goal04/generate-runtime-dispositions.mjs", ".", "--check"]],
  ["node", ["tools/goal04/verify-runtime-dispositions.mjs", "."]],
  ["node", ["tools/goal04/verify-foundation.mjs", "."]],
  ["node", ["tools/goal04/verify-catalog-bootstrap.mjs", "."]],
  ["node", ["tools/goal04/verify-structural-catalog.mjs", "."]],
  ["node", ["tools/goal04/verify-path-catalog.mjs", "."]],
  ["node", ["tools/goal04/verify-curio-catalog.mjs", "."]],
  ["node", ["tools/goal04/verify-run-catalog.mjs", "."]],
  ["node", ["tools/goal04/verify-encounter-catalog.mjs", "."]],
  ["node", ["tools/goal04/verify-activity-graph.mjs", "."]],
  ["node", ["tools/goal04/verify-activity-state.mjs", "."]],
  ["node", ["tools/goal04/verify-activity-transaction.mjs", "."]],
  ["node", ["tools/goal04/verify-activity-rng-state.mjs", "."]],
  ["node", ["tools/goal04/verify-battle-preparation.mjs", "."]],
  ["node", ["tools/goal04/verify-battle-settlement.mjs", "."]],
  ["node", ["tools/goal04/verify-activity-hardening.mjs", ".", "--run"]],
  ["node", ["tools/goal04/verify-universe-entry.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-topology.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-encounter-runtime.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-blessing-runtime.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-path-runtime.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-curio-runtime.mjs", "."]],
  ["node", ["tools/goal04/verify-universe-run-runtime.mjs", "."]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "activity_suite", "--all-features"]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "universe_suite", "--all-features"]],
  ["node", ["tools/goal04/verify-release-contract.mjs", ".", "--scaffold"]]
]) execFileSync(command, args, { stdio: "inherit" });

console.log("Goal 04 native foundation gate passed.");
