#!/usr/bin/env node
// Exact source facts and reviewed resource policy; not an Aha subsystem claim.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const rows = (folder, name) => JSON.parse(fs.readFileSync(path.join(root, folder, `${name}.json`), "utf8")).table.rows.map(r => r.values);
const decision = "config/divergent-universe-decisions-generated/debug-json";
const authored = rows(decision, "DuBattleTeamResources");
assert.equal(authored.length, 1);
const [row] = authored;
assert.equal(row.stable_key.String, "du.battle-team-resource.punchline");
assert.equal(row.resource_key.String, "shared.punchline");
assert.deepEqual(row.character_paths.List.map(v => v.String), ["Elation"]);
assert.deepEqual([row.initial_value.Integer, row.maximum_value.Integer], [0, 9999]);
assert.equal(row.policy.String, "OriginalElationZeroClampPersist");
for (const clause of ["VersionedProjectPolicy", "not observed", "Persist across waves", "No Aha actor"])
  assert.ok(row.policy_note.String.includes(clause), clause);
assert.ok(row.replacement_condition.String.includes("Low confidence"));
assert.deepEqual(row.source_ids.List.map(v => v.Integer), [111, 112, 113]);
const state = JSON.parse(fs.readFileSync(path.join(root, "policy/state.json"), "utf8")).divergent_universe.decision_authoring;
assert.equal(state.weighted_curio_elation_shared_meter_assembly_implemented, true);
assert.equal(state.weighted_curio_elation_battle_effect_implemented, false);
assert.equal(state.punchline_assembly_policy, "VersionedProjectPolicyOriginalElationZeroClampPersist");
assert.deepEqual([state.punchline_initial_value, state.punchline_maximum_value, state.punchline_wave_policy], [0, 9999, "Persist"]);
assert.equal(state.punchline_cross_battle_carry_implemented, false);
assert.equal(state.ability_team_resource_non_cost_deltas_implemented, false);
assert.equal(state.aha_actor_and_certified_banger_lifecycle_implemented, false);
const required = [
  [111, "global-cap", "Config/GlobalConfig/GameCoreConstValue.json", "ElationPointMax=9999",
    "5511ff36c631da99925c8aadae8ae46f50d620f4f937dbbed952fd61b16b80e3"],
  [112, "text-en", "TextMap/TextMapEN.json", "hash=8389201339365092983;",
    "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [113, "text-zh", "TextMap/TextMapCHS.json", "hash=8389201339365092983;",
    "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
];
const sources = rows(decision, "DuDecisionSources");
for (const [id, suffix, file, locator, digest] of required) {
  const source = sources.find(v => v.id.Integer === id);
  assert.ok(source);
  for (const [field, expected] of Object.entries({stable_key: `du.source.battle-team-resource.${suffix}`,
    url: "https://gitlab.com/Dimbreath/turnbasedgamedata", revision, game_version: "4.4",
    access_date: "2026-10-02", sha256: digest, quality: "ExactStructured"})) assert.equal(source[field].String, expected);
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
  if (process.argv.includes("--check-source")) {
    const bytes = execFileSync("git", ["-C", path.join(root, ".cache/content-reference/turnbasedgamedata"), "show", `${revision}:${file}`], {maxBuffer:128*1024*1024});
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"), digest);
    const doc = JSON.parse(bytes.toString("utf8"));
    if (id === 111) assert.equal(doc.ElationPointMax, 9999);
    if (id === 112) assert.match(doc["8389201339365092983"], /Punchline is shared by the whole team/u);
    if (id === 113) assert.match(doc["8389201339365092983"], /笑点为全队共享/u);
  }
}
const consumers = rows("config/generated/debug-json", "AbilityResourceDelta")
  .filter(v => v.resource_kind.String === "TeamResource" && v.character_resource_key.String === "shared.punchline");
assert.deepEqual(consumers.map(v => [v.ability_id.Integer, v.amount_decimal.String]), [[100050,"2"], [110050,"5"], [120045,"3"], [120046,"5"]]);
console.log("Shared Punchline source facts and independent assembly policy verified; non-cost production resource consumers, Sapient Pen and Aha remain unimplemented.");
