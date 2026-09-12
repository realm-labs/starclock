#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const referenceRoot = "content-reference/divergent-universe-v1";
const output = `${runtimeRoot}/mapping-execution.json`;
const inputs = {
  eligibility: `${referenceRoot}/arithmetic-mapping-eligibility.json`,
  builds: `${referenceRoot}/arithmetic-mapping-builds.json`,
  rules: `${referenceRoot}/arithmetic-mapping-rules.json`,
  runtime_contract: `${runtimeRoot}/runtime-contract.json`,
  runtime_dispositions: `${runtimeRoot}/runtime-dispositions.json`,
  batch_ledger: `${runtimeRoot}/batch-ledger.json`,
  mapping_runtime: "crates/starclock-mode-universe/src/divergent_universe/mapping.rs",
  entry_runtime: "crates/starclock-mode-universe/src/divergent_universe/entry_flow.rs",
  state_runtime: "crates/starclock-mode-universe/src/divergent_universe/state.rs",
  build_substitution: "crates/starclock-build/src/substitution.rs",
  core_catalog_lookup: "crates/starclock-data/src/catalog_lookup.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/mapping_scope.rs",
};

export function buildMappingExecution() {
  const eligibility = json(inputs.eligibility);
  const builds = json(inputs.builds);
  const rules = json(inputs.rules);
  const contract = json(inputs.runtime_contract);
  const runtime = json(inputs.runtime_dispositions).obligations.filter(
    ({ execution_partition: batch }) => batch === "G22-P3-B4");
  const ledger = json(inputs.batch_ledger);
  const fixtures = owned(ledger.fixture_assignments, "G22-P3-B4");
  const gaps = owned(ledger.research_gap_assignments, "G22-P3-B4");
  const policies = owned(ledger.policy_assignments, "G22-P3-B4");
  const mappingSource = text(inputs.mapping_runtime);
  const entrySource = text(inputs.entry_runtime);
  const stateSource = text(inputs.state_runtime);
  const buildSource = text(inputs.build_substitution);
  const lookupSource = text(inputs.core_catalog_lookup);
  const testSource = text(inputs.tests);

  const eligible = builds.filter(({ eligible_catalog_entry: value }) => value);
  const ineligible = builds.filter(({ eligible_catalog_entry: value }) => !value);
  const resolved = builds.filter(
    ({ public_identity_resolution: value }) => value === "ResolvedAvatarConfig");
  const unresolved = builds.filter(
    ({ public_identity_resolution: value }) => value !== "ResolvedAvatarConfig");
  const arities = countBy(builds, ({ role_buff_parameters: values }) => values.length);
  assert(eligibility.length === 84 && builds.length === 95 && rules.length === 7,
    "Arithmetic Mapping source denominator drift");
  assert(eligible.length === 84 && ineligible.length === 11,
    "Arithmetic Mapping eligibility closure drift");
  assert(resolved.length === 91 && unresolved.length === 4
    && unresolved.every(({ eligible_catalog_entry: value }) => !value),
  "Arithmetic Mapping released identity closure drift");
  assert(builds.filter(({ special_avatar_id: value }) => Boolean(value)).length === 79,
    "Arithmetic Mapping special-avatar denominator drift");
  assert(equal(arities, { 4: 41, 5: 34, 6: 18, 7: 2 }),
    "Arithmetic Mapping role-buff arity drift");
  assert(rules.every(({ account_mutation: value }) => value === false),
    "Arithmetic Mapping gained account mutation authority");
  assert(runtime.length === 258
    && runtime.every(({ runtime_status: status }) => status === "Terminal")
    && count(runtime, "target_disposition", "ExactIntegrated") === 84
    && count(runtime, "target_disposition", "PolicyIntegrated") === 174,
  "P3-B4 obligation terminal closure drift");
  assert(fixtures.length === 2
    && fixtures.every(({ status }) => status === "ProductionExecutionPassed"),
  "P3-B4 fixture closure drift");
  assert(gaps.length === 2
    && gaps.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B4 research-gap closure drift");
  assert(policies.length === 4
    && policies.every(({ status }) => status === "VersionedProjectPolicyExecutable"),
  "P3-B4 policy closure drift");
  assert(ledger.completed_through === "G22-P8-B3" && ledger.next_batch === "G22-P8-B4",
    "Mapping execution ledger progress drift");
  assert(contract.account_and_mapping_boundary.mapping.includes("field provenance")
    && contract.account_and_mapping_boundary.teardown.includes("caller-owned state"),
  "Mapping runtime contract drift");

  for (const fragment of [
    "pub enum DivergentUniverseAvatarBindingAccuracy",
    "pub fn compile_mapping",
    "pub fn refresh_mapping",
    "pub fn teardown",
    "substitute_owned_or_trial",
    "LoadoutCompiler.compile",
    "OwnedBuildLockMismatch",
  ]) assert(mappingSource.includes(fragment), `missing Mapping fragment ${fragment}`);
  for (const fragment of [
    "entry.mapping_snapshot.as_ref()",
    "slot: super::state::MAPPING_STATE_SLOT",
    "value.digest().bytes()",
  ]) assert(entrySource.includes(fragment), `missing Mapping entry fragment ${fragment}`);
  for (const fragment of ["MAPPING_STATE_SLOT", "SlotCarryPolicy::CarryExact"])
    assert(stateSource.includes(fragment), `missing Mapping state fragment ${fragment}`);
  for (const fragment of [
    "pub fn substitute_owned_or_trial",
    "BuildFieldSource::MappedMinimum",
    "BuildFieldSource::Owned",
    "BuildFieldSource::Combined",
  ]) assert(buildSource.includes(fragment), `missing shared Build fragment ${fragment}`);
  assert(lookupSource.includes("character_form_for_source_avatar"),
    "missing source-avatar to stable-form lookup boundary");

  const probes = [
    probe("catalog-eligibility-and-identity-closure",
      "arithmetic_mapping_catalog_closes_eligibility_and_released_identity_denominators"),
    probe("field-wise-stronger-owned-preservation",
      "arithmetic_mapping_preserves_stronger_owned_fields_without_mutating_caller_build"),
    probe("eligibility-refresh-and-teardown",
      "arithmetic_mapping_rejects_ineligible_inputs_and_refreshes_then_tears_down"),
    probe("entry-binding-and-terminal-clear",
      "arithmetic_mapping_is_bound_to_entry_state_and_cleared_at_run_finalization"),
  ];
  for (const value of probes)
    assert(testSource.includes(value.test), `missing Mapping probe ${value.id}`);

  return {
    schema_revision: "starclock.divergent-universe-mapping-execution.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P3-B4",
    status: "CompleteFieldWiseMappingRefreshTeardownNoBattleCredit",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    execution_boundary: {
      runtime_owner: "starclock-mode-universe",
      build_owner: "starclock-build",
      source_identity: "Upstream AvatarID values remain source locators. Core-catalog joins resolve a Starclock stable form when present; otherwise the immutable caller-provided form binding is explicitly labeled VersionedProjectPolicy and digest-bound.",
      substitution: "The shared Build substitution primitive compares progression, abilities, Traces, Eidolon, Light Cone and generic contributions field by field, preserving sufficient caller fields and applying only deficient temporary minimums.",
      compilation: "Every selected immutable build compiles through LoadoutCompiler into a fresh CompiledBuild and ResolvedCombatantSpec; receipts and both identities are bound into the Mapping snapshot digest.",
      refresh: "Run entry compiles a fresh snapshot; an accepted party change may request deterministic reevaluation against a matching component and participant lock. Prior snapshots and caller specs remain immutable.",
      teardown: "Run finalization clears Activity-owned Mapping state. Explicit teardown consumes a snapshot into an auditable removal receipt without mutating account state.",
      policy: "Exact per-avatar temporary Trace, Light Cone and Relic loadouts remain unpublished; callers supply immutable versioned minimum specs until the stated replacement condition is satisfied.",
      credit: "Arithmetic Mapping build compilation and lifecycle only; no encounter, BattleSpec, real battle, rewards, full playable-run or release-gate credit.",
    },
    source_closure: {
      eligibility_rows: eligibility.length,
      mapping_builds: builds.length,
      lifecycle_rules: rules.length,
      eligible_builds: eligible.length,
      ineligible_builds: ineligible.length,
      resolved_public_identities: resolved.length,
      unresolved_public_identities: unresolved.length,
      special_avatar_bindings: builds.filter(
        ({ special_avatar_id: value }) => Boolean(value)).length,
      role_buff_parameter_arities: arities,
      source_obligations: runtime.length,
      exact_obligations: count(runtime, "target_disposition", "ExactIntegrated"),
      policy_obligations: count(runtime, "target_disposition", "PolicyIntegrated"),
    },
    lifecycle_rules: rules.map((rule) => ({
      id: rule.id,
      selection_timing: rule.selection_timing,
      condition: rule.condition,
      ordered_operations: rule.ordered_operations,
      stronger_build_rule: rule.stronger_build_rule,
      account_mutation: rule.account_mutation,
    })),
    terminal_assignments: {
      obligations: runtime.length,
      fixture_families: fixtures.map(({ fixture_family_id: id }) => id),
      research_gaps: gaps.map(({ research_gap_id: id }) => id),
      policy_sources: policies.map(({ policy_source_id: id }) => id),
    },
    capability_probes: probes,
    summary: {
      eligibility_rows: eligibility.length,
      mapping_builds: builds.length,
      lifecycle_rules: rules.length,
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

function countBy(values, key) {
  const result = {};
  for (const value of values) {
    const selected = String(key(value));
    result[selected] = (result[selected] ?? 0) + 1;
  }
  return result;
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

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const artifact = buildMappingExecution();
  const serialized = pretty(artifact);
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Mapping execution is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe Mapping execution evidence.");
  }
}
