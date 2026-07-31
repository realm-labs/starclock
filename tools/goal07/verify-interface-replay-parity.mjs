#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const rootArgument = process.argv.slice(2).find((value) => !value.startsWith("--"));
const root = path.resolve(rootArgument ?? ".");
const bless = process.argv.includes("--bless");
const policyPath = "policy/goal07-interface-replay-parity.json";
const evidencePath =
  "evidence/standard-universe-mechanics-complete-v1/integration/interface-replay-parity.json";
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const sha256 = (value) =>
  crypto.createHash("sha256").update(value).digest("hex");
const assert = (condition, message) => {
  if (!condition) {
    throw new Error(message);
  }
};
const includes = (source, marker, label) =>
  assert(source.includes(marker), `${label} omits ${JSON.stringify(marker)}`);

const policy = JSON.parse(read(policyPath));
assert(
  policy.schema_revision === "starclock.goal07-interface-replay-parity.v1",
  "unexpected interface/replay parity policy revision",
);
assert(policy.batch === "G07-P6-B2", "unexpected interface/replay parity batch");
assert(
  JSON.stringify(policy.surfaces) ===
    JSON.stringify(["baseline-ai", "cli", "agent", "mcp"]),
  "production surface denominator drift",
);
assert(
  Object.values(policy.contracts).every((value) => value === true),
  "every interface/replay parity contract must be enabled",
);

const sources = {
  dynamic:
    "crates/starclock-test-kit/tests/suites/universe/dynamic_battle_assembly.rs",
  agent: "crates/starclock-test-kit/tests/suites/adapter/agent_api/activity_session_loop.rs",
  cli: "crates/starclock-cli/tests/universe_cli.rs",
  cliContract: "crates/starclock-cli/tests/cli_contract.rs",
  cliMcpStdio: "crates/starclock-cli/tests/mcp_stdio.rs",
  cliStandardBattle: "crates/starclock-cli/tests/standard_replay_smoke.rs",
  cliRuntime: "crates/starclock-cli/src/universe_v1.rs",
  mcp: "crates/starclock-test-kit/tests/suites/adapter/mcp/universe_surface_parity.rs",
  mcpHttp: "crates/starclock-test-kit/tests/suites/adapter/mcp/http_conformance.rs",
};
const text = Object.fromEntries(
  Object.entries(sources).map(([key, relative]) => [key, read(relative)]),
);
for (const [key, marker] of Object.entries({
  dynamic: "production_baseline_records_and_verifies_dynamic_replay_v3",
  agent: "baseline_and_agent_surfaces_emit_identical_authoritative_nested_trace",
  cli: "universe_run_round_trips_a_canonical_replay_and_detects_corruption",
  cliContract: "config_validation_uses_only_a_validated_sora_bundle",
  cliMcpStdio:
    "independent_stdio_client_proves_discovery_play_errors_cancellation_replay_and_shutdown",
  cliStandardBattle: "cli_runs_and_verifies_the_frozen_public_standard_scenario",
  cliRuntime: "record_baseline_run_v3(",
  mcp: "mcp_activity_surface_matches_agent_replay_and_fresh_verification",
  mcpHttp: "authorized_tcp_client_proves_conformance_trace_and_multi_session_load",
})) {
  includes(text[key], marker, `${key} surface fixture`);
}

for (const marker of [
  "verification must resolve one current Activity snapshot per battle",
  "dynamic_replay_reconstructs_each_snapshot_and_reports_first_divergence",
]) {
  includes(text.dynamic, marker, "fresh reconstruction fixture");
}
let prior = -1;
for (const kind of policy.first_divergence_order) {
  const marker = `ReplayV3DivergenceKind::${kind}`;
  const index = text.dynamic.indexOf(marker);
  assert(index > prior, `first-divergence order drift at ${kind}`);
  prior = index;
}

const golden = policy.goldens;
const compatibility = policy.compatibility_goldens;
for (const [surface, source] of Object.entries({
  agent: text.agent,
  cli: text.cli,
  mcp: text.mcp,
})) {
  includes(source, golden.final_state_hash, `${surface} final-state golden`);
}
for (const source of [text.agent, text.mcp]) {
  includes(
    source,
    golden.agent_mcp_replay_sha256,
    "Agent/MCP shared replay golden",
  );
}
const rustInteger = (value) =>
  value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, "_");
