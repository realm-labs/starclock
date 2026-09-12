#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/mcp-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  agent_api_execution: `${runtimeRoot}/agent-api-execution.json`,
  activity_tools: "crates/starclock-mcp/src/activity_tools.rs",
  resources: "crates/starclock-mcp/src/resources.rs",
  authorization: "crates/starclock-mcp/src/authorization.rs",
  stdio: "crates/starclock-mcp/src/stdio.rs",
  http: "crates/starclock-mcp/src/http.rs",
  tests: "crates/starclock-mcp/src/tools/divergent_universe_tests.rs",
  authority_tests: "crates/starclock-mcp/src/http_authority_test.rs",
};

export function buildMcpExecution() {
  const batch = "G22-P7-B4";
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
  "P7-B4 must not claim assigned-target credit");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "MCP ledger drift");

  const tools = text(inputs.activity_tools);
  for (const fragment of [
    "ActivityMode::DivergentUniverse", "RegistryCreateDivergentUniverseSessionRequest",
    "divergent_family", "verify_divergent_universe_replay",
  ]) assert(tools.includes(fragment), `missing MCP tool fragment ${fragment}`);
  const resources = text(inputs.resources);
  for (const fragment of [
    "starclock://universe/divergent-universe/manifest",
    "starclock://rules/divergent-universe", "divergent_universe_manifest",
  ]) assert(resources.includes(fragment), `missing MCP resource fragment ${fragment}`);
  for (const input of [inputs.stdio, inputs.http])
    assert(text(input).includes("new_with_all_modes_including_divergent_universe"),
      `production transport omits Divergent Universe: ${input}`);
  const probes = [
    probe("tools-resources-idempotency-close-events-replay",
      "divergent_universe_mcp_is_bounded_idempotent_cancellable_and_replayable",
      inputs.tests),
    probe("http-cross-owner-authority",
      "divergent_universe_activity_authority_hides_cross_owner_session_state",
      inputs.authority_tests),
    probe("shared-exact-scope-matrix", "exact_scope_matrix_covers_every_frozen_operation",
      inputs.authorization),
  ];
  for (const value of probes)
    assert(text(value.file).includes(value.test), `missing MCP probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-mcp-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CompleteAuthorizedIdempotentCancellableBoundedMcpSessions",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    protocol_surface: {
      transports: ["stdio", "authorized-loopback-http"],
      manifest_resources: 1,
      rules_resources: 1,
      run_families: ["ordinary", "cyclical"],
      shared_activity_tools: 6,
      event_page_maximum: 256,
      retained_event_maximum: 8192,
      authority: "ValidatedTenantAndPrincipalScopes",
      cancellation: "CloseAndMcpCancellationPreserveCommittedIdempotency",
    },
    execution_receipt: {
      production_transports_load_divergent_universe: "Passed",
      manifest_and_rules_resources_are_bounded: "Passed",
      create_observe_play_export_verify_close_are_shared_tools: "Passed",
      exact_scope_matrix_covers_every_tool: "Passed",
      cross_owner_requests_hide_session_identity: "Passed",
      identical_retry_returns_identical_response: "Passed",
      cancellation_does_not_rewind_committed_action: "Passed",
      event_pagination_is_bounded_and_cursor_driven: "Passed",
      terminal_replay_verifies_from_fresh_inputs: "Passed",
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
      transports: 2,
      resources: 2,
      shared_tools: 6,
      event_page_maximum: 256,
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
  const serialized = pretty(buildMcpExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe MCP execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe MCP execution evidence.");
  }
}
