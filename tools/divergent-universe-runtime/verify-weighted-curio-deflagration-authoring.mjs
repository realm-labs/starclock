// Released fact/provenance checks, not execution or decoded postfix parity.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const programPath = "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json";
const rows = name => JSON.parse(fs.readFileSync(path.join(root, "config/divergent-universe-decisions-generated/debug-json", `${name}.json`))).table.rows.map(row => row.values);
const hash = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
const referenceRoot = path.join(root,"content-reference/divergent-universe-v1");
const pack = JSON.parse(fs.readFileSync(path.join(referenceRoot,"pack-index.json")))[0];
assert.equal(pack.pack_digest,"59fe8211da025d6526f08f9d0a8e1a9955ae60219ba8b82fa6cab9f1f2b544b6");
const referenceBytes = fs.readFileSync(path.join(referenceRoot,"weighted-curios.json"));
assert.equal(hash(referenceBytes),pack.file_digests.find(value=>value.file==="weighted-curios.json").sha256);
const reference = JSON.parse(referenceBytes).filter(value=>value.id==="divergent-universe.weighted-curio.1012");
assert.equal(reference.length,1);
assert.equal(reference[0].maze_buff_id,"633412");
assert.equal(reference[0].runtime_lowered,false);
const definitions = rows("DuWeightedCurioDeflagrations");
assert.equal(definitions.length, 1);
const row = definitions[0];
assert.equal(row.id.Integer, 1);
assert.equal(row.stable_key.String, "du.weighted-curio-deflagration.most-raucous");
assert.equal(row.weighted_curio_key.String, "divergent-universe.weighted-curio.1012");
assert.equal(row.maze_buff_id.String, "633412");
assert.deepEqual(row.elements.List.map(value => value.String), ["Fire"]);
assert.deepEqual(row.burn_fractions.List.map(value => value.String), ["0.5", "1", "1.5", "2"]);
assert.equal(row.damage_parameter.Integer, 5);
assert.equal(row.damage_multiplier.String, "2");
assert.equal(row.duration_parameter.Integer, 6);
assert.equal(row.duration_turns.Integer, 2);
assert.equal(row.base_fixed_damage.String, "100");
assert.equal(row.base_hard_level_group.Integer, 1);
assert.equal(row.status.String, "AuthoredOperandsPendingNative");
assert.equal(row.policy.String, "OriginalFireAfterActionNaturalTickBurns");
assert.equal(row.base_policy.String, "TargetGroupOneHpRatioProtocolHpFloor");
assert.deepEqual(row.source_ids.List.map(value => value.Integer), [141,142,143,144,145,146,147,148]);
for (const prefix of ["runtime", "base"]) {
  assert.match(row[`${prefix}_policy_note`].String, /^VersionedProjectPolicy:/u);
  assert.match(row[`${prefix}_policy_note`].String, /low-confidence/u);
  assert.match(row[`${prefix}_replacement_condition`].String, /Alternatives/u);
  assert.match(row[`${prefix}_replacement_condition`].String, /Replace each field independently/u);
}
const sources = rows("DuDecisionSources");
const required = [
  [141,"hex","ExcelOutput/RogueTournHex.json","HexID=1012;","51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [142,"maze-buff","ExcelOutput/MazeBuff.json","ID=633412;","2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [143,"text-en","TextMap/TextMapEN.json","hash=8602529829111077351;","afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [144,"text-zh","TextMap/TextMapCHS.json","hash=8602529829111077351;","ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
  [145,"program",programPath,"Name=StageAbility_633412;","5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47"],
  [146,"hard-level","ExcelOutput/HardLevelGroup.json","HardLevelGroup=1; Level=1..95;","d185c09b5388f4eeb368199276ee8a815b406fd9902744027b72a5011b962978"],
];
function context(source, id, suffix, quality) {
  assert.ok(source, `missing source ${id}`);
  assert.equal(source.stable_key.String, `du.source.weighted-curio-deflagration.${suffix}`);
  assert.equal(source.url.String, "https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String, revision);
  assert.equal(source.game_version.String, "4.4");
  assert.equal(source.access_date.String, "2026-10-05");
  assert.equal(source.quality.String, quality);
}
for (const [id,suffix,file,locator,digest] of required) {
  const source = sources.find(source => source.id.Integer === id);
  context(source,id,suffix,"ExactStructured");
  assert.equal(source.sha256.String,digest);
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
for (const [id,prefix,anchor] of [[147,"base","base-damage-policy"],[148,"runtime","execution-policy"]]) {
  const source = sources.find(source => source.id.Integer === id);
  context(source,id,`${prefix}-policy`,"ProjectPolicy");
  assert.equal(source.sha256.String,hash(row[`${prefix}_policy_note`].String));
  assert.equal(source.locator.String,`docs/divergent-universe-weighted-curio-deflagration.md#${anchor}`);
  assert.match(source.note.String,/not an upstream blob/u);
}
const levels = rows("DuWeightedCurioDeflagrationLevels");
assert.equal(levels.length,95);
for (const [index,level] of levels.entries()) {
  assert.equal(level.id.Integer,index+1);
  assert.equal(level.unit_level.Integer,index+1);
  assert.equal(level.stable_key.String,`du.weighted-curio-deflagration.level.${String(index+1).padStart(3,"0")}`);
  assert.equal(level.weighted_curio_key.String,row.weighted_curio_key.String);
  assert.equal(level.hard_level_group.Integer,1);
  assert.deepEqual(level.source_ids.List.map(value=>value.Integer),[146]);
}
if (process.argv.includes("--check-source")) {
  const docs = new Map();
  for (const [, ,file, ,digest] of required) {
    const bytes = execFileSync("git",["-C",path.join(root,".cache/content-reference/turnbasedgamedata"),"show",`${revision}:${file}`],{maxBuffer:128*1024*1024});
    assert.equal(hash(bytes),digest);
    docs.set(file,JSON.parse(bytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu,'$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu,'$1"$2"')));
  }
  const hex = docs.get(required[0][2]).filter(value=>value.HexID===1012 && value.TournMode==="Tourn3");
  assert.equal(hex.length,1);
  assert.equal(hex[0].MazeBuffID,633412);
  assert.deepEqual(hex[0].AvatarType,[]);
  assert.deepEqual(hex[0].AvatarDamageType,["Fire"]);
  const buffs = objects(docs.get(required[1][2])).filter(value=>value.ID===633412 && value.Lv===1);
  assert.equal(buffs.length,1);
  assert.deepEqual(buffs[0].ParamList.map(value=>value.Value),["0.5","1","1.5","2","2","2"]);
  assert.equal(buffs[0].InBattleBindingKey,"StageAbility_633412");
  assert.equal(buffs[0].BuffDesc.Hash,"8602529829111077351");
  for (const [file,words] of [[required[2][2],["Deflagration","Burned","start of each turn","#1[i]%","#4[i]%","#5[i]%","#6[i]"]],
    [required[3][2],["爆燃","灼烧","每回合开始","#1[i]%","#4[i]%","#5[i]%","#6[i]"]]]) {
    for (const word of words) assert.ok(docs.get(file)[buffs[0].BuffDesc.Hash].includes(word));
  }
  const abilities = objects(docs.get(programPath)).filter(value=>value.Name==="StageAbility_633412");
  assert.equal(abilities.length,1);
  const a = abilities[0];
  const entry = objects(a.Modifiers.Modifier_StageAbility_633412._CallbackList.find(value=>value.Event==="OnEnterBattle"));
  assert.ok(entry.some(value=>value.$type==="RPG.GameCore.SetDynamicValueByCharacterCount" && value.AliveOnly===true && value.ReadTargetType.Alias==="AllLightTeam.RemoveServant"));
  assert.ok(entry.some(value=>value.$type==="RPG.GameCore.ByCharacterDamageType" && value.DamageType==="Fire"));
  assert.ok(objects(a).some(value=>value.Property==="HPRatio" && value.DynamicKey==="_Hardlevel_HpAddRatio"));
  assert.ok(objects(a).some(value=>value.FixedValues?.some(item=>item.Value==="100")));
  const effect = a.Modifiers.Modifier_StageAbility_633412_SuperBurn;
  assert.equal(effect.Stacking,"ReplaceByCaster");
  assert.deepEqual(effect._CallbackList.map(value=>value.Event),["OnCreate","OnPhase1","OnCustomEvent"]);
  const natural = objects(effect._CallbackList.find(value=>value.Event==="OnPhase1"));
  const custom = objects(effect._CallbackList.find(value=>value.Event==="OnCustomEvent"));
  assert.ok(natural.some(value=>value.$type==="RPG.GameCore.DamageByAttackProperty"));
  assert.ok(custom.some(value=>value.$type==="RPG.GameCore.DamageByAttackProperty"));
  const notifications = natural.filter(value=>value.$type==="RPG.GameCore.TriggerModifierCustomEvent");
  assert.equal(notifications.length,1);
  assert.equal(notifications[0].PreCheck.ModifierName,"Modifier_StageAbility_633412_SuperBurn");
  assert.equal(notifications[0].PreCheck.Inverse,true);
  assert.ok(!custom.some(value=>value.$type==="RPG.GameCore.TriggerModifierCustomEvent"));
  const curve = docs.get(required[5][2]).filter(value=>value.HardLevelGroup===1 && value.Level>=1 && value.Level<=95).sort((a,b)=>a.Level-b.Level);
  assert.equal(curve.length,95);
  for (const [index,level] of curve.entries()) {
    assert.equal(level.Level,index+1);
    assert.equal(level.HPRatio.Value,levels[index].hp_ratio.String);
  }
}
console.log("Deflagration six released operands, 95 exact HPRatios, natural/custom callback distinction and two independent policy digests verified; native execution remains pending.");
function objects(value) {
  return Array.isArray(value)?value.flatMap(objects):value && typeof value==="object"?[value,...Object.values(value).flatMap(objects)]:[];
}
