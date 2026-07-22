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
  ["node", "tools/ci/verify-golden-matrix.mjs"],
  ["node", "tools/agent-control/verify-agent-security-audit.mjs"],
  ["node", "tools/agent-control/verify-agent-contract-freeze.mjs"],
  ["node", "tools/repository-check/verify-source-policy.mjs"],
  ["node", "tools/repository-check/verify-native-handlers.mjs"],
  ["node", "tools/goal-hardening/verify-content-audits.mjs"],
  ["node", "tools/goal-hardening/verify-property-hardening.mjs"],
  ["node", "tools/goal-hardening/verify-architecture-audit.mjs"],
  ["node", "tools/benchmark/review-phase8.mjs", "--check"],
  ["node", "tools/goal-hardening/verify-release-contract.mjs"],
  ["node", "tools/repository-check/verify-generated-drift.mjs", ...(includeSourceCache ? ["--with-source-cache"] : [])],
  ["node", "tools/core-kernel/verify-phase4.mjs"],
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
