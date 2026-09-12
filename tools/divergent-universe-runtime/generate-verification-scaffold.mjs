#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const goalPath = "docs/goals/22-divergent-universe-runtime.md";
const outputPath = `${runtimeRoot}/verification-scaffold.json`;

export function buildVerificationScaffold() {
  const ledger = json(`${runtimeRoot}/batch-ledger.json`);
  const dispositions = json(`${runtimeRoot}/runtime-dispositions.json`).obligations;
  const mechanics = json(`${runtimeRoot}/mechanic-dispositions.json`).programs;
  const partitions = json(`${runtimeRoot}/mechanic-partitions.json`).partitions;
  const verification = json(`${runtimeRoot}/verification-contract.json`);
  const partitionByBatch = new Map(partitions.map((value) => [value.batch, value]));
  const boundary = required(ledger.batches.find(({ batch }) => batch === "G22-P0-B6"),
    "G22-P0-B6 ledger row");
  const later = ledger.batches.filter(({ ordinal }) => ordinal > boundary.ordinal);
  assert(later.length === 59, "later batch denominator drift");

  const batches = later.map((row) => {
    const partition = partitionByBatch.get(row.batch) ?? null;
    const assigned = assignedTargets(row.batch, dispositions, mechanics, ledger,
      verification, partition);
    const ownership = ownershipFor(row, partition);
    return {
      batch: row.batch,
      ordinal: row.ordinal,
      phase: row.phase,
      kind: row.kind,
      prerequisites: row.prerequisites,
      deliverable: row.deliverable,
      assigned_targets: assigned,
      owned_files: ownership,
      focused_gate: focusedGate(row, ownership.packages),
      terminal_evidence: terminalEvidence(row, assigned, partition),
      status: row.status,
    };
  });

  return {
    schema_revision: "starclock.divergent-universe-verification-scaffold.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P0-B6",
    status: "BehavioralAuditIncomplete",
    input_digests: Object.fromEntries([
      goalPath,
      `${runtimeRoot}/foundation.json`,
      `${runtimeRoot}/runtime-dispositions.json`,
      `${runtimeRoot}/mechanic-dispositions.json`,
      `${runtimeRoot}/mechanic-partitions.json`,
      `${runtimeRoot}/batch-ledger.json`,
      `${runtimeRoot}/runtime-contract.json`,
      `${runtimeRoot}/verification-contract.json`,
      `${runtimeRoot}/shared-capability-audit.json`,
      `${runtimeRoot}/entry-flow-execution.json`,
      `${runtimeRoot}/progression-execution.json`,
      `${runtimeRoot}/economy-persistence-execution.json`,
      `${runtimeRoot}/mapping-execution.json`,
      `${runtimeRoot}/scope-identity-execution.json`,
      `${runtimeRoot}/ordinary-vertical-slice-execution.json`,
      `${runtimeRoot}/equation-offer-execution.json`,
      `${runtimeRoot}/equation-progress-execution.json`,
      `${runtimeRoot}/equation-battle-execution.json`,
      `${runtimeRoot}/blessing-runtime-execution.json`,
      `${runtimeRoot}/blessing-interaction-execution.json`,
      `${runtimeRoot}/equation-blessing-hardening-execution.json`,
      `${runtimeRoot}/curio-runtime-execution.json`,
      `${runtimeRoot}/grand-miracle-gamble-execution.json`,
      `${runtimeRoot}/titan-runtime-execution.json`,
      `${runtimeRoot}/permanent-progression-execution.json`,
      `${runtimeRoot}/workbench-curse-execution.json`,
      `${runtimeRoot}/contribution-snapshot-execution.json`,
      `${runtimeRoot}/occurrence-execution.json`,
      `${runtimeRoot}/service-adventure-execution.json`,
      `${runtimeRoot}/encounter-reachability-execution.json`,
      `${runtimeRoot}/battle-assembly-execution.json`,
      `${runtimeRoot}/battle-settlement-execution.json`,
      `${runtimeRoot}/battle-transition-hardening-execution.json`,
      `${runtimeRoot}/activity-state-mechanic-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a02-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a03-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a04-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a05-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a06-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a07-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a08-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a09-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a10-execution.json`,
      `${runtimeRoot}/activity-decision-mechanic-a11-execution.json`,
      `${runtimeRoot}/activity-external-outcome-mechanic-execution.json`,
      `${runtimeRoot}/battle-mechanic-reachability-proof.json`,
      `${runtimeRoot}/battle-mechanic-execution.json`,
      `${runtimeRoot}/baseline-run-execution.json`,
      `${runtimeRoot}/cli-execution.json`,
      `${runtimeRoot}/agent-api-execution.json`,
      `${runtimeRoot}/mcp-execution.json`,
      `${runtimeRoot}/replay-divergence-execution.json`,
      `${runtimeRoot}/matrix-execution.json`,
      `${runtimeRoot}/hardening-execution.json`,
      `${runtimeRoot}/performance-execution.json`,
      `${runtimeRoot}/repository-audit-execution.json`,
    ].map((file) => [file, sha256(file)])),
    sequencing_rule: "Exactly one batch may be active. A batch starts only after every prerequisite is Complete in the regenerated current ledger; shared-file conflicts stop execution for reconciliation.",
    completion_rule: "Complete requires every focused command and assertion plus terminal current-state evidence for every assigned target. Catalog identities, status text, no-op handlers and inherited policy names earn no execution credit.",
    file_ownership_rule: "Owned patterns are the maximum expected write surface for the batch. A discovered dependency outside them requires an explicit boundary review and scaffold regeneration before editing.",
    common_rejection_rule: "Rejected, stale, malformed and wrong-phase commands preserve authoritative bytes, hashes, events, RNG counters, pending handoffs and cache authority unless the frozen contract declares a deterministic terminal fault.",
    summary: {
      later_batches: batches.length,
      fixed_batches: batches.filter(({ kind }) => kind === "Fixed").length,
      generated_mechanic_partitions: batches.filter(({ kind }) =>
        kind === "GeneratedMechanicPartition").length,
      phases: countBy(batches, ({ phase }) => String(phase)),
      statuses: countBy(batches, ({ status }) => status),
      assigned_catalog_obligations: sum(batches,
        ({ assigned_targets }) => assigned_targets.catalog_obligations),
      assigned_execution_obligations: sum(batches,
        ({ assigned_targets }) => assigned_targets.execution_obligations),
      assigned_mechanic_programs: sum(batches,
        ({ assigned_targets }) => assigned_targets.mechanic_programs),
      assigned_semantic_fixtures: sum(batches,
        ({ assigned_targets }) => assigned_targets.semantic_fixtures),
      assigned_research_gaps: sum(batches,
        ({ assigned_targets }) => assigned_targets.research_gaps),
      assigned_policy_sources: sum(batches,
        ({ assigned_targets }) => assigned_targets.policy_sources),
    },
    release_gates: releaseGates(verification),
    batches,
  };
}

