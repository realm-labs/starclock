#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/currency-wars-runtime-v1";
const goalPath = "docs/goals/21-currency-wars-runtime.md";
const outputPath = `${runtimeRoot}/verification-scaffold.json`;

export function buildVerificationScaffold() {
  const ledger = json(`${runtimeRoot}/batch-ledger.json`);
  const sources = json(`${runtimeRoot}/source-dispositions.json`).obligations;
  const mechanics = json(`${runtimeRoot}/mechanic-dispositions.json`).programs;
  const partitions = json(`${runtimeRoot}/mechanic-partitions.json`).partitions;
  const coverage = json(`${runtimeRoot}/coverage-and-release.json`);
  const capabilityInventory = json(`${runtimeRoot}/capability-inventory.json`);
  const deliverables = parseFixedDeliverables();
  const partitionByBatch = new Map(partitions.map((partition) => [partition.batch, partition]));
  const p0b6 = ledger.batches.find(({ batch }) => batch === "G21-P0-B6");
  assert(p0b6 !== undefined, "G21-P0-B6 is missing from the ledger");
  const later = ledger.batches.filter(({ ordinal }) => ordinal > p0b6.ordinal);
  assert(later.length === 88, "remaining Goal 21 batch denominator drift");
  assert(capabilityInventory.summary.mechanic_programs === 2_367,
    "capability inventory mechanic denominator drift");

  const batches = later.map((row) => {
    const partition = partitionByBatch.get(row.batch) ?? null;
    const counts = targetCounts(row.batch, sources, mechanics, ledger, coverage, partition);
    const packages = ownedPackages(row);
    const deliverable = partition === null
      ? deliverables.get(row.batch) : row.deliverable;
    assert(typeof deliverable === "string" && deliverable.length > 0,
      `${row.batch} has no deliverable`);
    return {
      batch: row.batch,
      ordinal: row.ordinal,
      phase: row.phase,
      kind: row.kind,
      owners: row.owner,
      prerequisites: row.prerequisites,
      deliverable,
      assigned_targets: counts,
      focused_gate: focusedGate(row, packages),
      terminal_evidence: terminalEvidence(row, counts, partition, deliverable),
      status: row.status,
    };
  });

  return {
    schema_revision: "starclock.currency-wars-verification-scaffold.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B6",
    status: "RuntimeReleaseComplete",
    input_digests: {
      [goalPath]: sha256(goalPath),
      [`${runtimeRoot}/batch-ledger.json`]: sha256(`${runtimeRoot}/batch-ledger.json`),
      [`${runtimeRoot}/source-dispositions.json`]: sha256(`${runtimeRoot}/source-dispositions.json`),
      [`${runtimeRoot}/mechanic-dispositions.json`]: sha256(`${runtimeRoot}/mechanic-dispositions.json`),
      [`${runtimeRoot}/mechanic-partitions.json`]: sha256(`${runtimeRoot}/mechanic-partitions.json`),
      [`${runtimeRoot}/coverage-and-release.json`]: sha256(`${runtimeRoot}/coverage-and-release.json`),
      [`${runtimeRoot}/capability-inventory.json`]: sha256(`${runtimeRoot}/capability-inventory.json`),
      [`${runtimeRoot}/shared-capability-audit.json`]: sha256(`${runtimeRoot}/shared-capability-audit.json`),
    },
    completion_rule: "A batch becomes Complete only after its focused commands pass and its assigned current-state dispositions, fixtures, policies and behavior artifacts satisfy terminal_evidence. Status text, IDs, catalog rows and no-op handlers are not evidence.",
    sequencing_rule: "Exactly one batch is InProgress. A batch may start only after every listed prerequisite is Complete in the regenerated ledger.",
    common_rejection_rule: "Rejected and stale commands, assembly and settlement preserve authoritative bytes, hashes, events, RNG counters and cache authority unless the frozen failure contract explicitly declares a deterministic terminal fault.",
    summary: {
      later_batches: batches.length,
      fixed_batches: batches.filter(({ kind }) => kind === "FixedBatch").length,
      generated_partitions: batches.filter(({ kind }) =>
        kind === "GeneratedMechanicPartition").length,
      statuses: countBy(batches, ({ status }) => status),
      phases: countBy(batches, ({ phase }) => String(phase)),
      assigned_source_catalog_obligations: sum(batches,
        ({ assigned_targets: value }) => value.source_catalog_obligations),
      assigned_source_execution_obligations: sum(batches,
        ({ assigned_targets: value }) => value.source_execution_obligations),
      assigned_mechanic_programs: sum(batches,
        ({ assigned_targets: value }) => value.mechanic_programs),
      assigned_fixture_families: sum(batches,
        ({ assigned_targets: value }) => value.semantic_fixture_families),
      assigned_policies: sum(batches,
        ({ assigned_targets: value }) => value.policies),
    },
    release_gates: releaseGates(coverage),
    batches,
  };
}

