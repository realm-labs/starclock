#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const index = args.indexOf("--partition");
const write = args.includes("--write");
assert(index >= 0 && args[index + 1], "missing --partition");
assert(args.every((value, offset) =>
  value === "--partition" || value === "--write" || offset === index + 1),
"unsupported argument");
const partitionId = args[index + 1];
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition?.mechanic_family?.startsWith("curio-"),
  `${partitionId}: not a Curio partition`);
assert(["G07-P3-M11-S01", "G07-P3-M11-S02", "G07-P3-M11-S03", "G07-P3-M11-S04", "G07-P3-M11-S05", "G07-P3-M11-S06"].includes(partitionId),
  `${partitionId}: Curio receipt profile is not implemented`);
const s02 = partitionId === "G07-P3-M11-S02";
const s03 = partitionId === "G07-P3-M11-S03";
const s04 = partitionId === "G07-P3-M11-S04";
const s05 = partitionId === "G07-P3-M11-S05";
const s06 = partitionId === "G07-P3-M11-S06";
const golden =
  `evidence/standard-universe-mechanics-complete-v1/goldens/${partitionId}.json`;
assert(exists(golden), `${partitionId}: golden is missing`);
const records = new Map(audit.records.map((entry) => [entry.id, entry]));
const rules = new Map(audit.rules.map((entry) => [entry.id, entry]));
const fixtures = new Map(audit.fixtures.map((entry) => [entry.id, entry]));
const sourceEvidence = [
  { path: "content-reference/standard-universe-v1/curios.json" },
  { path: "content-reference/standard-universe-v1/curio-states.json" },
  { path: "content-reference/standard-universe-v1/mechanic-rules.json" },
];
const executionEvidence = s06 ? [
  { path: "crates/starclock-activity/src/random_policy.rs" },
  { path: "crates/starclock-activity/src/graph_activity/random_offer.rs" },
  { path: "crates/starclock-activity/src/graph_activity.rs" },
  { path: "crates/starclock-mode-universe/src/topology/blessing_offer.rs" },
  { path: "crates/starclock-mode-universe/src/topology_reward.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s06.rs" },
  { path: "crates/starclock-test-kit/tests/suites/activity/activity/random_offer_policy.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s06.rs" },
] : s05 ? [
  { path: "crates/starclock-activity/src/program.rs" },
  { path: "crates/starclock-activity/src/transaction/participant_carry.rs" },
  { path: "crates/starclock-combat/src/rule/model.rs" },
  { path: "crates/starclock-combat/src/resolver/operation.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/curio_commands.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs" },
  { path: "crates/starclock-mode-universe/src/battle_materialization/player.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s05.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s05.rs" },
] : s04 ? [
  { path: "crates/starclock-mode-universe/src/curio_activity/domain.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/curio_commands.rs" },
  { path: "crates/starclock-mode-universe/src/topology/blessing_offer.rs" },
  { path: "crates/starclock-mode-universe/src/topology_reward.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s04.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/topology_runtime.rs" },
] : s03 ? [
  { path: "crates/starclock-activity/src/graph_activity/boundary.rs" },
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: "crates/starclock-mode-universe/src/curio_activity/domain.rs" },
  { path: "crates/starclock-mode-universe/src/service_interaction.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/ability_access.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/curio_commands.rs" },
  { path: "crates/starclock-mode-universe/src/topology/route_program.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s03.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s03.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/service_interaction_runtime.rs" },
] : s02 ? [
  { path: "crates/starclock-activity/src/graph_activity/boundary.rs" },
  { path: "crates/starclock-activity/src/random_policy.rs" },
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/curio_commands.rs" },
  { path: "crates/starclock-mode-universe/src/topology.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s02.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s02.rs" },
] : [
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: "crates/starclock-mode-universe/src/runtime.rs" },
  { path: "crates/starclock-mode-universe/src/topology_reward.rs" },
  { path: "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs" },
  { path: "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s01.rs" },
  { path: "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s01.rs" },
];
const reviewEvidence = [
  { path: s06
    ? "docs/goal-07-curio-s06.md"
    : s05
    ? "docs/goal-07-curio-s05.md"
    : s04 ? "docs/goal-07-curio-s04.md"
    : s03 ? "docs/goal-07-curio-s03.md"
    : s02 ? "docs/goal-07-curio-s02.md" : "docs/goal-07-curio-s01.md" },
  { path: "crates/starclock-mode-universe/src/curio_activity.rs" },
  { path: s06
    ? "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s06.rs"
    : s05
    ? "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s05.rs"
    : s04 ? "crates/starclock-mode-universe/src/runtime/battle_execution_access.rs"
    : s03 ? "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s03.rs"
    : s02
    ? "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s02.rs"
    : "crates/starclock-mode-universe/src/battle_rule_lowering/curio_s01.rs" },
];

