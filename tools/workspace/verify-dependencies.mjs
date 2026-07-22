import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.cwd());
const workspaceManifest = read(path.join(root, "Cargo.toml"));
assert(/\[workspace\.lints\.rust\][\s\S]*?unsafe_code\s*=\s*"forbid"/.test(workspaceManifest), "workspace must forbid unsafe Rust");
assert(/\[workspace\.lints\.rust\][\s\S]*?unexpected_cfgs\s*=\s*"deny"/.test(workspaceManifest), "workspace must deny unexpected cfg values");
assert(/\[workspace\.lints\.rust\][\s\S]*?unused_must_use\s*=\s*"deny"/.test(workspaceManifest), "workspace must deny unused must-use results");
const expected = new Map([
  ["starclock-agent-api", ["starclock-ai", "starclock-combat", "starclock-data", "starclock-replay"]],
  ["starclock-combat", []],
  ["starclock-build", ["starclock-combat"]],
  ["starclock-activity", ["starclock-combat"]],
  ["starclock-rules", ["starclock-activity", "starclock-combat"]],
  ["starclock-replay", ["starclock-activity", "starclock-combat"]],
  ["starclock-ai", ["starclock-combat"]],
  ["starclock-mode-standard", ["starclock-activity", "starclock-combat"]],
  ["starclock-mcp", ["starclock-agent-api"]],
  ["starclock-data", ["starclock-activity", "starclock-build", "starclock-combat", "starclock-mode-standard", "starclock-rules"]],
  ["starclock-cli", ["starclock-activity", "starclock-ai", "starclock-build", "starclock-combat", "starclock-data", "starclock-mode-standard", "starclock-replay", "starclock-rules"]],
]);
const expectedExternal = new Map([
  ["starclock-agent-api", [
    { name: "allocation-counter", requirement: "=0.8.1", features: [], kind: "dev" },
    { name: "proptest", requirement: "=1.11.0", features: ["std"], kind: "dev" },
    { name: "serde", requirement: "=1.0.228", features: ["derive", "rc", "std"] },
    { name: "serde_json", requirement: "=1.0.151", features: ["std"] },
    { name: "sha2", requirement: "=0.11.0", features: [] },
  ]],
  ["starclock-combat", [
    { name: "fixnum", requirement: "=0.9.5", features: ["i64", "std"] },
    { name: "proptest", requirement: "=1.11.0", features: ["std"], kind: "dev" },
    { name: "rand", requirement: "=0.10.2", features: ["chacha", "std"] },
    { name: "sha2", requirement: "=0.11.0", features: [] },
  ]],
  ["starclock-build", [
    { name: "sha2", requirement: "=0.11.0", features: [] },
  ]],
  ["starclock-activity", [
    { name: "sha2", requirement: "=0.11.0", features: [] },
  ]],
  ["starclock-data", [
    { name: "serde", requirement: "=1.0.228", features: ["derive", "rc", "std"] },
    { name: "sha2", requirement: "=0.11.0", features: [] },
    { name: "zstd", requirement: "=0.13.3", features: [] },
  ]],
  ["starclock-replay", [
    { name: "proptest", requirement: "=1.11.0", features: ["std"], kind: "dev" },
    { name: "sha2", requirement: "=0.11.0", features: [] },
  ]],
  ["starclock-mcp", [
    { name: "rmcp", requirement: "=2.2.0", features: ["client", "macros", "server", "transport-io", "transport-streamable-http-server"] },
    { name: "schemars", requirement: "=1.2.1", features: ["derive", "std"] },
    { name: "serde", requirement: "=1.0.228", features: ["derive", "rc", "std"] },
    { name: "serde_json", requirement: "=1.0.151", features: ["std"] },
    { name: "tokio", requirement: "=1.53.1", features: ["io-util", "macros", "rt-multi-thread", "time"], kind: "dev" },
  ]],
  ["starclock-cli", [
    { name: "allocation-counter", requirement: "=0.8.1", features: [], kind: "dev" },
  ]],
]);

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
  cwd: root,
  encoding: "utf8",
}));
const memberIds = new Set(metadata.workspace_members);
const packages = metadata.packages.filter((entry) => memberIds.has(entry.id));
const actualNames = packages.map((entry) => entry.name).sort();
const expectedNames = [...expected.keys()].sort();
assert(JSON.stringify(actualNames) === JSON.stringify(expectedNames), `workspace packages differ:\nexpected ${expectedNames.join(", ")}\nactual   ${actualNames.join(", ")}`);