function parseFixedDeliverables() {
  const source = text(goalPath);
  const entries = [...source.matchAll(/^\| `(?<batch>G21-P\d-B\d+)` \| (?<deliverable>.+) \|$/gmu)];
  assert(entries.length === 51, "fixed deliverable count drift in Goal 21 document");
  const values = new Map(entries.map(({ groups }) => [groups.batch, groups.deliverable]));
  assert(values.size === entries.length, "duplicate fixed batch in Goal 21 document");
  return values;
}

function targetCounts(batch, sources, mechanics, ledger, coverage, partition) {
  const catalog = sources.filter(({ catalog_batch: owner }) => owner === batch).length;
  const execution = sources.filter(({ execution_batch: owner }) => owner === batch).length;
  const programs = mechanics.filter(({ execution_batch: owner }) => owner === batch).length;
  const fixtureFamilies = ledger.fixture_assignments
    .filter(({ owner_batch: owner }) => owner === batch).length;
  const policies = ledger.policy_assignments
    .filter(({ owner_batch: owner }) => owner === batch).length;
  if (partition !== null)
    assert(programs === partition.program_count,
      `${batch} partition/program assignment drift`);
  return {
    source_catalog_obligations: catalog,
    source_execution_obligations: execution,
    mechanic_programs: programs,
    semantic_fixture_families: fixtureFamilies,
    policies,
    complete_run_targets: batch === "G21-P7-B6"
      ? coverage.summary.complete_runs : 0,
    investment_execution_fixtures: batch === "G21-P5-B3"
      ? coverage.summary.investment_fixtures : 0,
    bond_level_execution_fixtures: batch === "G21-P4-B4"
      ? coverage.summary.bond_level_fixtures : 0,
    encounter_axis_fixtures: batch === "G21-P6-B1"
      ? coverage.summary.encounter_group_fixtures
        + coverage.summary.encounter_wave_fixtures
        + coverage.summary.enemy_slot_fixtures
        + coverage.summary.boss_pool_fixtures
      : 0,
  };
}

function ownedPackages(row) {
  const packages = row.owner
    .filter((owner) => owner.startsWith("starclock-"))
    .map((owner) => owner === "starclock-mode-currency-wars"
      ? owner : owner);
  if (row.phase >= 1 && !packages.includes("starclock-mode-currency-wars"))
    packages.push("starclock-mode-currency-wars");
  if (row.batch === "G21-P3-B6")
    packages.push("starclock-data");
  return [...new Set(packages)].sort();
}

function focusedGate(row, packages) {
  const commands = [
    "node tools/currency-wars-runtime/verify-dispositions.mjs",
    "node tools/currency-wars-runtime/verify-runtime-contract.mjs",
    "node tools/currency-wars-runtime/verify-coverage-and-release.mjs",
  ];
  if (row.phase >= 2)
    commands.push("node tools/currency-wars-runtime/verify-capability-inventory.mjs");
  if (row.phase >= 3 || row.batch === "G21-P2-B5")
    commands.push("node tools/currency-wars-runtime/verify-shared-capability-audit.mjs");
  if (row.phase === 1)
    commands.push(
      "node tools/currency-wars-reference/verify-pack.mjs",
      "node tools/currency-wars-reference/verify-sora-reader.mjs config/currency-wars-generated/config.sora",
    );
  commands.push("cargo fmt --all -- --check");
  for (const packageName of packages) {
    commands.push(`cargo clippy -p ${packageName} --all-targets -- -D warnings`);
    commands.push(`cargo test -p ${packageName}`);
  }
  if (row.batch === "G21-P7-B6")
    commands.push("cargo test --release -p starclock-ai --test currency_wars_matrix -- --ignored --exact generated_legal_matrix_completes_real_battles_and_fresh_replay");
  if (row.batch === "G21-P8-B1")
    commands.push("cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite");
  if (row.batch === "G21-P8-B2")
    commands.push("cargo run --release -p starclock-agent-api --example currency_wars_benchmark --features benchmark-harness");
  if (row.batch === "G21-P8-B3")
    commands.push("node tools/repository-check/verify-data.mjs");
  if (row.batch === "G21-P8-B4")
    commands.push(
      "node tools/currency-wars-runtime/verify-dispositions.mjs",
      "node tools/currency-wars-runtime/verify-coverage-and-release.mjs",
      "node tools/currency-wars-runtime/verify-verification-scaffold.mjs",
    );
  if (row.batch === "G21-P8-B5")
    commands.push(
      "cargo test --workspace",
      "node tools/currency-wars-runtime/run-clean-checkout.mjs",
    );
  return {
    packages,
    commands,
    required_assertions: assertionsFor(row),
    scope_rule: "Run these package-scoped gates after the narrowest changed test. Add workspace-wide execution only at a shared boundary or the explicit final release gate.",
  };
}

