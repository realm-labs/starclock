import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const bless = process.argv.slice(2).includes("--bless");
assert(process.argv.slice(2).every((argument) => argument === "--bless"), "usage: verify-property-hardening.mjs [--bless]");
const policyBytes = fs.readFileSync(path.join(root, "policy/property-hardening.json"));
const policy = JSON.parse(policyBytes);
assert(policy.schema_revision === "starclock.property-hardening.v1", "unsupported property policy revision");
assert(policy.proptest_cases === 256 && policy.max_shrink_iterations === 4096, "property case or shrink budget changed without review");
assert(policy.families.length === 5, "property family inventory changed without review");

const coverage = new Set();
const seeds = new Set();
const families = policy.families.map((family) => {
  const file = path.join(root, family.source);
  assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(), `${family.id}: missing source`);
  const source = fs.readFileSync(file, "utf8").replaceAll("\r\n", "\n");
  for (const marker of family.required_markers) assert(source.includes(marker), `${family.id}: required hardening marker is absent: ${marker}`);
  for (const seed of family.seeds) {
    assert(source.includes(seed), `${family.id}: fixed seed ${seed} is absent`);
    assert(!seeds.has(seed), `${family.id}: seed ${seed} is reused`);
    seeds.add(seed);
  }
  for (const item of family.coverage) coverage.add(item);
  return {
    id: family.id,
    source: family.source,
    normalized_sha256: sha(source),
    seeds: family.seeds,
    coverage: family.coverage,
  };
});
for (const required of ["invalid commands", "state rollback", "selector validity", "effect duration timing", "content compilation", "arbitrary bytes", "512-command replay"]) {
  assert(coverage.has(required), `required property coverage is absent: ${required}`);
}
for (const relative of policy.failure_persistence) {
  const text = fs.readFileSync(path.join(root, relative), "utf8");
  assert(text.includes("Commit") && text.includes("failure"), `${relative}: corpus retention contract is incomplete`);
}

const report = {
  schema_revision: "starclock.goal01.property-hardening-evidence.v1",
  policy_sha256: sha(Buffer.from(policyBytes.toString("utf8").replaceAll("\r\n", "\n"))),
  fixed_seed_count: seeds.size,
  proptest_cases: policy.proptest_cases,
  max_shrink_iterations: policy.max_shrink_iterations,
  long_sequence_limits: { battle_commands: 512, rollback_prefix: 256, replay_records: 256, replay_payload_bytes: 1024, arbitrary_decoder_bytes: 4096, production_builds: 1024 },
  families,
  failure_persistence: policy.failure_persistence,
};
const output = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/core-combat-v1/hardening/property-hardening.json";
const outputPath = path.join(root, relative);
if (bless) fs.writeFileSync(outputPath, output);
else {
  assert(fs.existsSync(outputPath), `${relative}: missing; run with --bless`);
  assert(fs.readFileSync(outputPath, "utf8") === output, `${relative}: stale; run with --bless`);
}
console.log(`Property hardening verified (${sha(output)}; ${families.length} families, ${seeds.size} fixed seeds).`);

function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
