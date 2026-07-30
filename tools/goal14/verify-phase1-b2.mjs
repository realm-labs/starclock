#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/catalog/identity-composition.json",
);
const contract = json("policy/goal14-runtime-contract.json");

assert(evidence.schema_revision
  === "starclock.gold-and-gears-identity-composition.v1"
  && evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P1-B2"
  && evidence.result === "Pass",
"Goal 14 identity evidence drift");
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
assert(catalog.revision === "gold-and-gears-v4.4-runtime-v1"
  && catalog.profile_revision === "gold-gears-profile-v1"
  && catalog.content_revision === "gold-gears-content-v1"
  && catalog.shared_content_revision === "universe-shared-content-v1",
"catalog revision identity drift");
assert(catalog.gold_content_sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b"
  && catalog.shared_content_sha256
  === "5e5234ee3977f794ae9b1b833372f51c38408c205105c464f11827e9e9ae6a75"
  && catalog.gold_content_sha256 !== catalog.shared_content_sha256,
"Gold/shared component identity drift");
for (const digest of [
  catalog.profile_identity_sha256,
  catalog.composition_sha256,
  evidence.activity_registry.sha256,
  evidence.components.fixture_root_sha256,
])
  assert(/^[0-9a-f]{64}$/u.test(digest), `invalid identity digest: ${digest}`);
assert(new Set([
  catalog.gold_content_sha256,
  catalog.shared_content_sha256,
  catalog.profile_identity_sha256,
  catalog.composition_sha256,
  evidence.activity_registry.sha256,
  evidence.components.fixture_root_sha256,
]).size === 6, "identity digest domains collapsed");

const handler = text(
  "crates/starclock-mode-universe/src/gold_gears_handler_bundle.rs",
);
assert(handler.includes('"starclock.mode.gold-and-gears"')
  && handler.includes('"gold-and-gears-activity-handlers-v1"')
  && handler.includes('vec!["starclock.activity.core"]')
  && handler.includes("Vec::new()"),
"Gold Activity registry bundle drift");
assert(!handler.includes("standard-universe"),
  "Gold registry imports the Standard profile bundle");

const identity = text(
  "crates/starclock-mode-universe/src/gold_gears_identity.rs",
);
for (const needle of [
  "pub struct GoldAndGearsCatalogIdentity",
  `"${catalog.revision}"`,
  `"${catalog.profile_revision}"`,
  `"${catalog.content_revision}"`,
  `"${catalog.shared_content_revision}"`,
  "starclock.gold-and-gears.profile-identity.v1",
  "starclock.gold-and-gears.catalog-composition.v1",
])
  assert(identity.includes(needle), `catalog identity source drift: ${needle}`);

const components = text(
  "crates/starclock-mode-universe/src/gold_gears_components.rs",
);
for (const frozen of contract.component_contract.ordered_components) {
  if (frozen.id !== "selected by caller")
    assert(components.includes(`"${frozen.id}"`),
      `missing component identity ${frozen.id}`);
}
assert((components.match(/ConfigurationComponentKind::/gu) ?? []).length === 10,
  "component source does not contain exactly ten typed identities");

const compatibility = evidence.compatibility;
assert(blobAt(
  "b26f7d8dbafe5a83dd92ff8cb5a9c14466e69e8b",
  "crates/starclock-mode-universe/src/handler_bundle.rs",
) === compatibility.standard_handler_bundle_blob,
"Goal-start Standard handler bundle identity drift");
assert(blobAt(
  "HEAD",
  "crates/starclock-mode-universe/src/handler_bundle.rs",
) === compatibility.standard_handler_bundle_blob,
"current Standard handler bundle was changed");
assert(blobAt(
  "b26f7d8dbafe5a83dd92ff8cb5a9c14466e69e8b",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
) === compatibility.standard_component_composer_blob,
"Goal-start Standard component composer identity drift");
assert(blobAt(
  "HEAD",
  "crates/starclock-mode-universe/src/universe_replay_v2.rs",
) === compatibility.standard_component_composer_blob,
"current Standard component composer was changed");
assert(compatibility.standard_sources_unchanged_from_goal_start === true
  && compatibility.gold_registry_excludes_standard_bundle === true
  && compatibility.controller_digest_changes_component_root === true,
"compatibility evidence drift");
assert(evidence.components.gold_mode_content_components === 1
  && evidence.components.shared_mode_content_components === 1
  && evidence.components.unrelated_mode_components === 0
  && evidence.tests.integration_passed === 3
  && evidence.tests.golden_digest_vectors === 4,
"component/test denominator drift");

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| `G14-P1-B2` | `Complete` |"),
  "G14-P1-B2 is incomplete");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G14-P1-B3` |"),
"Goal 14 did not advance to G14-P1-B3");

console.log(
  "Goal 14 P1-B2 verified (10 components; two immutable Activity bundles; " +
  "zero admitted handlers; Gold/shared roots separate; Standard roots unchanged).",
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
