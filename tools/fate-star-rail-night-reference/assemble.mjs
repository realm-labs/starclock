#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const packRoot = path.join(root, "content-reference/fate-star-rail-night-v1");
const manifest = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json"));
const peerLock = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/peer-reconciliation-lock.json"));
const partitionNames = [
  "profile-graph.json", "participants.json", "noble-phantasms.json",
  "effects.json", "command-spells.json", "progression-traits.json",
  "fight-flow.json", "pool-audits.json", "battle-bindings.json",
  "encounters.json", "enemies.json", "enemy-programs.json",
];
const partitions = partitionNames.map((name) => ({
  name, bytes: fs.readFileSync(path.join(packRoot, name)),
  document: json(path.join(packRoot, name)),
}));
const records = partitions.flatMap(({ document }) => document.records);

const policies = manifest.obligations
  .filter(({ disposition }) => disposition === "ResearchRequired")
  .map((obligation) => ({
    policy_id: `fate-star-rail-night.policy.${slug(obligation.obligation_id)}`,
    obligation_id: obligation.obligation_id,
    unavailable_fact: `Typed operation meaning for ${obligation.family} ${obligation.locator}`,
    selected_policy: "IdentityOnlyNoOperationLowering",
    rejected_alternatives: ["AssumeScalarMatchDefinesOperation", "DiscardObligation"],
    rationale: "The scalar reference is exact, but no released typed field or program proves its operation semantics.",
    affected_fixtures: [`fate-star-rail-night.fixture.${slug(obligation.family)}`],
    replacement_condition: "Replace when a released typed reference, configuration program, or reproducible observation proves the operation and ordering semantics.",
    evidence_quality: "ProjectPolicy",
  }))
  .sort((left, right) => compareText(left.policy_id, right.policy_id));
const policyIds = new Set(policies.map(({ obligation_id: id }) => id));

const recordReceipts = new Set(records.flatMap(({ source_refs: refs }) => refs)
  .map((source) => receiptKey(source.path, source.locator, source.sha256)));
const coverageRows = manifest.obligations.map((obligation) => {
  const covered = recordReceipts.has(receiptKey(obligation.source_path,
    obligation.locator, obligation.source_sha256));
  assert(covered, `uncovered obligation ${obligation.obligation_id}`);
  const resolved = obligation.disposition === "ResearchRequired" &&
    policyIds.has(obligation.obligation_id);
  return {
    obligation_id: obligation.obligation_id,
    ownership: obligation.ownership,
    manifest_disposition: obligation.disposition,
    final_disposition: resolved ? "DataReadyPolicyBound" : obligation.disposition,
    normalized: true,
    policy_id: resolved
      ? policies.find(({ obligation_id: id }) => id === obligation.obligation_id).policy_id
      : "",
  };
});

const sourceMap = new Map();
for (const record of records)
  for (const source of record.source_refs) {
    const key = receiptKey(source.path, source.locator, source.sha256);
    sourceMap.set(key, {
      source_id: `source.${digest(key).slice(0, 24)}`,
      repository_or_url: source.path.startsWith("Config/") ||
        source.path.startsWith("ExcelOutput/")
        ? "https://gitlab.com/Dimbreath/turnbasedgamedata.git"
        : "starclock-repository",
      revision_or_access_date: source.path.startsWith("Config/") ||
        source.path.startsWith("ExcelOutput/")
        ? "fd978d6ef09f941fba644c731ab54abd6f7c3568" : "2026-08-01",
      game_version: "4.4",
      path_or_page: source.path,
      row_locator: source.locator,
      sha256: source.sha256,
      evidence_quality: "ExactStructured",
      mechanism_quality: "TransportedExact",
      note: "Generated from normalized source references.",
    });
  }
const sources = [...sourceMap.values()].sort((left, right) =>
  compareText(left.source_id, right.source_id));

const enabledFamilies = [...new Set(records.filter(({ enabled }) => enabled)
  .map(({ family }) => family))].sort(compareText);
