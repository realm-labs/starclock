import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.cwd());
const expected = new Map([
  ["starclock-combat", []],
  ["starclock-build", ["starclock-combat"]],
  ["starclock-activity", ["starclock-combat"]],
  ["starclock-rules", ["starclock-activity", "starclock-combat"]],
  ["starclock-replay", ["starclock-activity", "starclock-combat"]],
  ["starclock-ai", ["starclock-combat"]],
  ["starclock-mode-standard", ["starclock-activity", "starclock-combat"]],
  ["starclock-data", ["starclock-activity", "starclock-build", "starclock-combat", "starclock-mode-standard", "starclock-rules"]],
  ["starclock-cli", ["starclock-activity", "starclock-ai", "starclock-build", "starclock-combat", "starclock-data", "starclock-mode-standard", "starclock-replay", "starclock-rules"]],
]);
const expectedExternal = new Map([
  ["starclock-combat", [{ name: "fixnum", requirement: "=0.9.5", features: ["i64", "std"] }]],
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
  })).sort((a, b) => a.name.localeCompare(b.name));
  const allowedExternal = (expectedExternal.get(pkg.name) ?? []).map((dependency) => ({
    ...dependency,
    features: [...dependency.features].sort(),
    uses_default_features: false,
  }));
  assert(JSON.stringify(externalDependencies) === JSON.stringify(allowedExternal), `${pkg.name} external dependency policy differs:\nexpected ${JSON.stringify(allowedExternal)}\nactual   ${JSON.stringify(externalDependencies)}`);
}

const combat = packages.find((entry) => entry.name === "starclock-combat");
assert(combat.dependencies.every((dependency) => dependency.name === "fixnum"), "starclock-combat may depend only on the reviewed private numeric backend");
const cli = packages.find((entry) => entry.name === "starclock-cli");
const cliBinaries = cli.targets.filter((target) => target.kind.includes("bin")).map((target) => target.name);
assert(JSON.stringify(cliBinaries) === JSON.stringify(["starclock"]), "starclock-cli must own only the starclock binary");

console.log("Workspace dependency boundaries verified (9 crates; combat has only private fixnum backend).");

function normalize(value) { return path.resolve(value).replaceAll("\\", "/").toLowerCase(); }
function assert(condition, message) { if (!condition) throw new Error(message); }
