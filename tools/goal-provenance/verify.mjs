import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const PACK_SHA = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const MANIFEST_SHA = "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19";
const root = path.resolve(process.cwd());
const evidenceRoot = path.join(root, "evidence", "core-combat-v1", "reference-binding");
const manifestRoot = path.join(root, "content-manifests", "core-combat-v1");
const index = readJson(path.join(evidenceRoot, "evidence-index.json"));

for (const entry of index.files) assert(sha256File(path.join(evidenceRoot, entry.name)) === entry.sha256, `${entry.name} hash mismatch`);
const digest = sha256Text(index.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join(""));
assert(digest === index.evidence_sha256, "evidence index digest mismatch");

const provenance = readJson(path.join(evidenceRoot, "provenance-map.json"));
const cacheReport = readJson(path.join(evidenceRoot, "source-cache-report.json"));
const audit = readJson(path.join(evidenceRoot, "saber-archer-audit.json"));
assert(provenance.reference_pack_sha256 === PACK_SHA, "provenance pack digest mismatch");
assert(provenance.goal_manifest_sha256 === MANIFEST_SHA, "provenance manifest digest mismatch");
assert(cacheReport.reference_pack_sha256 === PACK_SHA, "cache report pack digest mismatch");

const characterManifest = readJson(path.join(manifestRoot, "released-character-forms.json"));
const coneManifest = readJson(path.join(manifestRoot, "released-light-cones.json"));
const standardManifest = readJson(path.join(manifestRoot, "standard-v1.json"));
const expected = [
  ...characterManifest.entries.map((entry) => `CharacterCombatForm\0${entry.id}`),
  ...coneManifest.entries.map((entry) => `LightCone\0${entry.id}`),
  ...standardManifest.enemies.map((entry) => `StandardEnemyVariant\0${entry.id}`),
  ...standardManifest.encounters.map((entry) => `StandardEncounter\0${entry.id}`),
  ...standardManifest.scenarios.map((entry) => `StandardScenario\0${entry.id}`),
  `StandardProfile\0${standardManifest.profile.id}`,
].sort();
const actual = provenance.mappings.map((entry) => `${entry.kind}\0${entry.id}`).sort();
assert(JSON.stringify(actual) === JSON.stringify(expected), "frozen goal entry mapping is not one-to-one and complete");
assert(provenance.mapping_count === 283, `expected 283 mappings, got ${provenance.mapping_count}`);
assert(provenance.required_reference_closure.source_text_evidence.unbound_approximations === 0, "unbound approximation present");
assert(cacheReport.source_inventory.expected_files === 1811 && cacheReport.source_inventory.verified_files === 1811, "source inventory is incomplete");
assert(cacheReport.source_inventory.all_hashes_match, "source hash verification failed");
assert(cacheReport.regeneration.all_files_match && cacheReport.regeneration.pack_sha256 === PACK_SHA, "reference regeneration did not match");
assert(audit.case_id === "G01-R-SABER-ARCHER-SOURCE" && audit.conclusion === "VerifiedPreviousReleaseFallback", "fallback audit unresolved");
assert(audit.characters.length === 2 && audit.characters.every((entry) => entry.absent_from_pinned_4_4_avatar_config && entry.released_text_hashes_match && entry.retained_quality === "ExactPreviousRelease"), "fallback audit checks failed");

console.log(`Goal provenance evidence verified (${index.evidence_sha256}; 283 mappings; 1811 source files).`);

function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function sha256File(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