const fixtures = enabledFamilies.map((family) => {
  const record = records.find((candidate) => candidate.enabled &&
    candidate.family === family);
  return {
    fixture_id: `fate-star-rail-night.fixture.${slug(family)}`,
    mechanic_family: family,
    initial_state: { stable_id: record.stable_id },
    commands: [{ kind: "InspectReferenceFact" }],
    expected_facts: [
      { op: "equals", path: "family", value: family },
      { op: "equals", path: "enabled", value: true },
    ],
    source_refs: record.source_refs,
    mechanism_quality: record.mechanism_quality,
  };
});
for (const family of [...new Set(manifest.obligations
  .filter(({ disposition }) => disposition === "ResearchRequired")
  .map(({ family }) => family))].sort(compareText)) {
  const record = records.find((candidate) =>
    candidate.family === family && candidate.disposition === "ResearchRequired");
  assert(record, `missing policy-bound fixture source for ${family}`);
  fixtures.push({
    fixture_id: `fate-star-rail-night.fixture.${slug(family)}`,
    mechanic_family: family,
    initial_state: { stable_id: record.stable_id },
    commands: [{ kind: "InspectReferenceFact" }],
    expected_facts: [
      { op: "equals", path: "family", value: family },
      { op: "equals", path: "enabled", value: false },
      { op: "equals", path: "disposition", value: "ResearchRequired" },
    ],
    source_refs: record.source_refs,
    mechanism_quality: "PolicyBoundary",
  });
}
fixtures.sort((left, right) => compareText(left.fixture_id, right.fixture_id));

const peerManifestPaths = findPeerManifests();
const localTriples = new Map(manifest.obligations.map((obligation) => [
  receiptKey(obligation.source_path, obligation.locator, obligation.source_sha256),
  obligation.obligation_id,
]));
const localLocators = new Map(manifest.obligations.map((obligation) => [
  locatorKey(obligation.source_path, obligation.locator),
  { obligation_id: obligation.obligation_id, sha256: obligation.source_sha256 },
]));
const reconciliation = peerManifestPaths.map((relative) => {
  const bytes = fs.readFileSync(path.join(root, relative));
  const document = JSON.parse(bytes.toString("utf8"));
  const triples = collectTriples(document);
  const peerLocators = collectLocators(document);
  const matches = [...triples].filter((key) => localTriples.has(key)).sort();
  const conflicts = [...peerLocators].flatMap(([key, digests]) => {
    const local = localLocators.get(key);
    if (!local || digests.has(local.sha256)) return [];
    return [{
      local_obligation_id: local.obligation_id,
      locator: key,
      local_sha256: local.sha256,
      peer_sha256: [...digests].sort(compareText),
    }];
  }).sort((left, right) => compareText(left.locator, right.locator));
  return {
    peer_goal: relative.split("/")[1],
    peer_manifest: relative,
    peer_manifest_sha256: digest(bytes),
    exact_receipt_matches: matches.map((key) => ({
      local_obligation_id: localTriples.get(key), receipt: key,
      semantic_result: "SharedIdentical",
    })),
    match_count: matches.length,
    conflicts,
    conflict_count: conflicts.length,
    decision: conflicts.length > 0 ? "ConflictingEvidenceDigest"
      : matches.length === 0 ? "DistinctByExactReceipt" : "ReferenceSharedIdentity",
    note: "Compared exact source path + row locator + evidence digest and separately rejected same-locator digest drift; names and ID adjacency were ignored.",
  };
});
for (const peer of peerLock.peers) {
  const expected = {
    peer_goal: peer.peer_goal,
    peer_manifest: peer.peer_manifest,
    peer_manifest_sha256: peer.peer_manifest_sha256,
    exact_receipt_matches: peer.exact_receipt_matches,
    match_count: peer.match_count,
    conflicts: peer.conflicts,
    conflict_count: peer.conflict_count,
    decision: peer.decision,
    note: "Compared exact source path + row locator + evidence digest and separately rejected same-locator digest drift; names and ID adjacency were ignored.",
  };
  const observed = reconciliation.find(({ peer_goal: goal }) =>
    goal === peer.peer_goal);
  if (observed) assert(canonical(observed) === canonical(expected),
    `peer reconciliation drift ${peer.peer_goal}`);
  else reconciliation.push(expected);
}
reconciliation.sort((left, right) => compareText(left.peer_goal, right.peer_goal));

const coverage = {
  schema_revision: "starclock.fate-star-rail-night-coverage.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  manifest_binding: manifest.canonical_obligations_sha256,
  counts: {
    required: coverageRows.length,
    accounted: coverageRows.length,
    data_ready: coverageRows.filter(({ final_disposition }) =>
      final_disposition === "DataReady" || final_disposition === "DataReadyPolicyBound").length,
    evidence_only: coverageRows.filter(({ final_disposition }) =>
      final_disposition === "EvidenceOnly").length,
    policy_bound: coverageRows.filter(({ final_disposition }) =>
      final_disposition === "DataReadyPolicyBound").length,
    unresolved: 0,
  },
  rows: coverageRows,
};