for (const pkg of packages) {
  assert(pkg.edition === "2024", `${pkg.name} must use edition 2024`);
  assert(Array.isArray(pkg.publish) && pkg.publish.length === 0, `${pkg.name} must inherit publish = false`);
  const manifestDirectory = normalize(path.dirname(pkg.manifest_path));
  assert(manifestDirectory === normalize(path.join(root, "crates", pkg.name)), `${pkg.name} is outside its required crate directory`);
  const manifest = read(pkg.manifest_path);
  assert(/\[lints\]\s*workspace\s*=\s*true/.test(manifest), `${pkg.name} must inherit workspace lints`);
  const localDependencies = pkg.dependencies
    .filter((dependency) => dependency.source === null)
    .map((dependency) => dependency.name)
    .sort();
  const expectedDependencies = [...expected.get(pkg.name)].sort();
  assert(JSON.stringify(localDependencies) === JSON.stringify(expectedDependencies), `${pkg.name} local dependencies differ:\nexpected ${expectedDependencies.join(", ") || "(none)"}\nactual   ${localDependencies.join(", ") || "(none)"}`);
  const externalDependencies = pkg.dependencies.filter((dependency) => dependency.source !== null).map((dependency) => ({
    name: dependency.name,
    requirement: dependency.req,
    features: [...dependency.features].sort(),
    uses_default_features: dependency.uses_default_features,
    kind: dependency.kind,
  })).sort((a, b) => a.name.localeCompare(b.name));
  const allowedExternal = (expectedExternal.get(pkg.name) ?? []).map((dependency) => ({
    name: dependency.name,
    requirement: dependency.requirement,
    features: [...dependency.features].sort(),
    uses_default_features: false,
    kind: dependency.kind ?? null,
  }));
  assert(JSON.stringify(externalDependencies) === JSON.stringify(allowedExternal), `${pkg.name} external dependency policy differs:\nexpected ${JSON.stringify(allowedExternal)}\nactual   ${JSON.stringify(externalDependencies)}`);
}

const combat = packages.find((entry) => entry.name === "starclock-combat");
assert(combat.dependencies.every((dependency) => dependency.kind === "dev" ? dependency.name === "proptest" : ["fixnum", "rand", "sha2"].includes(dependency.name)), "starclock-combat may depend only on the reviewed private numeric/RNG/hash backends plus the property dev-dependency");
const data = packages.find((entry) => entry.name === "starclock-data");
assert(data.dependencies.filter((dependency) => dependency.source !== null).every((dependency) => ["serde", "sha2", "zstd"].includes(dependency.name)), "starclock-data may use only generated-reader transport dependencies plus the reviewed private SHA-256 backend");
const replay = packages.find((entry) => entry.name === "starclock-replay");
assert(replay.dependencies.filter((dependency) => dependency.source !== null).every((dependency) => dependency.kind === "dev" ? dependency.name === "proptest" : dependency.name === "sha2"), "starclock-replay may use only the reviewed private SHA-256 backend plus the property dev-dependency");
const cli = packages.find((entry) => entry.name === "starclock-cli");
const cliBinaries = cli.targets.filter((target) => target.kind.includes("bin")).map((target) => target.name);
assert(JSON.stringify(cliBinaries) === JSON.stringify(["starclock"]), "starclock-cli must own only the starclock binary");
const agentApi = packages.find((entry) => entry.name === "starclock-agent-api");
assert(agentApi.dependencies.every((dependency) => ["starclock-ai", "starclock-combat", "starclock-data", "starclock-replay", "serde", "serde_json", "sha2"].includes(dependency.name) || (dependency.kind === "dev" && ["allocation-counter", "proptest"].includes(dependency.name))), "starclock-agent-api may use only reviewed Goal 01 controller/composition/replay boundaries, deterministic serialization/token-digest dependencies and property/benchmark tooling");

const mcp = packages.find((entry) => entry.name === "starclock-mcp");
assert(mcp.dependencies.every((dependency) => ["starclock-agent-api", "rmcp", "schemars", "serde", "serde_json", "tokio"].includes(dependency.name)), "starclock-mcp may depend only on the protocol-neutral agent API, frozen official MCP SDK, schema/JSON conversion and test-only async runtime");

console.log("Workspace dependency boundaries verified (11 crates; protocol-neutral agent boundary and one-way frozen MCP adapter dependency).");

function normalize(value) { return path.resolve(value).replaceAll("\\", "/").toLowerCase(); }
function read(file) { return fs.readFileSync(file, "utf8"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
