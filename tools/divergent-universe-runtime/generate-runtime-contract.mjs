#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputPath = "content-manifests/divergent-universe-runtime-v1/runtime-contract.json";
const mechanicDispositionPath =
  "content-manifests/divergent-universe-runtime-v1/mechanic-dispositions.json";
const runtimeDispositionPath =
  "content-manifests/divergent-universe-runtime-v1/runtime-dispositions.json";

export function buildRuntimeContract() {
  const mechanics = json(mechanicDispositionPath);
  const dispositions = json(runtimeDispositionPath);
  assert(mechanics.summary.native_handlers_admitted === 0,
    "runtime contract starts with zero admitted native handlers");
  const runtimeStatus = dispositions.summary.runtime_status;
  assert((runtimeStatus.Pending ?? 0) + (runtimeStatus.Terminal ?? 0) === 6215
    && (runtimeStatus.Terminal ?? 0) >= 182,
  "runtime contract must bind the exact current disposition boundary");
  return {
    schema_revision: "starclock.divergent-universe-runtime-contract.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P0-B4",
    status: "FrozenTargetBoundary",
    input_digests: {
      mechanic_dispositions_sha256: sha256(mechanicDispositionPath),
      runtime_dispositions_sha256: sha256(runtimeDispositionPath),
    },
    architecture: {
      production_path: [
        "Divergent Universe workbooks",
        "Sora 0.6.1 production bundle",
        "starclock-data private lowering",
        "immutable Divergent Universe catalogs and typed programs",
        "GraphActivity accepted command",
        "immutable contribution and mapped-build snapshots",
        "BattleSpec and battle commands",
        "verified BattleResult",
        "atomic Activity settlement",
      ],
      forbidden_paths: [
        "runtime workbook, normalized JSON or source-cache loading",
        "combat lookup by Divergent Universe content ID",
        "battle mutation of live Activity state",
        "Activity mutation of a live battle",
        "DivergentUniverseActivity::apply or a mode-specific battle state machine",
        "account lookup or mutation from Activity or combat",
        "global mutable registration, runtime scripting or filesystem-discovered handlers",
      ],
    },
    public_api: {
      assembly_owner: "starclock-data",
      facade_owner: "starclock-mode-universe",
      target_types: [
        "DivergentUniverseRuntimeFactory",
        "DivergentUniverseEntry",
        "DivergentUniverseRuntime",
        "DivergentUniverseObservation",
        "DivergentUniverseOfferedCommand",
        "DivergentUniverseRuntimeError",
      ],
      generic_types_reused: [
        "GraphActivityCommand",
        "ActivityPlayerView",
        "ActivityDebugView",
        "ActivityBattleHandoff",
        "BattleSpec",
        "BattleResult",
        "ConfigurationComponentSet",
        "ResolvedCombatantSpec",
      ],
      public_data: [
        "stable IDs and bounded mode-owned observations",
        "currently offered commands with state hash and decision ID",
        "opaque component, account-snapshot, mapping and contribution identities",
        "battle handoff and result contracts",
        "typed errors and first-divergence reports",
      ],
      private_data: [
        "generated Sora row types and table identities",
        "raw configuration programs, normalized rows and source-cache paths",
        "lowering intermediates, private catalogs and internal slot allocation",
        "handler payloads and unoffered candidate pools",
        "presentation text, source locators and ID dereferencing",
      ],
      mutation_rule: "Every adapter mutation applies one currently offered GraphActivityCommand; convenience methods only construct that command and never bypass GraphActivity::apply.",
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
        { kind: "Choice", uses: "entry, run family, difficulty, Mapping, Equation, Blessing, Curio, Titan and transformation choices" },
        { kind: "Route", uses: "area, layer, room and next-node traversal" },
        { kind: "Encounter", uses: "encounter and boss confirmation" },
        { kind: "Preparation", uses: "party snapshot, mapping refresh and prebattle contribution confirmation" },
        { kind: "Reward", uses: "Equation, Blessing, Curio, Grand Miracle, Boon and progression offers" },
        { kind: "Shop", uses: "workbench, gamble, Curse Chest and priced offers" },
        { kind: "Service", uses: "service NPC, room service and Adventure entry" },
        { kind: "Roster", uses: "immutable participant/loadout selection without account mutation" },
        { kind: "ExternalOutcome", uses: "typed Adventure or unavailable minigame result" },
        { kind: "BattleReady", uses: "sealed BattleSpec handoff" },
        { kind: "Checkpoint", uses: "save/load checkpoint or terminal confirmation" },
        { kind: "Abandon", uses: "explicit deterministic run termination" },
      ],
      adapter_rule: "Adapters submit only commands present in the current bounded observation and cannot name hidden candidates.",
      rejection_rule: "Unknown, disabled, stale, duplicate, malformed or wrong-phase commands preserve canonical Activity bytes, state hash, events, pending handoff, caches and every RNG draw counter.",
      acceptance_rule: "An accepted command commits one ordered event/operation transaction or enters the documented deterministic terminal Activity fault.",
    },
    account_and_mapping_boundary: {
      input: "The caller supplies one immutable account/loadout snapshot identity and immutable owned build specs before run construction or an explicitly offered roster refresh.",
      mapping: "Arithmetic Mapping compares fields against immutable trial minimums, compiles a new ResolvedCombatantSpec and records field provenance; it never queries or mutates live account state.",
      sufficient_field_rule: "An already-sufficient caller field is preserved; only a below-threshold or absent field may receive the authored temporary contribution.",
      teardown: "Temporary mapping contributions end at their declared Run, party-change or run-terminal boundary and do not escape into caller-owned state.",
    },
    external_outcomes: {
      boundary: "SubmitExternalOutcome",
      use: "Only an authored interaction whose result is produced outside deterministic Activity mechanics may cross this boundary; movement, aiming, placement, timing UI and dialogue are never simulated.",
      validation: [
        "the exact outcome ID and payload shape are currently offered",
        "the decision, component root and expected state hash match",
        "the result maps to bounded typed Activity operations",
        "an optional random policy names one Activity RNG label, non-zero purpose and canonical candidate count",
      ],
    },
    battle_contract: {
      assembly_input: [
        "profile, run family, area, layer, node, attempt and battle sequence",
        "participant lock, immutable caller snapshot and resolved Mapping builds",
        "difficulty, Threshold Protocol and Astronomical Division state",
        "Equation, Blessing, Curio, Grand Miracle, Titan and progression contributions",
        "room, encounter, waves, enemy definitions, boss pool and explicit selector policy identity",
        "mechanic partition and immutable rule-registry identities",
      ],
      snapshot_rule: "Assembly consumes one immutable canonical contribution snapshot and immutable resolved combatants; no live Activity or account lookup is allowed after BattleSpec construction begins.",
      identity: [
        "activity definition and configuration digests",
        "configuration component root",
        "account/loadout and participant lock digests",
        "mapping and contribution snapshot digests",
        "combat input and assembly digests",
        "scope identity, battle sequence and purpose-derived Battle seed",
      ],
      required_result_fields: [
        "Outcome",
        "FinalStateHash",
        "EventDigest",
        "TerminalFault",
        "ParticipantState for every locked participant",
      ],
      optional_metric_rule: "A lowered mechanic may declare a bounded typed metric with a stable key before battle start; undeclared, duplicate or wrong-kind metrics reject settlement.",
      settlement_rule: "A result is accepted only for the exact pending handoff identity and is validated, carried, rewarded, progressed and traversed in one Activity transaction.",
    },
    component_set: [
      component("CombatCatalog", "combat-catalog", "current combat catalog"),
      component("BuildCatalog", "build-catalog", "current build catalog and Mapping contribution definitions"),
      component("ActivityCore", "divergent-universe-activity", "compiled graph, state and typed programs"),
      component("ModeProfile", "divergent-universe-profile", "entry, Ordinary/Cyclical and versioned policy profile"),
      component("ModeContent", "divergent-universe-content", "exact production Sora bundle and private lowering"),
      component("ActivityHandlerRegistry", "divergent-universe-activity-handlers", "immutable composed Activity registry"),
      component("CombatRuleRegistry", "divergent-universe-combat-rules", "immutable mode combat-rule bundle"),
      component("EncounterOverlay", "divergent-universe-encounter-overlay", "room, encounter, wave, enemy and boss assembly inputs"),
      component("Controller", "divergent-universe-baseline-controller", "caller-selected controller identity"),
    ],
    rng: {
      labels: [
        "Graph", "Encounter", "Reward", "Shop", "Occurrence", "Spawn",
        "ExternalOutcomeTest", "Battle",
      ],
      ownership: {
        Graph: "area, layer, room and flow selection",
        Encounter: "encounter, wave and boss selection",
        Reward: "Equation, Blessing, Curio, Grand Miracle, Titan and progression offers",
        Shop: "workbench, gamble, Curse Chest and service offers",
        Occurrence: "Occurrence choice and settlement policies",
        Spawn: "bounded Activity-owned spawn or replacement selection",
        ExternalOutcomeTest: "test-only external outcome generation",
        Battle: "purpose-derived nested battle seed only",
      },
      rule: "Each draw uses a named label, non-zero purpose and stable ordered candidate set; empty candidates consume no draw and rejection restores every counter.",
    },
    persistence: {
      snapshot_contents: [
        "canonical Activity state and state hash",
        "configuration component set and root",
        "master seed and every Activity RNG stream snapshot",
        "account/loadout, participant lock, Mapping and contribution snapshot identities",
        "current scope, decision, offered commands and pending BattleResult identity",
      ],
      save_rule: "Save is a read-only canonical snapshot operation and cannot advance state, consume RNG or materialize a new offer.",
      load_rule: "Load reconstructs fresh immutable catalogs and registries, verifies the exact current component set and all embedded identities, then resumes the same offered decision.",
      rejection_rule: "Unknown components, digest mismatch, malformed bounds, stale handoff or noncanonical order fail before a live session exists.",
      compatibility: "Only the current tree and current runtime identity are supported; no legacy decoder or migration branch is retained.",
    },
    handler_admission: {
      default_admitted: 0,
      registry: "bounded immutable static registry",
      requirements: [
        "the generated capability audit proves shared typed Activity or Rule IR cannot express the behavior",
        "one reviewed source-program set, mechanic partition and owner batch are named",
        "inputs and outputs are bounded typed values or operations",
        "trigger, phase, priority, cause ownership, snapshot and once-scope are explicit",
        "determinism, rejection inertness and production execution fixtures pass",
        "content-ID resolver branching, no-op output and runtime registration are forbidden",
      ],
    },
    failure_semantics: [
      { boundary: "CatalogOrLowering", behavior: "Fail before a run exists; no partial factory is returned." },
      { boundary: "RunConstruction", behavior: "Reject invalid profile, component, account snapshot, participant or policy identity before authoritative state exists." },
      { boundary: "CommandValidation", behavior: "Reject with byte-, hash-, event-, handoff-, cache- and RNG-inert authoritative state." },
      { boundary: "AcceptedExecution", behavior: "Commit ordered events atomically or enter a deterministic terminal Activity fault." },
      { boundary: "BattleAssembly", behavior: "Reject stale, invalid or over-budget snapshots without changing Activity state or authoritative cache identity." },
      { boundary: "NestedBattleInfrastructure", behavior: "Restore the pre-start Activity identity and append no battle report." },
      { boundary: "CombatFault", behavior: "Settle only through the sealed declared BattleResult TerminalFault field." },
      { boundary: "BattleSettlement", behavior: "Reject mismatched, duplicate, malformed or undeclared result fields without mutation." },
      { boundary: "SaveOrLoad", behavior: "Reject noncanonical or component-mismatched snapshots before a live session exists." },
      { boundary: "ReplayVerification", behavior: "Reconstruct fresh immutable inputs, report first divergence and never mutate a live session." },
    ],
  };
}

