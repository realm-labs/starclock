import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evolution = read("docs/26-mode-extension-and-evolution.md");
const activity = read("docs/19-activity-core-and-mode-extension.md");
const rules = read("docs/11-rule-ir-and-native-handlers.md");
const coverage = read("docs/15-content-data-and-coverage.md");
const replay = read("docs/16-replay-cli-and-engine-integration.md");
const generatedPolicy = JSON.parse(read("policy/generated-drift.json"));
const workspacePolicy = JSON.parse(read("policy/workspace-dependencies.json"));
const workflow = read(".github/workflows/ci.yml");
const repositoryRunner = read("tools/repository-check/run.mjs");

for (const marker of [
  "ActivityTaskSpec",
  "LogicalScopeDefinition",
  "RuleRegistry::compose",
  "ConfigurationRootDigest",
  "`Experimental`",
  "completion commit/tree",
]) assert(evolution.includes(marker), `extension contract omits ${marker}`);
for (const marker of ["pending_tasks: Vec<PendingActivityTask>", "bounded logical scope tree", "handler/executor bundle"])
  assert(activity.includes(marker), `Activity extension contract omits ${marker}`);
assert(rules.includes("deterministic composition of immutable"), "Rule registry remains a central closed extension point");
assert(coverage.includes("| `Experimental` |") && coverage.includes("| `Candidate` |") && coverage.includes("| `Released` |"),
  "content delivery lanes are incomplete");
assert(replay.includes("consumed_component_digests[]"), "replay identity remains monolithic");

const commands = generatedPolicy.checks.map((check) => check.command.join(" "));
assert(commands.includes("node tools/repository-check/verify-release-snapshots.mjs"),
  "generated drift does not verify immutable release snapshots");
assert(!commands.some((command) => command.includes("tools/goal04/")),
  "global generated drift still binds current source to historical Goal 04 evidence");
assert(workflow.includes("fetch-depth: 0") && !workflow.includes("tools/goal04/run-native-ci.mjs"),
  "CI cannot resolve immutable snapshots or still reruns historical Goal 04 gates");
assert(workspacePolicy.schema_revision === "starclock.workspace-dependencies.v1"
  && workspacePolicy.packages.every((pkg) => Array.isArray(pkg.local) && Array.isArray(pkg.external)),
  "workspace dependency extension policy is not declarative");
for (const historical of [
  "verify-golden-matrix.mjs",
  "verify-agent-security-audit.mjs",
  "verify-agent-contract-freeze.mjs",
  "verify-goal02-clean-acceptance.mjs",
  "verify-property-hardening.mjs",
  "verify-architecture-audit.mjs",
]) assert(!repositoryRunner.includes(historical), `current repository gate still recomputes historical evidence through ${historical}`);

console.log("Mode extension contract verified (task sets, logical scopes, composed registries, component digests, delivery lanes, immutable releases).");

function read(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