function assignedTargets(batch, dispositions, mechanics, ledger, verification, partition) {
  const programs = mechanics.filter(({ execution_partition }) =>
    execution_partition === batch).length;
  if (partition !== null)
    assert(programs === partition.program_count, `${batch} mechanic assignment drift`);
  return {
    catalog_obligations: dispositions.filter(({ catalog_batch }) => catalog_batch === batch).length,
    execution_obligations: dispositions.filter(({ execution_partition }) =>
      execution_partition === batch).length,
    mechanic_programs: programs,
    semantic_fixtures: ledger.fixture_assignments.filter(({ owner_batch }) =>
      owner_batch === batch).length,
    research_gaps: ledger.research_gap_assignments.filter(({ owner_batch }) =>
      owner_batch === batch).length,
    policy_sources: ledger.policy_assignments.filter(({ owner_batch }) =>
      owner_batch === batch).length,
    matrix_cases: batch === "G22-P7-B6" ? verification.matrix_cases.length : 0,
    performance_workloads: batch === "G22-P8-B2"
      ? verification.performance_workloads.length : 0,
    native_platforms: batch === "G22-P8-B5"
      ? verification.native_ci_expectations.platforms.length : 0,
  };
}

function ownershipFor(row, partition) {
  let packages = phasePackages(row);
  if (partition !== null) packages = [...partition.execution_owners];
  packages = [...new Set(packages)].sort();
  const patterns = packages.map((packageName) =>
    `crates/${packageName}/src/divergent_universe/**`);
  patterns.push(
    `crates/starclock-test-kit/tests/suites/divergent_universe/${row.batch.toLowerCase()}/**`,
    `${runtimeRoot}/**`,
    "tools/divergent-universe-runtime/**",
  );
  if (row.phase === 1) patterns.push(
    "config/divergent-universe/**",
    "config/divergent-universe-generated/**",
    "tools/divergent-universe-reference/**",
  );
  if (row.phase === 7) patterns.push(
    "crates/starclock-cli/src/divergent_universe/**",
    "crates/starclock-agent-api/src/divergent_universe/**",
    "crates/starclock-mcp/src/divergent_universe/**",
  );
  if (row.phase === 8) patterns.push("docs/state.md", "policy/state.json");
  return { packages, patterns: [...new Set(patterns)].sort() };
}

