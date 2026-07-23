import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/generated-drift.json"), "utf8"));
assert(policy.schema_revision === "starclock.generated-drift.v1", "unsupported generated-drift policy revision");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--with-source-cache"), `unsupported generated-drift argument: ${arguments_.find((argument) => argument !== "--with-source-cache")}`);
const includeSourceCache = arguments_.includes("--with-source-cache");

let completed = 0;
let skipped = 0;
const timings = [];
for (const check of policy.checks) {
  assert(typeof check.name === "string" && check.name.length > 0, "generated-drift check requires a name");
  assert(Array.isArray(check.command) && check.command.length > 1, `${check.name}: command must contain a program and arguments`);
  if (check.requires === "source-cache" && !includeSourceCache) {
    console.log(`SKIP ${check.name} (pass --with-source-cache to verify ignored evidence inputs)`);
    skipped += 1;
    continue;
  }
  assert(check.requires === undefined || check.requires === "source-cache", `${check.name}: unsupported requirement`);
  console.log(`RUN  ${check.name}`);
  const began = Date.now();
  run(check.command);
  timings.push({ name: check.name, elapsed_ms: Date.now() - began });
  completed += 1;
}

const reportPath = path.join(root, ".cache", "repository-check", "generated-drift-timings.json");
fs.mkdirSync(path.dirname(reportPath), { recursive: true });
fs.writeFileSync(reportPath, `${JSON.stringify({
  schema_revision: "starclock.generated-drift-timings.v1",
  result: "pass",
  checks: timings,
  elapsed_ms: timings.reduce((total, entry) => total + entry.elapsed_ms, 0),
  slowest_checks: [...timings].sort((left, right) => right.elapsed_ms - left.elapsed_ms).slice(0, 12),
}, null, 2)}\n`);
console.log(`Generated-artifact policy verified (${completed} checks, ${skipped} cache-dependent check skipped).`);

function run(command) {
  const result = spawnSync(command[0], command.slice(1), {
    cwd: root,
    stdio: "inherit",
    env: { ...process.env, STARCLOCK_ARTIFACT_CHECK_ONLY: "1" },
  });
  if (result.error) throw result.error;
  assert(result.status === 0, `command failed (${result.status}): ${command.join(" ")}`);
}
function assert(condition, message) { if (!condition) throw new Error(message); }
