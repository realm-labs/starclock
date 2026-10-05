// Fact/provenance verification, not native execution or source-program parity.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const programPath = "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json";
const debug = path.join(root, "config/divergent-universe-decisions-generated/debug-json");
const rows = name => JSON.parse(fs.readFileSync(path.join(debug, `${name}.json`))).table.rows.map(row => row.values);
const hash = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
const definitions = rows("DuWeightedCurioFootsteps");
assert.equal(definitions.length, 1);
const row = definitions[0];
assert.equal(row.id.Integer, 1);
assert.equal(row.stable_key.String, "du.weighted-curio-footstep.footstep-of-gods");
assert.equal(row.weighted_curio_key.String, "divergent-universe.weighted-curio.1011");
assert.equal(row.maze_buff_id.String, "633411");
assert.deepEqual(row.character_paths.List.map(value => value.String), ["Warrior", "Memory"]);
for (const [parameter, index, field, decimal] of [
  ["loss_parameter", 1, "loss_fraction", "0.5"],
  ["damage_parameter", 2, "damage_per_stack", "0.08"],
]) {
  assert.equal(row[parameter].Integer, index);
  assert.equal(row[field].String, decimal);
}
assert.equal(row.cap_parameter.Integer, 3);
assert.equal(row.maximum_stacks.Integer, 10);
assert.equal(row.policy.String, "EffectiveHpLossAfterSkillOriginalDamage");
assert.deepEqual(row.source_ids.List.map(value => value.Integer), [134, 135, 136, 137, 138, 139, 140]);
for (const prefix of ["hp", "skill"]) {
  assert.match(row[`${prefix}_policy_note`].String, /^VersionedProjectPolicy:/u);
  assert.match(row[`${prefix}_policy_note`].String, /low-confidence/u);
  assert.match(row[`${prefix}_replacement_condition`].String, /Alternatives/u);
  assert.match(row[`${prefix}_replacement_condition`].String, /Replace each field independently/u);
}
const sources = rows("DuDecisionSources");
const required = [
  [134, "hex", "ExcelOutput/RogueTournHex.json", "HexID=1011;", "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [135, "maze-buff", "ExcelOutput/MazeBuff.json", "ID=633411;", "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [136, "text-en", "TextMap/TextMapEN.json", "hash=10599000866283908992;", "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [137, "text-zh", "TextMap/TextMapCHS.json", "hash=10599000866283908992;", "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
  [138, "program", programPath, "Name=StageAbility_633411;", "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47"],
];
function context(source, id, suffix, quality) {
  assert.ok(source, `missing source ${id}`);
  assert.equal(source.stable_key.String, `du.source.weighted-curio-footstep.${suffix}`);
  assert.equal(source.url.String, "https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String, revision);
  assert.equal(source.game_version.String, "4.4");
  assert.equal(source.access_date.String, "2026-10-05");
  assert.equal(source.quality.String, quality);
}
for (const [id, suffix, file, locator, digest] of required) {
  const source = sources.find(source => source.id.Integer === id);
  context(source, id, suffix, "ExactStructured");
  assert.equal(source.sha256.String, digest);
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
for (const [id, prefix, anchor] of [
  [139, "hp", "independently-replaceable-execution-policy"],
  [140, "skill", "after-skill-damage-policy"],
]) {
  const source = sources.find(source => source.id.Integer === id);
  context(source, id, `${prefix}-policy`, "ProjectPolicy");
  assert.equal(source.locator.String, `docs/divergent-universe-weighted-curio-footstep.md#${anchor}`);
  assert.equal(source.sha256.String, hash(row[`${prefix}_policy_note`].String));
  assert.match(source.note.String, /not an upstream blob/u);
}
if (process.argv.includes("--check-source")) {
  const docs = new Map();
  for (const [, , file, , digest] of required) {
    const bytes = execFileSync("git", ["-C", path.join(root, ".cache/content-reference/turnbasedgamedata"),
      "show", `${revision}:${file}`], {maxBuffer: 128 * 1024 * 1024});
    assert.equal(hash(bytes), digest);
    // Decimal facts and large TextMap locators must never pass through JS floats.
    docs.set(file, JSON.parse(bytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"')));
  }
  const hex = docs.get(required[0][2]).filter(value => value.HexID === 1011 && value.TournMode === "Tourn3");
  assert.equal(hex.length, 1);
  assert.equal(hex[0].MazeBuffID, 633411);
  assert.deepEqual(hex[0].AvatarType, ["Warrior", "Memory"]);
  assert.deepEqual(hex[0].AvatarDamageType, []);
  const buffs = objects(docs.get(required[1][2])).filter(value => value.ID === 633411 && value.Lv === 1);
  assert.equal(buffs.length, 1);
  assert.deepEqual(buffs[0].ParamList.map(value => value.Value), ["0.5", "0.08", "10"]);
  assert.equal(buffs[0].InBattleBindingKey, "StageAbility_633411");
  assert.equal(buffs[0].BuffDesc.Hash, "10599000866283908992");
  for (const [file, words] of [[required[2][2], ["Max HP lost", "Skill Point", "Skill is used", "#1[i]%", "#2[i]%", "#3[i]"]],
    [required[3][2], ["战技点", "战技", "#1[i]%", "#2[i]%", "#3[i]"]]]) {
    for (const word of words) assert.ok(docs.get(file)[buffs[0].BuffDesc.Hash].includes(word));
  }
  const abilities = objects(docs.get(programPath)).filter(value => value.Name === "StageAbility_633411");
  assert.equal(abilities.length, 1);
  const sub = abilities[0].Modifiers.Modifier_StageAbility_633411_Sub;
  assert.deepEqual(sub._CallbackList.map(callback => callback.Event), ["OnListenHPChange", "OnAfterSkillUse"]);
  const hpNodes = objects(sub._CallbackList[0]);
  assert.ok(hpNodes.some(value => value.$type === "RPG.GameCore.ModifyTeamBoostPoint"));
  for (const name of ["LoseHP", "TotalLoseHP", "AvatarMaxHP", "BoostBP"]) {
    assert.ok(hpNodes.some(value => value.DynamicKey === name || value.DynamicKey?.Value === name));
  }
  const skillNodes = objects(sub._CallbackList[1]);
  assert.ok(skillNodes.some(value => value.$type === "RPG.GameCore.ByCurrentSkillType" && value.SkillType === "Skill"));
  assert.ok(skillNodes.some(value => value.ModifierName?.Value === "Modifier_StageAbility_633411_Effect"));
  const effect = abilities[0].Modifiers.Modifier_StageAbility_633411_Effect;
  assert.ok(objects(effect).some(value => value.Property === "AllDamageTypeAddedRatio"));
  assert.ok(objects(effect).some(value => value.$type === "RPG.GameCore.SetDynamicValueByModifierValue"));
  // These checks do not decode postfix math, hidden layer timing or calculator reach.
}
console.log("Footstep released operands, five pinned facts and two independent policy digests verified; Cargo owns execution proof.");

function objects(value) {
  if (Array.isArray(value)) return value.flatMap(objects);
  return value && typeof value === "object" ? [value, ...Object.values(value).flatMap(objects)] : [];
}
