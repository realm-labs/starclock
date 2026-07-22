import crypto from "node:crypto";
import { readFile } from "node:fs/promises";

const corpusPath = "evidence/agent-control-mcp-v1/security/hardening-corpus.json";
const corpusBytes = await readFile(corpusPath);
const corpus = JSON.parse(corpusBytes);
const apiPolicy = JSON.parse(await readFile("policy/agent-api-v1.json", "utf8"));
const surfaces = JSON.parse(await readFile("policy/agent-control-surfaces.json", "utf8"));
const test = await readFile("crates/starclock-agent-api/tests/hardening_corpus.rs", "utf8");
const session = await readFile("crates/starclock-agent-api/src/session.rs", "utf8");
const propertySources = new Map();
for (const suite of corpus.property_suites) propertySources.set(suite.source, await readFile(suite.source, "utf8"));
const fail = (message) => { throw new Error(`Agent hardening corpus: ${message}`); };

if (corpus.schema_revision !== "starclock.agent-hardening-corpus.v1" || corpus.seed !== "0x676f322d70356231") fail("corpus identity drift");
const explicitCases = corpus.malformed_requests.length + corpus.tokens.length + corpus.idempotency_mutations.length + corpus.cursors.length + corpus.replay_mutations.length + corpus.settlement.scenario_ids.length + corpus.races.rounds;
if (explicitCases !== corpus.explicit_cases || explicitCases !== 60) fail("explicit corpus count drift");
if (corpus.races.contenders !== 2 || corpus.races.rounds !== 16) fail("race corpus drift");
if (JSON.stringify(corpus.property_suites.map(({ seed, seeds, cases, cases_per_property }) => ({ seed, seeds, cases, cases_per_property }))) !== JSON.stringify([
  { seed: "0x6167656e742d7631", cases: 512 },
  { seeds: ["0x636f6465632d7631", "0x7265706c61792d31", "0x6d616c666f726d31"], cases_per_property: 256 },
  { seed: "0x626174746c652d31", cases: 256 },
])) fail("property seed/count inventory drift");
if (JSON.stringify(corpus.settlement.scenario_ids) !== JSON.stringify(surfaces.standard_scenarios.map(({ stable_id }) => stable_id))) fail("settlement scenario denominator drift");
for (const [field, policyField] of [["maximum_accepted_commands", "max_accepted_commands_per_settlement"], ["maximum_emitted_events", "max_emitted_events_per_settlement"], ["maximum_resolver_operations", "max_resolver_operations_per_settlement"]]) {
  if (corpus.settlement[field] !== apiPolicy.limits[policyField]) fail(`${field} drift`);
}
for (const marker of [
  "malformed_request_and_token_corpus_is_total_and_bounded",
  "conflicting_idempotency_and_cursor_corpus_never_mutates",
  "corrupted_replay_corpus_fails_without_touching_live_state",
  "every_settlement_corpus_path_stays_within_all_three_budgets",
  "seeded_race_corpus_allows_exactly_one_commit_per_round",
  "IdempotencyConflict",
  "verify_replay(&corrupted).is_err()",
  "Barrier::new(3)",
]) {
  if (!test.includes(marker)) fail(`executable corpus is missing ${marker}`);
}
if (!session.includes("AgentUInt::parse(value).map_or_else")) fail("cursor canonical-integer rejection drift");
const expectedProperties = [
  ["crates/starclock-agent-api/tests/schema_property_contract.rs", "0x6167_656e_742d_7631", "PROPERTY_CASES: u32 = 512"],
  ["crates/starclock-replay/tests/property_contract.rs", "MALFORMED_SEED: u64 = 0x6d61_6c66_6f72_6d31", "cases: 256"],
  ["crates/starclock-replay/tests/battle_property_contract.rs", "BATTLE_REPLAY_CORRUPTION_SEED: u64 = 0x6261_7474_6c65_2d31", "cases: 256"],
];
for (const [source, seed, cases] of expectedProperties) {
  const text = propertySources.get(source);
  if (!text?.includes(seed) || !text.includes(cases) || !text.includes("RngAlgorithm::ChaCha")) fail(`${source} reproducibility drift`);
}

const digest = crypto.createHash("sha256").update(corpusBytes).digest("hex");
console.log(`Agent hardening corpus verified (${digest}; 60 explicit cases plus fixed-seed schema/replay properties)`);
