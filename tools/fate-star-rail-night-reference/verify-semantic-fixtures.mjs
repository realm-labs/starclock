#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const referenceRoot = resolve(root, "content-reference/fate-star-rail-night-v1");
const pack = json("pack-index.json");
const records = pack.files
  .filter(({ records: count }) => count > 0)
  .map(({ path }) => json(path))
  .filter((payload) => Array.isArray(payload.records))
  .flatMap(({ records: values }) => values);
const byId = new Map(records.map((row) => [row.stable_id, row]));
const fixtures = json("review-fixtures.json").fixtures;
const policies = json("research-gaps.json").policies;
assert(fixtures.length === 58 && new Set(fixtures.map(({ fixture_id }) => fixture_id)).size === 58, "fixture denominator drift");
let assertions = 0;
for (const fixture of fixtures) {
  assert(fixture.commands.length === 1 && fixture.commands[0].kind === "InspectReferenceFact", `${fixture.fixture_id}: invalid command`);
  const row = byId.get(fixture.initial_state.stable_id);
  assert(row, `${fixture.fixture_id}: missing source row`);
  assert(fixture.source_refs.every((reference) => row.source_refs.some((candidate) => candidate.path === reference.path && candidate.locator === reference.locator && candidate.sha256 === reference.sha256)), `${fixture.fixture_id}: source receipt mismatch`);
  for (const expected of fixture.expected_facts) {
    assert(expected.op === "equals", `${fixture.fixture_id}: unsupported assertion`);
    assert(row[expected.path] === expected.value, `${fixture.fixture_id}: expected ${expected.path}=${expected.value}`);
    assertions += 1;
  }
}
const fixtureIds = new Set(fixtures.map(({ fixture_id }) => fixture_id));
assert(policies.length === 13 && new Set(policies.map(({ policy_id }) => policy_id)).size === 13, "policy denominator drift");
for (const policy of policies) {
  assert(policy.selected_policy === "IdentityOnlyNoOperationLowering", `${policy.policy_id}: policy drift`);
  assert(policy.evidence_quality === "ProjectPolicy", `${policy.policy_id}: evidence quality drift`);
  assert(policy.rejected_alternatives.length >= 2, `${policy.policy_id}: alternatives missing`);
  assert(policy.affected_fixtures.length > 0 && policy.affected_fixtures.every((id) => fixtureIds.has(id)), `${policy.policy_id}: fixture binding drift`);
  assert(typeof policy.replacement_condition === "string" && policy.replacement_condition.startsWith("Replace when a released"), `${policy.policy_id}: replacement condition drift`);
}
console.log(`Executed 58 semantic fixtures / ${assertions} source-backed assertions and verified 13 bounded replacement policies.`);

function json(file) { return JSON.parse(readFileSync(resolve(referenceRoot, file), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
