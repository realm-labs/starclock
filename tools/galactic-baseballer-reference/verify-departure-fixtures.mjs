#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-departure-fixtures.mjs",
  ),
  "--check",
], { cwd: root, stdio: "inherit" });
const rules = JSON.parse(await readFile(path.join(
  fragmentRoot,
  "departure-mechanic-rules.json",
), "utf8"));
const fixtures = JSON.parse(await readFile(path.join(
  fragmentRoot,
  "departure-review-fixtures.json",
), "utf8"));
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
const expectedFamilies = [
  "profile-version-selection",
  "stage-difficulty-selection",
  "wave-battle-phase-progression",
  "experience-team-level-up",
  "random-upgrade-candidates",
  "weapon-acquisition-duplicate-upgrade",
  "accessory-acquisition-duplicate-upgrade",
  "slot-capacity-expansion-replacement",
  "weapon-automatic-action",
  "character-action-triggered-weapon",
  "resonance-accessory-binding",
  "legendary-weapon-synthesis",
  "adventure-strategy",
  "team-bonus",
  "score-rating-clear",
  "boss-phase-final-settlement",
  "no-legal-candidate-failure-invariance",
].sort();
assert(rules.length === 17 && fixtures.length === 17, "Departure fixture count drift");
assert(
  JSON.stringify(rules.map(({ family_id: id }) => id).sort())
    === JSON.stringify(expectedFamilies),
  "Departure rule family set drift",
);
assert(
  JSON.stringify(fixtures.map(({ family_id: id }) => id).sort())
    === JSON.stringify(expectedFamilies),
  "Departure fixture family set drift",
);
for (const fixture of fixtures) {
  for (const field of [
    "source_record_ids",
    "trigger_point",
    "state_owner",
    "preconditions",
    "input",
    "ordered_operations",
    "expected_facts",
    "evidence_refs",
    "evidence_quality",
    "mechanism_quality",
  ]) {
    assert(fixture[field] !== undefined, `missing ${field}: ${fixture.id}`);
  }
  assert(
    fixture.source_record_ids.length >= 1
      && fixture.ordered_operations.length >= 1
      && fixture.evidence_refs.length >= 1
      && fixture.runtime_executable === false,
    `incomplete ReferenceOnly fixture: ${fixture.id}`,
  );
}
for (const rule of rules) {
  assert(
    rule.fixture_ids.length === 1
      && fixtures.some(({ id }) => id === rule.fixture_ids[0])
      && rule.ordered_operations.length >= 1
      && rule.runtime_executable === false,
    `rule/fixture binding drift: ${rule.id}`,
  );
}
console.log(
  "Departure semantic fragments verified: 17 mechanism families, "
  + "17 ReferenceOnly rules and 17 concrete review fixtures",
);
