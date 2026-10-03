// Released operand and callback verification only; native Cargo owns execution.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const programPath = "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json";
const debug = path.join(root,"config/divergent-universe-decisions-generated/debug-json");
const rows = name => JSON.parse(fs.readFileSync(path.join(debug,`${name}.json`))).table.rows.map(row=>row.values);
const definitions = rows("DuWeightedCurioOverflows");
assert.equal(definitions.length,1);
const row = definitions[0];
assert.equal(row.stable_key.String,"du.weighted-curio-overflow.parallel-universe-walkie-talkie");
assert.equal(row.weighted_curio_key.String,"divergent-universe.weighted-curio.1016");
assert.equal(row.maze_buff_id.String,"633416");
assert.deepEqual(row.character_paths.List.map(value=>value.String),["Mage","Rogue"]);
for (const [field,operand,decimal] of [["base",1,"10"],["overflow",2,"1"],["attack",3,"0.8"]]) {
  assert.equal(row[`${field}_parameter`].Integer,operand);
  assert.equal(row[{base:"base_multiplier",overflow:"overflow_multiplier",attack:"attack_increase"}[field]].String,decimal);
}
assert.equal(row.status.String,"PendingNativeDeathCallbackAndBaseDamage");
assert.match(row.source_semantics.String,/random ties/u);
assert.match(row.unresolved_runtime.String,/^Unimplemented:/u);
assert.match(row.unresolved_runtime.String,/not established/u);
assert.deepEqual(row.source_ids.List.map(value=>value.Integer),[126,127,128,129,130]);
const required = [
  [126,"hex","ExcelOutput/RogueTournHex.json","HexID=1016;","51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [127,"maze-buff","ExcelOutput/MazeBuff.json","ID=633416;","2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [128,"text-en","TextMap/TextMapEN.json","hash=12104045893670599658;","afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [129,"text-zh","TextMap/TextMapCHS.json","hash=12104045893670599658;","ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
  [130,"program",programPath,"Name=StageAbility_633416;","5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47"],
];
for (const [id,suffix,file,locator,digest] of required) {
  const source = rows("DuDecisionSources").find(row=>row.id.Integer===id);
  assert.equal(source.stable_key.String,`du.source.weighted-curio-overflow.${suffix}`);
  assert.equal(source.url.String,"https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String,revision);
  assert.equal(source.game_version.String,"4.4");
  assert.equal(source.access_date.String,"2026-10-03");
  assert.equal(source.sha256.String,digest);
  assert.equal(source.quality.String,"ExactStructured");
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
if (process.argv.includes("--check-source")) {
  const docs = new Map();
  for (const [,,file,,digest] of required) {
    const bytes = execFileSync("git",["-C",path.join(root,".cache/content-reference/turnbasedgamedata"),"show",`${revision}:${file}`],{maxBuffer:128*1024*1024});
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"),digest);
    docs.set(file,JSON.parse(bytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu,'$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu,'$1"$2"')));
  }
  const hex = docs.get("ExcelOutput/RogueTournHex.json").filter(row=>row.HexID===1016 && row.TournMode==="Tourn3");
  assert.equal(hex.length,1);
  assert.equal(hex[0].MazeBuffID,633416);
  assert.equal(hex[0].DisplayID,1007);
  assert.deepEqual(hex[0].AvatarType,["Mage","Rogue"]);
  assert.deepEqual(hex[0].AvatarDamageType,[]);
  const buffs = objects(docs.get("ExcelOutput/MazeBuff.json")).filter(row=>row.ID===633416 && row.Lv===1);
  assert.equal(buffs.length,1);
  const buff = buffs[0];
  assert.deepEqual(buff.ParamList.map(value=>value.Value),["10","1","0.8"]);
  assert.equal(buff.InBattleBindingType,"StageAbilityBeforeCharacterBorn");
  assert.equal(buff.InBattleBindingKey,"StageAbility_633416");
  assert.equal(buff.BuffDesc.Hash,"12104045893670599658");
  assert.equal(buff.BuffDescBattle.Hash,"12104045893670599658");
  for (const [file,words] of [
    ["TextMap/TextMapEN.json",["Erudition or The Hunt","highest HP","True DMG","#1[i]%","#2[i]%","#3[i]%"]],
    ["TextMap/TextMapCHS.json",["智识","巡猎","当前生命值最高","真实伤害","#1[i]%","#2[i]%","#3[i]%"]],
  ]) for (const word of words) assert.ok(docs.get(file)["12104045893670599658"].includes(word));
  const abilities = objects(docs.get(programPath)).filter(row=>row.Name==="StageAbility_633416");
  assert.equal(abilities.length,1,"search merged ability names, not filename membership");
  const ability = abilities[0];
  const modifiers = ability.Modifiers;
  const creation = objects(modifiers.Modifier_StageAbility_633416);
  assert.ok(creation.some(row=>JSON.stringify(row.BaseTypeList)==='["Rogue","Mage"]'));
  assert.equal(modifiers.StageAbility_633416_Modifier_Sub.Stacking,"Replace");
  assert.ok(objects(modifiers.StageAbility_633416_Modifier_Sub).some(row=>row.Property==="AttackAddedRatio"));
  const callbacks = modifiers.Modifier_StageAbility_633416_Character._CallbackList;
  assert.deepEqual(callbacks.map(row=>row.Event),["OnAfterHitAll","OnTriggerDeath","OnTriggerDeathrattle","OnBeforeAttack","OnAfterAttackEnd"]);
  const hit = objects(callbacks.find(row=>row.Event==="OnAfterHitAll"));
  assert.ok(hit.some(row=>row.Property==="Result_OverflowHPDamage"));
  assert.ok(hit.some(row=>row.Value==="CurrentHP"));
  for (const event of ["OnTriggerDeath","OnTriggerDeathrattle"]) {
    const tasks = objects(callbacks.find(row=>row.Event===event));
    assert.ok(tasks.some(row=>row.MinOrMax==="Max" && row.PropertyType==="CurrentHP"));
    assert.ok(tasks.some(row=>row.ByRandom===true && row.MaxNumber?.FixedValue?.Value==="1"));
    assert.ok(tasks.some(row=>row.Property==="HPRatio"));
    assert.ok(tasks.some(row=>row.AttackType==="TrueDamage" && row.FinalFormulaType==="ByBaseDamage"));
    assert.ok(tasks.some(row=>row.CanTriggerLastKill===true && row.IsConvert===true));
  }
  // Do not interpret unknown task types, opcode bytes or hash operands here.
}
console.log("Walkie-Talkie exact operands, merged callback evidence and pending admission verified; no battle-execution claim.");

function objects(value) {
  if (Array.isArray(value)) return value.flatMap(objects);
  return value && typeof value==="object" ? [value,...Object.values(value).flatMap(objects)] : [];
}
