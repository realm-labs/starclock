#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/gold-and-gears-runtime-v1/catalog/bundle-loading.json",
);

assert(evidence.schema_revision === "starclock.gold-and-gears-bundle-loading.v1",
  "unsupported Goal 14 bundle-loading evidence revision");
assert(evidence.goal_id === "gold-and-gears-runtime-v1"
  && evidence.batch === "G14-P1-B1"
  && evidence.result === "Pass",
"Goal 14 bundle-loading evidence identity drift");

const bundle = evidence.bundle;
assert(bundle.path === "config/gold-and-gears-generated/config.sora",
  "candidate bundle path drift");
assert(sha256(bundle.path) === bundle.sha256,
  "candidate bundle digest drift");
assert(bundle.sha256
  === "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b",
"candidate bundle identity drift");
assert(bundle.schema_fingerprint === "5d5e76d3dbe1afca"
  && bundle.tables_loaded === 52
  && bundle.rows_loaded === 29_140,
"generated bundle denominator drift");

const generated = text("config/gold-and-gears-generated/rust/mod.rs");
assert(generated.includes(
  `pub const SCHEMA_FINGERPRINT: &str = "${bundle.schema_fingerprint}";`,
), "generated schema fingerprint drift");
assert(generated.includes("sora_map_with_capacity(52)"),
  "generated table capacity drift");
assert((generated.match(/tables\.insert\(/gu) ?? []).length === 52,
  "generated table loader closure drift");

const facade = text("crates/starclock-mode-universe/src/lib.rs");
assert(facade.includes(
  "#[path = \"../../../config/gold-and-gears-generated/rust/mod.rs\"]\n"
    + "mod gold_gears_generated;",
), "generated Gold and Gears module is not private");
assert(facade.includes("pub mod gold_gears_catalog;"),
  "generated-free Gold and Gears catalog boundary is unavailable");

const loader = text(
  "crates/starclock-mode-universe/src/gold_gears_catalog.rs",
);
for (const needle of [
  "pub struct GoldAndGearsBundleSummary",
  "pub enum GoldAndGearsBundleLoadError",
  "pub fn validate_gold_and_gears_bundle(",
  'const EXPECTED_SCHEMA_FINGERPRINT: &str = "5d5e76d3dbe1afca";',
  "const EXPECTED_TABLES: usize = 52;",
  "const EXPECTED_ROWS: usize = 29_140;",
])
  assert(loader.includes(needle), `bundle-loading boundary drift: ${needle}`);
assert(!/pub (?:struct|enum|type|fn)[^\n]*(?:Sora|GoldGearsManifest)/u.test(loader),
  "generated Sora type escaped the public boundary");

const manifest = evidence.manifest;
assert(manifest.goal_id === "gold-and-gears-reference-v1"
  && manifest.profile_id === "gold-gears.profile.v1"
  && manifest.source_obligations === 7_913
  && manifest.mechanic_rules === 1_224
  && manifest.semantic_fixture_families === 18
  && manifest.policy_boundaries === 16,
"manifest denominator evidence drift");
assert(evidence.boundary.generated_module_visibility === "private"
  && evidence.boundary.public_generated_types === 0
  && evidence.boundary.stable_error_families === 6
  && evidence.boundary.digest_checked_before_lowering === true
  && evidence.boundary.exact_table_closure_checked === true,
"bundle-loading boundary evidence drift");
assert(evidence.tests.integration_passed === 2
  && evidence.tests.unit_passed === 3
  && evidence.tests.rejection_classes_exercised.length === 6,
"bundle-loading test evidence drift");

for (const protectedRoot of [
  "evidence/gold-and-gears-reference-v1",
  "content-manifests/gold-and-gears-v1",
  "content-reference/gold-and-gears-v1",
  "config/gold-and-gears/data",
  "config/gold-and-gears-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(status.includes("| `G14-P1-B1` | `Complete` |"),
  "G14-P1-B1 is incomplete");
assert(!status.includes("| Active batch | `G14-P1-B1` |")
  && !status.includes("| Next unblocked batch | `G14-P1-B1` |"),
"Goal 14 regressed to G14-P1-B1");

console.log(
  "Goal 14 P1-B1 verified (52 private Sora tables; 29,140 rows; " +
  "7,913 obligations; six stable rejection families; zero generated public types).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
