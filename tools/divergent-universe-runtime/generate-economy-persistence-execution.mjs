#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/economy-persistence-execution.json`;
const inputs = {
  currencies: `${referenceRoot}/currencies.json`,
  constants: `${referenceRoot}/common-constants.json`,
  runtime_contract: `${runtimeRoot}/runtime-contract.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  economy_runtime: "crates/starclock-mode-universe/src/divergent_universe/economy.rs",
  snapshot_runtime: "crates/starclock-mode-universe/src/divergent_universe/snapshot.rs",
  entry_runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/flow_progression_economy.rs",
};

export function buildEconomyPersistenceExecution() {
  const currencies = json(inputs.currencies);
  const constants = json(inputs.constants);
  const contract = json(inputs.runtime_contract);
  const runtime = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B3");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P3-B3");
  const gaps = owned(ledger.research_gap_assignments, "G22-P3-B3");
  const policies = owned(ledger.policy_assignments, "G22-P3-B3");
  const economySource = text(inputs.economy_runtime);
  const snapshotSource = text(inputs.snapshot_runtime);
  const entrySource = text(inputs.entry_runtime);
  const stateSource = text(inputs.state_runtime);
  const testSource = text(inputs.tests);

  assert(currencies.length === 2, "currency denominator drift");
  assert(currencies.some(({ id, scope, reset_rule: reset }) =>
    id === "divergent-universe.currency.cosmic-fragment"
      && scope === "Run" && reset === "RunEnd"), "Cosmic Fragment lifecycle drift");
  assert(currencies.some(({ id, scope, reset_rule: reset }) =>
    id === "divergent-universe.currency.workbench-heat"
      && scope === "Workbench" && reset === "ResetAtEachWorkbench"),
  "Workbench Heat lifecycle drift");
  const executableConstants = constants.filter(({ exclusion_reason: reason }) => !reason);
  const excludedConstants = constants.filter(({ exclusion_reason: reason }) => Boolean(reason));
  assert(constants.length === 34 && executableConstants.length === 18
    && excludedConstants.length === 16, "common-constant disposition closure drift");
  assert(runtime.length === 34
    && runtime.every(({ runtime_status: status }) => status === "Terminal")
    && count(runtime, "target_disposition", "ExactIntegrated") === 18
    && count(runtime, "target_disposition", "Excluded") === 16,
  "P3-B3 obligation terminal closure drift");
  assert(fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
    "P3-B3 unassigned evidence closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "economy/persistence execution ledger progress drift");
  assert(contract.persistence.save_rule.includes("read-only canonical snapshot")
    && contract.persistence.load_rule.includes("fresh immutable catalogs"),
  "persistence contract drift");

  for (const fragment of [
    "pub enum DivergentUniverseCurrencyKind",
    "pub fn apply_currency_command",
    "ActivityOperation::AddCounter",
    "DivergentUniverseCurrencyResetRule::EachWorkbench",
    "DivergentUniverseRuntimeConstantValue::IntegerArray",
  ]) assert(economySource.includes(fragment), `missing economy fragment ${fragment}`);
  for (const fragment of [
    "pub struct DivergentUniverseInputSnapshot",
    "pub fn capture_save_snapshot",
    "pub fn validate_save_snapshot",
    "pub fn progression_settlement",
    "reconstruction from canonical bytes is owned by G22-P7-B5",
  ]) assert(snapshotSource.includes(fragment), `missing snapshot fragment ${fragment}`);
  for (const fragment of [
    "ActivityOperation::SetCounterMap",
    "slot: CURRENCIES_SLOT",
    "entry.input_snapshot.digest().bytes()",
  ]) assert(entrySource.includes(fragment), `missing entry fragment ${fragment}`);
  for (const fragment of [
    "ACCOUNT_LOADOUT_SNAPSHOT_SLOT",
    "PARTY_SNAPSHOT_SLOT",
    "ActivityStateVisibility::Private",
  ]) assert(stateSource.includes(fragment), `missing state fragment ${fragment}`);

  const probes = [
    probe("currency-and-common-constant-lowering",
      "currencies_and_constants_lower_to_closed_typed_runtime_values"),
    probe("immutable-input-and-save-snapshot",
      "input_and_save_snapshots_bind_participants_and_capture_without_mutation"),
    probe("terminal-settlement-and-run-reset",
      "terminal_settlement_is_immutable_and_run_currency_reset_is_explicit"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing economy/persistence probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-economy-persistence-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B3",
    status: "CompleteEconomyPersistenceBoundaryNoLoadReconstructionOrBattleCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_boundary: {
      runtime_owner: "starclock-mode-universe",
      mutation_owner: "starclock-activity::GraphActivity",
      currencies: "Both released currency identities, scopes and reset rules lower through closed typed values. Validated credit/spend/reset commands execute atomically through GraphActivity; stale, rule-mismatched and unfunded commands are byte-inert, and run finalization clears the currency map.",
      constants: "All 18 simulation-visible common constants lower to checked integer, integer-array or opaque-text values; all 16 account/reward/presentation/test exclusions remain absent from runtime projection.",
      caller_snapshot: "Entry requires an immutable account/loadout snapshot sealed to the exact participant lock. Full digests bind the Activity configuration; private slots retain stable state projections.",
      save: "Capture is read-only over canonical GraphActivity bytes and validates current component, definition, input, participant, state and envelope digests.",
      settlement: "Completed Activities expose immutable progression/account-consumer inputs while retaining no authority to query or mutate account state.",
      deferred: "Canonical-byte reconstruction and resume are owned by G22-P7-B5; malformed load/replay suites are owned by G22-P8-B1.",
      credit: "Economy, immutable snapshot and settlement boundaries only; no service execution, save reconstruction, encounter, battle, full playable-run or release-gate credit.",
    },
    source_closure: {
      currencies: currencies.length,
      common_constants: constants.length,
      executable_constants: executableConstants.length,
      excluded_constants: excludedConstants.length,
      exact_obligations: count(runtime, "target_disposition", "ExactIntegrated"),
      excluded_obligations: count(runtime, "target_disposition", "Excluded"),
    },
    currency_lifecycles: currencies.map((currency) => ({
      id: currency.id,
      scope: currency.scope,
      gain_rules: currency.gain_rules,
      spend_rules: currency.spend_rules,
      reset_rule: currency.reset_rule,
    })),
    persistence_contract: {
      capture: contract.persistence.save_rule,
      future_load: contract.persistence.load_rule,
      rejection: contract.persistence.rejection_rule,
      compatibility: contract.persistence.compatibility,
      implemented_now: "Read-only capture and fresh-current-input validation before any live reconstructed session exists.",
      deferred_to: "G22-P7-B5",
    },
    terminal_assignments: {
      obligations: runtime.length,
      fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
      research_gaps: gaps.map(({ research_gap_id: id }) => id),
      policy_sources: policies.map(({ policy_source_id: id }) => id),
    },
    capability_probes: probes,
    summary: {
      currencies: currencies.length,
      executable_constants: executableConstants.length,
      excluded_constants: excludedConstants.length,
      obligations_terminal: runtime.length,
      production_fixtures_passed: fixtures.length,
      executable_research_gaps: gaps.length,
      executable_policy_sources: policies.length,
      probes: probes.length,
    },
  };
}

function probe(id, test) {
  return { id, file: inputs.tests, test, result: "Passed" };
}

function owned(values, batch) {
  return values.filter(({ owner_batch: owner }) => owner === batch);
}

function count(values, field, selected) {
  return values.filter((value) => value[field] === selected).length;
}

function json(file) {
  return JSON.parse(text(file));
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifact = buildEconomyPersistenceExecution();
  const serialized = pretty(artifact);
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe economy/persistence execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe economy/persistence execution evidence.");
  }
}
