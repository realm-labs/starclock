import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(process.argv.slice(2).every((argument) => argument === "--write"), "usage: generate-bootstrap-policy.mjs [--write]");
const manifest = path.join(root, "tools/workbook-bootstrap/Cargo.toml");
const lock = path.join(root, "tools/workbook-bootstrap/Cargo.lock");
const metadata = JSON.parse(capture("cargo", ["metadata", "--manifest-path", manifest, "--locked", "--format-version", "1"]));
const rootPackage = metadata.packages.find((entry) => entry.name === "starclock-workbook-bootstrap");
assert(rootPackage, "bootstrap package is absent from Cargo metadata");
const direct = rootPackage.dependencies.map((entry) => entry.name).sort();
assert(JSON.stringify(direct) === JSON.stringify(["calamine", "rust_xlsxwriter"]), "bootstrap direct dependency set differs");
const checksums = lockChecksums(lock);
const packages = metadata.packages.filter((entry) => entry.source?.startsWith("registry+")).map((entry) => ({
  name: entry.name,
  version: entry.version,
  relationship: direct.includes(entry.name) ? "Direct" : "Transitive",
  license: entry.license,
  checksum: checksums.get(`${entry.name}@${entry.version}`),
})).sort((left, right) => left.name.localeCompare(right.name));
assert(packages.every((entry) => entry.license && entry.checksum), "bootstrap dependency license/checksum inventory is incomplete");
const policy = {
  schema_revision: "starclock.workbook-bootstrap-dependencies.v1",
  reviewed_on: "2026-07-17",
  scope: "Standalone deterministic workbook authoring tool; no package enters a production runtime crate.",
  lockfile_sha256: sha256(lock),
  direct_packages: [
    { name: "calamine", version: "0.35.0", default_features: false, features: ["dates"] },
    { name: "rust_xlsxwriter", version: "0.96.0", default_features: false, features: ["constant_memory"] },
  ],
  source_url_template: "https://crates.io/crates/{name}/{version}",
  purpose: "Read Sora-generated schema projections and materialize deterministic bootstrap .xlsx cell values without editing live designer workbooks.",
  deterministic_impact: "Tooling only. Exact cell projections are validated by Sora and exported bundle/debug bytes; workbook ZIP bytes are not runtime or replay identities.",
  compile_cost: "Fresh Windows x86_64 MSVC cargo check observed at 6,590 ms under Rust/Cargo 1.97.0; not a performance budget.",
  rejected_alternatives: [
    "Handwritten XLSX ZIP/XML output would duplicate schema and archive logic.",
    "Editing designer workbooks in place would violate the no-overwrite authoring contract.",
    "JSON-direct runtime loading would bypass Excel/Sora authority."
  ],
  packages,
};
const output = `${JSON.stringify(policy, null, 2)}\n`;
const destination = path.join(root, "policy/workbook-bootstrap-dependencies.json");
if (write) {
  fs.writeFileSync(destination, output);
  console.log(`Wrote workbook-bootstrap dependency policy (${packages.length} packages).`);
} else {
  assert(fs.existsSync(destination) && fs.readFileSync(destination, "utf8") === output, "workbook-bootstrap dependency policy drifted");
  console.log(`Workbook-bootstrap dependency policy verified (${packages.length} packages).`);
}

function lockChecksums(file) {
  return new Map(fs.readFileSync(file, "utf8").split("[[package]]").slice(1).map((block) => {
    const name = block.match(/^name = "([^"]+)"/m)?.[1];
    const version = block.match(/^version = "([^"]+)"/m)?.[1];
    const checksum = block.match(/^checksum = "([^"]+)"/m)?.[1];
    return checksum ? [`${name}@${version}`, checksum] : undefined;
  }).filter(Boolean));
}
function capture(command, arguments_) { const result = spawnSync(command, arguments_, { cwd: root, encoding: "utf8", env: { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} failed: ${result.stderr}`); return result.stdout; }
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