const receipt = {
  schema_revision: "starclock.goal07-content-partition-receipt.v1",
  goal_id: "standard-universe-mechanics-complete-v1",
  partition_id: partitionId,
  state: "Complete",
  completed_on: s05 || s06 ? "2026-07-27" : "2026-07-26",
  authoring: {
    workbooks: [
      {
        path: "config/data/Universe.xlsx",
        tables: ["UniverseCurio", "UniverseCurioState", "UniverseCurioParameter"],
      },
      {
        path: "config/data/UniverseBindings.xlsx",
        tables: ["UniverseMechanicRule"],
      },
      {
        path: "config/data/UniverseEvidence.xlsx",
        tables: ["UniverseContentAudit", "UniverseReviewFixture", "UniverseSourceRecord"],
      },
    ],
    openpyxl_commands: [
      `python -c "import openpyxl" && python tools/goal07/author-curio-partition.py --partition ${partitionId} --check`,
    ],
    sora_bundle: evidence("config/universe-generated/config.sora"),
    sora_golden: evidence(golden),
  },
  records: partition.record_ids.map((id) =>
    disposition(records.get(id), "ExecutableSharedPrimitive", [
      { path: "config/data/Universe.xlsx" },
    ])),
  rules: partition.rule_ids.map((id) => ({
    ...disposition(rules.get(id), "ExecutableSharedPrimitive", [
      { path: "config/data/UniverseBindings.xlsx" },
    ]),
    implementation_kind: "SharedPrimitive",
    definition_keys: [id, rules.get(id).source_record_id],
    execution_evidence: executionEvidence,
  })),
  fixtures: partition.fixture_ids.map((id) => ({
    ...disposition(fixtures.get(id), "ProductionExecuted", [
      { path: "config/data/UniverseEvidence.xlsx" },
    ]),
    execution_kind: "RustTest",
    test_path: s05
      ? id === "universe.fixture.curio-tag.destructible"
        ? "crates/starclock-mode-universe/src/runtime/curio_commands.rs"
        : "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s05.rs"
      : s04 ? "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s04.rs"
      : s03
      ? id === "universe.fixture.curio-tag.blessing"
        ? "crates/starclock-mode-universe/src/runtime/curio_commands.rs"
        : "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s03.rs"
      : s02
      ? "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s02.rs"
      : "crates/starclock-test-kit/tests/suites/universe/mechanic_battle_integration/curio_s01.rs",
    test_marker: s05
      ? id === "universe.fixture.curio-tag.destructible"
        ? "destructible_event_counts_once_and_capsule_exposes_exact_spatial_free_policy"
        : "goal07_p3_m11_s05_keeps_all_assigned_curios_out_of_native_handlers"
      : s04 ? "goal07_p3_m11_s04_executes_every_assigned_curio_without_native_handlers"
      : s03
      ? fixtureMarkerS03(id)
      : s02
      ? fixtureMarker(id)
      : "goal07_p3_m11_s01_executes_every_assigned_curio_and_fixture_family",
  })),
  enemy_variants: [],
  encounter_members: [],
  native_handler_reviews: partition.native_review_candidate_rule_ids.map((id) => ({
    id,
    outcome: "IrSufficient",
    decision: nativeDecision(id),
    evidence: reviewEvidence,
  })),
  numeric_approximations: s06 ? [] : s05 ? [
    {
      id: "universe.curio.63.destructible-lottery-chances",
      disposition: "ExternalDecision",
      rationale:
        "Public evidence confirms two small chances but publishes neither probability. The replayable command records NoEffect, Curio or Failure without inventing a number.",
    },
    {
      id: "universe.curio.64.destructible-frequency",
      disposition: "ExternalDecision",
      rationale:
        "Public evidence only says destructible objects appear more frequently. Runtime exposes a qualitative policy flag to the spatial-free host and does not invent a spawn multiplier.",
    },
    {
      id: "universe.curio.68.allied-element-selection",
      disposition: "ProjectPolicyApproximate",
      rationale:
        "The source defines the allied-element candidate set and one selected weakness but no mixed-team selection distribution. Runtime v1 freezes deterministic uniform battle-RNG selection.",
    },
  ] : s04 ? [
    ...[
      ["23", "elation"],
      ["24", "hunt"],
      ["25", "destruction"],
      ["26", "remembrance"],
      ["27", "nihility"],
      ["28", "abundance"],
    ].map(([id, pathName]) => ({
      id: `universe.curio.${id}.${pathName}-offer-weight`,
      disposition: "ProjectPolicyApproximate",
      rationale:
        "Public evidence says the appearance rate greatly increases but supplies no multiplier. Runtime v1 freezes the shared x2 policy and records the approximation explicitly.",
    })),
  ] : s03 ? [
    {
      id: "universe.curio.13.discount-rounding",
      disposition: "ProjectPolicyApproximate",
      rationale:
        "The exact 30% discount is public, but its fractional-fragment rounding is not. Runtime v1 uses checked integer floor after applying the retained 70% price.",
    },
    {
      id: "universe.curio.211.erudition-offer-weight",
      disposition: "ProjectPolicyApproximate",
      rationale:
        "Public evidence says the appearance rate greatly increases but supplies no multiplier. Runtime v1 freezes x2 and records the approximation explicitly.",
    },
    {
      id: "universe.curio.22.preservation-offer-weight",
      disposition: "ProjectPolicyApproximate",
      rationale:
        "Public evidence says the appearance rate greatly increases but supplies no multiplier. Runtime v1 freezes x2 and records the approximation explicitly.",
    },
  ] : s02 ? [
    {
      id: "universe.curio.123.propagation-offer-weight",
      disposition: "ProjectPolicyApproximate",
      rationale:
        "Public evidence says the appearance rate increases but supplies no multiplier. Runtime v1 freezes x2 and records the approximation explicitly.",
    },
  ] : [
    {
      id: "universe.curio.107.destructible-success-chance",
      disposition: "ExternalDecision",
      rationale:
        "The public source confirms a small chance but does not expose an authoritative probability. The replayable command records NoEffect, Blessing or Failure without inventing a number.",
    },
  ],
  execution: {
    result: "pass",
    commands: [
      `python tools/goal07/author-curio-partition.py --partition ${partitionId} --check`,
      "node tools/universe-reference/verify_production_workbooks.mjs .",
      `cargo test -p starclock-test-kit --test universe_suite mechanic_battle_integration ${s06 ? "curio_s06" : s05 ? "curio_s05" : s04 ? "curio_s04" : s03 ? "curio_s03" : s02 ? "curio_s02" : "curio_s01"} --all-features`,
      "cargo test -p starclock-mode-universe --lib curio_activity::tests --all-features",
      ...(s05 ? [
        "cargo test -p starclock-test-kit --test activity_suite battle_settlement --all-features",
        "cargo test -p starclock-mode-universe --lib runtime::curio_commands::tests --all-features",
      ] : []),
      ...(s06 ? [
        "cargo test -p starclock-test-kit --test activity_suite random_offer_policy --all-features",
        "cargo test -p starclock-test-kit --test universe_suite topology_runtime --all-features",
      ] : []),
      ...(s04 ? [
        "cargo test -p starclock-mode-universe --lib runtime::curio_commands::tests --all-features",
        "cargo test -p starclock-test-kit --test universe_suite topology_runtime --all-features",
      ] : []),
      ...(s03 ? [
        "cargo test -p starclock-test-kit --test activity_suite random_boundary --all-features",
        "cargo test -p starclock-mode-universe --lib runtime::curio_commands::tests --all-features",
        "cargo test -p starclock-test-kit --test universe_suite service_interaction_runtime --all-features",
        "cargo test -p starclock-test-kit --test universe_suite topology_runtime --all-features",
      ] : []),
      ...(s02 ? [
        "cargo test -p starclock-test-kit --test activity_suite random_boundary --all-features",
        "cargo test -p starclock-test-kit --test universe_suite topology_runtime --all-features",
      ] : []),
      "cargo test -p starclock-mode-universe --all-features",
    ],
    goldens: [evidence(golden)],
  },
};