function assertionsFor(row) {
  const common = [
    "generated manifests regenerate byte-identically",
    "assigned identities remain exact-once and no denominator decreases",
    "rejected paths preserve state and RNG when the batch exposes a command boundary",
  ];
  const byPhase = {
    1: "private Sora lowering validates references, bounds and exact component identity without exposing generated rows",
    2: "shared capability is content-agnostic, deterministic and covered by boundary vectors",
    3: "Activity behavior advances only through offered GraphActivityCommand values and ordered operations",
    4: "build, equipment, Bond or override behavior changes the immutable contribution snapshot at its declared boundary",
    5: "cross-battle content executes lifecycle, eligibility, stacking and teardown rather than retaining IDs",
    6: "encounter or battle behavior produces real immutable combat inputs without mode-ID resolver branches",
    7: "adapter/replay behavior is surface-equivalent and reconstructs from fresh immutable inputs",
    8: "hardening/release evidence is fresh, deterministic and bound to the current tree",
  };
  return [...common, byPhase[row.phase]];
}

function terminalEvidence(row, counts, partition, deliverable) {
  const artifacts = [
    "current generated disposition/coverage manifests",
    "production-lowered behavior tests and nearest control/rejection",
    "updated current state/docs when runtime facts change",
  ];
  if (partition !== null)
    artifacts.push(
      `exact-once terminal dispositions for ${partition.program_count} assigned programs`,
      `production execution fixture for ${partition.fixture_family}`,
    );
  if (row.phase === 6)
    artifacts.push("real BattleSpec/BattleResult identity and replay evidence");
  if (row.phase === 7)
    artifacts.push("fresh component-addressed replay or adapter parity evidence");
  if (row.phase === 8)
    artifacts.push("explicit exhaustive, performance, audit or native release output owned by this batch");
  return {
    assigned_counts: counts,
    required_artifacts: artifacts,
    terminal_states: [
      "ExactExecutable", "VersionedProjectPolicyExecutable", "MetadataOnlyAudited",
      "ExcludedWithProof",
    ],
    forbidden_states: [
      "Pending", "Blocked", "CatalogOnly", "IdentityOnly", "NoOpHandler",
      "InheritedPolicy", "AssignedPendingResolution",
    ],
    completion_assertion: `The current tree proves: ${deliverable}`,
  };
}

function releaseGates(coverage) {
  return [
    gate("exact-once-runtime", "G21-P8-B4",
      "19,250 source obligations, 2,367 programs, 28 fixture families and 12 policies have terminal dispositions."),
    gate("complete-run-matrix", "G21-P7-B6",
      `${coverage.summary.complete_runs} production runs and every assigned axis fixture execute and fresh-replay.`),
    gate("surface-parity", "G21-P7-B5",
      "CLI, Agent API and MCP expose the same offered-command semantics and component-addressed replay."),
    gate("hardening", "G21-P8-B1",
      "Malformed, stale, RNG-isolation, overflow, recursion and replay-corruption suites pass."),
    gate("performance", "G21-P8-B2",
      "Eight frozen release workloads satisfy structural and measured host-class budgets."),
    gate("audits", "G21-P8-B3",
      "Dependency, architecture, unsafe, generated drift, provenance, handler and prior-release isolation audits pass."),
    gate("native-and-clean-checkout", "G21-P8-B5",
      "Three native runtime runners agree and fresh clean-checkout acceptance passes before completion registration."),
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

function json(relativePath) {
  return JSON.parse(text(relativePath));
}

function text(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function sha256(relativePath) {
  return crypto.createHash("sha256").update(text(relativePath)).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildVerificationScaffold());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate Goal 21 verification scaffolding`);
    console.log("Currency Wars verification scaffold is current.");
  } else {
    fs.writeFileSync(output, expected);
    console.log(`Wrote ${outputPath}.`);
  }
}
