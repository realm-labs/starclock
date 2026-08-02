#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => option === "--bless"),
  "usage: verify-determinism-hardening.mjs [root] [--bless]");
const bless = options.includes("--bless");
const policy = json("policy/goal20-determinism-hardening.json");
assert(policy.schema_revision === "starclock.goal20-determinism-hardening.v1"
  && policy.goal_id === "swarm-disaster-runtime-v1" && policy.batch === "G20-P8-B1",
"Goal 20 hardening policy identity drift");

const ci = json("policy/ci-matrix.json");
assert(equal(ci.native_profiles.map((profile) => profile.id), policy.native_profiles),
  "native profile inventory drift");
assert(equal(ci.compile_only_profiles.map((profile) => profile.id), policy.compile_only_profiles),
  "compile-only profile inventory drift");
const workflow = normalizedText(".github/workflows/ci.yml");
assert(workflow.includes(`run: ${ci.repository_gate}`)
  && !workflow.includes(`run: ${policy.native_gate}`),
"native CI must run one full repository pass without replaying P8-B1");
const runner = normalizedText("tools/goal20/run-native-ci.mjs");
for (const marker of [
  "swarm_disaster_entry::hardening_tests",
  "component_replay_reexecutes_real_battles_and_reports_every_first_boundary",
  "swarm_disaster_hardening", "\"replay\"", "verify-determinism-hardening.mjs",
]) assert(runner.includes(marker), `native hardening gate omits ${marker}`);

const expectedSuites = [
  "swarm-replay-command-event-state-goldens",
  "swarm-rng-property-and-fault-hardening",
  "swarm-agent-rejection-and-malformed-replay-corpora",
  "inherited-replay-codec-properties",
];
assert(equal(policy.suites.map((suite) => suite.id), expectedSuites),
  "Goal 20 hardening suite inventory drift");
const suites = policy.suites.map((suite) => ({
  id: suite.id,
  targets: suite.targets.map((target) => {
    assert(fs.statSync(path.join(root, target), { throwIfNoEntry: false })?.isFile(),
      `${suite.id}: missing ${target}`);
    return { path: target, sha256: sha256(target) };
  }),
}));

const replay = policy.suites[0];
const replaySource = text(replay.targets[0]);
for (const literal of [
  `replay.records().len(), ${replay.records}`,
  `verified.action_count(), ${replay.activity_actions}`,
  `verified.battle_count(), ${replay.nested_battles}`,
  `verified.battle_command_count(), ${replay.battle_commands}`,
  `bytes.len(), ${numberLiteral(replay.replay_bytes)}`,
  replay.replay_sha256, replay.activity_command_sha256, replay.battle_command_sha256,
  replay.battle_event_state_sha256, replay.activity_state_sha256,
]) assert(replaySource.includes(literal), `replay golden omits ${literal}`);

const rng = policy.suites[1];
const rngSource = text(rng.targets[0]);
assert(rngSource.includes(`const DOMAINS: [(&str, ActivityRngLabel, u16); ${rng.rng_domains}]`),
  "Swarm RNG domain denominator drift");
assert(rngSource.includes(`1..=${rng.draws_per_perturbation}_u16`),
  "Swarm RNG perturbation denominator drift");
assert(rngSource.includes("20_120..20_184_u64"), "Swarm seed-property corpus drift");
assert(rngSource.includes("[0, BUNDLE.len() / 3, BUNDLE.len() - 1]"),
  "Swarm corrupted-candidate corpus drift");
assert(rngSource.includes("swarm_state_fault_is_deterministic_and_discards_partial_mutation")
  && rngSource.includes(rng.rng_domain_sha256), "Swarm hardening vector drift");

const agent = policy.suites[2];
const agentSource = text(agent.targets[0]);
assert(agentSource.includes(`0..${numberLiteral(agent.invalid_actions)}_u32`)
  && agentSource.includes(`cases: ${agent.malformed_replay_cases}`)
  && agentSource.includes(`0..=${numberLiteral(agent.maximum_arbitrary_replay_bytes)}`)
  && agentSource.includes("prop_assert_eq!(&first, &second)"),
"Swarm agent corruption corpus drift");
const inheritedReplay = policy.suites[3];
for (const target of inheritedReplay.targets)
  assert(text(target).includes(`cases: ${inheritedReplay.cases_per_property}`),
    `${target} property denominator drift`);

const generated = json("policy/generated-drift.json");
assert(generated.checks.some((check) => equal(check.command, [
  "node", "tools/goal20/verify-determinism-hardening.mjs", ".",
])), "generated drift does not own Goal 20 hardening evidence");
const ledger = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(/^\| `G20-P8-B1` \| `(InProgress|Complete)` \|/mu.test(ledger),
  "G20-P8-B1 is not active or complete");

const report = {
  schema_revision: "starclock.goal20-determinism-hardening-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "cross-platform-native-contract-and-local-macos-arm64-vectors-frozen",
  native_gate: policy.native_gate,
  goldens: {
    records: replay.records,
    activity_actions: replay.activity_actions,
    battle_commands: replay.battle_commands,
    nested_battles: replay.nested_battles,
    replay_bytes: replay.replay_bytes,
    replay_sha256: replay.replay_sha256,
    activity_command_sha256: replay.activity_command_sha256,
    battle_command_sha256: replay.battle_command_sha256,
    battle_event_state_sha256: replay.battle_event_state_sha256,
    activity_state_sha256: replay.activity_state_sha256,
    rng_domain_sha256: rng.rng_domain_sha256,
  },
  corpora: {
    rng_domains: rng.rng_domains,
    draws_per_perturbation: rng.draws_per_perturbation,
    seed_property_cases: rng.seed_property_cases,
    corrupted_candidate_cases: rng.corrupted_candidate_cases,
    deterministic_fault_cases: rng.deterministic_fault_cases,
    invalid_actions: agent.invalid_actions,
    malformed_replay_cases: agent.malformed_replay_cases,
    maximum_arbitrary_replay_bytes: agent.maximum_arbitrary_replay_bytes,
    inherited_cases_per_property: inheritedReplay.cases_per_property,
  },
  suites,
  profiles: [
    ...policy.native_profiles.map((id) => ({
      id, execution: "required-native-on-success", suites: expectedSuites,
    })),
    ...policy.compile_only_profiles.map((id) => ({
      id, execution: "compiled-not-executed", suites: [],
    })),
  ],
  evidence_boundary: policy.evidence_boundary,
  contract_sha256: {
    policy: sha256("policy/goal20-determinism-hardening.json"),
    workflow: sha256(".github/workflows/ci.yml"),
    native_runner: sha256("tools/goal20/run-native-ci.mjs"),
  },
};
const relative = "evidence/swarm-disaster-runtime-v1/hardening/determinism-hardening.json";
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(),
    `${relative} is missing; run with --bless`);
  assert(normalizedText(relative) === output, `${relative} is stale; run with --bless`);
}
console.log(`Goal 20 determinism hardening verified (${policy.native_profiles.length} native profiles, ${expectedSuites.length} suites, ${agent.invalid_actions} rejection cases).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function normalizedText(relative) { return text(relative).replaceAll("\r\n", "\n"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/gu, "_"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
