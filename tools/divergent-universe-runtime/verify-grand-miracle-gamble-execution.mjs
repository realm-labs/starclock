#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { buildGrandMiracleGambleExecution } from "./generate-grand-miracle-gamble-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/grand-miracle-gamble-execution.json";
const artifact = buildGrandMiracleGambleExecution();
assert(fs.readFileSync(path.join(root, output), "utf8")
  === `${JSON.stringify(artifact, null, 2)}\n`, "Hex taxonomy/Gamble inventory drift");
assert(artifact.current_boundary.grand_miracles_admitted_from_hex === 0
  && artifact.current_boundary.terminal_coverage_credit === 0
  && artifact.current_boundary.reference_transport_names_aligned
  && !artifact.current_boundary.source_obligation_and_fixture_labels_aligned,
"reference identities must not imply Grand Miracle execution");
assert(artifact.current_boundary.weighted_curio_accepted_loadout_boundary
  && artifact.current_boundary.weighted_curio_accepted_loadout_maximum === 3
  && artifact.current_boundary.weighted_curio_unsupported_equipment_rejects_battle_contribution
  && !artifact.current_boundary.weighted_curio_loadout_implemented
  && !artifact.current_boundary.weighted_curio_battle_effects_implemented
  && artifact.current_boundary.weighted_curio_battle_effect_definitions === 8
  && !artifact.current_boundary.forge_room_payload_implemented,
"an accepted equipment primitive is not a complete Forge or battle-effect implementation");