function phasePackages(row) {
  if (row.phase === 1) return ["starclock-data", "starclock-mode-universe"];
  if (row.phase === 2) {
    const byBatch = {
      "G22-P2-B1": ["starclock-activity", "starclock-build", "starclock-combat", "starclock-data", "starclock-mode-universe"],
      "G22-P2-B2": ["starclock-activity"],
      "G22-P2-B3": ["starclock-combat", "starclock-rules"],
      "G22-P2-B4": ["starclock-build"],
      "G22-P2-B5": ["starclock-activity", "starclock-build", "starclock-combat", "starclock-rules", "starclock-mode-universe"],
    };
    return required(byBatch[row.batch], `${row.batch} phase-2 ownership`);
  }
  if (row.phase === 3)
    return row.batch === "G22-P3-B4"
      ? ["starclock-build", "starclock-mode-universe"]
      : ["starclock-activity", "starclock-mode-universe"];
  if (row.phase === 4 || row.phase === 5)
    return ["starclock-data", "starclock-mode-universe"];
  if (row.phase === 6)
    return ["starclock-activity", "starclock-combat", "starclock-data", "starclock-mode-universe"];
  if (row.phase === 7)
    return ["starclock-agent-api", "starclock-cli", "starclock-mcp", "starclock-mode-universe", "starclock-replay"];
  if (row.phase === 8)
    return [
      "starclock-agent-api", "starclock-data", "starclock-mode-universe",
      "starclock-replay", "starclock-test-kit",
    ];
  throw new Error(`unsupported phase ${row.phase}`);
}

