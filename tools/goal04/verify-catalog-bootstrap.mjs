import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-catalog-bootstrap.json");
assert(policy.schema_revision === "starclock.goal04-catalog-bootstrap.v1", "unexpected catalog-bootstrap revision");
assert(sha256("config/generated/config.sora") === policy.digests.core_bundle, "core bundle changed");
assert(sha256("config/universe-generated/config.sora") === policy.digests.universe_bundle, "Universe bundle changed");

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: root, encoding: "utf8" }));
const universe = metadata.packages.find((entry) => entry.name === policy.crate);
assert(universe, "starclock-mode-universe is not a workspace crate");
const local = universe.dependencies.filter((dependency) => dependency.source === null).map((dependency) => dependency.name).sort();
const external = universe.dependencies.filter((dependency) => dependency.source !== null).map((dependency) => dependency.name).sort();
assert(equal(local, policy.local_dependencies), "Universe local dependency boundary differs");
assert(equal(external, policy.external_dependencies), "Universe external dependency boundary differs");
const combat = metadata.packages.find((entry) => entry.name === "starclock-combat");
assert(!combat.dependencies.some((dependency) => dependency.name === policy.crate), "combat has a reverse Universe dependency");

const facade = text("crates/starclock-mode-universe/src/lib.rs");
assert(facade.includes("mod generated;") && !facade.includes("pub mod generated"), "generated Sora module is not private");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
for (const marker of [
  `UNIVERSE_CATALOG_REVISION: &str = "${policy.revisions.catalog}"`,
  `STANDARD_UNIVERSE_PROFILE_REVISION: &str = "${policy.revisions.profile}"`,
  `ACTIVITY_CONFIGURATION_REVISION: &str = "${policy.revisions.configuration}"`,
  "pub struct UniverseCatalogIdentity",
  "pub struct UniverseCatalog",
  "pub fn load(",
  "UniverseCatalogLoadErrorKind::UniverseBundleDigest",
  "UniverseCatalogLoadErrorKind::UniverseRevision",
  "UniverseCatalogLoadErrorKind::CoreCompatibility"
]) assert(catalog.includes(marker), `catalog bootstrap source omits ${marker}`);
for (const digest of [policy.digests.composed_configuration, policy.digests.universe_profile])
  assert(hexBytesAppear(catalog, digest), `catalog golden omits ${digest}`);
assert((catalog.match(/^    #\[test\]$/gm) ?? []).length === policy.focused_tests, "focused test count differs");

const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B1` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B1 is not active or complete");

const evidence = {
  schema_revision: "starclock.goal04-catalog-bootstrap-evidence.v1",
  goal_id: policy.goal_id,
  result: "private-bundle-loaded",
  revisions: policy.revisions,
  digests: policy.digests,
  summary: policy.summary,
  boundaries: {
    generated_rows_public: false,
    core_reverse_dependency: false,
    runtime_json_or_xlsx: false,
    wrong_bundle_revision_digest_rejected: true
  },
  dependencies: { local, external, new_registry_packages: policy.new_registry_packages },
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/bootstrap.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "catalog bootstrap evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "catalog bootstrap evidence is stale; run with --bless");
}
console.log(`Goal 04 catalog bootstrap verified (${policy.summary.private_sora_tables} private tables, ${policy.summary.content_records} content rows, composed ${policy.digests.composed_configuration.slice(0, 12)}).`);

function hexBytesAppear(source, hex) { return [...Buffer.from(hex, "hex")].every((byte) => source.includes(`0x${byte.toString(16).padStart(2, "0")}`)); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