function slotFamilies() {
  const activity = [
    slot("entry_profile", "StableId", "Player", "CarryExact"),
    slot("entry", "StableId", "Player", "CarryExact"),
    slot("module", "StableId", "Player", "CarryExact"),
    slot("run_family", "StableId", "Player", "CarryExact"),
    slot("difficulty", "StableId", "Player", "CarryExact"),
    slot("threshold_protocol", "OptionalId", "Player", "CarryExact"),
    slot("astronomical_division", "OptionalId", "Player", "CarryExact"),
    slot("weekly_modifier", "OptionalId", "Player", "CarryExact"),
    slot("account_loadout_snapshot", "StableId", "Private", "CarryExact"),
    slot("party_snapshot", "StableId", "Private", "CarryExact"),
    slot("mapping_state", "BoundedCounterMap", "Player", "CarryExact"),
    slot("currencies", "BoundedCounterMap", "Player", "CarryExact"),
    slot("equations", "OrderedIdSet", "Player", "CarryExact"),
    slot("equation_progress", "BoundedCounterMap", "Player", "CarryExact"),
    slot("blessings", "OrderedIdSet", "Player", "CarryExact"),
    slot("blessing_levels", "BoundedCounterMap", "Player", "CarryExact"),
    slot("curios", "OrderedIdSet", "Player", "CarryExact"),
    slot("curio_state", "BoundedCounterMap", "Player", "CarryExact"),
    slot("grand_miracles", "OrderedIdSet", "Player", "CarryExact"),
    slot("titan_type", "OptionalId", "Player", "CarryExact"),
    slot("titan_boons", "BoundedCounterMap", "Player", "CarryExact"),
    slot("titan_talents", "BoundedCounterMap", "Player", "CarryExact"),
    slot("permanent_progression", "OrderedIdSet", "Player", "CarryExact"),
    slot("progression_state", "BoundedCounterMap", "Player", "CarryExact"),
    slot("run_flags", "OrderedIdSet", "DebugOnly", "CarryExact"),
  ].map((value) => ({ ...value, owner: "Activity", resets: ["ActivityStart"] }));
  const section = [
    slot("area", "StableId", "Player", "Replace"),
    slot("layer", "StableId", "Player", "Replace"),
    slot("layer_sequence", "BoundedInteger", "DebugOnly", "Replace"),
    slot("plane_carry", "BoundedCounterMap", "Private", "CarryExact"),
    slot("plane_flags", "OrderedIdSet", "DebugOnly", "Reset"),
  ].map((value) => ({ ...value, owner: "Section", resets: ["SectionStart"] }));
  const node = [
    slot("node", "StableId", "Player", "Replace"),
    slot("room", "OptionalId", "Player", "Replace"),
    slot("room_mark", "OptionalId", "Player", "Replace"),
    slot("encounter", "OptionalId", "Player", "Replace"),
    slot("service", "OptionalId", "Player", "Replace"),
    slot("occurrence", "OptionalId", "Player", "Replace"),
    slot("equation_offers", "OrderedIdSet", "Player", "Reset"),
    slot("blessing_offers", "OrderedIdSet", "Player", "Reset"),
    slot("curio_offers", "OrderedIdSet", "Player", "Reset"),
    slot("titan_offers", "OrderedIdSet", "Player", "Reset"),
    slot("node_flags", "OrderedIdSet", "DebugOnly", "Reset"),
  ].map((value) => ({ ...value, owner: "Node", resets: ["NodeStart"] }));
  const attempt = [
    slot("battle_sequence", "BoundedInteger", "DebugOnly", "Replace"),
    slot("pending_handoff", "OptionalId", "Private", "Reset"),
    slot("contribution_snapshot", "OptionalId", "Private", "Reset"),
    slot("external_outcome", "OptionalId", "DebugOnly", "Reset"),
    slot("last_battle_outcome", "OptionalId", "Player", "Reset"),
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

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath)))
    .digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildRuntimeContract());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate the Goal 22 runtime contract`);
    console.log("Divergent Universe runtime contract is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