function focusedGate(row, packages) {
  const commands = [
    "node tools/divergent-universe-runtime/verify-foundation.mjs --source-cache .cache/content-reference",
    "node tools/divergent-universe-runtime/verify-dispositions.mjs",
    "node tools/divergent-universe-runtime/verify-runtime-contract.mjs",
    "node tools/divergent-universe-runtime/verify-verification-contract.mjs",
    "node tools/divergent-universe-runtime/verify-verification-scaffold.mjs",
  ];
  if (row.phase === 1) commands.push(
    "node tools/divergent-universe-reference/verify-sora-schema.mjs",
    "node tools/divergent-universe-reference/verify-sora-release.mjs",
  );
  if (row.phase >= 2) commands.push(
    "node tools/divergent-universe-runtime/verify-capability-inventory.mjs",
    "node tools/divergent-universe-runtime/verify-activity-capability-closure.mjs",
    "node tools/divergent-universe-runtime/verify-combat-capability-closure.mjs",
    "node tools/divergent-universe-runtime/verify-build-capability-closure.mjs",
    "node tools/divergent-universe-runtime/verify-shared-capability-audit.mjs",
  );
  if (row.phase >= 3) commands.push(
    "node tools/divergent-universe-runtime/verify-entry-flow-execution.mjs",
    "node tools/divergent-universe-runtime/verify-progression-execution.mjs",
    "node tools/divergent-universe-runtime/verify-economy-persistence-execution.mjs",
    "node tools/divergent-universe-runtime/verify-mapping-execution.mjs",
    "node tools/divergent-universe-runtime/verify-scope-identity-execution.mjs",
    "node tools/divergent-universe-runtime/verify-ordinary-vertical-slice-execution.mjs",
    "node tools/divergent-universe-runtime/verify-equation-offer-execution.mjs",
    "node tools/divergent-universe-runtime/verify-equation-progress-execution.mjs",
    "node tools/divergent-universe-runtime/verify-equation-battle-execution.mjs",
    "node tools/divergent-universe-runtime/verify-blessing-runtime-execution.mjs",
    "node tools/divergent-universe-runtime/verify-blessing-interaction-execution.mjs",
    "node tools/divergent-universe-runtime/verify-equation-blessing-hardening-execution.mjs",
    "node tools/divergent-universe-runtime/verify-curio-runtime-execution.mjs",
    "node tools/divergent-universe-runtime/verify-grand-miracle-gamble-execution.mjs",
    "node tools/divergent-universe-runtime/verify-titan-runtime-execution.mjs",
    "node tools/divergent-universe-runtime/verify-permanent-progression-execution.mjs",
    "node tools/divergent-universe-runtime/verify-workbench-curse-execution.mjs",
    "node tools/divergent-universe-runtime/verify-contribution-snapshot-execution.mjs",
    "node tools/divergent-universe-runtime/verify-occurrence-execution.mjs",
    "node tools/divergent-universe-runtime/verify-service-adventure-execution.mjs",
    "node tools/divergent-universe-runtime/verify-encounter-reachability-execution.mjs",
    "node tools/divergent-universe-runtime/verify-battle-assembly-execution.mjs",
    "node tools/divergent-universe-runtime/verify-battle-settlement-execution.mjs",
    "node tools/divergent-universe-runtime/verify-battle-transition-hardening-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-state-mechanic-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a02-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a03-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a04-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a05-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a06-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a07-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a08-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a09-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a10-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-decision-mechanic-a11-execution.mjs",
    "node tools/divergent-universe-runtime/verify-activity-external-outcome-mechanic-execution.mjs",
    "node tools/divergent-universe-runtime/verify-battle-mechanic-reachability-proof.mjs",
    "node tools/divergent-universe-runtime/verify-battle-mechanic-execution.mjs",
  );
  if (row.phase >= 7) commands.push(
    "node tools/divergent-universe-runtime/verify-baseline-run-execution.mjs",
    "node tools/divergent-universe-runtime/verify-cli-execution.mjs",
    "node tools/divergent-universe-runtime/verify-agent-api-execution.mjs",
    "node tools/divergent-universe-runtime/verify-mcp-execution.mjs",
    "node tools/divergent-universe-runtime/verify-replay-divergence-execution.mjs",
    "node tools/divergent-universe-runtime/verify-matrix-execution.mjs",
  );
  if (row.phase >= 8) commands.push(
    "node tools/divergent-universe-runtime/verify-hardening-execution.mjs",
    "node tools/divergent-universe-runtime/verify-performance-execution.mjs",
    "node tools/divergent-universe-runtime/verify-repository-audit-execution.mjs",
  );
  commands.push("cargo fmt --all -- --check");
  for (const packageName of packages) commands.push(
    `cargo clippy -p ${packageName} --all-targets -- -D warnings`,
    `cargo test -p ${packageName}`,
  );
  if (row.batch === "G22-P7-B6") commands.push(
    "cargo test --release -p starclock-mode-universe --test divergent_universe_matrix -- --ignored --exact generated_legal_matrix_completes_real_battles_and_fresh_replay",
  );
  if (row.batch === "G22-P8-B1") commands.push(
    "cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite",
  );
  if (row.batch === "G22-P8-B2") commands.push(
    "cargo run --release -p starclock-agent-api --example divergent_universe_benchmark --features benchmark-harness",
  );
  if (row.batch === "G22-P8-B3") commands.push(
    "node tools/repository-check/verify-data.mjs",
  );
  if (row.batch === "G22-P8-B4" || row.batch === "G22-P8-B5") commands.push(
    "cargo test -p starclock-mode-universe --lib divergent_universe::tests::release_room_completion_changes_gameplay_beyond_a_lifecycle_marker -- --ignored --exact",
  );
  if (row.batch === "G22-P8-B5") commands.push(
    "cargo test --workspace",
    "node tools/divergent-universe-runtime/run-clean-checkout.mjs",
  );
  return {
    commands,
    required_assertions: phaseAssertions(row.phase),
    scope_rule: "Begin with the narrowest changed test; run package gates once per completed batch and workspace/native gates only where explicitly owned.",
  };
}

