import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(import.meta.dirname, "../..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const policy = JSON.parse(read("policy/goal05-integration-probes.json"));
const fail = (message) => {
  throw new Error(`Goal 05 integration probe failed: ${message}`);
};

if (policy.schema !== "starclock.goal05-integration-probes.v1") {
  fail(`unexpected policy schema ${policy.schema}`);
}
if (policy.focused_budget.wall_seconds < 60 || policy.focused_budget.wall_seconds > 180) {
  fail("focused wall budget must remain between one and three minutes");
}

const cli = read("crates/starclock-cli/src/universe_v1.rs");
const agent = read("crates/starclock-agent-api/src/activity_reference.rs");
if (cli.includes("reference_won_result") || cli.includes("verified-reference-projection-v1")) {
  fail("CLI reverted to the frozen Goal 04 reference-settlement baseline");
}
for (const marker of [
  "UniverseNestedBattleExecutor",
  "record_baseline_run_v2",
  "verify_standard_universe_replay_v2",
  "standard_universe_component_set"
]) {
  if (!cli.includes(marker)) fail(`CLI is missing real-combat migration marker ${marker}`);
}
if (!agent.includes("reference_won_result") || !agent.includes("verified-reference-projection-v1")) {
  fail("agent API moved before its owning G05-P4-B3 batch");
}

const runtime = read("crates/starclock-mode-universe/src/runtime.rs");
const pathMethods = [
  "preservation_effects",
  "remembrance_effects",
  "nihility_effects",
  "abundance_effects",
  "hunt_effects",
  "destruction_effects",
  "elation_effects",
  "propagation_effects",
  "erudition_effects"
];
for (const method of pathMethods) {
  if (!runtime.includes(`pub fn ${method}`)) {
    fail(`missing frozen path effect entry point ${method}`);
  }
}
for (const method of policy.frozen_snapshot.effect_plan_only_entry_points) {
  if (!runtime.includes(`pub fn ${method}`)) {
    fail(`missing frozen effect-plan entry point ${method}`);
  }
}

const topology = read("crates/starclock-mode-universe/src/topology.rs");
for (const node of [
  "resolution_node",
  "content_node",
  "member_node",
  "battle_node",
  "reward_node",
  "formation_node",
  "route_node"
]) {
  if (!topology.includes(`fn ${node}`) && !topology.includes(` ${node}:`)) {
    fail(`missing frozen domain micrograph node ${node}`);
  }
}

const maps = JSON.parse(read("content-reference/standard-universe-v1/maps.json"));
const byMap = new Map();
for (const node of maps) {
  const entries = byMap.get(node.map_id) ?? [];
  entries.push(node);
  byMap.set(node.map_id, entries);
}
let cycles = 0;
for (const entries of byMap.values()) {
  const graph = new Map(entries.map((node) => [node.id, node.next_node_ids]));
  const visited = new Set();
  const active = new Set();
  const visit = (id) => {
    if (active.has(id)) {
      cycles += 1;
      return;
    }
    if (visited.has(id)) return;
    visited.add(id);
    active.add(id);
    for (const target of graph.get(id) ?? []) visit(target);
    active.delete(id);
  };
  for (const id of graph.keys()) visit(id);
}
if (byMap.size !== policy.frozen_snapshot.topology_templates) {
  fail(`expected ${policy.frozen_snapshot.topology_templates} maps, got ${byMap.size}`);
}
if (maps.length !== policy.frozen_snapshot.source_topology_nodes) {
  fail(`expected ${policy.frozen_snapshot.source_topology_nodes} nodes, got ${maps.length}`);
}
if (cycles !== policy.frozen_snapshot.source_topology_cycles) {
  fail(`expected ${policy.frozen_snapshot.source_topology_cycles} cycles, got ${cycles}`);
}

process.stdout.write(
  `Goal 05 integration probes verified with CLI real combat (${byMap.size} maps, ${maps.length} source nodes, ` +
    `${pathMethods.length} path entry points, ${cycles} source cycles, <=${policy.focused_budget.wall_seconds}s focused gate).\n`
);
