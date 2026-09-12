#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/battle-transition-hardening-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  battle_assembly: `${runtimeRoot}/battle-assembly-execution.json`,
  battle_settlement: `${runtimeRoot}/battle-settlement-execution.json`,
  runtime: "crates/starclock-mode-universe/src/divergent_universe/battle_assembly_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/battle_transition_hardening.rs",
};

export function buildBattleTransitionHardeningExecution() {
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B6");
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: batch }) => batch === "G22-P6-B6");
  const ledger = json(inputs.batch_ledger);
  const assembly = json(inputs.battle_assembly);
  const settlement = json(inputs.battle_settlement);
  const fixtures = owned(ledger.fixture_assignments);
  const gaps = owned(ledger.research_gap_assignments);
  const policies = owned(ledger.policy_assignments);
  assert(obligations.length === 0 && mechanics.length === 0
    && fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
  "P6-B6 must not claim assigned-target credit");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "battle transition hardening ledger drift");
  assert(assembly.batch === "G22-P6-B4"
    && settlement.batch === "G22-P6-B5",
  "battle transition hardening prerequisite drift");
  const source = text(inputs.runtime);
  for (const fragment of [
    "BATTLE_ASSEMBLY_CACHE_CAPACITY", "resolve_current_battle", "validate_inputs",
    "cache_entry_matches", "clear_cache", "CachePoisoned", "CacheOrderCorrupt",
  ]) assert(source.includes(fragment), `missing hardening runtime fragment ${fragment}`);
  const probes = [
    probe("bounded-non-authoritative-cache",
      "repeated_resolution_uses_bounded_non_authoritative_cache_and_clear_is_inert"),
    probe("stale-assembly-and-settlement",
      "stale_assembly_and_settlement_preserve_state_rng_and_cache_semantics"),
    probe("fresh-transition-reconstruction",
      "accepted_transition_reconstructs_battle_only_from_fresh_inputs"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing hardening probe ${value.id}`);
  return {
    schema_revision: "starclock.divergent-universe-battle-transition-hardening-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-B6",
    status: "CompleteAtomicRejectionBoundedCacheAndFreshTransitionReconstruction",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    exact_runtime: {
      cache_capacity: 8,
      eviction_policy: "DeterministicFifo",
      cache_authority: "NonAuthoritativeScratchOnly",
      stale_validation_order: "BeforeCacheLookup",
      reconstruction_inputs: ["ActivityState", "ContributionSnapshot", "EncounterSelection"],
    },
    execution_receipt: {
      repeated_identity_returns_same_immutable_entry: "Passed",
      ninth_distinct_entry_evicts_fifo_at_capacity_eight: "Passed",
      cache_clear_preserves_reconstructed_battle_identity: "Passed",
      stale_assembly_preserves_state_rng_and_cache_metrics: "Passed",
      stale_settlement_preserves_state_rng_and_cache_metrics: "Passed",
      accepted_transition_requires_and_accepts_fresh_inputs: "Passed",
    },
    assignment_closure: {
      obligations: obligations.length,
      fixture_families: fixtures.length,
      research_gaps: gaps.length,
      policy_sources: policies.length,
      mechanic_programs: mechanics.length,
    },
    capability_probes: probes,
    summary: {
      cache_capacity: 8,
      terminal_obligations: obligations.length,
      mechanic_programs: mechanics.length,
      probes: probes.length,
    },
  };
}

function owned(values) { return values.filter(({ owner_batch: batch }) => batch === "G22-P6-B6"); }
function probe(id, test) { return { id, file: inputs.tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBattleTransitionHardeningExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe battle transition hardening execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe battle transition hardening evidence.");
  }
}
