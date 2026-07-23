import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => option === "--bless"), "usage: verify-determinism-hardening.mjs [root] [--bless]");
const bless = options.includes("--bless");
const policy = json("policy/goal04-determinism-hardening.json");
assert(policy.schema_revision === "starclock.goal04-determinism-hardening.v1", "unexpected determinism-hardening policy revision");

const inherited = json("policy/ci-matrix.json");
assert(equal(inherited.native_profiles.map((profile) => profile.id), policy.native_profiles), "native profile inventory drift");
assert(equal(inherited.compile_only_profiles.map((profile) => profile.id), policy.compile_only_profiles), "compile-only profile inventory drift");
const workflow = normalizedText(".github/workflows/ci.yml");
const foundationGate = "run: node tools/goal04/run-native-ci.mjs --foundation";
const hardeningGate = `run: ${policy.native_gate}`;
assert(workflow.includes(foundationGate) && workflow.includes(hardeningGate), "workflow omits a Goal 04 native gate");
assert(workflow.indexOf(hardeningGate) > workflow.indexOf(foundationGate), "hardening must run after the frozen foundation gate");

const runner = normalizedText("tools/goal04/run-native-ci.mjs");
for (const marker of [
  "verify-seeded-matrix.mjs", "activity_hardening", "property_contract", "battle_property_contract",
  "activity_replay", "activity_session_loop", "verify-generated-drift.mjs",
  "goal-hardening/verify-release-contract.mjs", "verify-goal02-release-contract.mjs",
  "universe-reference/verify-release.mjs",
]) assert(runner.includes(marker), `native hardening gate omits ${marker}`);

const suiteIds = policy.suites.map((suite) => suite.id);
assert(equal(suiteIds, ["universe-seeded-matrix", "activity-rng-isolation", "replay-property-corpora", "clean-generated-drift", "prior-release-compatibility"]), "hardening suite inventory drift");
const suites = policy.suites.map((suite) => ({
  id: suite.id,
  targets: suite.targets.map((target) => {
    assert(fs.statSync(path.join(root, target), { throwIfNoEntry: false })?.isFile(), `${suite.id}: missing ${target}`);
    return { path: target, sha256: sha256(target) };
  }),
}));

const seeded = json("evidence/standard-universe-runtime-v1/hardening/seeded-matrix.json");
const seededPolicy = policy.suites.find((suite) => suite.id === "universe-seeded-matrix");
assert(sha256("evidence/standard-universe-runtime-v1/hardening/seeded-matrix.json") === seededPolicy.expected_evidence_sha256, "seeded matrix evidence digest drift");
assert(seeded.matrix.coverage.worlds === 9 && seeded.matrix.coverage.difficulties === 33, "seeded matrix coverage drift");
assert(seeded.matrix.coverage.distinct_path_options === 9 && seeded.matrix.coverage.complete_runs === 33, "seeded matrix Path/run coverage drift");

const activitySource = text("crates/starclock-activity/tests/activity_hardening.rs");
const rngSuite = policy.suites.find((suite) => suite.id === "activity-rng-isolation");
assert(activitySource.includes(`0..${numberLiteral(rngSuite.invalid_commands)}_u32`), "invalid-command property denominator drift");
assert(activitySource.includes("for perturbed_label in ActivityRngLabel::ALL"), "not every Activity RNG stream is perturbed");
assert(activitySource.includes(`1..=${rngSuite.draws_per_perturbation}_u16`), "RNG perturbation draw denominator drift");
assert(json("policy/goal04-activity-hardening.json").corpora.rng_streams === rngSuite.streams, "RNG stream denominator drift");

const replaySuite = policy.suites.find((suite) => suite.id === "replay-property-corpora");
for (const target of ["crates/starclock-replay/tests/property_contract.rs", "crates/starclock-replay/tests/battle_property_contract.rs"])
  assert(text(target).includes(`cases: ${replaySuite.proptest_cases_per_property}`), `${target} property case denominator drift`);
assert(text("crates/starclock-agent-api/tests/activity_session_loop.rs").includes(`CORPUS_CASES: usize = ${replaySuite.full_activity_corruption_cases}`), "full Activity replay corruption corpus drift");

const generated = json("policy/generated-drift.json");
const cleanSuite = policy.suites.find((suite) => suite.id === "clean-generated-drift");
const cacheDependent = generated.checks.filter((check) => check.requires === "source-cache").length;
assert(cacheDependent === cleanSuite.cache_dependent_checks, "cache-dependent regeneration denominator drift");
assert(generated.checks.length - cacheDependent === cleanSuite.cache_independent_checks, "clean regeneration denominator drift");
assert(generated.checks.some((check) => equal(check.command, ["node", "tools/goal04/verify-determinism-hardening.mjs", "."])), "generated drift does not own Goal 04 hardening evidence");

const prior = policy.suites.find((suite) => suite.id === "prior-release-compatibility");
assert(equal(prior.goals, ["core-combat-v1", "agent-control-mcp-v1", "standard-universe-reference-v1"]), "prior Goal inventory drift");
const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P6-B2` \| `(InProgress|Complete)` \|/m.test(status), "G04-P6-B2 is not active or complete");

const report = {
  schema_revision: "starclock.goal04-determinism-hardening-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "cross-platform-native-contract-and-local-hardening-vectors-frozen",
  native_gate: policy.native_gate,
  matrix: {
    worlds: seeded.matrix.coverage.worlds,
    paths: seeded.matrix.coverage.distinct_path_options,
    difficulties: seeded.matrix.coverage.difficulties,
    runs: seeded.matrix.coverage.complete_runs,
    nested_battles: seeded.matrix.coverage.nested_battles,
    replay_sha256: seededPolicy.expected_evidence_sha256,
  },
  corpora: {
    invalid_commands: rngSuite.invalid_commands,
    rng_streams: rngSuite.streams,
    perturbation_draws_per_stream: rngSuite.draws_per_perturbation,
    proptest_cases_per_property: replaySuite.proptest_cases_per_property,
    full_activity_corruption_cases: replaySuite.full_activity_corruption_cases,
  },
  regeneration: {
    cache_independent_checks: cleanSuite.cache_independent_checks,
    cache_dependent_checks: cleanSuite.cache_dependent_checks,
    source_cache_optional_in_clean_checkout: policy.evidence_boundary.source_cache_is_optional_for_clean_regeneration,
  },
  prior_release_contracts: prior.goals,
  suites,
  profiles: [
    ...policy.native_profiles.map((id) => ({ id, execution: "required-native-on-success", suites: suiteIds })),
    ...policy.compile_only_profiles.map((id) => ({ id, execution: "compiled-not-executed", suites: [] })),
  ],
  evidence_boundary: policy.evidence_boundary,
  contract_sha256: {
    policy: sha256("policy/goal04-determinism-hardening.json"),
    workflow: sha256(".github/workflows/ci.yml"),
    native_runner: sha256("tools/goal04/run-native-ci.mjs"),
  },
  new_registry_packages: [],
};
const relative = "evidence/standard-universe-runtime-v1/hardening/determinism-hardening.json";
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile(), `${relative} is missing; run with --bless`);
  assert(normalizedText(relative) === output, `${relative} is stale; run with --bless`);
}
console.log(`Goal 04 determinism hardening verified (${policy.native_profiles.length} native profiles, ${suiteIds.length} suites, ${cleanSuite.cache_independent_checks} clean checks).`);

function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function normalizedText(relative) { return text(relative).replaceAll("\r\n", "\n"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function numberLiteral(value) { return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, "_"); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
