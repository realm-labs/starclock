import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));
const fail = (message) => {
  throw new Error(`agent-control surface audit: ${message}`);
};

const policy = await readJson("policy/agent-control-surfaces.json");
const manifest = await readJson("content-manifests/core-combat-v1/standard-v1.json");
const release = await readJson("policy/release-contract.json");

if (policy.schema_revision !== "starclock.agent-control.surface-audit.v1") {
  fail("unexpected policy revision");
}
if (policy.standard_scenarios.length !== 6 || manifest.scenarios.length !== 6) {
  fail("the Standard denominator must remain exactly six");
}

const frozen = new Map(policy.standard_scenarios.map((scenario) => [scenario.stable_id, scenario]));
for (const scenario of manifest.scenarios) {
  const expected = frozen.get(scenario.id);
  if (!expected) fail(`manifest scenario ${scenario.id} is not frozen`);
  if (expected.encounter_stable_id !== scenario.encounter_id) {
    fail(`${scenario.id} encounter drifted`);
  }
  if (String(expected.default_seed) !== scenario.seed) {
    fail(`${scenario.id} seed drifted`);
  }
  frozen.delete(scenario.id);
}
if (frozen.size !== 0) fail("policy contains a scenario absent from the manifest");

if (policy.production_bundle_sha256 !== release.production.bundle_sha256) {
  fail("production bundle binding disagrees with the Goal 01 release contract");
}
const bundle = await readFile(release.production.bundle_path);
const digest = createHash("sha256").update(bundle).digest("hex");
if (digest !== policy.production_bundle_sha256) fail("production bundle bytes drifted");

const requiredOwners = new Set(["System", "Team(Player)", "Team(Enemy)"]);
for (const row of policy.decision_owner_matrix) requiredOwners.delete(row.owner);
if (requiredOwners.size !== 0) fail("decision-owner matrix is incomplete");
if (policy.narrow_application_seams.length !== 3 || policy.forbidden_core_changes.length < 6) {
  fail("application seams or forbidden-change boundary is incomplete");
}

console.log(`agent-control surface audit verified: 6 scenarios, bundle ${digest}`);
