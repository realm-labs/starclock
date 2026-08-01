#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(
  options.every((option) => option === "--bless"),
  "usage: verify-determinism-hardening.mjs [root] [--bless]",
);
const bless = options.includes("--bless");
const policy = json("policy/goal14-determinism-hardening.json");
assert(
  policy.schema_revision === "starclock.goal14-determinism-hardening.v1"
    && policy.goal_id === "gold-and-gears-runtime-v1"
    && policy.batch === "G14-P8-B1",
  "Goal 14 hardening policy identity drift",
);

const inherited = json("policy/ci-matrix.json");
assert(
  equal(inherited.native_profiles.map((profile) => profile.id), policy.native_profiles),
  "native profile inventory drift",
);
assert(
  equal(
    inherited.compile_only_profiles.map((profile) => profile.id),
    policy.compile_only_profiles,
  ),
  "compile-only profile inventory drift",
);
const workflow = normalizedText(".github/workflows/ci.yml");
assert(workflow.includes(`run: ${policy.native_gate}`),
  "native CI omits the Goal 14 hardening gate");
const runner = normalizedText("tools/goal14/run-native-ci.mjs");
for (const marker of [
  "gold_gears_entry::hardening_tests",
  "component_replay_reexecutes_real_battles_and_reports_every_first_boundary",
  "gold_gears_hardening",
  "\"replay\"",
  "verify-determinism-hardening.mjs",
]) assert(runner.includes(marker), `native hardening gate omits ${marker}`);

const expectedSuites = [
  "gold-replay-command-event-state-goldens",
  "gold-rng-property-and-fault-hardening",
  "gold-agent-rejection-and-malformed-replay-corpora",
  "inherited-replay-codec-properties",
];
assert(equal(policy.suites.map((suite) => suite.id), expectedSuites),
  "Goal 14 hardening suite inventory drift");
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
  replay.replay_sha256,
  replay.activity_command_sha256,
  replay.battle_command_sha256,
  replay.battle_event_state_sha256,
  replay.activity_state_sha256,
]) assert(replaySource.includes(literal), `replay golden omits ${literal}`);

const rng = policy.suites[1];
const rngSource = text(rng.targets[0]);
assert(rngSource.includes(`const DOMAINS: [(&str, ActivityRngLabel, u16); ${rng.rng_domains}]`),
  "Gold RNG domain denominator drift");
assert(rngSource.includes(`1..=${rng.draws_per_perturbation}_u16`),
  "Gold RNG perturbation denominator drift");
assert(rngSource.includes("14_820..14_884_u64"),
  "Gold seed-property corpus drift");
assert(rngSource.includes("[0, BUNDLE.len() / 3, BUNDLE.len() - 1]"),
  "Gold corrupted-candidate corpus drift");
assert(rngSource.includes("gold_state_fault_is_deterministic_and_discards_partial_mutation"),
  "Gold deterministic fault regression missing");
assert(rngSource.includes(rng.rng_domain_sha256), "Gold RNG digest drift");

const agent = policy.suites[2];
const agentSource = text(agent.targets[0]);
assert(agentSource.includes(`0..${numberLiteral(agent.invalid_actions)}_u32`),
  "Gold invalid-action denominator drift");
assert(agentSource.includes(`cases: ${agent.malformed_replay_cases}`),
  "Gold malformed replay denominator drift");
assert(agentSource.includes(`0..=${numberLiteral(agent.maximum_arbitrary_replay_bytes)}`),
  "Gold malformed replay byte bound drift");
assert(agentSource.includes("prop_assert_eq!(&first, &second)"),
  "Gold malformed replay failures are not compared repeatably");

const inheritedReplay = policy.suites[3];
for (const target of inheritedReplay.targets)
  assert(text(target).includes(`cases: ${inheritedReplay.cases_per_property}`),
    `${target} property denominator drift`);

const generated = json("policy/generated-drift.json");
assert(generated.checks.some((check) => equal(check.command, [
  "node", "tools/goal14/verify-determinism-hardening.mjs", ".",
])), "generated drift does not own Goal 14 hardening evidence");
const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(/^\| `G14-P8-B1` \| `(InProgress|Complete)` \|/mu.test(ledger),
  "G14-P8-B1 is not active or complete");

const report = {
  schema_revision: "starclock.goal14-determinism-hardening-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "cross-platform-native-contract-and-local-windows-vectors-frozen",
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
      id,
      execution: "required-native-on-success",
      suites: expectedSuites,
    })),
    ...policy.compile_only_profiles.map((id) => ({
      id,
      execution: "compiled-not-executed",
      suites: [],
    })),
  ],
  evidence_boundary: policy.evidence_boundary,
  contract_sha256: {
    policy: sha256("policy/goal14-determinism-hardening.json"),
    workflow: sha256(".github/workflows/ci.yml"),
    native_runner: sha256("tools/goal14/run-native-ci.mjs"),
  },
};
const relative = "evidence/gold-and-gears-runtime-v1/hardening/determinism-hardening.json";
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(),
    `${relative} is missing; run with --bless`);
  assert(normalizedText(relative) === output,
    `${relative} is stale; run with --bless`);
}
console.log(
  `Goal 14 determinism hardening verified (${policy.native_profiles.length} native profiles, ${expectedSuites.length} suites, ${agent.invalid_actions} rejection cases).`,
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function normalizedText(relative) {
  return text(relative).replaceAll("\r\n", "\n");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function numberLiteral(value) {
  return String(value).replace(/\B(?=(\d{3})+(?!\d))/gu, "_");
}
function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