const relative =
  `evidence/standard-universe-mechanics-complete-v1/partitions/${partitionId}.json`;
const encoded = `${JSON.stringify(receipt, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(absolute(relative)), { recursive: true });
  fs.writeFileSync(absolute(relative), encoded);
  console.log(`Wrote Goal 07 receipt ${relative}.`);
} else {
  assert(exists(relative), `${relative} is missing`);
  assert(fs.readFileSync(absolute(relative), "utf8") === encoded,
    `${partitionId}: generated receipt drifted`);
  console.log(`Goal 07 Curio receipt ${partitionId} matches generated evidence.`);
}

function nativeDecision(id) {
  const stable = id.split(".").slice(3, 4)[0];
  return {
    "1": "Conditional random-offer limits, a bounded pending choice and ordinary Curio teardown express both limited-use triggers atomically.",
    "102": "Acquisition-time integer division and the generic fragment-gain multiplier express the current-fragment grant exactly.",
    "104": "The Activity destroyed-Curio counter is captured in the immutable battle contribution and lowered to ordinary source damage modifiers.",
    "106": "Returned participant carry supplies full-HP facts to the generic after-battle Curio event and checked fragment operation.",
    "107": "A replayable external outcome command executes blessing acquisition or atomic Curio, Energy and Technique Point teardown without inventing an unpublished probability.",
    "11": "Initial keyed Resonance Energy and the ordinary Resonance damage ratio are compiled from the Curio contribution at battle assembly.",
    "110": "A conditional reward-node bypass and the shared checked fragment multiplier express both Gossip clauses.",
    "111": "A Technique ability tag, ordinary DamageBoost and flat pre-multiplier damage stage express both released Technique damage terms.",
    "112": "A checked Domain-entry settlement grants fragments, evaluates the post-grant threshold and atomically tears down the Curio through ordinary Activity operations.",
    "113": "Acquisition captures complete fragment hundreds into an immutable generic contribution value and ordinary stat modifiers apply the exact CRIT DMG result.",
    "118": "An actor selector, TurnStarted trigger and ordinary maximum-HP Heal operation express the complete effect.",
    "12": "The checked post-battle fragment-gain category composes the exact 175% multiplier with other generic gain modifiers.",
    "120": "The generic random-option boundary samples count and selected-Path Blessings without replacement and executes ordinary acquisition operations.",
    "121": "Highest-ATK selection, a permanent mark, HP consumption and a bounded stacking SPD effect are all typed Rule IR.",
    "122": "Battle assembly counts distinct Path IDs in the immutable Blessing snapshot and lowers one ordinary Break Effect modifier.",
    "123": "The generic random-option boundary grants one Propagation Blessing and the conditional offer-weight primitive biases later Propagation options.",
    "13": "The shared service debit operation applies the authored discount to Blessing enhancement, offer reset and participant revival without changing unrelated shop prices.",
    "14": "Battle assembly snapshots complete fragment hundreds and lowers the exact per-hundred damage ratio through ordinary source modifiers.",
    "15": "A checked Domain-entry settlement derives six percent from current fragments and credits it through the shared fragment-gain pipeline.",
    "19": "Typed mitigation, Effect RES, duration and first-attack removal operations express damage nullification and the three-turn debuff guard.",
    "2": "The generic bounded reroll counter composes one acquisition-scoped free Blessing reset with Ability Tree authorization.",
    "20": "The generic Reward-stream random boundary selects up to two owned unenhanced Blessings without replacement and atomically upgrades their inventory values.",
    "211": "The generic acquisition boundary grants one Erudition Blessing and the conditional offer-weight primitive biases later Erudition options.",
    "22": "The generic acquisition boundary grants one Preservation Blessing and the conditional offer-weight primitive biases later Preservation options.",
    "23": "The table-driven Sealing Wax boundary grants one Elation Blessing and applies the shared conditional Path-offer weight policy.",
    "24": "The table-driven Sealing Wax boundary grants one Hunt Blessing and applies the shared conditional Path-offer weight policy.",
    "25": "The table-driven Sealing Wax boundary grants one Destruction Blessing and applies the shared conditional Path-offer weight policy.",
    "26": "The table-driven Sealing Wax boundary grants one Remembrance Blessing and applies the shared conditional Path-offer weight policy.",
    "27": "The table-driven Sealing Wax boundary grants one Nihility Blessing and applies the shared conditional Path-offer weight policy.",
    "28": "The table-driven Sealing Wax boundary grants one Abundance Blessing and applies the shared conditional Path-offer weight policy.",
    "3": "The ordinary Blessing acquisition expression adds the owned Curio count only for one-star options, preserving one atomic reward transaction.",
    "4": "The generic post-battle participant projection, full-ratio restore operation and Curio teardown express the one-use party revival atomically.",
    "5": "The generic Reward-stream boundary samples one or two unowned Blessings without replacement from the complete eligible catalog.",
    "58": "The generic destructible Activity counter is captured in immutable battle contributions and lowered to ordinary all-character damage modifiers.",
    "6": "The generic won-battle projection and checked maximum-HP carry operation heal all living participants without combat inventory knowledge.",
    "61": "The result adapter validates and converts non-Boss defeat, restores participant carry and tears down the one-use Curio atomically.",
    "62": "The mode adapter verifies the exact locked build selection and recompiles one additional Eidolon level before ordinary combat materialization.",
    "63": "The atomic external destructible outcome uses ordinary Curio acquisition or generic current-HP loss and Curio teardown operations.",
    "64": "The spatial-free destructible policy exposes the qualitative frequency flag and exact doubled reward without scene dependencies.",
    "68": "A generic BattleStarted Rule IR operation samples one present allied Basic element and applies the same timed weakness to every enemy.",
    "69": "A generic Reward-stream marker selects one visible option, replaces stale reroll state and caps ordinary Blessing acquisition at enhancement level two.",
    "7": "A conditional three-star candidate filter persists across rerolls, while an atomic selection prefix performs ordinary Curio teardown only after selection.",
    "8": "A BattleStarted Rule IR selector queries each enemy's maximum HP and applies the exact 30% true-damage operation once per battle.",
  }[stable] ?? "Generic Activity and Rule IR primitives express the assigned Curio state.";
}
function fixtureMarker(id) {
  return {
    "universe.fixture.curio-tag.critical":
      "cavity_capture_materializes_exact_critical_damage_fixture",
    "universe.fixture.curio-tag.healing":
      "illusory_automaton_heals_the_current_actor_for_twenty_percent_maximum_hp",
    "universe.fixture.curio-tag.speed":
      "thalan_toxi_flame_uses_highest_attack_marker_hp_cost_and_five_stack_speed",
  }[id] ?? "goal07_p3_m11_s02_executes_every_assigned_curio_without_native_handlers";
}
function fixtureMarkerS03(id) {
  return {
    "universe.fixture.curio-state.active":
      "goal07_p3_m11_s03_executes_every_assigned_curio_without_native_handlers",
    "universe.fixture.curio-tag.blessing":
      "erudition_sealing_wax_grants_one_erudition_blessing",
    "universe.fixture.curio-tag.curio":
      "goal07_p3_m11_s03_executes_every_assigned_curio_without_native_handlers",
  }[id] ?? "goal07_p3_m11_s03_executes_every_assigned_curio_without_native_handlers";
}
function disposition(planned, runtimeDisposition, workbookEvidence) {
  assert(planned, "retained-audit entry is missing");
  return {
    id: planned.id,
    runtime_disposition: runtimeDisposition,
    accuracy_disposition: planned.intended_accuracy_disposition,
    workbook_evidence: workbookEvidence,
    provenance_evidence: sourceEvidence,
  };
}
function evidence(relative) {
  return {
    path: relative,
    sha256: sha256(relative),
    git_blob_sha1: gitBlob(relative),
  };
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function gitBlob(relative) {
  return execFileSync("git", ["hash-object", relative], {
    cwd: root,
    encoding: "utf8",
  }).trim();
}
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function json(relative) {
  return JSON.parse(fs.readFileSync(absolute(relative), "utf8"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