function phaseAssertions(phase) {
  const specific = {
    1: "private Sora lowering validates every row, join, bound and component digest without exposing generated row types",
    2: "shared capability is content-agnostic, typed, deterministic and cannot branch on mode/content IDs",
    3: "entry, progression and Mapping mutate only through offered commands and immutable caller/build snapshots",
    4: "Equation and Blessing ownership, contribution, expansion, trigger order and teardown execute rather than retain IDs",
    5: "Curio, Miracle, Titan, progression and service lifecycles change typed authoritative state at declared scopes",
    6: "Occurrence, external outcome, encounter and mechanic programs lower to real operations/BattleSpec inputs with audited handlers",
    7: "all adapters expose equivalent offered commands and replay reconstructs fresh production inputs through real battles",
    8: "hardening, performance, audits, exact-once closure and native clean-checkout evidence bind the current tree",
  };
  return [
    "generated artifacts reproduce byte-identically and denominators do not decrease",
    "assigned targets are exact-once and terminal evidence is production-executed or proven non-runtime",
    "rejected boundaries preserve bytes, hashes, events, RNG, handoff and cache authority",
    required(specific[phase], `phase ${phase} assertion`),
  ];
}

function terminalEvidence(row, assigned, partition) {
  const artifacts = [
    "regenerated current disposition, coverage and status manifests",
    "focused production-lowered behavior test with nearest control and rejection case",
    "canonical current hashes/digests when authoritative state or replay identity changes",
  ];
  if (row.phase === 1) artifacts.push("real Sora bundle reader-load and invalid-join fixture");
  if (partition !== null) artifacts.push(
    `terminal exact-once dispositions for ${partition.program_count} mechanic programs`,
    `production execution fixtures for ${partition.fixture_family_ids.join(", ")}`,
  );
  if (row.phase === 6) artifacts.push("real BattleSpec/BattleResult or typed external-outcome settlement receipt");
  if (row.phase === 7) artifacts.push("fresh component-addressed replay and adapter parity receipt");
  if (row.phase === 8) artifacts.push("fresh exhaustive, performance, audit or native release output owned by this batch");
  return {
    assigned_counts: assigned,
    required_artifacts: artifacts,
    terminal_states: [
      "ExactExecutable", "VersionedProjectPolicyExecutable", "SharedExecutable",
      "ExternalOutcomeExecutable", "MetadataOnlyAudited", "ExcludedWithProof",
    ],
    forbidden_states: [
      "Pending", "CatalogOnly", "IdentityOnly", "NoOpHandler", "InheritedPolicy",
      "AssignedPendingResolution", "ReferenceFixtureOnly",
    ],
    completion_assertion: `The current production tree proves: ${row.deliverable}`,
  };
}

function releaseGates(verification) {
  return [
    gate("production-lowering", "G22-P1-B6", "All 80 Sora tables lower privately with exact joins and component identity."),
    gate("shared-capability", "G22-P2-B5", "Every required shared shape is implemented or assigned to an admitted audited handler."),
    gate("vertical-slice", "G22-P3-B6", "The frozen Ordinary slice reaches terminal state through a real battle and fresh replay."),
    gate("complete-run-matrix", "G22-P7-B6", `${verification.matrix_cases.length} seeded legal cases complete and fresh-replay production inputs.`),
    gate("performance", "G22-P8-B2", `${verification.performance_workloads.length} frozen workloads satisfy structural and measured budgets.`),
    gate("exact-once-release", "G22-P8-B4", "6,215 obligations, 669 programs, 25 fixtures, 25 gaps and 54 policies have terminal dispositions."),
    gate("native-clean-checkout", "G22-P8-B5", `${verification.native_ci_expectations.platforms.length} native platforms agree and clean-checkout acceptance passes.`),
  ];
}

function gate(id, ownerBatch, acceptance) {
  return { id, owner_batch: ownerBatch, acceptance };
}

function countBy(values, keyOf) {
  const result = {};
  for (const value of values) {
    const key = keyOf(value);
    result[key] = (result[key] ?? 0) + 1;
  }
  return result;
}

function sum(values, valueOf) {
  return values.reduce((total, value) => total + valueOf(value), 0);
}

function required(value, label) {
  assert(value !== undefined && value !== null, `${label} is missing`);
  return value;
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath))).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildVerificationScaffold());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate Goal 22 verification scaffolding`);
    console.log("Divergent Universe verification scaffold is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
