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

const session = read("crates/starclock-agent-api/src/activity_session.rs");
const sessionTest = read(
  "crates/starclock-test-kit/tests/suites/adapter/agent_api/activity_session_loop.rs",
);
const executor = read(
  "crates/starclock-mode-universe/src/nested_battle_executor.rs",
);
const activityTools = read("crates/starclock-mcp/src/activity_tools.rs");
const authorization = read("crates/starclock-mcp/src/authorization.rs");
const evidence = read("docs/goal-06-agent-mcp-dynamic-assembly.md");
const status = read(
  "docs/goals/06-combat-identity-and-dynamic-assembly-status.md",
);

for (const [text, needle, label] of [
  [session, "Arc<StandardUniverseBattleAssembler>", "shared Agent assembler"],
  [session, "runtime.into_dynamic_parts()", "dynamic runtime parts"],
  [session, "standard_universe_header_v3(", "Agent replay-v3 header"],
  [session, "UniverseNestedBattleExecutor::dynamic()", "dynamic Agent executor"],
  [
    session,
    "execute_dynamic_pending_activity_battle",
    "atomic Agent battle settlement",
  ],
  [
    session,
    "encode_standard_universe_trace_parts_v3(",
    "Agent replay-v3 export",
  ],
  [
    session,
    "verify_standard_universe_replay_v3_dynamic(",
    "Agent dynamic replay verification",
  ],
  [
    executor,
    "pub fn execute_dynamic_pending_activity_battle",
    "shared atomic executor operation",
  ],
  [sessionTest, "decode_replay_v3(replay.bytes())", "Agent v3 integration assertion"],
  [
    activityTools,
    "self.activity_registry.create(",
    "MCP create delegates to Agent registry",
  ],
  [
    activityTools,
    "self.activity_registry.apply_action(",
    "MCP play delegates to Agent registry",
  ],
  [
    activityTools,
    "self.activity_registry.export_replay(",
    "MCP export delegates to Agent registry",
  ],
  [authorization, "SCOPE_ACTIVITY_CREATE", "unchanged create scope"],
  [authorization, "SCOPE_ACTIVITY_READ", "unchanged read scope"],
  [authorization, "SCOPE_ACTIVITY_ACT", "unchanged act scope"],
  [authorization, "SCOPE_ACTIVITY_REPLAY", "unchanged replay scope"],
  [authorization, "SCOPE_ACTIVITY_CLOSE", "unchanged close scope"],
  [evidence, "`agent-api-v1`", "stable public schema statement"],
  [evidence, "opaque action-token authority", "stable authority statement"],
  [status, "| `G06-P3-B2` | `Complete` |", "completed ledger row"],
]) {
  has(text, needle, label);
}

console.log(
  "Goal 06 P3-B2 verified (Agent and MCP use dynamic per-battle assembly).",
);
