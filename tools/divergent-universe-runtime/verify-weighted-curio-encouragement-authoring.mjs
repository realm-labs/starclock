#!/usr/bin/env node
// Exact released operands and joins, never battle-execution credit.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const json = file => JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
const debug = "config/divergent-universe-decisions-generated/debug-json";
const rows = table => json(`${debug}/${table}.json`).table.rows.map(row => row.values);
const authored = rows("DuWeightedCurioEncouragements");
assert.equal(authored.length, 1);
const row = authored[0];
assert.equal(row.stable_key.String, "du.weighted-curio-encouragement.encouragement-for-you");
assert.equal(row.weighted_curio_key.String, "divergent-universe.weighted-curio.1008");
assert.equal(row.maze_buff_id.String, "633408");
assert.deepEqual(row.character_paths.List.map(value => value.String), ["Elation"]);
assert.equal(row.crit_parameter.Integer, 1);
assert.equal(row.follow_up_crit_damage.String, "1.5");
assert.equal(row.policy.String, "OriginalElationFollowUpDamage");
assert.match(row.policy_note.String, /^VersionedProjectPolicy:/u);
assert.match(row.policy_note.String, /without changing DamageClass::Elation/u);
assert.match(row.policy_note.String, /does not implement or admit the effect/u);
assert.match(row.policy_note.String, /BattleStarted\/AfterEvent/u);
assert.match(row.policy_note.String, /unitless timeline actors/u);
assert.match(row.policy_note.String, /actual original-unit producer/u);
assert.match(row.replacement_condition.String, /do not prove battle execution/u);
assert.deepEqual(row.source_ids.List.map(value => value.Integer), [118, 119, 120, 121]);
const state = json("policy/state.json").divergent_universe.decision_authoring;
const tables = fs.readdirSync(path.join(root, debug)).filter(file => file.endsWith(".json"));
assert.equal(state.tables, tables.length);
assert.equal(state.rows, tables.reduce((total, file) => total + json(`${debug}/${file}`).table.rows.length, 0));
assert.equal(state.reader_files, fs.readdirSync(path.join(root, "config/divergent-universe-decisions-generated/reader")).filter(file => file.endsWith(".rs")).length);
const bundle = fs.readFileSync(path.join(root, "config/divergent-universe-decisions-generated/config.sora"));
assert.equal(state.bundle_bytes, bundle.length);
assert.equal(state.bundle_sha256, crypto.createHash("sha256").update(bundle).digest("hex"));
assert.equal(state.weighted_curio_encouragement_operands_authored, true);
assert.equal(state.weighted_curio_encouragement_battle_effect_implemented, true);
const required = [
  [118, "hex", "ExcelOutput/RogueTournHex.json", "HexID=1008;", "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [119, "maze-buff", "ExcelOutput/MazeBuff.json", "ID=633408;", "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [120, "text-en", "TextMap/TextMapEN.json", "hash=2693397682820933078;", "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [121, "text-zh", "TextMap/TextMapCHS.json", "hash=2693397682820933078;", "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
];
const sources = rows("DuDecisionSources");
for (const [id, suffix, file, locator, digest] of required) {
  const source = sources.find(value => value.id.Integer === id);
  assert.ok(source, `missing source ${id}`);
  assert.equal(source.stable_key.String, `du.source.weighted-curio-encouragement.${suffix}`);
  assert.equal(source.url.String, "https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String, revision);
  assert.equal(source.game_version.String, "4.4");
  assert.equal(source.access_date.String, "2026-10-02");
  assert.equal(source.sha256.String, digest);
  assert.equal(source.quality.String, "ExactStructured");
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
if (process.argv.includes("--check-source")) {
  const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
  const docs = new Map();
  for (const [, , file, , expected] of required) {
    const bytes = execFileSync("git", ["-C", sourceRoot, "show", `${revision}:${file}`], {maxBuffer: 128 * 1024 * 1024});
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"), expected, `source drift: ${file}`);
    docs.set(file, JSON.parse(bytes.toString("utf8")
      .replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"')));
  }
  const hex = docs.get("ExcelOutput/RogueTournHex.json").filter(value => value.HexID === 1008 && value.TournMode === "Tourn3");
  assert.equal(hex.length, 1);
  assert.equal(hex[0].MazeBuffID, 633408);
  assert.equal(hex[0].DisplayID, 1029);
  assert.deepEqual(hex[0].AvatarType, ["Elation"]);
  assert.deepEqual(hex[0].AvatarDamageType, []);
  const flatten = value => Array.isArray(value) ? value.flatMap(flatten)
    : value && typeof value === "object" && !Object.hasOwn(value, "ID") ? Object.values(value).flatMap(flatten) : [value];
  const buffs = flatten(docs.get("ExcelOutput/MazeBuff.json")).filter(value => value?.ID === 633408 && value.Lv === 1);
  assert.equal(buffs.length, 1);
  assert.equal(buffs[0].InBattleBindingType, "StageAbilityBeforeCharacterBorn");
  assert.equal(buffs[0].InBattleBindingKey, "StageAbility_633408");
  assert.deepEqual(buffs[0].ParamList.map(value => value.Value), ["1.5"]);
  assert.equal(buffs[0].BuffDesc.Hash, "2693397682820933078");
  assert.equal(buffs[0].BuffDescBattle.Hash, "2693397682820933078");
  for (const [file, words] of [
    ["TextMap/TextMapEN.json", ["Elation", "Follow-Up ATK DMG", "CRIT DMG", "#1[i]%"]],
    ["TextMap/TextMapCHS.json", ["欢愉", "追加攻击伤害", "暴击伤害", "#1[i]%"]],
  ]) for (const word of words) assert.ok(docs.get(file)["2693397682820933078"].includes(word));
  const tree = execFileSync("git", ["-C", sourceRoot, "ls-tree", "-r", "--name-only", revision], {encoding: "utf8", maxBuffer: 128 * 1024 * 1024});
  assert.equal(tree.split("\n").filter(file => file.includes("633408")).length, 0,
    "released program now exists; replace the missing-program policy premise");
}
console.log("Encouragement for You operands and provenance verified; no native battle execution or original-game parity claim.");
