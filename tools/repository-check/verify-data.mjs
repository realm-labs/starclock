import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/data-checks.json"), "utf8"));

if (!Array.isArray(policy.checks) || policy.checks.length === 0) {
  throw new Error("current data check list is empty");
}

for (const check of policy.checks) {
  if (typeof check.name !== "string" || !Array.isArray(check.command) || check.command.length === 0) {
    throw new Error("invalid current data check");
  }
  console.log(`\n==> ${check.name}`);
  const result = spawnSync(check.command[0], check.command.slice(1), {
    cwd: root,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

console.log(`\nCurrent data checks passed (${policy.checks.length} checks).`);
