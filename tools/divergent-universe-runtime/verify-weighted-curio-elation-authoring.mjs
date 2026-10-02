#!/usr/bin/env node
// Released operands and production source joins, never battle-execution credit.
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
const authored = rows("DuWeightedCurioElations");
assert.equal(authored.length, 1);
const row = authored[0];
assert.equal(row.stable_key.String, "du.weighted-curio-elation.sapient-pen");
assert.equal(row.weighted_curio_key.String, "divergent-universe.weighted-curio.1015");
assert.equal(row.maze_buff_id.String, "633415");
assert.deepEqual(row.character_paths.List.map(value => value.String), ["Elation"]);
assert.deepEqual([row.gain_parameter.Integer, row.bonus_parameter.Integer, row.duration_parameter.Integer], [1, 2, 3]);
assert.deepEqual([row.punchline_gain.Integer, row.elation_bonus.String, row.duration_turns.Integer], [2, "0.5", 2]);
assert.equal(row.policy.String, "ActionResolvedOriginalPartyRefresh");
assert.match(row.policy_note.String, /authored but not yet executed/u);
assert.match(row.policy_note.String, /separate BattleTeamResources assembly policy/u);
assert.match(row.replacement_condition.String, /do not prove battle execution/u);
assert.deepEqual(row.source_ids.List.map(value => value.Integer), [107, 108, 109, 110]);
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
assert.equal(state.weighted_curio_elation_operands_authored, true);
assert.equal(state.weighted_curio_elation_battle_effect_implemented, false);
assert.equal(state.weighted_curio_elation_shared_meter_assembly_implemented, true);
const required = [
  [107, "hex", "ExcelOutput/RogueTournHex.json", "HexID=1015;",
    "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [108, "maze-buff", "ExcelOutput/MazeBuff.json", "ID=633415;",
    "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [109, "text-en", "TextMap/TextMapEN.json", "hash=7976287788830292485;",
    "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [110, "text-zh", "TextMap/TextMapCHS.json", "hash=7976287788830292485;",
    "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
];
for (const [id, suffix, file, locator, digest] of required) {
  const source = sources.find(value => value.id.Integer === id);
  assert.ok(source, `missing source ${id}`);
  assert.equal(source.stable_key.String, `du.source.weighted-curio-elation.${suffix}`);
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
    // Hash locators exceed JS exact-integer range; decimal operands stay strings.
    docs.set(file, JSON.parse(bytes.toString("utf8")
      .replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"')));
  }
  const hex = docs.get("ExcelOutput/RogueTournHex.json").filter(value => value.HexID === 1015 && value.TournMode === "Tourn3");
  assert.equal(hex.length, 1);
  assert.equal(hex[0].MazeBuffID, 633415);
  assert.equal(hex[0].DisplayID, 1027);
  assert.deepEqual(hex[0].AvatarType, ["Elation"]);
  assert.deepEqual(hex[0].AvatarDamageType, []);
  const flatten = value => Array.isArray(value) ? value.flatMap(flatten)
    : value && typeof value === "object" && !Object.hasOwn(value, "ID") ? Object.values(value).flatMap(flatten) : [value];
  const buffs = flatten(docs.get("ExcelOutput/MazeBuff.json")).filter(value => value?.ID === 633415 && value.Lv === 1);
  assert.equal(buffs.length, 1);
  assert.equal(buffs[0].InBattleBindingType, "StageAbilityBeforeCharacterBorn");
  assert.equal(buffs[0].InBattleBindingKey, "StageAbility_633415");
  assert.deepEqual(buffs[0].ParamList.map(value => value.Value), ["2", "0.5", "2"]);
  assert.equal(buffs[0].BuffDesc.Hash, "7976287788830292485");
  assert.equal(buffs[0].BuffDescBattle.Hash, "7976287788830292485");
  const en = docs.get("TextMap/TextMapEN.json")["7976287788830292485"];
  const zh = docs.get("TextMap/TextMapCHS.json")["7976287788830292485"];
  for (const word of ["Elation", "Basic ATK or Skill", "Punchline", "#1[i]", "#2[i]%", "#3[i]"]) assert.ok(en.includes(word));
  for (const word of ["欢愉", "非", "普攻/战技", "笑点", "欢愉度", "#1[i]", "#2[i]%", "#3[i]"]) assert.ok(zh.includes(word));
}
console.log("Sapient Pen production operands and provenance verified; its battle effect remains unimplemented independently of shared-meter assembly.");