includes(text.agent, rustInteger(golden.agent_mcp_replay_bytes), "Agent replay size");
includes(text.cli, rustInteger(golden.cli_replay_bytes), "CLI replay size");
includes(
  text.cliContract,
  golden.cli_catalog_bundle_sha256,
  "CLI complete-catalog bundle golden",
);
includes(
  text.cliContract,
  `identities=${golden.cli_catalog_identities} enabled=${golden.cli_catalog_identities}`,
  "CLI complete-catalog identity denominator",
);
includes(text.agent, "nested_authority_digest", "baseline/Agent nested parity");
includes(text.agent, "CORPUS_CASES: usize = 16", "Agent corruption corpus");
includes(text.mcp, `"${golden.replay_actions}"`, "MCP replay action golden");
for (const source of [
  text.cliMcpStdio,
  text.cliStandardBattle,
  text.mcpHttp,
]) {
  includes(
    source,
    compatibility.standard_battle_final_state_hash,
    "current Standard Battle compatibility golden",
  );
}
includes(
  text.cliMcpStdio,
  `assert_eq!(step, ${compatibility.standard_battle_external_actions})`,
  "stdio external-action golden",
);
includes(
  text.cliStandardBattle,
  `\\"commands\\":${compatibility.standard_battle_replay_commands}`,
  "CLI Standard Battle replay-command golden",
);
includes(
  text.cliStandardBattle,
  `\\"replay_bytes\\":${compatibility.standard_battle_cli_replay_bytes}`,
  "CLI Standard Battle replay-size golden",
);
includes(
  text.mcpHttp,
  `const CLIENTS: usize = ${compatibility.http_concurrent_clients}`,
  "HTTP concurrent-client denominator",
);
includes(
  text.mcpHttp,
  `CURRENT_COMBAT_STATE_HASHES: [&str; ${compatibility.http_state_hashes}]`,
  "HTTP current state-hash denominator",
);
const cliDigestMatch = text.cli.match(
  /replay_hash\.finalize\(\),\s*Sha256Digest::new\(\[([\s\S]*?)\]\)/,
);
assert(cliDigestMatch, "CLI replay SHA-256 byte golden is missing");
const cliDigestHex = [...cliDigestMatch[1].matchAll(/\d+/g)]
  .map((match) => Number(match[0]).toString(16).padStart(2, "0"))
  .join("");
assert(
  cliDigestHex === golden.cli_replay_sha256,
  "CLI replay SHA-256 policy differs from the executable golden",
);

for (const command of policy.focused_commands) {
  execFileSync(command[0], command.slice(1), {
    cwd: root,
    stdio: "inherit",
    windowsHide: true,
  });
}

const evidence = {
  schema_revision: "starclock.goal07-interface-replay-parity-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "pass",
  scenario: {
    world: policy.world,
    difficulty_index: policy.difficulty_index,
    seed: policy.seed,
    terminal: golden.terminal,
    final_state_hash: golden.final_state_hash,
    replay_actions: golden.replay_actions,
    nested_battles: golden.nested_battles,
    battle_commands: golden.battle_commands,
  },
  surface_parity: {
    surfaces: policy.surfaces,
    baseline_agent_nested_authority: "byte-equivalent",
    agent_mcp_full_replay: "byte-equivalent",
    cli: {
      catalog_identities: golden.cli_catalog_identities,
      catalog_bundle_sha256: golden.cli_catalog_bundle_sha256,
      replay_bytes: golden.cli_replay_bytes,
      replay_sha256: golden.cli_replay_sha256,
      fresh_verification: "pass",
    },
    agent_mcp: {
      replay_bytes: golden.agent_mcp_replay_bytes,
      replay_sha256: golden.agent_mcp_replay_sha256,
      fresh_verification: "pass",
    },
  },
  reconstruction: {
    current_activity_snapshots_per_nested_battle: 1,
    first_divergence_order: policy.first_divergence_order,
    corruption_classes: policy.first_divergence_order.length,
    malformed_agent_replay_cases: 16,
    live_session_mutation_after_corruption: 0,
  },
  compatibility: {
    immutable_goal02_evidence_mutated: false,
    standard_battle_external_actions:
      compatibility.standard_battle_external_actions,
    standard_battle_replay_commands:
      compatibility.standard_battle_replay_commands,
    standard_battle_final_state_hash:
      compatibility.standard_battle_final_state_hash,
    standard_battle_cli_replay_bytes:
      compatibility.standard_battle_cli_replay_bytes,
    http_concurrent_clients: compatibility.http_concurrent_clients,
    http_state_hashes: compatibility.http_state_hashes,
  },
  contracts: policy.contracts,
  inputs: {
    policy_sha256: sha256(read(policyPath)),
    sources: Object.fromEntries(
      Object.entries(sources).map(([key, relative]) => [
        key,
        { path: relative, sha256: sha256(read(relative)) },
      ]),
    ),
  },
};
const output = `${JSON.stringify(evidence, null, 2)}\n`;
const absoluteEvidence = path.join(root, evidencePath);
if (bless) {
  fs.mkdirSync(path.dirname(absoluteEvidence), { recursive: true });
  fs.writeFileSync(absoluteEvidence, output);
} else {
  assert(
    fs.existsSync(absoluteEvidence),
    `interface/replay parity evidence is missing; run with --bless`,
  );
  assert(
    read(evidencePath).replaceAll("\r\n", "\n") === output,
    "interface/replay parity evidence is stale; run with --bless",
  );
}
console.log(
  `Goal 07 interface/replay parity verified (${policy.surfaces.length} surfaces, ` +
    `${policy.first_divergence_order.length} divergence classes).`,
);
