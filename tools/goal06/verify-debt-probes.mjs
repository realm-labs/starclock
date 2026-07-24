import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal06-debt-probes.json");
assert(policy.schema_revision === "starclock.goal06-debt-probes.v1",
  "unsupported Goal 06 debt-probe revision");

const rustFiles = files("crates", ".rs");
const specCallFiles = rustFiles.filter((file) => text(file).includes("BattleSpec::new("));
const specCalls = specCallFiles.reduce(
  (total, file) => total + occurrences(text(file), "BattleSpec::new("),
  0,
);
assert(specCallFiles.length === policy.caller_supplied_identity.source_files,
  `BattleSpec caller file count drifted to ${specCallFiles.length}`);
assert(specCalls === policy.caller_supplied_identity.call_sites,
  `BattleSpec call-site count drifted to ${specCalls}`);

const spec = text("crates/starclock-combat/src/battle/spec.rs");
for (const marker of [
  "pub struct BattleSpec",
  "digest: BattleSpecDigest",
  "pub fn new(",
  "digest: BattleSpecDigest,",
  "pub const fn digest(&self) -> BattleSpecDigest",
]) assert(spec.includes(marker), `caller-supplied digest marker is missing: ${marker}`);
assert(!spec.includes("CombatInputDigest"), "combat-owned identity was introduced before P1-B1");

const production = text(policy.entry_time_materialization.factory_source);
for (const marker of [
  "pub struct StandardUniverseRuntimeFactory",
  "materialization: Arc<UniverseBattleMaterialization>",
  "let contributions = initial_contributions(&catalog)?;",
  "let materialization = UniverseBattleMaterializer",
  "with_encounter_overlay(self.materialization.overlay().clone())",
  "combat_catalog: Arc::clone(self.materialization.combat_catalog())",
]) assert(production.includes(marker), `entry-time materialization marker is missing: ${marker}`);
assert(!production.includes(".battle_contributions("),
  "factory unexpectedly consumes current Activity contributions");
const dynamicAccess = text(
  "crates/starclock-mode-universe/src/runtime/battle_contribution_access.rs",
);
assert(dynamicAccess.includes("pub fn battle_contributions"),
  "current Activity projection seam is missing");

for (const file of policy.production_surfaces.shared_factory_sources)
  assert(text(file).includes("StandardUniverseRuntimeFactory"),
    `production surface no longer uses the shared factory: ${file}`);
const mcp = [
  text("crates/starclock-mcp/src/activity_tools.rs"),
  text("crates/starclock-mcp/src/resources.rs"),
].join("\n");
for (const marker of [
  "activity_registry.apply_action",
  "activity_registry.export_replay",
  "activity_factory.verify_replay",
]) assert(mcp.includes(marker), `MCP delegation marker is missing: ${marker}`);

const content = [
  ...json("content-reference/standard-universe-v1/blessings.json"),
  ...json("content-reference/standard-universe-v1/curios.json"),
  ...json("content-reference/standard-universe-v1/paths.json"),
  ...json("content-reference/standard-universe-v1/resonances.json"),
  ...json("content-reference/standard-universe-v1/ability-tree.json"),
];
const contentIds = new Set(content.map((entry) => entry.id));
for (const scenario of policy.representative_scenarios) {
  assert(scenario.transitions.length >= 2, `${scenario.id}: scenario is not a transition`);
  if (scenario.source.startsWith("universe."))
    assert(contentIds.has(scenario.source), `${scenario.id}: source is not in the frozen pack`);
}
assert(new Set(policy.representative_scenarios.map((entry) => entry.id)).size
  === policy.representative_scenarios.length, "representative scenario IDs are not unique");

const dispositions = json(
  "content-manifests/standard-universe-end-to-end-v1/integration-dispositions.json",
);
for (const source of [
  "universe.blessing.612344.level.2",
  "universe.curio.8.state.active",
  "universe.resonance.612420",
]) {
  const entry = dispositions.rules.find((candidate) => candidate.id === `universe.rule.${source.slice(9)}`);
  assert(entry?.integration_state === "Integrated",
    `representative executable rule is not integrated: ${source}`);
}

console.log(
  `Goal 06 debt probes verified (${specCalls} BattleSpec callers, entry-time materialization, `
    + `${policy.representative_scenarios.length} frozen scenarios).`,
);

function files(relative, suffix) {
  const result = [];
  const visit = (directory) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(absolute);
      else if (entry.name.endsWith(suffix))
        result.push(path.relative(root, absolute).replaceAll("\\", "/"));
    }
  };
  visit(path.join(root, relative));
  return result.sort();
}
function occurrences(value, needle) {
  return value.split(needle).length - 1;
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