if (process.argv.includes("--check-source")) {
  const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
  const evidence = artifact.source_taxonomy;
  const chineseText = { path: "TextMap/TextMapCHS.json",
    sha256: "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147" };
  for (const source of [evidence.source_table, evidence.meaning_evidence, evidence.text_map, chineseText]) {
    const bytes = fs.readFileSync(path.join(sourceRoot, source.path));
    assert(crypto.createHash("sha256").update(bytes).digest("hex") === source.sha256,
      `pinned source drift: ${source.path}`);
  }
  function sourceJson(file) {
    const raw = fs.readFileSync(path.join(sourceRoot, file), "utf8");
    return JSON.parse(raw.replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"'));
  }
  const rows = sourceJson(evidence.source_table.path).filter((row) => row.TournMode === "Tourn3");
  assert(rows.length === artifact.summary.weighted_curio_references, "released Hex selector drift");
  const func = sourceJson(evidence.meaning_evidence.path).find((row) => row.FuncID === 11);
  assert(func?.FuncType === "HexEquipment"
    && String(func.FuncName.Hash) === evidence.meaning_evidence.title_hash
    && String(func.FuncDesc.Hash) === evidence.meaning_evidence.description_hash,
  "released HexEquipment meaning locator drift");
  const text = sourceJson(evidence.text_map.path);
  assert(text[evidence.meaning_evidence.title_hash] === "Select Weighted Curio"
    && text[evidence.meaning_evidence.description_hash].includes("Weighted Curios"),
  "released HexEquipment text must identify Weighted Curios");
  const mazeBytes = execFileSync("git", ["-C", sourceRoot, "show",
    `${evidence.revision}:ExcelOutput/MazeBuff.json`], { maxBuffer: 128 * 1024 * 1024 });
  assert(crypto.createHash("sha256").update(mazeBytes).digest("hex")
    === "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac", "generic MazeBuff digest drift");
  const maze = JSON.parse(mazeBytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
    .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"'));
  const flat = (value) => Array.isArray(value) ? value.flatMap(flat)
    : value && typeof value === "object" && !Object.hasOwn(value, "ID") ? Object.values(value).flatMap(flat) : [value];
  const buffs = flat(maze).filter((row) => row && Object.hasOwn(row, "ID"));
  const released = rows.map((row) => buffs.find((buff) => buff.ID === row.MazeBuffID && buff.Lv === 1));
  assert(released.length === 17 && released.every((row) => row
    && row.InBattleBindingType === "StageAbilityBeforeCharacterBorn"), "all current Hex MazeBuff joins must resolve in the generic table");
  const splash = released.find((row) => row.ID === 633401);
  assert(splash?.ParamList[0]?.Value === "0.3"
    && splash.InBattleBindingKey === "StageAbility_633401"
    && String(splash.BuffDesc.Hash) === "8678628073262894449", "splash operand/binding locator drift");
  const chinese = sourceJson(chineseText.path);
  assert(text["8678628073262894449"].includes("The Hunt")
    && text["8678628073262894449"].includes("#1[i]%")
    && text["8678628073262894449"].includes("adjacent targets")
    && chinese["8678628073262894449"].includes("巡猎")
    && chinese["8678628073262894449"].includes("#1[i]%")
    && chinese["8678628073262894449"].includes("相邻目标"), "released bilingual splash selector/operand reference drift");
  const authored = JSON.parse(fs.readFileSync(path.join(root, artifact.input_digests.weighted_splash_data.path), "utf8"));
  const authoredRows = authored.table.rows;
  assert(authoredRows.length === 1 && authoredRows[0].values.weighted_curio_key.String === "divergent-universe.weighted-curio.1001"
    && authoredRows[0].values.maze_buff_id.String === "633401" && authoredRows[0].values.damage_fraction.String === "0.3",
    "production splash must preserve the released membership and exact operand");
  const shield = released.find((row) => row.ID === 633402);
  const shieldHex = rows.find((row) => row.HexID === 1002);
  assert(shieldHex?.MazeBuffID === 633402 && JSON.stringify(shieldHex.AvatarType) === '["Shaman"]'
    && shieldHex.AvatarDamageType.length === 0
    && shield?.ParamList[0]?.Value === "0.35" && shield.ParamList[1]?.Value === "2"
    && shield.InBattleBindingKey === "StageAbility_633402"
    && String(shield.BuffDesc.Hash) === "18303557854201536316", "Harmony shield released operand/eligibility drift");
  assert(text["18303557854201536316"].includes("Basic ATK/Skill/Ultimate")
    && text["18303557854201536316"].includes("respective ally's Max HP")
    && chinese["18303557854201536316"].includes("对我方目标"), "Harmony shield ally direction and recipient basis drift");
  const shieldRows = JSON.parse(fs.readFileSync(path.join(root, artifact.input_digests.weighted_shield_data.path), "utf8")).table.rows;
  assert(shieldRows.length === 1 && shieldRows[0].values.weighted_curio_key.String === "divergent-universe.weighted-curio.1002"
    && shieldRows[0].values.maze_buff_id.String === "633402"
    && shieldRows[0].values.shield_fraction.String === "0.35"
    && shieldRows[0].values.duration_turns.Integer === 2, "production Harmony shield must preserve exact released operands");
  const attack = released.find((row) => row.ID === 633414);
  const attackHex = rows.find((row) => row.HexID === 1014);
  assert(attackHex?.MazeBuffID === 633414 && JSON.stringify(attackHex.AvatarType) === '["Warrior","Warlock"]'
    && attackHex.AvatarDamageType.length === 0
    && JSON.stringify(attack?.ParamList.map((parameter) => parameter.Value)) === '["0.2","0.3","1"]'
    && attack.InBattleBindingKey === "StageAbility_633414"
    && String(attack.BuffDesc.Hash) === "4217722633224787979", "attack debuff operand/eligibility drift");
  assert(text["4217722633224787979"].includes("Destruction or Nihility")
    && text["4217722633224787979"].includes("attacked enemy")
    && chinese["4217722633224787979"].includes("施放攻击后"), "bilingual attack condition/direction drift");
  const attackRows = JSON.parse(fs.readFileSync(path.join(root, artifact.input_digests.weighted_attack_debuff_data.path), "utf8")).table.rows;
  const value = attackRows[0]?.values;
  assert(attackRows.length === 1 && value.weighted_curio_key.String === "divergent-universe.weighted-curio.1014"
    && value.maze_buff_id.String === "633414" && value.advance_fraction.String === "0.2"
    && value.reduction_fraction.String === "0.3" && value.duration_turns.Integer === 1,
    "production attack debuff must preserve both released effects");
  const support = released.find((row) => row.ID === 633417);
  const supportHex = rows.find((row) => row.HexID === 1017);
  assert(supportHex?.MazeBuffID === 633417
    && JSON.stringify(supportHex.AvatarType) === '["Shaman","Priest","Knight"]'
    && supportHex.AvatarDamageType.length === 0
    && JSON.stringify(support?.ParamList.map((parameter) => parameter.Value)) === '["0.15","0.3","1"]'
    && support.InBattleBindingKey === "StageAbility_633417"
    && String(support.BuffDesc.Hash) === "18107300873516931386", "support attack operand/eligibility drift");
  assert(text["18107300873516931386"].includes("Harmony, Abundance, or Preservation")
    && text["18107300873516931386"].includes("Max HP, DEF, and ATK")
    && chinese["18107300873516931386"].includes("队伍中每有1名"), "support attack roster/stat basis drift");
  const supportRows = JSON.parse(fs.readFileSync(path.join(root, artifact.input_digests.weighted_support_attack_data.path), "utf8")).table.rows;
  const supportValue = supportRows[0]?.values;
  assert(supportRows.length === 1 && supportValue.weighted_curio_key.String === "divergent-universe.weighted-curio.1017"
    && supportValue.maze_buff_id.String === "633417" && supportValue.crit_rate_fraction.String === "0.15"
    && supportValue.crit_damage_fraction.String === "0.3" && supportValue.additional_multiplier.String === "1",
    "production support attack must preserve all released operands, not another seasonal variant");
  const prayer = released.find((row) => row.ID === 633404);
  const prayerHex = rows.find((row) => row.HexID === 1004);
  assert(prayerHex?.MazeBuffID === 633404
    && JSON.stringify(prayerHex.AvatarType) === '["Mage","Warlock"]'
    && prayerHex.AvatarDamageType.length === 0
    && JSON.stringify(prayer?.ParamList.map((parameter) => parameter.Value)) === '["0.6","0.15","0.25"]'
    && prayer.InBattleBindingKey === "StageAbility_633404"
    && String(prayer.BuffDesc.Hash) === "16252001468855081658", "prayer operand/eligibility drift");
  const prayerRows = JSON.parse(fs.readFileSync(path.join(root, artifact.input_digests.weighted_prayer_data.path), "utf8")).table.rows;
  const prayerValue = prayerRows[0]?.values;
  assert(prayerRows.length === 1 && prayerValue.weighted_curio_key.String === "divergent-universe.weighted-curio.1004"
    && prayerValue.maze_buff_id.String === "633404" && prayerValue.hp_fraction.String === "0.6"
    && prayerValue.consume_fraction.String === "0.15" && prayerValue.shield_fraction.String === "0.25",
    "production prayer must preserve all three released resource operands");
  const retaliation = released.find((row) => row.ID === 633413);
  const retaliationHex = rows.find((row) => row.HexID === 1013);
  assert(retaliationHex?.MazeBuffID === 633413 && retaliationHex.DisplayID === 1026
    && retaliationHex.AvatarType.length === 0
    && JSON.stringify(retaliationHex.AvatarDamageType) === '["Physical"]'
    && JSON.stringify(retaliation?.ParamList.map((parameter) => parameter.Value)) === '["4","0.3"]'
    && retaliation.InBattleBindingKey === "StageAbility_633413"
    && String(retaliation.BuffDesc.Hash) === "6457248194266440334", "retaliation released operand/eligibility drift");
  assert(text["6457248194266440334"].includes("Physical")
    && text["6457248194266440334"].includes("Additional DMG")
    && text["6457248194266440334"].includes("cannot defeat")
    && chinese["6457248194266440334"].includes("无法消灭"), "retaliation owner additional/nonlethal text drift");
  const retaliationRows = JSON.parse(fs.readFileSync(path.join(root,
    artifact.input_digests.weighted_retaliation_data.path), "utf8")).table.rows;
  const retaliationValue = retaliationRows[0]?.values;
  assert(retaliationRows.length === 1
    && retaliationValue.weighted_curio_key.String === "divergent-universe.weighted-curio.1013"
    && retaliationValue.maze_buff_id.String === "633413"
    && retaliationValue.additional_multiplier.String === "4"
    && retaliationValue.aggro_fraction.String === "0.3", "production retaliation operand drift");
  const breakEffect = released.find((row) => row.ID === 633405);
  const breakEffectHex = rows.find((row) => row.HexID === 1005);
  assert(breakEffectHex?.MazeBuffID === 633405 && breakEffectHex.DisplayID === 1018
    && breakEffectHex.AvatarType.length === 0
    && JSON.stringify(breakEffectHex.AvatarDamageType) === '["Wind","Thunder"]'
    && JSON.stringify(breakEffect?.ParamList.map((parameter) => parameter.Value)) === '["1.2"]'
    && breakEffect.InBattleBindingKey === "StageAbility_633405"
    && String(breakEffect.BuffDesc.Hash) === "6003923322476481427",
    "Converse of Entropy released operand/eligibility drift");
  assert(text["6003923322476481427"].includes("Wind")
    && text["6003923322476481427"].includes("Lightning")
    && text["6003923322476481427"].includes("highest Break Effect")
    && chinese["6003923322476481427"].includes("提高至"),
    "Converse of Entropy must raise to the team maximum, not add that whole value");
  const breakEffectRows = JSON.parse(fs.readFileSync(path.join(root,
    artifact.input_digests.weighted_break_effect_data.path), "utf8")).table.rows;
  const breakEffectValue = breakEffectRows[0]?.values;
  assert(breakEffectRows.length === 1
    && breakEffectValue.weighted_curio_key.String === "divergent-universe.weighted-curio.1005"
    && breakEffectValue.maze_buff_id.String === "633405"
    && JSON.stringify(breakEffectValue.character_elements.List.map((item) => item.String)) === '["Wind","Thunder"]'
    && breakEffectValue.multiplier_parameter.Integer === 1
    && breakEffectValue.multiplier.String === "1.2",
    "production Converse of Entropy must preserve the current exact released operand");
  const necrosis = released.find((row) => row.ID === 633403);
  const necrosisHex = rows.find((row) => row.HexID === 1003);
  assert(necrosisHex?.MazeBuffID === 633403 && necrosisHex.DisplayID === 1016
    && JSON.stringify(necrosisHex.AvatarType) === '["Priest"]'
    && necrosisHex.AvatarDamageType.length === 0
    && JSON.stringify(necrosis?.ParamList.map((parameter) => parameter.Value)) === '["1.5","6","3","2"]'
    && necrosis.InBattleBindingKey === "StageAbility_633403"
    && String(necrosis.BuffDesc.Hash) === "14457895563947008111",
    "Mock Crimson Moon released operand/eligibility drift");
  assert(text["14457895563947008111"].includes("Abundance")
    && text["14457895563947008111"].includes("Necrosis")
    && text["14457895563947008111"].includes("Burn")
    && chinese["14457895563947008111"].includes("坏死")
    && chinese["14457895563947008111"].includes("灼烧"),
    "Mock Crimson Moon bilingual effect identity drift");
  const necrosisRows = JSON.parse(fs.readFileSync(path.join(root,
    artifact.input_digests.weighted_necrosis_data.path), "utf8")).table.rows;
  const necrosisValue = necrosisRows[0]?.values;
  assert(necrosisRows.length === 1
    && necrosisValue.weighted_curio_key.String === "divergent-universe.weighted-curio.1003"
    && necrosisValue.maze_buff_id.String === "633403"
    && JSON.stringify(necrosisValue.character_paths.List.map((item) => item.String)) === '["Priest"]'
    && necrosisValue.chance_parameter.Integer === 1 && necrosisValue.base_chance.String === "1.5"
    && necrosisValue.damage_parameter.Integer === 2 && necrosisValue.attack_multiplier.String === "6"
    && necrosisValue.duration_parameter.Integer === 3 && necrosisValue.duration_turns.Integer === 3
    && necrosisValue.detonation_parameter.Integer === 4 && necrosisValue.detonation_fraction.String === "2"
    && necrosisValue.policy.String === "AttackResolvedNecrosisAndDotDamageBurnDetonation"
    && JSON.stringify(necrosisValue.source_ids.List.map((item) => item.Integer)) === '[103,104,105,106]',
    "production Mock Crimson Moon must preserve all four current released operands and explicit policy");
  const paths = [[1001,"Knight","preservation","150"], [1002,"Rogue","hunt","75"],
    [1003,"Mage","erudition","75"], [1004,"Warlock","nihility","100"],
    [1008,"Warrior","destruction","125"], [1009,"Shaman","harmony","100"],
    [1105,"Priest","abundance","100"], [1402,"Memory","remembrance","100"],
    [1501,"Elation","elation","100"]];
  const exactSource = (file, digest) => {
    const bytes = execFileSync("git", ["-C", sourceRoot, "show", `${evidence.revision}:${file}`],
      {maxBuffer: 128 * 1024 * 1024});
    assert(crypto.createHash("sha256").update(bytes).digest("hex") === digest, `path baseline source drift: ${file}`);
    return JSON.parse(bytes.toString().replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"')
      .replace(/("Value"\s*:\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu, '$1"$2"'));
  };
  const avatars = exactSource("ExcelOutput/AvatarConfig.json",
    "c14584519e2feda70e9932501f7b79d67242d700585fe44119bbce9da9678e98");
  const promotions = exactSource("ExcelOutput/AvatarPromotionConfig.json",
    "4453f206d6b79658128f22ce4d923e2e608f92b48175ba5c913ed2be322d24c5");
  const flattenBy = (value, field) => Array.isArray(value) ? value.flatMap(v => flattenBy(v,field))
    : value && typeof value === "object" && !Object.hasOwn(value,field)
      ? Object.values(value).flatMap(v => flattenBy(v,field)) : [value];
  for (const [avatar, path, field, weight] of paths) {
    assert(flattenBy(avatars,"AvatarID").some(row => row?.AvatarID === avatar && row.AvatarBaseType === path),
      `representative path join drift: ${avatar}`);
    const weights = flattenBy(promotions,"AvatarID").filter(row => row?.AvatarID === avatar);
    assert(weights.length === 7 && weights.every(row => row.BaseAggro?.Value === weight)
      && retaliationValue[`base_aggro_${field}`].String === weight, `representative path baseline drift: ${avatar}`);
  }
}
console.log("Hex taxonomy/Gamble inventory verified; no runtime completion or test-pass receipt emitted.");
function assert(condition, message) { if (!condition) throw new Error(message); }
