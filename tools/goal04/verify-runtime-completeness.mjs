import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-runtime-completeness-audit.json");
assert(policy.schema_revision === "starclock.goal04-runtime-completeness-audit.v1", "unexpected completeness policy revision");
const dispositions = json("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json");
assert(dispositions.records.length === policy.expected.content_records, "content denominator differs");
assert(dispositions.rules.length === policy.expected.rule_bindings, "rule denominator differs");
assert(dispositions.fixtures.length === policy.expected.semantic_fixtures, "fixture denominator differs");
for (const row of [...dispositions.records, ...dispositions.rules, ...dispositions.fixtures])
  assert(row.implementation_state === "Executable", `${row.id}: enabled runtime obligation is not executable`);

const targetNames = new Set([...dispositions.records, ...dispositions.rules].map((row) => row.target));
assert(targetNames.size === policy.expected.runtime_targets, "runtime target denominator differs");
assert(canonical([...targetNames].sort()) === canonical(Object.keys(policy.target_counts).sort()), "runtime target set differs");
const classificationCounts = { RuntimeReachable: 0, IntentionalMetadata: 0, ExplicitPolicy: 0 };
for (const [target, [expectedContent, expectedRules, classification]] of Object.entries(policy.target_counts)) {
  const content = dispositions.records.filter((row) => row.target === target);
  const rules = dispositions.rules.filter((row) => row.target === target);
  assert(content.length === expectedContent, `${target}: content reachability denominator differs`);
  assert(rules.length === expectedRules, `${target}: rule reachability denominator differs`);
  assert(Object.hasOwn(classificationCounts, classification), `${target}: unknown classification`);
  classificationCounts[classification] += content.length;
  const owners = policy.target_owner_files[target];
  assert(Array.isArray(owners) && owners.length > 0, `${target}: owner files are missing`);
  for (const owner of owners) assert(fs.existsSync(path.join(root, owner)), `${target}: owner file ${owner} is missing`);
}
assert(classificationCounts.RuntimeReachable === policy.expected.runtime_reachable_content, "runtime-reachable content denominator differs");
assert(classificationCounts.IntentionalMetadata === policy.expected.intentional_metadata_content, "metadata content denominator differs");
assert(classificationCounts.ExplicitPolicy === policy.expected.explicit_policy_content, "explicit-policy content denominator differs");
assert(dispositions.rules.length === policy.expected.runtime_reachable_rules, "runtime-reachable rule denominator differs");

const actualLiterals = new Map();
for (const relative of rustSources("crates/starclock-mode-universe/src")) {
  const matches = [...text(relative).matchAll(/"(universe\.[a-z0-9._-]+)"/g)].map((match) => match[1]);
  if (matches.length > 0) actualLiterals.set(relative, [...new Set(matches)].sort());
}
const allowed = new Map(Object.entries(policy.allowed_stable_id_literals).map(([file, values]) => [file, [...values].sort()]));
assert(canonical([...actualLiterals.keys()].sort()) === canonical([...allowed.keys()].sort()), "stable-ID literal owner set differs");
let literalCount = 0;
for (const [relative, values] of actualLiterals) {
  assert(canonical(values) === canonical(allowed.get(relative)), `${relative}: unreviewed stable-ID literal or missing allowlist entry`);
  literalCount += values.length;
}
assert(literalCount === policy.expected.stable_id_literals, "stable-ID literal denominator differs");

const ownerFiles = [...new Set(Object.values(policy.target_owner_files).flat())].sort();
const evidence = {
  schema_revision: "starclock.goal04-runtime-completeness-audit-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "every-enabled-content-rule-and-fixture-is-executable-and-every-target-has-one-reviewed-runtime-or-metadata-owner",
  counts: {
    content_records: dispositions.records.length,
    rule_bindings: dispositions.rules.length,
    semantic_fixtures: dispositions.fixtures.length,
    runtime_reachable_content: classificationCounts.RuntimeReachable,
    intentional_metadata_content: classificationCounts.IntentionalMetadata,
    explicit_policy_content: classificationCounts.ExplicitPolicy,
    runtime_targets: targetNames.size,
    reviewed_stable_id_literals: literalCount,
    target_owner_files: ownerFiles.length
  },
  contracts: {
    unimplemented_enabled_records: 0,
    unimplemented_enabled_rules: 0,
    unimplemented_enabled_fixtures: 0,
    unowned_runtime_targets: 0,
    unreviewed_stable_id_literals: 0,
    generated_rows_public: false,
    runtime_json_or_excel_reads: false
  },
  classifications: classificationCounts,
  target_counts: policy.target_counts,
  digests: {
    runtime_dispositions_sha256: sha256("content-manifests/standard-universe-runtime-v1/runtime-dispositions.json"),
    fixture_audit_sha256: sha256("evidence/standard-universe-runtime-v1/profile/mechanic-fixture-audit.json"),
    policy_sha256: sha256("policy/goal04-runtime-completeness-audit.json")
  },
  source_sha256: Object.fromEntries(ownerFiles.map((relative) => [relative, sha256(relative)])),
  new_registry_packages: []
};
const relativeEvidence = "evidence/standard-universe-runtime-v1/profile/runtime-completeness-audit.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relativeEvidence)), { recursive: true });
  fs.writeFileSync(path.join(root, relativeEvidence), output);
} else {
  assert(fs.existsSync(path.join(root, relativeEvidence)), "runtime completeness evidence is missing; run with --bless");
  assert(text(relativeEvidence).replaceAll("\r\n", "\n") === output, "runtime completeness evidence is stale; run with --bless");
}
console.log(`Goal 04 runtime completeness verified (${dispositions.records.length}/${dispositions.rules.length}/${dispositions.fixtures.length}; ${targetNames.size} targets, ${literalCount} reviewed stable IDs).`);

function rustSources(relativeRoot) {
  const output = [];
  const walk = (relative) => {
    for (const entry of fs.readdirSync(path.join(root, relative), { withFileTypes: true })) {
      const child = path.posix.join(relative.replaceAll("\\", "/"), entry.name);
      if (entry.isDirectory()) walk(child);
      else if (entry.isFile() && entry.name.endsWith(".rs")) output.push(child);
    }
  };
  walk(relativeRoot);
  return output.sort();
}
function canonical(value) { return JSON.stringify(value); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