const outputs = new Map([
  ["sources.json", envelope("sources", sources)],
  ["coverage.json", coverage],
  ["research-gaps.json", envelope("policies", policies)],
  ["reconciliation.json", envelope("receipts", reconciliation)],
  ["review-fixtures.json", envelope("fixtures", fixtures)],
]);
const packFiles = [...partitions.map(({ name, bytes, document }) => ({
  path: name, bytes: bytes.length, sha256: digest(bytes), records: document.counts.records,
})), ...[...outputs].map(([name, value]) => {
  const bytes = Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8");
  return { path: name, bytes: bytes.length, sha256: digest(bytes),
    records: Array.isArray(value.records) ? value.records.length : 0 };
})].sort((left, right) => compareText(left.path, right.path));
const packIndex = {
  schema_revision: "starclock.fate-star-rail-night-pack-index.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P2-B5",
  manifest_binding: manifest.canonical_obligations_sha256,
  counts: { files: packFiles.length, normalized_records: records.length,
    fixtures: fixtures.length, sources: sources.length, policies: policies.length,
    reconciliation_receipts: reconciliation.length },
  files: packFiles,
};
packIndex.pack_sha256 = digest(canonical(packFiles));
outputs.set("pack-index.json", packIndex);

for (const [name, value] of outputs) {
  const target = path.join(packRoot, name);
  const serialized = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    assert(fs.existsSync(target), `missing ${name}`);
    assert(fs.readFileSync(target, "utf8") === serialized, `pack drift ${name}`);
  } else fs.writeFileSync(target, serialized);
}
console.log(`Goal 19 reference pack ${check ? "verified" : "wrote"} (` +
  `${coverage.counts.required}/${coverage.counts.accounted}, ${records.length} normalized records, ` +
  `${fixtures.length} fixtures, ${packIndex.pack_sha256}).`);

function envelope(field, rows) {
  return {
    schema_revision: `starclock.fate-star-rail-night-${field}.v1`,
    goal_id: "fate-star-rail-night-reference-v1", batch: "G19-P2-B5",
    manifest_binding: manifest.canonical_obligations_sha256,
    count: rows.length, [field]: rows,
  };
}

function findPeerManifests() {
  return fs.readdirSync(path.join(root, "content-manifests"), { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name !== "fate-star-rail-night-v1")
    .map((entry) => `content-manifests/${entry.name}/content-manifest.json`)
    .filter((relative) => fs.existsSync(path.join(root, relative))).sort(compareText);
}

function collectTriples(value, output = new Set()) {
  if (Array.isArray(value)) for (const entry of value) collectTriples(entry, output);
  else if (value && typeof value === "object") {
    const sourcePath = value.source_path ?? value.path ?? value.relative_path;
    const locator = value.locator ?? value.source_locator ?? value.row_locator;
    const sha256 = value.source_sha256 ?? value.sha256 ??
      value.evidence_digest ?? value.evidence_sha256;
    if (typeof sourcePath === "string" && typeof locator === "string" &&
      typeof sha256 === "string" && /^[a-f0-9]{64}$/u.test(sha256))
      output.add(receiptKey(sourcePath, locator, sha256));
    for (const child of Object.values(value)) collectTriples(child, output);
  }
  return output;
}

function collectLocators(value, output = new Map()) {
  if (Array.isArray(value)) for (const entry of value) collectLocators(entry, output);
  else if (value && typeof value === "object") {
    const sourcePath = value.source_path ?? value.path ?? value.relative_path;
    const locator = value.locator ?? value.source_locator ?? value.row_locator;
    const sha256 = value.source_sha256 ?? value.sha256 ??
      value.evidence_digest ?? value.evidence_sha256;
    if (typeof sourcePath === "string" && typeof locator === "string" &&
      typeof sha256 === "string" && /^[a-f0-9]{64}$/u.test(sha256)) {
      const key = locatorKey(sourcePath, locator);
      const digests = output.get(key) ?? new Set();
      digests.add(sha256);
      output.set(key, digests);
    }
    for (const child of Object.values(value)) collectLocators(child, output);
  }
  return output;
}

function receiptKey(sourcePath, locator, sha256) {
  return `${sourcePath}\u001f${locator}\u001f${sha256}`;
}

function locatorKey(sourcePath, locator) {
  return `${sourcePath}\u001f${locator}`;
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value).sort(compareText)
    .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function json(absolute) {
  return JSON.parse(fs.readFileSync(absolute, "utf8"));
}

function slug(value) {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2")
    .replace(/[^A-Za-z0-9]+/gu, "-").replace(/^-|-$/gu, "").toLowerCase();
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
