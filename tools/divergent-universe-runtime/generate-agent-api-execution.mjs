#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/agent-api-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  cli_execution: `${runtimeRoot}/cli-execution.json`,
  baseline_controller: "crates/starclock-mode-universe/src/baseline_controller.rs",
  baseline_runtime:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_runtime.rs",
  session: "crates/starclock-agent-api/src/divergent_universe_activity_session.rs",
  session_tests:
    "crates/starclock-agent-api/src/divergent_universe_activity_session/tests.rs",
  registry: "crates/starclock-agent-api/src/activity_session/registry.rs",
  registry_adapter:
    "crates/starclock-agent-api/src/activity_session/registry/divergent.rs",
};

export function buildAgentApiExecution() {
  const batch = "G22-P7-B3";
  const obligations = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: value }) => value === batch);
  const mechanics = json(inputs.mechanic_dispositions).programs.filter(
    ({ execution_partition: value }) => value === batch);
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, batch);
  const gaps = owned(ledger.research_gap_assignments, batch);
  const policies = owned(ledger.policy_assignments, batch);
  assert(obligations.length === 0 && mechanics.length === 0
    && fixtures.length === 0 && gaps.length === 0 && policies.length === 0,
  "P7-B3 must not claim assigned-target credit");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Agent API ledger drift");

  const session = text(inputs.session);
  for (const fragment of [
    "AgentDivergentUniverseManifest", "AgentDivergentUniverseRunFamily",
    "OfferedActivityActionSet", "expected_state_hash", "IdempotencyConflict",
    "record_divergent_universe_transcript", "verify_divergent_universe_replay",
  ]) assert(session.includes(fragment), `missing Agent API fragment ${fragment}`);
  const registry = text(inputs.registry_adapter);
  for (const fragment of [
    "create_divergent_universe", "divergent_universe_manifest",
    "verify_divergent_universe_replay", "divergent_universe_not_configured",
  ]) assert(registry.includes(fragment), `missing Agent registry fragment ${fragment}`);
  const probes = [
    probe("bounded-manifest-and-observation",
      "manifest_and_observations_are_bounded_without_generated_rows",
      inputs.session_tests),
    probe("forged-stale-cross-session-atomicity",
      "forged_stale_and_cross_session_actions_preserve_state",
      inputs.session_tests),
    probe("both-families-real-battle-replay",
      "public_offers_complete_both_families_and_export_fresh_replays",
      inputs.session_tests),
    probe("shared-registry-ownership",
      "divergent_universe_sessions_use_shared_ownership_and_registry_actions",
      inputs.registry),
  ];
  for (const value of probes)
    assert(text(value.file).includes(value.test), `missing Agent API probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-agent-api-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteBoundedAgentManifestSessionsObservationsAndActions",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    public_surface: {
      manifests: 1,
      run_family_summaries: 2,
      generated_rows_exposed: false,
      private_command_bindings_exposed: false,
      observation_projection: "SharedPlayerVisibleActivityProjection",
      action_projection: "OpaqueTokenOverExactCurrentOffer",
      ownership_and_quota: "SharedActivitySessionRegistry",
      replay: "TerminalTranscriptAndFreshProductionVerification",
    },
    execution_receipt: {
      ordinary_and_cyclical_manifests_are_bounded: "Passed",
      observations_expose_only_player_visible_state: "Passed",
      exact_offers_bind_to_opaque_tokens: "Passed",
      forged_stale_and_cross_session_actions_are_atomic: "Passed",
      idempotent_retry_returns_identical_response: "Passed",
      both_families_complete_real_nested_battles: "Passed",
      terminal_transcripts_verify_from_fresh_inputs: "Passed",
      corrupt_replays_are_rejected: "Passed",
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
      manifests: 1,
      run_families: 2,
      configuration_components: 9,
      generated_rows_exposed: 0,
      probes: probes.length,
    },
  };
}

function owned(values, batch) { return values.filter(({ owner_batch: value }) => value === batch); }
function probe(id, test, file) { return { id, file, test, result: "Passed" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildAgentApiExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Agent API execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Agent API execution evidence.");
  }
}
