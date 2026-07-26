#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) {
    throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
  }
};

const runtime = read(
  "crates/starclock-mode-universe/src/production_runtime.rs",
);
const dynamicTest = read(
  "crates/starclock-mode-universe/tests/dynamic_battle_assembly.rs",
);
const agentTest = read(
  "crates/starclock-agent-api/tests/activity_session_loop.rs",
);
const mcpTest = read(
  "crates/starclock-mcp/tests/universe_surface_parity.rs",
);
const httpTest = read("crates/starclock-mcp/tests/http_conformance.rs");
const evidence = read(
  "docs/goal-06-replay-reconstruction-surface-parity.md",
);
const status = read(
  "docs/goals/06-combat-identity-and-dynamic-assembly-status.md",
);

for (const [text, needle, label] of [
  [
    dynamicTest,
    "dynamic_replay_reconstructs_each_snapshot_and_reports_first_divergence",
    "dynamic reconstruction fixture",
  ],
  [
    dynamicTest,
    "verification must resolve one current Activity snapshot per battle",
    "per-battle reconstruction count",
  ],
  [dynamicTest, "ReplayV3DivergenceKind::Component", "component divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::Assembly", "assembly divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::CombatInput", "input divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::Command", "command divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::Event", "event divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::State", "state divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::Result", "result divergence"],
  [dynamicTest, "ReplayV3DivergenceKind::Activity", "Activity divergence"],
  [
    agentTest,
    "baseline_and_agent_surfaces_emit_identical_authoritative_nested_trace",
    "CLI baseline and Agent nested parity",
  ],
  [
    agentTest,
    "nested_authority_digest",
    "authoritative nested projection",
  ],
  [
    mcpTest,
    "mcp_activity_surface_matches_agent_replay_and_fresh_verification",
    "MCP and Agent replay parity",
  ],
  [
    httpTest,
    "CURRENT_COMBAT_STATE_HASHES",
    "current-codec HTTP state golden",
  ],
  [
    runtime,
    "into_replay_v2_compatibility_parts",
    "named replay-v2 compatibility seam",
  ],
  [runtime, "pub fn into_dynamic_parts", "dynamic production seam"],
  [
    runtime,
    "baseline_materialization_coverage",
    "coverage-only historical evidence",
  ],
  [evidence, "exactly one", "normative reconstruction rule"],
  [status, "| `G06-P3-B3` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

const agentGoldens = new Set(agentTest.match(/[0-9a-f]{64}/g) ?? []);
const sharedSurfaceGoldens = (mcpTest.match(/[0-9a-f]{64}/g) ?? [])
  .filter((value) => agentGoldens.has(value));
if (new Set(sharedSurfaceGoldens).size < 2) {
  throw new Error("Agent and MCP no longer share current state/replay goldens");
}

for (const forbidden of [
  "pub const fn materialization(",
  "pub fn materialization(",
  "pub fn into_parts(",
]) {
  if (runtime.includes(forbidden)) {
    throw new Error(`generic frozen runtime access remains: ${forbidden}`);
  }
}

console.log(
  "Goal 06 P3-B3 verified (dynamic replay reconstruction and production-surface parity).",
);
