// Current released operand/provenance checks; native Cargo owns execution evidence.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const debug = path.join(root, "config/divergent-universe-decisions-generated/debug-json");
const rows = name => JSON.parse(fs.readFileSync(path.join(debug, `${name}.json`))).table.rows.map(row => row.values);
const definitions = rows("DuWeightedCurioTransfers");
assert.equal(definitions.length,1);
const row = definitions[0];
assert.equal(row.stable_key.String,"du.weighted-curio-transfer.dignity-and-passion");
assert.equal(row.weighted_curio_key.String,"divergent-universe.weighted-curio.1009");
assert.equal(row.maze_buff_id.String,"633409");
assert.equal(row.character_path.String,"Knight");
for (const [field, ordinal, decimal] of [["transfer",1,"0.75"],["threshold",2,"0.3"],["decay",3,"0.9"],["heal",4,"0.1"]]) {
  assert.equal(row[`${field}_parameter`].Integer,ordinal);
  assert.equal(row[`${field}_fraction`].String,decimal);
}
assert.equal(row.policy.String,"OtherShieldAppliedOwnerTurnExcessDecay");
assert.match(row.policy_note.String,/^VersionedProjectPolicy:/u);
assert.match(row.policy_note.String,/negative actual delta/u);
assert.match(row.replacement_condition.String,/not observed parity/u);
assert.deepEqual(row.source_ids.List.map(value => value.Integer),[122,123,124,125]);
const required = [
  [122,"hex","ExcelOutput/RogueTournHex.json","HexID=1009;","51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"],
  [123,"maze-buff","ExcelOutput/MazeBuff.json","ID=633409;","2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"],
  [124,"text-en","TextMap/TextMapEN.json","hash=14114601434821166552;","afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"],
  [125,"text-zh","TextMap/TextMapCHS.json","hash=14114601434821166552;","ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"],
];
for (const [id,suffix,file,locator,digest] of required) {
  const source = rows("DuDecisionSources").find(row => row.id.Integer === id);
  assert.equal(source.stable_key.String,`du.source.weighted-curio-transfer.${suffix}`);
  assert.equal(source.url.String,"https://gitlab.com/Dimbreath/turnbasedgamedata");
  assert.equal(source.revision.String,revision);
  assert.equal(source.game_version.String,"4.4");
  assert.equal(source.access_date.String,"2026-10-02");
  assert.equal(source.sha256.String,digest);
  assert.equal(source.quality.String,"ExactStructured");
  assert.ok(source.locator.String.startsWith(`${file};`) && source.locator.String.includes(locator));
}
if (process.argv.includes("--check-source")) {
  const sourceRoot = path.join(root,".cache/content-reference/turnbasedgamedata");
  const docs = new Map();
  for (const [,,file,,expected] of required) {
    const bytes = execFileSync("git",["-C",sourceRoot,"show",`${revision}:${file}`],{maxBuffer:128*1024*1024});
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"),expected);
    docs.set(file,JSON.parse(bytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu,'$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu,'$1"$2"')));
  }
  const hex = docs.get("ExcelOutput/RogueTournHex.json").filter(row => row.HexID === 1009 && row.TournMode === "Tourn3");
  assert.equal(hex.length,1);
  assert.equal(hex[0].MazeBuffID,633409);
  assert.equal(hex[0].DisplayID,1002);
  assert.deepEqual(hex[0].AvatarType,["Knight"]);
  assert.deepEqual(hex[0].AvatarDamageType,[]);
  const flatten = value => Array.isArray(value) ? value.flatMap(flatten) : value && typeof value === "object" && !Object.hasOwn(value,"ID") ? Object.values(value).flatMap(flatten) : [value];
  const buffs = flatten(docs.get("ExcelOutput/MazeBuff.json")).filter(row => row?.ID === 633409 && row.Lv === 1);
  assert.equal(buffs.length,1);
  const buff = buffs[0];
  assert.deepEqual(buff.ParamList.map(value => value.Value),["0.75","0.3","0.9","0.1"]);
  assert.equal(buff.InBattleBindingType,"StageAbilityBeforeCharacterBorn");
  assert.equal(buff.InBattleBindingKey,"StageAbility_633409");
  assert.equal(buff.BuffDesc.Hash,"14114601434821166552");
  assert.equal(buff.BuffDescBattle.Hash,"14114601434821166552");
  for (const [file,words] of [
    ["TextMap/TextMapEN.json",["Preservation","other Shields","start of the turn","#1[i]%","#2[i]%","#3[i]%","#4[i]%"]],
    ["TextMap/TextMapCHS.json",["存护","其他护盾","回合开始","#1[i]%","#2[i]%","#3[i]%","#4[i]%"]],
  ]) for (const word of words) assert.ok(docs.get(file)["14114601434821166552"].includes(word));
  const tree = execFileSync("git",["-C",sourceRoot,"ls-tree","-r","--name-only",revision],{encoding:"utf8",maxBuffer:128*1024*1024});
  assert.equal(tree.split("\n").filter(file => file.includes("633409")).length,0,
    "released program now exists; replace the missing-program premise");
}
console.log("Dignity and Passion four operands and exact provenance verified; not a battle-execution or parity claim.");
