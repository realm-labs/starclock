#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputPath = "content-manifests/currency-wars-runtime-v1/runtime-contract.json";

export function buildRuntimeContract() {
  const dispositions = json(
    "content-manifests/currency-wars-runtime-v1/runtime-dispositions.json",
  );
  assert(dispositions.summary.native_handlers_admitted === 0,
    "runtime contract starts with zero admitted native handlers");
  return {
    schema_revision: "starclock.currency-wars-runtime-contract.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B4",
    status: "FrozenTargetBoundary",
    architecture: {
      production_path: [
        "Currency Wars workbooks",
        "Sora 0.6.1 production bundle",
        "starclock-data private lowering",
        "immutable mode catalogs and typed programs",
        "GraphActivity accepted command",
        "immutable contribution snapshot",
        "BattleSpec and battle commands",
        "verified BattleResult",
        "atomic Activity settlement",
      ],
      forbidden_paths: [
        "runtime workbook or normalized JSON loading",
        "combat catalog lookup by Currency Wars content ID",
        "battle mutation of live Activity state",
        "Activity mutation of a live battle",
        "mode-specific command processor or battle state machine",
        "global mutable registration or filesystem-discovered handlers",
      ],
    },
    public_api: {
      assembly_owner: "starclock-data",
      facade_owner: "starclock-mode-currency-wars",
      target_types: [
        "CurrencyWarsRuntimeFactory",
        "CurrencyWarsEntry",
        "CurrencyWarsRuntime",
        "CurrencyWarsObservation",
        "CurrencyWarsOfferedCommand",
        "CurrencyWarsRuntimeError",
      ],
      generic_types_reused: [
        "GraphActivityCommand",
        "ActivityPlayerView",
        "ActivityDebugView",
        "ActivityBattleHandoff",
        "BattleSpec",
        "BattleResult",
        "ConfigurationComponentSet",
      ],
      public_data: [
        "stable IDs and bounded mode-owned observations",
        "currently offered commands with state hash and decision ID",
        "opaque component identities and digests",
        "battle handoff and result contracts",
        "typed errors and first-divergence reports",
      ],
      private_data: [
        "generated Sora row types",
        "raw configuration programs and source-cache paths",
        "lowering intermediates and private catalogs",
        "handler payloads and internal slot allocation",
        "presentation text and ID dereferencing",
      ],
      mutation_rule: "Every adapter mutation applies one offered GraphActivityCommand; convenience methods may only construct such commands and may not bypass apply.",
    },
    scopes: [
      { authored: "Run", generic: "Activity" },
      { authored: "Plane", generic: "Section" },
      { authored: "NodeVisit", generic: "Node" },
      { authored: "BattleOrExternalAttempt", generic: "Attempt" },
    ],
    slot_families: slotFamilies(),
    command_contract: {
      envelope: ["expected_state_hash", "decision_id", "kind"],
      kinds: [
        "ChooseOption",
        "StartBattle",
        "SubmitBattleResult",
        "SubmitExternalOutcome",
        "Abandon",
      ],
      decisions: [
        "Choice", "Route", "Encounter", "Preparation", "Reward", "Shop",
        "Service", "Roster", "ExternalOutcome", "BattleReady", "Checkpoint",
        "Abandon",
      ],
      adapter_rule: "Adapters may submit only commands present in the current bounded observation.",
      rejection_rule: "Unknown, disabled, stale, duplicate, malformed or wrong-phase commands preserve canonical Activity bytes, state hash, events and every RNG draw counter.",
      acceptance_rule: "Accepted commands commit one ordered event/operation transaction or enter the documented deterministic Activity fault state.",
    },
    external_outcomes: {
      boundary: "SubmitExternalOutcome",
      use: "Only an authored interaction whose result is produced outside deterministic Activity mechanics may cross this boundary.",
      validation: [
        "outcome must be currently offered",
        "payload and component identity are bounded",
        "a typed static handler must be present in the immutable registry",
        "optional random policy names one Activity RNG label, purpose and candidate count",
      ],
    },
    battle_contract: {
      assembly_input: [
        "route, difficulty, Gambit, Plane, node, attempt and battle sequence",
        "participant lock and deployment positions",
        "resolved builds, equipment, stars and Character Empowerments",
        "Bond levels and contributions",
        "investment, formula, progression and node contributions",
        "encounter waves, enemy definitions, affixes, scaling and boss selection",
      ],
      snapshot_rule: "Assembly consumes one immutable canonical contribution snapshot; no live Activity lookup is allowed after BattleSpec construction begins.",
      identity: [
        "activity definition and configuration digests",
        "participant lock digest",
        "combat input digest",
        "assembly digest",
        "scope identity and battle sequence",
        "purpose-derived Battle seed",
      ],
      required_result_fields: [
        "Outcome",
        "FinalStateHash",
        "EventDigest",
        "TerminalFault",
        "ParticipantState for every locked participant",
        "currency_wars_battle_progress ratio metric",
        "currency_wars_action_value_remaining action-value metric",
      ],
      settlement_rule: "A result is accepted only for the exact pending handoff identity and is projected, carried, rewarded and traversed in one Activity transaction.",
    },
    component_set: [
      component("CombatCatalog", "combat-catalog", "current combat catalog"),
      component("BuildCatalog", "build-catalog", "current build catalog"),
      component("ActivityCore", "currency-wars-activity", "compiled graph, state and programs"),
      component("ModeProfile", "currency-wars-profile", "entry, route and policy profile"),
      component("ModeContent", "currency-wars-content", "exact production Sora bundle and lowering"),
      component("ActivityHandlerRegistry", "currency-wars-activity-handlers", "immutable composed registry"),
      component("CombatRuleRegistry", "currency-wars-combat-rules", "immutable mode combat-rule bundle"),
      component("EncounterOverlay", "currency-wars-encounter-overlay", "encounter and enemy assembly inputs"),
      component("Controller", "currency-wars-baseline-controller", "caller-selected controller identity"),
    ],
    rng: {
      labels: [
        "Graph", "Encounter", "Reward", "Shop", "Occurrence", "Spawn",
        "ExternalOutcomeTest", "Battle",
      ],
      rule: "Each draw uses a named label, non-zero purpose and stable ordered candidate set; empty candidates consume no draw and rejected commands restore all counters.",
    },
    handler_admission: {
      default_admitted: 0,
      registry: "bounded immutable static registry",
      requirements: [
        "the generated opcode/capability audit proves shared typed IR cannot express the behavior",
        "one reviewed source-program set and owner batch are named",
        "inputs and outputs are bounded typed values or operations",
        "trigger, phase, priority, snapshot and once-scope are explicit",
        "determinism, rejection inertness and production execution fixtures pass",
        "content-ID resolver branching, no-op output and runtime registration are forbidden",
      ],
    },
    failure_semantics: [
      { boundary: "CatalogOrLowering", behavior: "Fail before a run exists; no partial factory is returned." },
      { boundary: "CommandValidation", behavior: "Reject with byte-, hash-, event- and RNG-inert authoritative state." },
      { boundary: "AcceptedExecution", behavior: "Commit ordered events atomically or enter a deterministic terminal fault." },
      { boundary: "BattleAssembly", behavior: "Reject stale or invalid snapshots without changing Activity state or cache authority." },
      { boundary: "NestedBattleInfrastructure", behavior: "Restore the pre-start Activity identity and append no battle report." },
      { boundary: "CombatFault", behavior: "Settle only through the sealed declared BattleResult terminal-fault field." },
      { boundary: "BattleSettlement", behavior: "Reject mismatched, duplicate or malformed results without mutation." },
      { boundary: "ReplayVerification", behavior: "Reconstruct fresh immutable inputs, report first divergence and never mutate a live session." },
    ],
  };
}

