#!/usr/bin/env node
// Released operands and source joins, never battle-execution credit.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
const debug = "config/divergent-universe-decisions-generated/debug-json";
const json = file => JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
const rows = file => json(`${debug}/${file}.json`).table.rows.map(row => row.values);
const authored = rows("DuWeightedCurioExcitations");
assert.equal(authored.length, 1);
const row = authored[0];
assert.equal(row.stable_key.String, "du.weighted-curio-excitation.genius-confusion");
assert.equal(row.weighted_curio_key.String, "divergent-universe.weighted-curio.1006");
assert.equal(row.maze_buff_id.String, "633406");
assert.deepEqual(row.character_elements.List.map(value => value.String), ["Quantum"]);
assert.deepEqual([row.gain_parameter.Integer, row.consume_parameter.Integer,
  row.damage_parameter.Integer, row.chance_parameter.Integer, row.duration_parameter.Integer], [1, 2, 3, 4, 5]);
assert.deepEqual([row.stack_gain.Integer, row.stack_consumption.Integer,
  row.attack_multiplier.String, row.base_chance.String, row.duration_turns.Integer], [2, 1, "2.5", "0.5", 1]);
assert.equal(row.policy_maximum_stacks.Integer, 65535);
assert.equal(row.policy.String, "EffectiveGainActionResolvedTeamConsumption");
assert.match(row.policy_note.String, /^VersionedProjectPolicy:/u);
assert.match(row.policy_note.String, /policy maximum 65535/u);
assert.match(row.policy_note.String, /does not implement or admit the effect/u);
assert.match(row.replacement_condition.String, /do not prove battle execution/u);
assert.deepEqual(row.source_ids.List.map(value => value.Integer), [114, 115, 116, 117]);
const sources = rows("DuDecisionSources");
const state = json("policy/state.json").divergent_universe.decision_authoring;
const tableFiles = fs.readdirSync(path.join(root, debug)).filter(file => file.endsWith(".json")).sort();
assert.equal(state.tables, tableFiles.length);
assert.equal(state.rows, tableFiles.reduce((total, file) => total + json(`${debug}/${file}`).table.rows.length, 0));
assert.equal(state.reader_files, fs.readdirSync(path.join(root, "config/divergent-universe-decisions-generated/reader"))
  .filter(file => file.endsWith(".rs")).length);
const bundle = fs.readFileSync(path.join(root, "config/divergent-universe-decisions-generated/config.sora"));
assert.equal(state.bundle_bytes, bundle.length);
assert.equal(state.bundle_sha256, crypto.createHash("sha256").update(bundle).digest("hex"));
assert.equal(state.weighted_curio_excitation_operands_authored, true);
assert.equal(state.weighted_curio_excitation_battle_effect_implemented, true);
assert.equal(state.weighted_curio_battle_effects_implemented, false);
const required = [
  [114, "hex", "ExcelOutput/RogueTournHex.json", "HexID=1006;",
    "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [115, "maze-buff", "ExcelOutput/MazeBuff.json", "ID=633406;",
    "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [116, "text-en", "TextMap/TextMapEN.json", "hash=332970425552739522;",
    "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [117, "text-zh", "TextMap/TextMapCHS.json", "hash=332970425552739522;",
    "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
];
for (const [id, suffix, file, locator, digest] of required) {
  const source = sources.find(value => value.id.Integer === id);
  assert.ok(source, `missing source ${id}`);
  assert.equal(source.stable_key.String, `du.source.weighted-curio-excitation.${suffix}`);
  assert.equal(source.url.String, "https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String, revision);
  assert.equal(source.game_version.String, "4.4");
  assert.equal(source.access_date.String, "2026-10-02");
  assert.equal(source.sha256.String, digest);
  assert.equal(source.quality.String, "ExactStructured");
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
if (process.argv.includes("--check-source")) {
  const docs = new Map();
  for (const [, , file, , expected] of required) {
    const bytes = execFileSync("git", ["-C", sourceRoot, "show", `${revision}:${file}`], {maxBuffer: 128 * 1024 * 1024});
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"), expected, `source drift: ${file}`);
    docs.set(file, JSON.parse(bytes.toString("utf8")
      .replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"')));
  }
  const hex = docs.get("ExcelOutput/RogueTournHex.json").filter(value => value.HexID === 1006 && value.TournMode === "Tourn3");
  assert.equal(hex.length, 1);
  assert.equal(hex[0].MazeBuffID, 633406);
  assert.equal(hex[0].DisplayID, 1011);
  assert.deepEqual(hex[0].AvatarType, []);
  assert.deepEqual(hex[0].AvatarDamageType, ["Quantum"]);
  const flatten = value => Array.isArray(value) ? value.flatMap(flatten)
    : value && typeof value === "object" && !Object.hasOwn(value, "ID") ? Object.values(value).flatMap(flatten) : [value];
  const buffs = flatten(docs.get("ExcelOutput/MazeBuff.json")).filter(value => value?.ID === 633406 && value.Lv === 1);
  assert.equal(buffs.length, 1);
  assert.equal(buffs[0].InBattleBindingType, "StageAbilityBeforeCharacterBorn");
  assert.equal(buffs[0].InBattleBindingKey, "StageAbility_633406");
  assert.deepEqual(buffs[0].ParamList.map(value => value.Value), ["2", "1", "2.5", "0.5", "1"]);
  assert.equal(buffs[0].BuffDesc.Hash, "332970425552739522");
  assert.equal(buffs[0].BuffDescBattle.Hash, "332970425552739522");
  const en = docs.get("TextMap/TextMapEN.json")["332970425552739522"];
  const zh = docs.get("TextMap/TextMapCHS.json")["332970425552739522"];
  for (const word of ["Quantum", "Skill Point", "Excitation", "Basic ATK/Skill", "Additional DMG", "Entanglement", "#1[i]", "#2[i]", "#3[i]%", "#4[i]%", "#5[i]"]) assert.ok(en.includes(word));
  for (const word of ["量子", "战技点", "激发", "普攻/战技", "附加伤害", "纠缠", "#1[i]", "#2[i]", "#3[i]%", "#4[i]%", "#5[i]"]) assert.ok(zh.includes(word));
  const tree = execFileSync("git", ["-C", sourceRoot, "ls-tree", "-r", "--name-only", revision], {encoding: "utf8", maxBuffer: 128 * 1024 * 1024});
  assert.equal(tree.split("\n").filter(file => file.includes("633406")).length, 0,
    "released program now exists; replace the missing-program policy premise");
}
console.log("Genius' Confusion operands and provenance verified; authoring checks do not prove native battle execution or original-game parity.");
