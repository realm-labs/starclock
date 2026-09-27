#!/usr/bin/env node

// Current source taxonomy and capability inventory, never an execution receipt.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ref = "content-reference/divergent-universe-v1";
const output = "content-manifests/divergent-universe-runtime-v1/grand-miracle-gamble-execution.json";
const inputs = {
  hex_references: `${ref}/weighted-curios.json`,
  hex_eligibility: `${ref}/weighted-curio-eligibility.json`,
  hex_states: `${ref}/weighted-curio-states.json`,
  gamble_groups: `${ref}/gamble-groups.json`,
  gamble_units: `${ref}/gamble-units.json`,
  curio_catalog: "crates/starclock-data/src/divergent_universe_curio_catalog.rs",
  curio_lowering: "crates/starclock-data/src/divergent_universe_curio.rs",
  taxonomy_tests: "crates/starclock-data/src/divergent_universe_tests.rs",
  mode_facade: "crates/starclock-mode-universe/src/divergent_universe/mod.rs",
  gamble_runtime: "crates/starclock-mode-universe/src/divergent_universe/gamble_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/gamble_runtime.rs",
};
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

export function buildGrandMiracleGambleExecution() {
  const hex = json(inputs.hex_references);
  const eligibility = json(inputs.hex_eligibility);
  const states = json(inputs.hex_states);
  const groups = json(inputs.gamble_groups);
  const units = json(inputs.gamble_units);
  const current = eligibility.filter((row) => row.selector_scope === "Tourn3");
  const otherMode = eligibility.filter((row) => row.selector_scope !== "Tourn3");
  const coins = units.filter((row) => row.outcome_program.operation === "GainRunCurrency");
  assert(hex.length === 17 && current.length === 17 && otherMode.length === 57
    && states.length === 34, "Hex reference denominator drift");
  assert(hex.every((row) => row.content_kind === "WeightedCurio"
    && row.kind === "DivergentUniverseWeightedCurio"
    && row.id.startsWith("divergent-universe.weighted-curio.")
    && !row.runtime_lowered && row.source_refs.some((source) =>
    source.path === "ExcelOutput/RogueTournHex.json" && source.revision === revision)),
  "Hex source/reference boundary drift");
  assert(groups.length === 126 && groups.every((row) => row.unit_ids.length === 0
    && row.fallback === "RejectWithoutMutation" && !row.runtime_lowered),
  "unresolved Gamble groups must remain fail-closed");
  assert(units.length === 89 && coins.length === 2
    && equal(coins.map((row) => row.outcome_program.amount), ["20", "40"]),
  "Gamble Coin source drift");
  assert(text(inputs.curio_lowering).includes(
    "content_kind(r.content_kind, &v.content_kind)?"),
  "owned Hex lowering must bind the authored column and payload classification");
  assert(text(inputs.curio_catalog).includes(
    "Hex references are Weighted Curios, not Grand Miracles"),
  "owned catalog must reject Grand Miracle classification of Hex references");
  assert(!text(inputs.mode_facade).includes("grand_miracle_runtime"),
    "reference flags must not be exposed as a Grand Miracle runtime");
  const probes = [
    { file: inputs.taxonomy_tests,
      test: "production_hex_source_references_are_weighted_curios_not_grand_miracles" },
    ...[
      "gamble_catalogs_compile_exact_policy_boundaries_without_hex_runtime_admission",
      "gamble_exact_coin_units_execute_and_unresolved_outcomes_fail_closed",
      "all_gamble_groups_preserve_state_and_rng",
    ].map((test) => ({ file: inputs.tests, test })),
  ];
  for (const probe of probes)
    assert(text(probe.file).includes(`fn ${probe.test}(`), `missing test ${probe.test}`);
  return {
    status: "HexWeightedCurioReferencesRuntimePending",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    source_taxonomy: {
      game_version: "4.4",
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision, access_date: "2026-09-27", evidence_quality: "ExactStructured",
      semantic_kind: "WeightedCurio", selector: "RogueTournHex.TournMode=Tourn3",
      source_table: { path: "ExcelOutput/RogueTournHex.json",
        sha256: "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455" },
      meaning_evidence: { path: "ExcelOutput/RogueTournWorkbenchFunc.json",
        sha256: "b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078",
        locator: "FuncID=11; FuncType=HexEquipment",
        title_hash: "1823313480930548529", description_hash: "14848339980219349693",
        summary_en: "The HexEquipment service equips and adjusts Weighted Curios.",
        summary_zh_cn: "HexEquipment 服务用于装备及调整加权奇物。" },
      text_map: { path: "TextMap/TextMapEN.json",
        sha256: "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789" },
    },
    current_boundary: {
      reference_transport_names_aligned: true,
      source_obligation_and_fixture_labels_aligned: false,
      owned_hex_semantic_classification: true,
      grand_miracles_admitted_from_hex: 0,
      weighted_curio_loadout_implemented: false,
      weighted_curio_battle_effects_implemented: false,
      grand_miracle_runtime_implemented: false,
      forge_room_payload_implemented: false,
      terminal_coverage_credit: 0,
      required_next_work: "Implement real Weighted Curio loadout and effects; establish Grand Miracle selectors independently and repair its separate source/fixture audit labels without shrinking obligations.",
    },
    summary: { weighted_curio_references: hex.length,
      current_hex_eligibility_rules: current.length,
      other_mode_eligibility_references: otherMode.length,
      reference_state_boundaries: states.length, unresolved_gamble_groups: groups.length,
      gamble_units: units.length, accepted_coin_units: coins.length,
      unresolved_gamble_units: units.length - coins.length },
    // Existence is checked here; only native Cargo execution verifies behavior.
    native_test_targets: probes,
  };
}

function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildGrandMiracleGambleExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Hex taxonomy/Gamble inventory is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated current Hex taxonomy/Gamble inventory, without execution credit.");
  }
}
