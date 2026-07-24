import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.cwd());
const workspaceManifest = read(path.join(root, "Cargo.toml"));
assert(/\[workspace\.lints\.rust\][\s\S]*?unsafe_code\s*=\s*"forbid"/.test(workspaceManifest), "workspace must forbid unsafe Rust");
assert(/\[workspace\.lints\.rust\][\s\S]*?unexpected_cfgs\s*=\s*"deny"/.test(workspaceManifest), "workspace must deny unexpected cfg values");
assert(/\[workspace\.lints\.rust\][\s\S]*?unused_must_use\s*=\s*"deny"/.test(workspaceManifest), "workspace must deny unused must-use results");
const dependencyPolicy = JSON.parse(read(path.join(root, "policy/workspace-dependencies.json")));
assert(dependencyPolicy.schema_revision === "starclock.workspace-dependencies.v1", "unsupported workspace dependency policy");
assert(Array.isArray(dependencyPolicy.packages) && dependencyPolicy.packages.length > 0, "workspace dependency policy is empty");
const expected = new Map(dependencyPolicy.packages.map((pkg) => [pkg.name, pkg.local]));
const expectedExternal = new Map(dependencyPolicy.packages.map((pkg) => [pkg.name, pkg.external]));
assert(expected.size === dependencyPolicy.packages.length, "workspace dependency policy contains duplicate packages");

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
const activity = packages.find((entry) => entry.name === "starclock-activity");
assert(activity.dependencies.every((dependency) => dependency.kind === "dev" ? dependency.name === "allocation-counter" : ["rand", "sha2", "starclock-combat"].includes(dependency.name)), "starclock-activity may depend only on combat domain types, the reviewed private RNG/hash backends and the benchmark-only allocator counter");
const data = packages.find((entry) => entry.name === "starclock-data");
assert(data.dependencies.filter((dependency) => dependency.source !== null).every((dependency) => ["serde", "sha2", "zstd"].includes(dependency.name)), "starclock-data may use only generated-reader transport dependencies plus the reviewed private SHA-256 backend");
const universe = packages.find((entry) => entry.name === "starclock-mode-universe");
assert(universe.dependencies.every((dependency) => ["starclock-activity", "starclock-combat", "starclock-data", "starclock-replay", "serde", "serde_json", "sha2", "zstd"].includes(dependency.name)), "starclock-mode-universe may use only generic Activity/combat/replay boundaries, stable data catalogs and generated-reader transport/hash dependencies");
const replay = packages.find((entry) => entry.name === "starclock-replay");
assert(replay.dependencies.filter((dependency) => dependency.source !== null).every((dependency) => dependency.kind === "dev" ? dependency.name === "proptest" : dependency.name === "sha2"), "starclock-replay may use only the reviewed private SHA-256 backend plus the property dev-dependency");
const cli = packages.find((entry) => entry.name === "starclock-cli");
const cliBinaries = cli.targets.filter((target) => target.kind.includes("bin")).map((target) => target.name);
assert(JSON.stringify(cliBinaries) === JSON.stringify(["starclock"]), "starclock-cli must own only the starclock binary");
const agentApi = packages.find((entry) => entry.name === "starclock-agent-api");
assert(agentApi.dependencies.every((dependency) => ["starclock-activity", "starclock-ai", "starclock-combat", "starclock-data", "starclock-mode-universe", "starclock-replay", "serde", "serde_json", "sha2"].includes(dependency.name) || (dependency.kind === "dev" && ["allocation-counter", "proptest"].includes(dependency.name))), "starclock-agent-api may use only reviewed combat/Activity controller, composition and replay boundaries, deterministic serialization/token-digest dependencies and property/benchmark tooling");

const mcp = packages.find((entry) => entry.name === "starclock-mcp");
assert(mcp.dependencies.every((dependency) => ["starclock-agent-api", "allocation-counter", "axum", "rmcp", "schemars", "serde", "serde_json", "tokio", "tower"].includes(dependency.name)), "starclock-mcp may depend only on the protocol-neutral agent API, frozen official MCP SDK, reviewed HTTP service boundary, schema/JSON conversion, async runtime and benchmark-only allocator counter");

console.log(`Workspace dependency boundaries verified (${packages.length} crates; declarative reviewed dependency graph).`);

function normalize(value) { return path.resolve(value).replaceAll("\\", "/").toLowerCase(); }
function read(file) { return fs.readFileSync(file, "utf8"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