function slotFamilies() {
  const activity = [
    slot("entry_profile", "StableId", "Player", "CarryExact"),
    slot("gambit", "StableId", "Player", "CarryExact"),
    slot("difficulty", "StableId", "Player", "CarryExact"),
    slot("route", "StableId", "Player", "CarryExact"),
    slot("gold", "BoundedInteger", "Player", "CarryExact"),
    slot("experience", "BoundedInteger", "Player", "CarryExact"),
    slot("team_level", "BoundedInteger", "Player", "CarryExact"),
    slot("back_capacity", "BoundedInteger", "Player", "CarryExact"),
    slot("shop_locked", "Boolean", "Player", "CarryExact"),
    slot("locked_shop_offers", "OrderedIdSet", "Player", "CarryExact"),
    slot("squad_hp", "BoundedInteger", "Player", "CarryExact"),
    slot("roster", "BoundedCounterMap", "Player", "CarryExact"),
    slot("deployment", "BoundedCounterMap", "Player", "CarryExact"),
    slot("equipment", "BoundedCounterMap", "Player", "CarryExact"),
    slot("bond_levels", "BoundedCounterMap", "Player", "CarryExact"),
    slot("empowerments", "OrderedIdSet", "Player", "CarryExact"),
    slot("investments", "OrderedIdSet", "Player", "CarryExact"),
    slot("investment_state", "BoundedCounterMap", "Private", "CarryExact"),
    slot("formula_state", "BoundedCounterMap", "Private", "CarryExact"),
    slot("permanent_progression", "OrderedIdSet", "Player", "CarryExact"),
    slot("run_flags", "OrderedIdSet", "DebugOnly", "CarryExact"),
  ].map((value) => ({ ...value, owner: "Activity", resets: ["ActivityStart"] }));
  const section = [
    slot("plane", "StableId", "Player", "Replace"),
    slot("rank", "BoundedInteger", "Player", "CarryExact"),
    slot("plane_carry", "BoundedCounterMap", "Private", "CarryExact"),
    slot("plane_flags", "OrderedIdSet", "DebugOnly", "Reset"),
  ].map((value) => ({ ...value, owner: "Section", resets: ["SectionStart"] }));
  const node = [
    slot("node", "StableId", "Player", "Replace"),
    slot("room", "OptionalId", "Player", "Replace"),
    slot("domain", "OptionalId", "Player", "Replace"),
    slot("shop_offers", "OrderedIdSet", "Player", "Reset"),
    slot("service_offers", "OrderedIdSet", "Player", "Reset"),
    slot("occurrence_choices", "OrderedIdSet", "Player", "Reset"),
    slot("node_flags", "OrderedIdSet", "DebugOnly", "Reset"),
  ].map((value) => ({ ...value, owner: "Node", resets: ["NodeStart"] }));
  const attempt = [
    slot("battle_sequence", "BoundedInteger", "DebugOnly", "Replace"),
    slot("last_squad_hp_loss", "BoundedInteger", "Player", "Reset"),
    slot("last_action_value_remaining", "BoundedInteger", "Player", "Reset"),
    slot("external_outcome", "OptionalId", "DebugOnly", "Reset"),
    slot("attempt_flags", "OrderedIdSet", "DebugOnly", "Reset"),
  ].map((value) => ({ ...value, owner: "Attempt", resets: ["AttemptStart"] }));
  return [...activity, ...section, ...node, ...attempt];
}

function slot(name, valueKind, visibility, carry) {
  return { name, value_kind: valueKind, visibility, carry };
}

function component(kind, id, digestSource) {
  return { kind, id, digest_source: digestSource };
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildRuntimeContract());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate the Goal 21 runtime contract`);
    console.log("Currency Wars runtime contract is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
