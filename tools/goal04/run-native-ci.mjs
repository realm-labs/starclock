import process from "node:process";
import { execFileSync } from "node:child_process";

if (process.argv.length !== 3 || process.argv[2] !== "--foundation")
  throw new Error("usage: run-native-ci.mjs --foundation");

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
  ["cargo", ["test", "-p", "starclock-activity", "--all-targets", "--all-features"]],
  ["cargo", ["test", "-p", "starclock-mode-universe", "--all-targets", "--all-features"]],
  ["node", ["tools/goal04/verify-release-contract.mjs", ".", "--scaffold"]]
]) execFileSync(command, args, { stdio: "inherit" });

console.log("Goal 04 native foundation gate passed.");
