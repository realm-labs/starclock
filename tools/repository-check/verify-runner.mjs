import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runner = read("tools/repository-check/run.mjs");
const workspaceTests = read("tools/repository-check/run-workspace-tests.mjs");
const generatedDrift = read("tools/repository-check/verify-generated-drift.mjs");
const phase4 = read("tools/core-kernel/verify-phase4.mjs");
const goal04Fixture = read("tools/goal04/verify-mechanic-fixtures.mjs");
const workbookVerifier = read("tools/universe-reference/verify_production_workbooks.mjs");
const cargoManifest = read("Cargo.toml");
const release = read("tools/release/run-goal02-clean-checkout.mjs");
const readme = read("tools/repository-check/README.md");
const standards = read("docs/08-engineering-standards.md");

for (const marker of [
  "STARCLOCK_QUICK_BUDGET_SECONDS ?? \"180\"",
  "quick-rust-receipt.json",
  'CARGO_INCREMENTAL: process.env.CARGO_INCREMENTAL ?? "1"',
  "cargo\", \"clippy\"",
  "cargo\", \"test\"",
  "cargo\", \"check\"",
  "STARCLOCK_REPOSITORY_PROFILE === \"full\"",
  "process.env.CI === \"true\"",
  "tools/repository-check/verify-generated-drift.mjs",
  "cargo\", \"clippy\", \"--workspace\"",
  "tools/repository-check/run-workspace-tests.mjs",
]) assert(runner.includes(marker), `repository runner omits ${marker}`);

for (const marker of [
  '"--no-run", "--message-format=json"',
  "entry.profile?.test",
  "STARCLOCK_TEST_JOBS",
  "STARCLOCK_TEST_THREADS",
  '"--workspace", "--doc", "--all-features"',
  "workspace-test-timings.json",
]) assert(workspaceTests.includes(marker), `workspace test runner omits ${marker}`);
assert(generatedDrift.includes('STARCLOCK_ARTIFACT_CHECK_ONLY: "1"'), "artifact verification can recursively rerun Rust tests");
assert(phase4.includes("--artifacts-only"), "Phase 4 verifier lacks its non-test acceptance mode");
assert(goal04Fixture.includes('process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1"'), "Goal 04 artifact checks cannot defer workspace-owned tests");
for (const marker of ["inputFingerprint()", "universe-production-workbooks.json", "STARCLOCK_NO_ARTIFACT_CACHE"])
  assert(workbookVerifier.includes(marker), `production workbook verifier omits ${marker}`);
assert(cargoManifest.includes("[profile.test]") && cargoManifest.includes("opt-level = 1"), "simulation-heavy tests lack the bounded optimized profile");
assert(release.includes('STARCLOCK_REPOSITORY_PROFILE: "full"'), "isolated release acceptance does not force the full profile");
for (const document of [readme, standards]) {
  assert(document.includes("node tools/repository-check/run.mjs"), "quick command is undocumented");
  assert(document.includes("node tools/repository-check/run.mjs --full"), "full command is undocumented");
  assert(document.includes("180"), "quick budget is undocumented");
}
console.log("Repository acceptance profiles verified (quick 180s, cached incremental Rust scope, explicit full/release boundary).");

function read(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
