#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/catalog/bundle-loading.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-bundle-loading.v1",
  "unsupported Goal 20 bundle-loading evidence revision");
assert(evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P1-B1"
  && evidence.result === "Pass",
"Goal 20 bundle-loading evidence identity drift");

const bundle = evidence.bundle;
assert(bundle.path === "config/swarm-disaster-generated/config.sora",
  "candidate bundle path drift");
assert(sha256(bundle.path) === bundle.sha256,
  "candidate bundle digest drift");
assert(bundle.sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362",
"candidate bundle identity drift");
assert(bundle.schema_fingerprint === "e1a4fc5af6b64ee9"
  && bundle.tables_loaded === 65
  && bundle.rows_loaded === 33_380,
"generated bundle denominator drift");

const generated = text("config/swarm-disaster-generated/rust/mod.rs");
assert(generated.includes(
  `pub const SCHEMA_FINGERPRINT: &str = "${bundle.schema_fingerprint}";`,
), "generated schema fingerprint drift");
assert(generated.includes("sora_map_with_capacity(65)"),
  "generated table capacity drift");
assert((generated.match(/tables\.insert\(/gu) ?? []).length === 65,
  "generated table loader closure drift");

const facade = text("crates/starclock-mode-universe/src/lib.rs");
assert(facade.includes(
  "#[path = \"../../../config/swarm-disaster-generated/rust/mod.rs\"]\n"
    + "mod swarm_disaster_generated;",
), "generated Swarm Disaster module is not private");
assert(facade.includes("pub mod swarm_disaster_catalog;"),
  "generated-type-free Swarm Disaster validation boundary is unavailable");
assert(!facade.includes("pub use swarm_disaster"),
  "Swarm Disaster public re-export was added");

const loader = text(
  "crates/starclock-mode-universe/src/swarm_disaster_catalog.rs",
);
for (const needle of [
  "pub fn validate_swarm_disaster_bundle(",
  'const EXPECTED_SCHEMA_FINGERPRINT: &str = "e1a4fc5af6b64ee9";',
  "const EXPECTED_TABLES: usize = 65;",
  "const EXPECTED_ROWS: usize = 33_380;",
])
  assert(loader.includes(needle), `bundle-loading boundary drift: ${needle}`);
assert(!/pub (?:struct|enum|type)[^\n]*(?:Sora|SwarmDisaster)/u.test(loader),
  "generated or new domain type escaped the public boundary");

const manifest = evidence.manifest;
assert(manifest.goal_id === "swarm-disaster-reference-v1"
  && manifest.profile_id === "swarm-disaster.profile.v1"
  && manifest.source_obligations === 6_963
  && manifest.mechanic_rules === 23
  && manifest.semantic_fixture_families === 23
  && manifest.policy_boundaries === 31,
"manifest denominator evidence drift");
assert(evidence.boundary.generated_module_visibility === "private"
  && evidence.boundary.public_generated_types === 0
  && evidence.boundary.new_public_domain_types === 0
  && evidence.boundary.public_reexports_added === 0
  && evidence.boundary.stable_private_rejection_families === 6
  && evidence.boundary.digest_checked_before_lowering === true
  && evidence.boundary.exact_table_closure_checked === true,
"bundle-loading boundary evidence drift");
assert(evidence.tests.integration_passed === 2
  && evidence.tests.unit_passed === 3
  && evidence.tests.rejection_classes_exercised.length === 6,
"bundle-loading test evidence drift");

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
])
  assert(captureGit([
    "status",
    "--porcelain=v1",
    "--untracked-files=all",
    "--",
    protectedRoot,
  ]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P1-B1` | `Complete` |"),
  "G20-P1-B1 is incomplete");
assert(!status.includes("| Active batch | `G20-P1-B1` |")
  && !status.includes("| Next unblocked batch | `G20-P1-B1` |"),
"Goal 20 regressed to G20-P1-B1");

console.log(
  "Goal 20 P1-B1 verified (65 private Sora tables; 33,380 rows; "
    + "6,963 obligations; six rejection families; zero generated public types).",
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
