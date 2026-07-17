import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.cwd());
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy", "dependency-and-tool-policy.json"), "utf8"));
const soraPolicy = JSON.parse(fs.readFileSync(path.join(root, "policy", "sora-toolchain.json"), "utf8"));
assert(policy.compile_cost_measurement.elapsed_milliseconds > 0 && policy.compile_cost_measurement.command && policy.compile_cost_measurement.runner, "compile-cost measurement is incomplete");
assert(policy.production_reader_compile_cost_measurement.elapsed_milliseconds > 0 && policy.production_reader_compile_cost_measurement.command && policy.production_reader_compile_cost_measurement.runner, "production reader compile-cost measurement is incomplete");
assert(policy.rng_hash_compile_cost_measurement.elapsed_milliseconds > 0 && policy.rng_hash_compile_cost_measurement.command && policy.rng_hash_compile_cost_measurement.runner, "RNG/hash compile-cost measurement is incomplete");
assert(policy.replay_codec_compile_cost_measurement.elapsed_milliseconds > 0 && policy.replay_codec_compile_cost_measurement.command && policy.replay_codec_compile_cost_measurement.runner, "replay codec compile-cost measurement is incomplete");
assert(policy.property_harness_compile_cost_measurement.elapsed_milliseconds > 0 && policy.property_harness_compile_cost_measurement.command && policy.property_harness_compile_cost_measurement.runner, "property harness compile-cost measurement is incomplete");
const requiredFields = ["license", "source_url", "purpose", "deterministic_impact", "compile_cost", "rejected_alternatives"];
for (const kind of ["packages", "tools"]) {
  for (const entry of policy[kind]) {
    for (const field of requiredFields) assert(entry[field] && (!Array.isArray(entry[field]) || entry[field].length > 0), `${kind}:${entry.name} lacks ${field}`);
    assert(/^https:\/\//.test(entry.source_url), `${kind}:${entry.name} lacks a source URL`);
  }
}
for (const group of policy.package_groups ?? []) {
  for (const field of ["relationship", "owner", "source_url", "purpose", "deterministic_impact", "compile_cost", "rejected_alternatives"]) {
    assert(group[field] && (!Array.isArray(group[field]) || group[field].length > 0), `package group ${group.name} lacks ${field}`);
  }
  assert(Array.isArray(group.packages) && group.packages.length > 0, `package group ${group.name} is empty`);
}

const metadata = JSON.parse(run("cargo", ["metadata", "--format-version", "1"]));
const registry = metadata.packages.filter((entry) => entry.source?.startsWith("registry+")).map((entry) => ({
  name: entry.name,
  version: entry.version,
  license: entry.license,
})).sort(comparePackage);
const groupedPackages = (policy.package_groups ?? []).flatMap((group) => group.packages);
const reviewed = [...policy.packages, ...groupedPackages].map((entry) => ({ name: entry.name, version: entry.version, license: entry.license })).sort(comparePackage);
assert(new Set(reviewed.map((entry) => `${entry.name}@${entry.version}`)).size === reviewed.length, "package inventory contains duplicate name/version pairs");
assert(JSON.stringify(registry) === JSON.stringify(reviewed), `registry package policy differs:\nreviewed ${JSON.stringify(reviewed)}\nresolved ${JSON.stringify(registry)}`);

const toolchain = fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8");
assert(toolchain.includes('channel = "1.97.0"'), "Rust toolchain is not pinned to 1.97.0");
assert(toolchain.includes('components = ["clippy", "rustfmt"]'), "required Rust components are not pinned");
assert(fs.readFileSync(path.join(root, ".node-version"), "utf8").trim() === "24.15.0", "Node pin differs");
assert(run("rustc", ["--version"]) === "rustc 1.97.0 (2d8144b78 2026-07-07)", "active rustc differs from policy");
assert(run("cargo", ["--version"]) === "cargo 1.97.0 (c980f4866 2026-06-30)", "active Cargo differs from policy");
assert(run("rustfmt", ["--version"]) === "rustfmt 1.9.0-stable (2d8144b788 2026-07-07)", "active rustfmt differs from policy");
assert(run("cargo", ["clippy", "--version"]) === "clippy 0.1.97 (2d8144b788 2026-07-07)", "active Clippy differs from policy");
assert(run("node", ["--version"]) === "v24.15.0", "active Node differs from policy");
const soraEntry = policy.tools.find((entry) => entry.name === "sora-cli");
assert(soraEntry?.version === "0.3.0" && soraEntry.license === soraPolicy.license, "Sora tool inventory differs from its checksum policy");
assert(soraPolicy.version === "0.3.0" && /^[a-f0-9]{64}$/.test(soraPolicy.crate_sha256), "Sora checksum policy is incomplete");

const combatSource = path.join(root, "crates", "starclock-combat", "src");
const rustFiles = walk(combatSource).filter((file) => file.endsWith(".rs"));
const backendUsers = rustFiles.filter((file) => fs.readFileSync(file, "utf8").includes("fixnum"));
assert(backendUsers.length === 1 && path.relative(root, backendUsers[0]).replaceAll("\\", "/") === "crates/starclock-combat/src/numeric/scalar.rs", `fixnum escaped the private scalar backend: ${backendUsers.join(", ")}`);
const combatRoot = fs.readFileSync(path.join(combatSource, "lib.rs"), "utf8");
const workspaceManifest = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const combatManifest = fs.readFileSync(path.join(root, "crates", "starclock-combat", "Cargo.toml"), "utf8");
assert(workspaceManifest.includes('rand = { version = "=0.10.2", default-features = false, features = ["chacha", "std"] }'), "authoritative rand pin/features differ");
assert(workspaceManifest.includes('proptest = { version = "=1.11.0", default-features = false, features = ["std"] }'), "property harness pin/features differ");
assert(workspaceManifest.includes('sha2 = { version = "=0.11.0", default-features = false }'), "authoritative sha2 pin/features differ");
assert(combatManifest.includes("rand.workspace = true") && combatManifest.includes("sha2.workspace = true"), "combat RNG/hash dependencies differ");
const replayManifest = fs.readFileSync(path.join(root, "crates", "starclock-replay", "Cargo.toml"), "utf8");
assert(replayManifest.includes("sha2.workspace = true"), "replay SHA-256 dependency differs");
assert(combatManifest.includes("[dev-dependencies]") && combatManifest.includes("proptest.workspace = true"), "combat property dev-dependency differs");
assert(replayManifest.includes("[dev-dependencies]") && replayManifest.includes("proptest.workspace = true"), "replay property dev-dependency differs");
assert(combatRoot.includes("mod numeric;") && !combatRoot.includes("pub mod numeric"), "numeric backend module must remain private");
assert(!combatRoot.includes("pub use fixnum"), "fixnum must not be re-exported");
const randUsers = rustFiles.filter((file) => /\brand::/.test(fs.readFileSync(file, "utf8")));
assert(randUsers.length === 1 && path.relative(root, randUsers[0]).replaceAll("\\", "/") === "crates/starclock-combat/src/rng/engine.rs", `rand escaped the private RNG wrapper: ${randUsers.join(", ")}`);
const randSource = fs.readFileSync(randUsers[0], "utf8");
assert(!/rand::distr|random_range|random_iter|thread_rng|sys_rng/.test(randSource), "private RNG wrapper uses a forbidden generic/system rand API");
const shaUsers = rustFiles.filter((file) => /\bsha2::/.test(fs.readFileSync(file, "utf8")));
const shaOwners = shaUsers.map((file) => path.relative(root, file).replaceAll("\\", "/")).sort();
assert(JSON.stringify(shaOwners) === JSON.stringify([
  "crates/starclock-combat/src/codec/state.rs",
  "crates/starclock-combat/src/rng/derive.rs",
]), `sha2 escaped the private RNG/state-codec owners: ${shaUsers.join(", ")}`);
assert(!combatRoot.includes("pub use rand") && !combatRoot.includes("pub use sha2"), "RNG/hash dependencies must not be re-exported");

console.log(`Dependency policy verified (${reviewed.length} locked registry packages; ${policy.tools.length} pinned tools; private numeric/RNG/hash boundaries).`);

function run(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim(); }
function comparePackage(left, right) { return left.name.localeCompare(right.name) || left.version.localeCompare(right.version); }
function walk(directory) { return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => { const file = path.join(directory, entry.name); return entry.isDirectory() ? walk(file) : [file]; }); }
function assert(condition, message) { if (!condition) throw new Error(message); }
