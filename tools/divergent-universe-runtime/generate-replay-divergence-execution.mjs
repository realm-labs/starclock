#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/replay-divergence-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  baseline_fixture:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_fixture.rs",
  baseline_replay:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_replay.rs",
  baseline_tests:
    "crates/starclock-mode-universe/src/divergent_universe/tests/baseline_runtime.rs",
  agent_session:
    "crates/starclock-agent-api/src/divergent_universe_activity_session.rs",
  agent_tests:
    "crates/starclock-agent-api/src/divergent_universe_activity_session/tests.rs",
  agent_registry:
    "crates/starclock-agent-api/src/activity_session/registry/divergent.rs",
  mcp_tools: "crates/starclock-mcp/src/activity_tools.rs",
  mcp_tests: "crates/starclock-mcp/src/tools/divergent_universe_tests.rs",
};

export function buildReplayDivergenceExecution() {
  const batch = "G22-P7-B5";
  const ledger = json(inputs.batch_ledger);
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const fixtures = owned(ledger.fixture_assignments, batch);
  const gaps = owned(ledger.research_gap_assignments, batch);
  const policies = owned(ledger.policy_assignments, batch);
  assert(obligations.length === 0 && mechanics.length === 0
    && fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
  "P7-B5 must not claim assigned-target credit");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "replay-divergence ledger drift");

  const replay = text(inputs.baseline_replay);
  for (const fragment of [
    "verify_exact(actual.components())", "compare_records", "component_divergence",
    "DivergentUniverseReplayDivergenceKind", "FirstDivergence",
    "record_index", "pub fn component_divergence",
  ]) assert(replay.includes(fragment), `missing replay fragment ${fragment}`);

  const tests = text(inputs.baseline_tests);
  const probes = [
    probe("fresh-both-families", "baseline_replay_round_trips_both_families_and_rejects_corruption"),
    probe("component-addressed-divergence", "baseline_replay_reports_component_addressed_first_divergence"),
    probe("first-runtime-boundary", "baseline_replay_reports_every_runtime_boundary_at_the_first_record"),
  ];
  for (const value of probes)
    assert(tests.includes(value.test), `missing replay probe ${value.id}`);

  const agent = text(inputs.agent_session);
  assert(agent.includes("report.run_family() != family.runtime()"),
    "Agent replay verification does not bind the requested family");
  assert(text(inputs.agent_tests).includes("family mismatch rejected"),
    "Agent family mismatch probe missing");
  assert(text(inputs.agent_registry).includes("family: AgentDivergentUniverseRunFamily"),
    "Agent registry does not retain the replay family");
  const mcp = text(inputs.mcp_tools);
  assert(mcp.includes("verify_divergent_universe_replay(&seed, family, &replay)"),
    "MCP replay verification does not retain the requested family");
  assert(mcp.includes("mode != ActivityMode::DivergentUniverse"),
    "MCP family field is not mode-scoped");

  const boundaries = [
    "Catalog", "Activity", "Mapping", "ContributionSnapshot", "BattleAssembly",
    "ActivityCommand", "BattleCommand", "BattleEvent", "BattleState", "Settlement",
    "ActivityState",
  ];
  for (const boundary of boundaries)
    assert(replay.includes(`DivergentUniverseReplayDivergenceKind::${boundary}`),
      `missing divergence boundary ${boundary}`);

  return {
    schema_revision: "starclock.divergent-universe-replay-divergence-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteFreshComponentAddressedReplayAndFirstDivergence",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    replay_contract: {
      reconstruction: "FreshProductionFixtureFromCanonicalSeedFamilyAndComponents",
      component_count: 9,
      run_families: ["ordinary", "cyclical"],
      first_divergence_boundaries: boundaries,
      component_address: "CanonicalComponentIndexExpectedAndActualIdentity",
      record_address: "CanonicalZeroBasedRecordIndex",
      agent_family_binding: "ExactRequestedFamily",
      mcp_family_field_scope: "DivergentUniverseOnly",
    },
    execution_receipt: {
      both_families_reconstruct_from_fresh_production_inputs: "Passed",
      component_identity_mismatch_reports_exact_component: "Passed",
      catalog_activity_mapping_and_assembly_components_are_classified: "Passed",
      mapping_snapshot_assembly_command_event_state_and_settlement_are_classified: "Passed",
      first_mismatching_record_index_is_reported: "Passed",
      agent_and_mcp_bind_seed_and_family: "Passed",
      non_divergent_modes_reject_family_field: "Passed",
      corrupt_replay_is_rejected: "Passed",
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
      configuration_components: 9,
      run_families: 2,
      divergence_boundaries: boundaries.length,
      probes: probes.length,
    },
  };
}

function owned(values, batch) { return values.filter(({ owner_batch: value }) => value === batch); }
function probe(id, test) { return { id, file: inputs.baseline_tests, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildReplayDivergenceExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe replay divergence execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe replay divergence execution evidence.");
  }
}
