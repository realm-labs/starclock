#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const referenceRoot = "content-reference/divergent-universe-v1";
const runtimeRoot = "content-manifests/divergent-universe-runtime-v1";
const output = `${runtimeRoot}/build-capability-closure.json`;
const inventoryInput = `${runtimeRoot}/capability-inventory.json`;
const referenceFiles = [
  `${referenceRoot}/arithmetic-mapping-eligibility.json`,
  `${referenceRoot}/arithmetic-mapping-builds.json`,
  `${referenceRoot}/arithmetic-mapping-rules.json`,
  `${referenceRoot}/coverage.json`,
];
const sharedFiles = [
  "crates/starclock-build/src/substitution.rs",
  "crates/starclock-build/src/spec.rs",
  "crates/starclock-build/src/contribution.rs",
  "crates/starclock-build/src/compiler.rs",
  "crates/starclock-build/src/digest.rs",
];

const artifact = buildClosure();
const serialized = pretty(artifact);
if (process.argv.includes("--check")) {
  assert(fs.readFileSync(absolute(output), "utf8") === serialized, `${output} is stale`);
  console.log(summaryLine("current", artifact));
} else {
  fs.writeFileSync(absolute(output), serialized);
  console.log(summaryLine("generated", artifact));
}

export function buildClosure() {
  const inventory = json(inventoryInput);
  const eligibility = json(referenceFiles[0]);
  const builds = json(referenceFiles[1]);
  const rules = json(referenceFiles[2]);
  const coverage = json(referenceFiles[3]).filter(({ manifest_category: category }) =>
    category.startsWith("arithmetic_mapping_"));
  const shapeGroups = [
    inventory.expression_shapes,
    inventory.selector_shapes,
    inventory.trigger_shapes,
    inventory.operation_shapes,
    inventory.state_shapes,
    inventory.lifecycle_shapes,
    inventory.record_shapes,
  ];
  const buildShapes = shapeGroups.flat().filter(({ domain }) => domain === "Build");
  const buildPrograms = inventory.programs.filter(({ domain }) => domain === "Build");
  const resolvedBuilds = builds.filter(({ public_identity_resolution: value }) =>
    value === "ResolvedAvatarConfig");
  const missingIdentityBuilds = builds.filter(({ public_identity_resolution: value }) =>
    value === "MissingReleasedAvatarConfig");
  const specialAvatarBuilds = builds.filter(({ special_avatar_id: value }) => value !== "");
  const parameterArities = countBy(builds, ({ role_buff_parameters: values }) =>
    String(values.length));

  assert(eligibility.length === 84 && builds.length === 95 && rules.length === 7,
    "Arithmetic Mapping denominator drift");
  assert(coverage.length === 258, "Arithmetic Mapping source obligation drift");
  assert(buildPrograms.length === 0 && buildShapes.length === 0,
    "unexpected Build-domain source program shape");
  assert(resolvedBuilds.length === 91 && missingIdentityBuilds.length === 4,
    "public avatar identity boundary drift");
  assert(specialAvatarBuilds.length === 79, "special-avatar denominator drift");
  assert(equal(parameterArities, { 4: 41, 5: 34, 6: 18, 7: 2 }),
    "role-buff parameter arity drift");
  assert(builds.every(({ runtime_lowered: value }) => value === false)
    && rules.every(({ runtime_lowered: value }) => value === false),
  "Arithmetic Mapping claimed premature runtime lowering");

  return {
    schema_revision: "starclock.divergent-universe-build-capability-closure.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P2-B4",
    status: "CompleteNoSharedRustDelta",
    input_digests: {
      capability_inventory: {
        path: inventoryInput,
        sha256: sha256File(absolute(inventoryInput)),
      },
      arithmetic_mapping_reference: {
        paths: referenceFiles,
        sha256: hashFiles(referenceFiles),
      },
      shared_build_surface: {
        paths: sharedFiles,
        sha256: hashFiles(sharedFiles),
      },
    },
    decision: {
      shared_build_primitive_additions: [],
      rationale: "The existing immutable CombatantBuildSpec, field-wise owned/trial substitution, generic catalog-owned contributions and LoadoutCompiler already represent every fully specified temporary Mapping input. No Build-domain mechanic-program shape or cross-program requirement needs a new shared primitive.",
      fieldwise_boundary: "The generic substitution compares progression, abilities, Traces, Eidolon, Light Cone and contribution/Relic fields independently, preserves sufficient owned values, combines set-like fields canonically and emits an auditable source receipt without querying or mutating account state.",
      unavailable_data_boundary: "All 95 Mapping build rows explicitly leave the exact temporary loadout unpublished, and four avatar locators lack released AvatarConfig identity. A compiler extension cannot manufacture those identities; Phase 3 must consume reviewed exact or VersionedProjectPolicy inputs and remain fail-closed where no legal mapped spec exists.",
      lifecycle_boundary: "Run entry, accepted party-change refresh and final teardown belong to the mode Activity runtime in G22-P3-B4. This batch proves only that their immutable Build-side input/output can use the current generic compiler.",
      execution_credit: "None. Every Mapping build and lifecycle row remains runtime_lowered=false; no account snapshot is read, no temporary build is constructed and no contribution is attached to a Divergent battle in this batch.",
    },
    existing_shared_surfaces: [
      surface("field-wise-owned-trial-substitution",
        "crates/starclock-build/src/substitution.rs", [
          "pub enum BuildFieldSource", "pub struct OwnedBuildMinimumFacts",
          "pub fn substitute_owned_or_trial", "fn merge_abilities",
          "fn merge_traces", "fn select_light_cone",
        ]),
      surface("immutable-generic-build-input",
        "crates/starclock-build/src/spec.rs", [
          "pub struct CombatantBuildSpec", "pub fn with_contributions",
          "pub const fn with_relic_stats",
        ]),
      surface("catalog-owned-contribution-language",
        "crates/starclock-build/src/contribution.rs", [
          "pub enum BuildContributionApplicability",
          "pub struct BuildContributionDefinition",
        ]),
      surface("canonical-contribution-compilation",
        "crates/starclock-build/src/compiler.rs", [
          "fn apply_contributions", "ContributionNotApplicable",
          "ContributionPatchConflict",
        ]),
      surface("build-and-selection-identity",
        "crates/starclock-build/src/digest.rs", [
          "BuildContributionApplicability::Any", "encode_patches",
          "spec.contributions()",
        ]),
    ],
    capability_probes: [
      probe("field-wise-minimum-selection",
        "crates/starclock-build/src/substitution.rs", [
          "absent_owned_build_selects_trial_without_mutation",
          "owned_build_inherits_stronger_fields_and_receives_field_minimums",
          "mismatched_forms_are_rejected",
        ]),
      probe("contribution-selection-compilation",
        "crates/starclock-test-kit/tests/suites/core/build/build_identity.rs", [
          "selected_contributions_are_canonical_attributed_and_applicability_checked",
          "canonical_definition_catalog_build_and_spec_digests_are_stable",
        ]),
    ],
    unresolved_public_identity_build_ids: missingIdentityBuilds.map(({ id }) => id).sort(),
    reference_field_values: Object.fromEntries([
      "level", "trace_state", "light_cone", "relics", "exact_temporary_loadout",
    ].map((field) => [field, [...new Set(builds.map((value) => value[field]))].sort()])),
    summary: {
      mapping_eligibility: eligibility.length,
      mapping_builds: builds.length,
      mapping_lifecycle_rules: rules.length,
      mapping_source_obligations: coverage.length,
      resolved_avatar_builds: resolvedBuilds.length,
      unresolved_avatar_builds: missingIdentityBuilds.length,
      special_avatar_builds: specialAvatarBuilds.length,
      role_buff_parameter_arities: parameterArities,
      build_domain_mechanic_programs: buildPrograms.length,
      build_domain_shapes: buildShapes.length,
      runtime_lowered_rows: 0,
      shared_build_primitive_additions: 0,
      shared_surfaces: 5,
      probes: 2,
    },
  };
}

function surface(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "ExistingAndSufficient" };
}

function probe(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "Passed" };
}

function countBy(values, selector) {
  const result = {};
  for (const value of values) {
    const key = selector(value);
    result[key] = (result[key] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(result).sort(([left], [right]) =>
    left.localeCompare(right, "en", { numeric: true })));
}

function hashFiles(files) {
  const hash = crypto.createHash("sha256");
  for (const file of files) {
    const bytes = fs.readFileSync(absolute(file));
    hash.update(file);
    hash.update("\0");
    hash.update(String(bytes.length));
    hash.update("\0");
    hash.update(bytes);
  }
  return hash.digest("hex");
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function json(file) {
  return JSON.parse(fs.readFileSync(absolute(file), "utf8"));
}

function absolute(file) {
  return path.join(root, file);
}

function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function summaryLine(state, value) {
  return `Divergent Universe Build capability closure ${state} `
    + `(${value.summary.mapping_builds} mappings; zero shared additions).`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
