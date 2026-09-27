#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
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

if (process.argv.includes("--check-source")) {
  const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
  const evidence = artifact.source_taxonomy;
  for (const source of [evidence.source_table, evidence.meaning_evidence, evidence.text_map]) {
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
}
console.log("Hex taxonomy/Gamble inventory verified; no runtime completion or test-pass receipt emitted.");
function assert(condition, message) { if (!condition) throw new Error(message); }
