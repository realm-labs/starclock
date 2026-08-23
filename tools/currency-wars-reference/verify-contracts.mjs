#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/currency-wars-reference/contracts.mjs", "--check", "--root", root],
  { cwd: root, stdio: "inherit" },
);
const manifest = json(
  "content-manifests/currency-wars-v1/content-manifest.json",
);
const schema = json(
  "content-manifests/currency-wars-v1/normalized-schema.json",
);
const authoring = json(
  "content-manifests/currency-wars-v1/authoring-contract.json",
);
const fixtures = json(
  "content-manifests/currency-wars-v1/fixture-contract.json",
);
const manifestSha = digestBytes(fs.readFileSync(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
)));

assert(
  schema.schema_revision === "starclock.currency-wars-normalized-schema.v1"
    && authoring.schema_revision
      === "starclock.currency-wars-authoring-contract.v1"
    && fixtures.schema_revision
      === "starclock.currency-wars-fixture-contract.v1",
  "Currency Wars contract revision drift",
);
assert(
  [schema, authoring, fixtures].every((contract) =>
    contract.goal_id === "currency-wars-reference-v1"
      && contract.bound_content_manifest_sha256 === manifestSha),
  "contract-to-manifest binding drift",
);

const fileNames = schema.files.map(({ file }) => file);
assert(unique(fileNames), "normalized file names are not unique");
assert(schema.files.length >= 90, "normalized family contract unexpectedly shrank");
assert(schema.files.every((entry) =>
  /^[a-z0-9]+(?:-[a-z0-9]+)*\.json$/u.test(entry.file)
    && /^CurrencyWars[A-Z][A-Za-z0-9]+$/u.test(entry.record_kind)
    && /^P[1-4]-B[1-9]$/u.test(entry.phase)
    && entry.ordering_keys.join(",") === "id"
    && entry.required_domain_fields.length > 0),
"normalized file contract contains an invalid entry");

const categoryIds = new Set(Object.keys(manifest.categories));
const mappedCategories = new Set();
for (const entry of schema.files)
  for (const categoryId of entry.manifest_category_inputs) {
    assert(categoryIds.has(categoryId),
      `${entry.file} references unknown manifest category ${categoryId}`);
    mappedCategories.add(categoryId);
  }
assert(
  setEqual(categoryIds, mappedCategories),
  "manifest categories are not all mapped by the normalized schema",
);

const workbookNames = authoring.workbooks.map(({ file }) => file);
assert(
  workbookNames.join(",")
    === "CurrencyWars.xlsx,CurrencyWarsBindings.xlsx,CurrencyWarsReview.xlsx",
  "workbook identity drift",
);
const assigned = authoring.workbooks.flatMap(({ normalized_files: values }) => values);
assert(unique(assigned) && setEqual(new Set(assigned), new Set(fileNames)),
  "normalized files are not partitioned exactly once across workbooks");
assert(
  authoring.authority.authoritative_format === "xlsx"
    && authoring.authority.editor === "python-openpyxl"
    && authoring.authority.schema_exporter_version === "0.6.1"
    && authoring.authority.runtime_loading === false,
  "Excel/openpyxl/Sora authority drift",
);
assert(
  authoring.isolation.project === "config/currency-wars-project.toml"
    && authoring.isolation.generated_reader_root
      === "config/currency-wars-generated/reader/"
    && authoring.isolation.forbidden_outputs.includes("config/generated/")
    && authoring.isolation.forbidden_outputs.includes(
      "config/unknowable-domain-generated/",
    )
    && authoring.isolation.forbidden_outputs.includes(
      "config/divergent-universe-generated/",
    ),
  "isolated Sora/output boundary drift",
);
assert(
  authoring.generation.clean_target_required
    && authoring.generation.double_generation_byte_identical
    && authoring.generation.patch_designer_workbook === false,
  "deterministic workbook generation contract drift",
);

