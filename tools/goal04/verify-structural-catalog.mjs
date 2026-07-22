import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-structural-catalog.json");
assert(policy.schema_revision === "starclock.goal04-structural-catalog.v1", "unexpected structural-catalog policy revision");

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: root, encoding: "utf8" }));
const universe = metadata.packages.find((entry) => entry.name === "starclock-mode-universe");
assert(universe, "Universe mode crate is absent");
const local = universe.dependencies.filter((dependency) => dependency.source === null).map((dependency) => dependency.name).sort();
const external = universe.dependencies.filter((dependency) => dependency.source !== null).map((dependency) => dependency.name).sort();
assert(equal(local, policy.local_dependencies), "structural catalog local dependencies differ");
assert(equal(external, policy.external_dependencies), "structural catalog external dependencies differ");

const facade = text("crates/starclock-mode-universe/src/lib.rs");
assert(facade.includes("pub mod definition;") && facade.includes("pub mod id;") && facade.includes("mod lowering;"), "Universe facade ownership differs");
assert(!facade.includes("pub mod generated") && !facade.includes("pub mod lowering"), "private lowering/generated modules escaped");
const definitions = text("crates/starclock-mode-universe/src/definition.rs");
const ids = text("crates/starclock-mode-universe/src/id.rs");
const lowering = text("crates/starclock-mode-universe/src/lowering.rs");
const catalog = text("crates/starclock-mode-universe/src/catalog.rs");
for (const marker of [
  "pub struct UniverseProfileDefinition", "pub struct WorldDefinition", "pub struct DifficultyDefinition",
  "pub struct DomainDefinition", "pub struct TopologyDefinition", "pub struct TopologyNodeDefinition",
  "pub struct RoomDefinition", "pub struct UniverseActivityBindingDefinition"
]) assert(definitions.includes(marker), `missing domain definition ${marker}`);
for (const marker of ["UniverseProfileId", "WorldId", "DifficultyId", "DomainId", "TopologyNodeId", "TopologyId", "RoomId", "ActivityBindingId"])
  assert(new RegExp(`id_type!\\(\\s*${marker}`).test(ids), `missing stable ID ${marker}`);
for (const [name, count] of Object.entries(policy.counts)) assert(Number.isInteger(count) && count > 0, `${name} count is invalid`);
for (const contract of Object.values(policy.contracts)) assert(lowering.includes(`"${contract}"`), `lowering omits ${contract}`);
assert(hexSequenceAppears(catalog, policy.definitions_digest), "definition digest golden is absent");
assert(!/position_[xy]/.test(definitions), "spatial position leaked into authoritative definitions");
assert(!/crate::generated|UniverseMapNode|SoraConfig/.test(definitions), "generated transport leaked into public definitions");
assert((lowering.match(/serde_json::/g) ?? []).length === 1, "embedded JSON parsing escaped its single private boundary");
assert(!/from_(json|xlsx)|load_(json|xlsx)/i.test(`${facade}\n${catalog}\n${definitions}`), "a competing runtime loader escaped");
assert((catalog.match(/^    #\[test\]$/gm) ?? []).length + (lowering.match(/^    #\[test\]$/gm) ?? []).length === policy.focused_tests, "focused test count differs");
for (const marker of ["pub fn worlds(", "pub fn difficulty(", "pub fn domain(", "pub fn topology(", "pub fn room(", "pub const fn activity_binding("])
  assert(catalog.includes(marker), `catalog query surface omits ${marker}`);

const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P1-B2` \| `(InProgress|Complete)` \|/m.test(status), "G04-P1-B2 is not active or complete");
const evidence = {
  schema_revision: "starclock.goal04-structural-catalog-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "structural-domain-catalog-lowered",
  definitions_digest: policy.definitions_digest,
  counts: policy.counts,
  contracts: policy.contracts,
  boundaries: {
    generated_rows_public: false,
    spatial_coordinates_authoritative: false,
    external_json_or_xlsx_loader: false,
    embedded_score_curve_typed_and_bounded: true,
    graph_references_reachable_acyclic_and_terminal: true,
    activity_domain_coverage_complete: true
  },
  dependencies: { local, external, new_registry_packages: policy.new_registry_packages },
  focused_tests: policy.focused_tests
};
const relative = "evidence/standard-universe-runtime-v1/catalog/structural-definitions.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "structural catalog evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "structural catalog evidence is stale; run with --bless");
}
console.log(`Goal 04 structural catalog verified (${policy.counts.worlds} Worlds, ${policy.counts.topology_nodes} graph nodes, ${policy.counts.rooms} rooms; ${policy.definitions_digest.slice(0, 12)}).`);

function hexSequenceAppears(source, hex) {
  const tokens = [...source.matchAll(/0x([0-9a-f]{2})/g)].map((match) => match[1]).join("");
  return tokens.includes(hex);
}
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function assert(condition, message) { if (!condition) throw new Error(message); }
