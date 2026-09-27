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
  && artifact.current_boundary.weighted_curio_battle_effect_definitions === 1
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
}
console.log("Hex taxonomy/Gamble inventory verified; no runtime completion or test-pass receipt emitted.");
function assert(condition, message) { if (!condition) throw new Error(message); }