assert(
  schema.common_envelope.required_fields.includes("name_en")
    && schema.common_envelope.required_fields.includes("name_zh_cn")
    && schema.common_envelope.required_fields.includes("source_refs")
    && schema.common_envelope.summary_en.mechanical_only
    && schema.common_envelope.summary_zh_cn.mechanical_only,
  "bilingual/source envelope drift",
);
assert(
  schema.types.canonical_decimal.storage === "string"
    && schema.canonical_encoding.encoding === "UTF-8"
    && schema.canonical_encoding.line_endings === "LF"
    && schema.canonical_encoding.unicode_normalization === "NFC"
    && schema.canonical_encoding.null_policy.includes("never use null"),
  "canonical encoding contract drift",
);
assert(
  schema.types.source_ref.required_fields.join(",") === [
    "source_id",
    "repository",
    "revision",
    "path",
    "locator",
    "sha256",
    "access_date",
    "game_version",
    "evidence_quality",
    "mechanism_quality",
  ].join(","),
  "row source contract drift",
);

const manifestFixtures = manifest.categories.semantic_fixtures.records
  .map(({ id }) => id).sort(compare);
const contractFixtures = fixtures.required_families
  .map(({ id }) => id).sort(compare);
assert(
  JSON.stringify(manifestFixtures) === JSON.stringify(contractFixtures)
    && contractFixtures.length === 28
    && fixtures.required_families.every((family) =>
      family.minimum_cases >= 1 && family.must_cover.length >= 3),
  "semantic fixture family contract drift",
);
assert(
  fixtures.fixture_role.includes("do not claim runtime lowering")
    && fixtures.coverage_rule.candidate_coverage.includes("EvidenceOnly")
    && fixtures.determinism.no_legal_target.includes("explicit"),
  "fixture runtime/candidate/fallback boundary drift",
);

const checkpoints = schema.reconciliation_policy.checkpoints;
assert(checkpoints.length === 4, "reconciliation checkpoint count drift");
const expectedCheckpoints = {
  "gold-and-gears-reference-v1": {
    commit: "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    manifest_sha256:
      "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
    records: 7913,
    required_now: true,
  },
  "swarm-disaster-reference-v1": {
    commit: "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    manifest_sha256:
      "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850",
    records: 6963,
    required_now: true,
  },
  "unknowable-domain-reference-v1": {
    commit: "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    manifest_sha256:
      "7416da5808a771a6c0bc78eb11371f51b4f7abb9cb273dd47123f4842800a758",
    records: 5377,
    required_now: true,
  },
  "divergent-universe-reference-v1": {
    commit: "3071d2c2fa7764c133931756769c9efe7f9dabd2",
    manifest_sha256:
      "5cbfa748406204e2d7a2c10c452ac6a87b3864b76461e85bd449c0739f3fc13e",
    records: 6215,
    required_now: true,
  },
};
for (const checkpoint of checkpoints) {
  const expected = expectedCheckpoints[checkpoint.goal];
  assert(expected && Object.entries(expected).every(
    ([field, value]) => checkpoint[field] === value),
  `${checkpoint.goal} reconciliation checkpoint drift`);
  execFileSync("git", ["cat-file", "-e", `${checkpoint.commit}^{commit}`], {
    cwd: root,
    stdio: "ignore",
  });
  const manifestPath = `content-manifests/${checkpoint.goal.replace("-reference-v1", "-v1")}/content-manifest.json`;
  const committedManifest = execFileSync(
    "git",
    ["show", `${checkpoint.commit}:${manifestPath}`],
    { cwd: root, maxBuffer: 100 * 1024 * 1024 },
  );
  assert(digestBytes(committedManifest) === checkpoint.manifest_sha256,
    `${checkpoint.goal} committed manifest digest drift`);
}
assert(
  schema.reconciliation_policy.join_key.join(",")
    === "source_path,row_locator,evidence_sha256"
    && schema.reconciliation_policy.conflict_behavior.includes("Blocked"),
  "reconciliation join/conflict contract drift",
);

console.log(
  `Currency Wars contracts verified (${schema.files.length} normalized ` +
  `files; ${fixtures.required_families.length} fixture families; three ` +
  `isolated workbooks; four ownership checkpoints).`,
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function digestBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function unique(values) {
  return new Set(values).size === values.length;
}

function setEqual(left, right) {
  return left.size === right.size && [...left].every((value) => right.has(value));
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
