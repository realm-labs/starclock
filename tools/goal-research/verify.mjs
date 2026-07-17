import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.cwd());
const evidenceRoot = path.join(root, "evidence", "core-combat-v1", "research-register");
const refRoot = path.join(root, "content-reference", "v4.4");
const index = read("evidence-index.json");
for (const entry of index.files) assert(sha(path.join(evidenceRoot, entry.name)) === entry.sha256, `${entry.name} hash mismatch`);
assert(index.evidence_sha256 === hashText(index.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("")), "index digest mismatch");

const register = read("research-cases.json");
const fixtures = read("fixture-specifications.json");
const decisions = read("decision-records.json");
const sources = read("source-register.json");
const cases = register.cases;
assert(cases.length === 37 && register.case_count === 37, `expected 37 cases, got ${cases.length}`);
assert(new Set(cases.map((entry) => entry.id)).size === cases.length, "duplicate research case ID");
assert(fixtures.fixtures.length === cases.length && new Set(fixtures.fixtures.map((entry) => entry.case_id)).size === cases.length, "fixture coverage mismatch");
assert(decisions.decisions.length === 6, "decision record coverage mismatch");
assert(sources.sources.length === 3 && sources.sources.every((entry) => /^https:\/\//.test(entry.url) && /^2026-\d\d-\d\d$/.test(entry.accessed_on) && /^[0-9a-f]{64}$/.test(entry.evidence_sha256) && entry.version && entry.confidence && entry.note), "source metadata incomplete");

for (const entry of cases) {
  assert(["Researching", "Observed"].includes(entry.state) && entry.owner_batch, `${entry.id} has no research owner`);
  assert(entry.question && entry.fixed_expectations.length && entry.observations_required.length, `${entry.id} lacks a bounded question`);
  assert(entry.evidence.length && entry.evidence.every((binding) => /^[0-9a-f]{64}$/.test(binding.source_text_sha256)), `${entry.id} lacks evidence hashes`);
  assert(entry.source_ids.length && entry.source_ids.every((id) => sources.sources.some((source) => source.id === id)), `${entry.id} has an unknown source`);
  assert(fixtures.fixtures.some((fixture) => fixture.case_id === entry.id && fixture.assertions.length && fixture.replay_requirements.length === 4), `${entry.id} lacks a reproducible fixture`);
}
const observed = cases.filter((entry) => entry.state === "Observed");
assert(observed.length === 2 && observed.every((entry) => entry.owner_batch === "G01-P4-B2" && /^[0-9a-f]{64}$/.test(entry.observation.source_payload_sha256) && /^[0-9a-f]{64}$/.test(entry.observation.executable_bundle_sha256) && entry.observation.evidence_paths.length === 2 && entry.observation.validation_commands.length === 2), "B2 observed cases lack executable evidence");
assert(fixtures.fixtures.filter((fixture) => fixture.state === "GoldenVerified").length === 2, "B2 golden fixture state differs");

for (const family of ["V1aAsta", "V1aKafka", "V1aClara", "V1aFirefly", "V1aAglaea", "SharedElation", "HimekoNovaApproximation"]) {
  assert(cases.some((entry) => entry.family === family), `missing ${family} cases`);
}
const elationRecords = new Set(cases.filter((entry) => entry.family === "SharedElation").flatMap((entry) => entry.evidence.map((binding) => binding.record_id.split(".ability.")[0])));
assert(elationRecords.size >= 2, "shared Elation cases use fewer than two released forms");

const abilities = JSON.parse(fs.readFileSync(path.join(refRoot, "character-abilities.json"), "utf8"));
const expectedHimeko = abilities.filter((row) => row.character_id === "character.himeko-nova" && row.mechanism_quality === "ApproximateFromReleasedText").map((row) => row.id).sort();
const actualHimeko = cases.filter((entry) => entry.family === "HimekoNovaApproximation").flatMap((entry) => entry.evidence.map((binding) => binding.record_id)).sort();
assert(JSON.stringify(expectedHimeko) === JSON.stringify(actualHimeko), "Himeko Nova approximation coverage mismatch");
assert(cases.filter((entry) => entry.family === "HimekoNovaApproximation").every((entry) => entry.owner_batch === "G01-P7-M01" && entry.dependent_batch === "G01-P7-C04"), "Himeko dependency ownership mismatch");

console.log(`Goal research register verified (${index.evidence_sha256}; 37 named cases; ${elationRecords.size} Elation forms).`);

function read(name) { return JSON.parse(fs.readFileSync(path.join(evidenceRoot, name), "utf8")); }
function sha(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function hashText(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
