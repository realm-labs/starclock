import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
if (arguments_.some((argument) => argument !== "--with-source-cache")) {
  throw new Error(`unsupported repository-check argument: ${arguments_.find((argument) => argument !== "--with-source-cache")}`);
}
const includeSourceCache = arguments_.includes("--with-source-cache");
const commands = [
  ["node", "tools/dependency-policy/verify.mjs"],
  ["node", "tools/workspace/verify-dependencies.mjs"],
  ["node", "tools/ci/verify-workflow.mjs"],
  ["node", "tools/repository-check/verify-source-policy.mjs"],
  ["node", "tools/repository-check/verify-native-handlers.mjs"],
  ["node", "tools/repository-check/verify-generated-drift.mjs", ...(includeSourceCache ? ["--with-source-cache"] : [])],
  ["node", "tools/benchmark/verify.mjs"],
  ["node", "tools/config-probes/verify-asta-modifier.mjs"],
  ["node", "tools/config-probes/verify-firefly-damage.mjs"],
  ["node", "tools/config-probes/verify-firefly-transform.mjs"],
  ["node", "tools/config-probes/verify-kafka-dot.mjs"],
  ["node", "tools/config-probes/verify-clara-counter.mjs"],
  ["node", "tools/config-probes/verify-aglaea-memosprite.mjs"],
  ["node", "tools/config-probes/verify-elation-probes.mjs"],
  ["cargo", "fmt", "--all", "--", "--check"],
  ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
  ["cargo", "test", "--workspace", "--all-targets", "--all-features"],
];

for (const command of commands) {
  console.log(`\n==> ${command.join(" ")}`);
  const result = spawnSync(command[0], command.slice(1), { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

console.log("\nRepository checks passed.");
