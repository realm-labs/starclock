#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/cli-execution.json`;
const inputs = {
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  mechanic_dispositions: `${runtimeRoot}/mechanic-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  cli: "crates/starclock-cli/src/divergent_universe.rs",
  cli_router: "crates/starclock-cli/src/main.rs",
  baseline_fixture:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_fixture.rs",
  baseline_replay:
    "crates/starclock-mode-universe/src/divergent_universe/baseline_replay.rs",
  tests: "crates/starclock-cli/tests/divergent_universe_cli.rs",
};

export function buildCliExecution() {
  const batch = "G22-P7-B2";
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
  "P7-B2 must not claim assigned-target credit");
  const cli = text(inputs.cli);
  for (const fragment of [
    "universe-config-validation", "universe-coverage", "universe-run",
    "--family", "--replay-out", "configuration_components",
    "verify_divergent_universe_replay", "DivergentUniverseCliError",
  ]) assert(cli.includes(fragment), `missing CLI fragment ${fragment}`);
  const router = text(inputs.cli_router);
  for (const fragment of [
    "divergent_universe::requested", "divergent_universe::run",
    "divergent_universe::coverage", "divergent_universe::config_validate",
    "divergent_universe::is_replay", "divergent_universe::verify_replay",
  ]) assert(router.includes(fragment), `missing CLI router fragment ${fragment}`);
  const probes = [
    probe("configuration-and-coverage",
      "divergent_universe_configuration_and_coverage_are_machine_readable"),
    probe("ordinary-and-cyclical-run-replay",
      "divergent_universe_both_families_complete_and_replay_verifies"),
    probe("corruption-and-invalid-options",
      "divergent_universe_rejects_corrupt_replay_and_invalid_options"),
  ];
  const testSource = text(inputs.tests);
  for (const value of probes)
    assert(testSource.includes(value.test), `missing CLI probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-cli-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch,
    status: "CliSurfaceDeclaredBehavioralAuditIncomplete",
    runtime_release_ready: false,
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    command_surface: {
      mode: "divergent-universe",
      run_families: ["ordinary", "cyclical"],
      controller: "baseline",
      configuration_components: 9,
      configuration_validation: "ProductionSoraAndCoreBundleLoad",
      coverage: "BehavioralAuditIncompleteNoTerminalCountClaim",
      replay_export: "CanonicalComponentAddressedActivityReplay",
      replay_verify: "FreshProductionFixtureExactByteReconstruction",
    },
    verification_requirements: {
      command: "cargo test -p starclock-cli --test divergent_universe_cli",
      result: "NotExecutedByGenerator",
      scope: "CLI smoke and replay checks, not complete gameplay acceptance",
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
      commands: 4,
      run_families: 2,
      configuration_components: 9,
      terminal_obligations: null,
      terminal_mechanic_programs: null,
      probes: probes.length,
    },
  };
}

function owned(values, batch) { return values.filter(({ owner_batch: value }) => value === batch); }
function probe(id, test) { return { id, file: inputs.tests, test, result: "DeclaredNotExecuted" }; }
function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildCliExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe CLI execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe CLI surface declaration (no execution claim).");
  }
}
