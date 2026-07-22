import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(process.argv.slice(2).every((argument) => argument === "--write"), "usage: generate-policy.mjs [--write]");
const manifest = path.join(root, "tools/universe-bundle-loader/Cargo.toml");
const lock = path.join(root, "tools/universe-bundle-loader/Cargo.lock");
const metadata = JSON.parse(capture("cargo", ["metadata", "--manifest-path", manifest, "--locked", "--format-version", "1"]));
const package_ = metadata.packages.find((entry) => entry.name === "starclock-universe-bundle-loader");
assert(package_, "Universe bundle-loader package is absent from Cargo metadata");
const direct = package_.dependencies.map((entry) => entry.name).sort();
assert(JSON.stringify(direct) === JSON.stringify(["serde", "zstd"]), "Universe bundle-loader direct dependencies differ");
const checksums = lockChecksums(lock);
const packages = metadata.packages.filter((entry) => entry.source?.startsWith("registry+")).map((entry) => ({
  name: entry.name,
  version: entry.version,
  relationship: direct.includes(entry.name) ? "Direct" : "Transitive",
  license: entry.license,
  checksum: checksums.get(`${entry.name}@${entry.version}`),
})).sort((left, right) => left.name.localeCompare(right.name));
assert(packages.every((entry) => entry.license && entry.checksum), "Universe bundle-loader dependency review is incomplete");
const policy = {
  schema_revision: "starclock.universe-bundle-loader-dependencies.v1",
  reviewed_on: "2026-07-22",
  scope: "Standalone generated-reader acceptance tool; no dependency or Universe row enters a runtime workspace crate.",
  lockfile_sha256: sha256(lock),
  direct_packages: [
    { name: "serde", version: "1.0.228", license: "MIT OR Apache-2.0" },
    { name: "zstd", version: "0.13.3", license: "MIT" },
  ],
  source_url_template: "https://crates.io/crates/{name}/{version}",
  purpose: "Compile the isolated Sora-generated Universe readers and prove that staged binary bundles decode every table.",
  deterministic_impact: "Tooling only. It reads and counts immutable bundle rows; it does not lower data or participate in simulation.",
  compile_cost: "Shares the already-reviewed serde/zstd reader stack; compilation is acceptance-only and has no runtime budget.",
  rejected_alternatives: [
    "Loading the staged Universe bundle through starclock-data would violate the no-runtime-lowering boundary.",
    "Treating Sora debug JSON as a reader proof would bypass the binary format under review.",
  ],
  packages,
};
const output = `${JSON.stringify(policy, null, 2)}\n`;
const destination = path.join(root, "policy/universe-bundle-loader-dependencies.json");
if (write) {
  fs.writeFileSync(destination, output);
  console.log(`Wrote Universe bundle-loader dependency policy (${packages.length} packages).`);
} else {
  assert(fs.existsSync(destination) && fs.readFileSync(destination, "utf8") === output, "Universe bundle-loader dependency policy drifted");
  console.log(`Universe bundle-loader dependency policy verified (${packages.length} packages).`);
}

function lockChecksums(file) {
  return new Map(fs.readFileSync(file, "utf8").split("[[package]]").slice(1).map((block) => {
    const name = block.match(/^name = "([^"]+)"/m)?.[1];
    const version = block.match(/^version = "([^"]+)"/m)?.[1];
    const checksum = block.match(/^checksum = "([^"]+)"/m)?.[1];
    return checksum ? [`${name}@${version}`, checksum] : undefined;
  }).filter(Boolean));
}
function capture(command, arguments_) {
  const result = spawnSync(command, arguments_, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache", "universe-bundle-loader-target") },
  });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} ${arguments_.join(" ")} failed: ${result.stderr}`);
  return result.stdout;
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
