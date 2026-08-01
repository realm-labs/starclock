#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/catalog/identity-composition.json",
);
const contract = json("policy/goal20-runtime-contract.json");

assert(evidence.schema_revision
  === "starclock.swarm-disaster-identity-composition.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P1-B2"
  && evidence.result === "Pass",
"Goal 20 identity evidence drift");
assert(contract.component_contract.component_set_revision
  === evidence.components.revision,
"component-set revision drift");
assert(contract.component_contract.ordered_components.length
  === evidence.components.count,
"component closure drift");
assert(contract.registry_contract.activity.p0_admitted_handlers
  === evidence.activity_registry.admitted_handlers,
"unreviewed Activity handler admission");
assert(contract.registry_contract.combat.p0_admitted_handlers === 0,
  "unreviewed combat handler admission");

const catalog = evidence.catalog;
assert(catalog.revision === "swarm-disaster-v4.4-runtime-v1"
  && catalog.profile_revision === "swarm-disaster-profile-v1"
  && catalog.content_revision === "swarm-disaster-content-v1"
  && catalog.shared_content_revision === "universe-shared-content-v1",
"catalog revision identity drift");
assert(catalog.swarm_content_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && catalog.shared_content_sha256
  === "5e5234ee3977f794ae9b1b833372f51c38408c205105c464f11827e9e9ae6a75"
  && catalog.swarm_content_sha256 !== catalog.shared_content_sha256,
"Swarm/shared component identity drift");
for (const digest of [
  catalog.profile_identity_sha256,
  catalog.composition_sha256,
  evidence.activity_registry.sha256,
  evidence.components.fixture_root_sha256,
])
  assert(/^[0-9a-f]{64}$/u.test(digest), `invalid identity digest: ${digest}`);
assert(new Set([
  catalog.swarm_content_sha256,
  catalog.shared_content_sha256,
  catalog.profile_identity_sha256,
  catalog.composition_sha256,
  evidence.activity_registry.sha256,
  evidence.components.fixture_root_sha256,
]).size === 6, "identity digest domains collapsed");

const handler = text(
  "crates/starclock-mode-universe/src/swarm_disaster_handler_bundle.rs",
);
assert(handler.includes('"starclock.mode.swarm-disaster"')
  && handler.includes('"swarm-disaster-activity-handlers-v1"')
  && handler.includes('vec!["starclock.activity.core"]')
  && handler.includes("Vec::new()"),
"Swarm Activity registry bundle drift");
assert(!handler.includes("standard-universe")
  && !handler.includes("gold-and-gears"),
"Swarm registry imports an unrelated mode bundle");

const identity = text(
  "crates/starclock-mode-universe/src/swarm_disaster_identity.rs",
);
for (const needle of [
  "struct SwarmDisasterCatalogIdentity",
  `"${catalog.revision}"`,
  `"${catalog.profile_revision}"`,
  `"${catalog.content_revision}"`,
  `"${catalog.shared_content_revision}"`,
  "starclock.swarm-disaster.profile-identity.v1",
  "starclock.swarm-disaster.catalog-composition.v1",
])
  assert(identity.includes(needle), `catalog identity source drift: ${needle}`);
assert(!identity.includes("pub struct SwarmDisasterCatalogIdentity"),
  "catalog identity escaped the private lowering boundary");

const components = text(
  "crates/starclock-mode-universe/src/swarm_disaster_components.rs",
);
for (const frozen of contract.component_contract.ordered_components) {
  if (frozen.id !== "selected by caller")
    assert(components.includes(`"${frozen.id}"`),
      `missing component identity ${frozen.id}`);
}
assert((components.match(/ConfigurationComponentKind::/gu) ?? []).length === 10,
  "component source does not contain exactly ten typed identities");

const compatibility = evidence.compatibility;
for (const [relative, expected] of [
  ["crates/starclock-mode-universe/src/handler_bundle.rs",
    compatibility.standard_handler_bundle_blob],
  ["crates/starclock-mode-universe/src/universe_replay_v2.rs",
    compatibility.standard_component_composer_blob],
  ["crates/starclock-mode-universe/src/gold_gears_components.rs",
    compatibility.gold_components_blob],
  ["crates/starclock-mode-universe/src/gold_gears_identity.rs",
    compatibility.gold_identity_blob],
  ["crates/starclock-mode-universe/src/gold_gears_handler_bundle.rs",
    compatibility.gold_handler_bundle_blob],
]) {
  assert(blobAt(compatibility.baseline_commit, relative) === expected,
    `P1-B1 baseline blob drift: ${relative}`);
  assert(blobAt("HEAD", relative) === expected,
    `current Standard/Gold source changed: ${relative}`);
}
assert(compatibility.gold_fixture_root_sha256
  === "93c50f430cf8950bb40fc180d355adbc6719d8f56ae434da4f8aac3068509b18"
  && compatibility.standard_and_gold_sources_unchanged === true
  && compatibility.swarm_registry_excludes_other_mode_bundles === true
  && compatibility.controller_digest_changes_component_root === true,
"compatibility evidence drift");
assert(evidence.components.swarm_mode_content_components === 1
  && evidence.components.shared_mode_content_components === 1
  && evidence.components.unrelated_mode_components === 0
  && evidence.boundary.new_public_domain_types === 0
  && evidence.boundary.public_reexports_added === 0
  && evidence.boundary.private_catalog_identity === true
  && evidence.boundary.component_composer_returns_existing_generic_type === true
  && evidence.tests.new_integration_passed === 2
  && evidence.tests.new_unit_passed === 2
  && evidence.tests.golden_digest_vectors === 5,
"component/test denominator drift");

const facade = text("crates/starclock-mode-universe/src/lib.rs");
assert(!facade.includes("pub use swarm_disaster"),
  "Swarm Disaster public re-export was added");

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P1-B2` | `Complete` |"),
  "G20-P1-B2 is incomplete");
assert(!status.includes("| Active batch | `G20-P1-B2` |")
  && !status.includes("| Next unblocked batch | `G20-P1-B2` |"),
"Goal 20 regressed to G20-P1-B2");

console.log(
  "Goal 20 P1-B2 verified (10 components; two immutable Activity bundles; "
    + "zero admitted handlers; Swarm/shared roots separate; Standard/Gold unchanged).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function blobAt(revision, relative) {
  return execFileSync("git", ["rev-parse", `${revision}:${relative}`], {
    cwd: root,
    encoding: "utf8",
  }).trim();
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
