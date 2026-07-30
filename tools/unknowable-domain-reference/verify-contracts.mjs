#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/contracts.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const manifestPath =
  "content-manifests/unknowable-domain-v1/content-manifest.json";
const manifest = json(manifestPath);
const schema = json(
  "content-manifests/unknowable-domain-v1/normalized-schema.json",
);
const authoring = json(
  "content-manifests/unknowable-domain-v1/authoring-contract.json",
);
const fixtures = json(
  "content-manifests/unknowable-domain-v1/fixture-contract.json",
);

assert(sha256(manifestPath)
  === "7416da5808a771a6c0bc78eb11371f51b4f7abb9cb273dd47123f4842800a758",
"contracts are not bound to the frozen P0-B3 content manifest");
assert([schema, authoring, fixtures].every(
  ({ bound_content_manifest_sha256: hash }) => hash === sha256(manifestPath),
), "Goal 10 contract manifest binding drift");
assert(schema.schema_revision
  === "starclock.unknowable-domain-normalized-schema.v1"
  && authoring.schema_revision
    === "starclock.unknowable-domain-authoring-contract.v1"
  && fixtures.schema_revision
    === "starclock.unknowable-domain-fixture-contract.v1",
"unsupported Goal 10 contract revision");
assert([schema, authoring, fixtures].every(({ goal_id: goalId }) =>
  goalId === "unknowable-domain-reference-v1"),
"Goal 10 contract identity drift");

const files = schema.files;
assert(files.length === 65, "normalized file-family denominator drift");
assert(unique(files.map(({ file }) => file)), "duplicate normalized file name");
assert(files.every(({
  file,
  record_kind: recordKind,
  phase,
  manifest_category_inputs: inputs,
  ordering_keys: keys,
  required_domain_fields: fields,
}) =>
  /^[a-z0-9-]+\.json$/u.test(file)
    && /^[A-Z][A-Za-z0-9]+$/u.test(recordKind)
    && /^P[12]-B[1-8]$/u.test(phase)
    && Array.isArray(inputs)
    && inputs.length > 0
    && Array.isArray(keys)
    && keys.length > 0
    && Array.isArray(fields)
    && fields.length > 0),
"normalized file contract is incomplete");
const categoryUses = new Map();
for (const file of files)
  for (const categoryId of file.manifest_category_inputs) {
    assert(manifest.categories[categoryId] !== undefined,
      `unknown manifest category input ${categoryId}`);
    categoryUses.set(categoryId, (categoryUses.get(categoryId) ?? 0) + 1);
  }
for (const categoryId of Object.keys(manifest.categories))
  assert((categoryUses.get(categoryId) ?? 0) >= 1,
    `manifest category has no normalized mapping ${categoryId}`);

const requiredEnvelope = [
  "id", "schema_revision", "kind", "name_en", "name_zh_cn", "summary_en",
  "summary_zh_cn", "ownership", "coverage_state", "evidence_quality",
  "source_refs", "tags",
];
assert(JSON.stringify(schema.common_envelope.required_fields)
  === JSON.stringify(requiredEnvelope), "normalized common envelope drift");
assert(JSON.stringify(schema.common_envelope.evidence_quality.enum)
  === JSON.stringify([
    "ExactStructured",
    "ExactPublicText",
    "Observed",
    "ApproximateFromReleasedText",
    "ProjectPolicy",
  ]), "evidence-quality vocabulary drift");
assert(schema.common_envelope.ownership.enum.join(",")
  === "UnknowableDomain,Shared", "ownership vocabulary drift");
assert(schema.common_envelope.coverage_state.enum.includes("Blocked")
  && !schema.common_envelope.coverage_state.enum.includes("GoldenVerified"),
"reference-only coverage vocabulary drift");
assert(schema.types.source_ref.required_fields.join(",") === [
  "source_id", "repository", "revision", "path", "locator", "sha256",
  "access_date", "game_version", "evidence_quality", "mechanism_quality",
].join(","), "row-level evidence envelope drift");
assert(schema.types.source_ref.approximation_rule.includes(
  "note and replacement_condition",
), "approximation source-ref contract drift");

const decimalPattern = new RegExp(schema.types.canonical_decimal.pattern, "u");
for (const valid of ["0", "1", "-1", "0.5", "-0.5", "100.0001"])
  assert(decimalPattern.test(valid), `canonical decimal rejects ${valid}`);
for (const invalid of [
  "-0", "+1", "01", "1.", ".5", "1.0", "1.20", "1e2", "NaN", "Infinity",
])
  assert(!decimalPattern.test(invalid), `canonical decimal accepts ${invalid}`);
assert(schema.canonical_encoding.encoding === "UTF-8"
  && schema.canonical_encoding.line_endings === "LF"
  && schema.canonical_encoding.indent_spaces === 2
  && schema.canonical_encoding.terminal_newline === true
  && schema.canonical_encoding.unicode_normalization === "NFC",
"canonical byte encoding drift");
assert(schema.canonical_encoding.decimal_policy
  === "canonical_decimal strings only"
  && schema.canonical_encoding.array_order.includes("ordering_keys"),
"canonical numeric/order contract drift");

const reconciliation = schema.reconciliation_policy;
assert(reconciliation.checkpoint_proof_path
  === "evidence/unknowable-domain-reference-v1/reconciliation-checkpoints.json",
"reconciliation checkpoint proof path drift");
const checkpointProof = json(reconciliation.checkpoint_proof_path);
assert(checkpointProof.schema_revision
  === "starclock.unknowable-domain-reconciliation-checkpoints.v1"
  && checkpointProof.result === "pass"
  && checkpointProof.checkpoints.length === 2,
"reconciliation checkpoint proof envelope drift");
assert(reconciliation.join_key.join(",")
  === "source_path,row_locator,evidence_sha256",
"reconciliation join key drift");
assert(reconciliation.conflict_behavior.startsWith("Blocked"),
  "reconciliation conflicts do not fail closed");
assert(reconciliation.required_receipt_fields.join(",") === [
  "id", "source_path", "row_locator", "evidence_sha256", "checkpoint_goal",
  "checkpoint_commit", "checkpoint_ownership", "goal10_ownership", "outcome",
  "note",
].join(","), "reconciliation receipt envelope drift");
assert(reconciliation.checkpoints.length === 2,
  "reconciliation checkpoint denominator drift");
const gold = reconciliation.checkpoints.find(({ goal }) =>
  goal === "gold-and-gears-reference-v1");
assert(gold?.required_now === true
  && gold.completion_state === "Complete"
  && gold.checkpoint_transport === "LocalCommittedReleaseRegistration",
"Goal 08 completed checkpoint contract drift");
const goldProof = checkpointProof.checkpoints.find(
  ({ goal }) => goal === "Goal08",
);
assert(goldProof?.commit === gold.commit
  && goldProof.registration_commit === gold.registration_commit
  && goldProof.manifest_sha256 === gold.manifest_sha256,
"Goal 08 compact checkpoint proof drift");
if (gitObjectExists(gold.commit) && gitObjectExists(gold.registration_commit)) {
  execFileSync("git", [
    "merge-base", "--is-ancestor", gold.commit, gold.registration_commit,
  ], { cwd: root, stdio: "ignore" });
  assert(gitBlobSha256(gold.commit,
    "content-manifests/gold-and-gears-v1/content-manifest.json")
    === gold.manifest_sha256,
  "Goal 08 manifest checkpoint drift");
  const goldStatus = execFileSync("git", [
    "show", `${gold.commit}:docs/goals/08-gold-and-gears-reference-data-status.md`,
  ], { cwd: root, encoding: "utf8" }).includes("| State | `Complete` |");
  assert(goldStatus === true, "Goal 08 completion status drift");
}
const swarm = reconciliation.checkpoints.find(({ goal }) =>
  goal === "swarm-disaster-reference-v1");
assert(swarm?.required_now === true
  && swarm.checkpoint_transport === "RemoteBranch",
  "Goal 09 checkpoint is not required");
const swarmProof = checkpointProof.checkpoints.find(
  ({ goal }) => goal === "Goal09",
);
assert(swarmProof?.commit === swarm.commit
  && swarmProof.manifest_sha256 === swarm.manifest_sha256
  && swarmProof.remote_ref === swarm.remote_ancestor,
"Goal 09 compact checkpoint proof drift");
if (gitObjectExists(swarm.commit)) {
  if (gitObjectExists(swarm.remote_ancestor))
    execFileSync("git", [
      "merge-base", "--is-ancestor", swarm.commit, swarm.remote_ancestor,
    ], { cwd: root, stdio: "ignore" });
  assert(gitBlobSha256(swarm.commit,
    "content-manifests/swarm-disaster-v1/content-manifest.json")
    === swarm.manifest_sha256,
  "Goal 09 manifest checkpoint drift");
}

assert(authoring.authority.authoritative_format === "xlsx"
  && authoring.authority.editor === "python-openpyxl"
  && authoring.authority.editor_version === "3.1.5"
  && authoring.authority.schema_exporter_version === "0.3.0"
  && authoring.authority.production_artifact === "sora"
  && authoring.authority.json_role === "research-staging-debug-only"
  && authoring.authority.runtime_loading === false,
"Excel/Sora authoring authority drift");
const workbookNames = authoring.workbooks.map(({ file }) => file);
assert(workbookNames.join(",") === [
  "UnknowableDomain.xlsx",
  "UnknowableDomainBindings.xlsx",
  "UnknowableDomainReview.xlsx",
].join(","), "workbook-family denominator drift");
const workbookFiles = authoring.workbooks.flatMap(
  ({ normalized_files: normalized }) => normalized,
);
assert(unique(workbookFiles) && workbookFiles.length === files.length,
  "normalized file appears in multiple workbook families");
assert(JSON.stringify([...workbookFiles].sort())
  === JSON.stringify(files.map(({ file }) => file).sort()),
"workbook families do not cover every normalized file");
assert(authoring.workbooks.map(({ normalized_files: normalized }) =>
  normalized.length).join(",") === "36,19,10",
"workbook partition count drift");
assert(authoring.isolation.project === "config/unknowable-domain/project.toml"
  && authoring.isolation.workbook_root.startsWith("config/unknowable-domain/")
  && authoring.isolation.generated_root
    === "config/unknowable-domain-generated/"
  && authoring.isolation.generated_reader_root.startsWith(
    "config/unknowable-domain-generated/",
  )
  && authoring.isolation.forbidden_outputs.every((output) =>
    !output.startsWith("config/unknowable-domain/")),
"Goal 10 authoring paths are not isolated");
assert(authoring.generation.overwrite_existing_target === false
  && authoring.generation.patch_designer_workbook === false
  && authoring.generation.double_generation_byte_identical === true,
"workbook generation safety drift");
assert(authoring.sheet_contract.data_start_row === 8
  && authoring.sheet_contract.freeze_panes === "A8"
  && authoring.sheet_contract.canonical_decimal_cells === "text",
"workbook sheet contract drift");
assert(authoring.acceptance.reader_load.includes("isolated reader")
  && authoring.acceptance.visual_review.includes("every authored sheet"),
"generated-reader/visual acceptance contract drift");
assert(authoring.reconciliation_sheet.workbook === "UnknowableDomainReview.xlsx"
  && authoring.reconciliation_sheet.conflict_behavior.includes("Block"),
"workbook reconciliation contract drift");

const manifestFamilies = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id).sort();
const contractFamilies = fixtures.required_families.map(({ id }) => id).sort();
assert(JSON.stringify(manifestFamilies) === JSON.stringify(contractFamilies),
  "semantic fixture-family denominator drift");
assert(fixtures.minimum_cases_per_family === 1
  && fixtures.required_families.length === 24
  && fixtures.required_families.every(
    ({ minimum_cases: minimum, must_cover: cover }) =>
      minimum >= 1 && Array.isArray(cover) && cover.length >= 3,
  ), "semantic fixture minimum contract drift");
assert(fixtures.fixture_role.includes("no runtime executability claim"),
  "review fixture contract claims runtime execution");
assert(fixtures.approximation.exact_claim_forbidden === true
  && fixtures.approximation.required_fields.join(",")
    === "note,replacement_condition",
"approximation replacement contract drift");
assert(fixtures.determinism.random_selection.includes("explicit seed")
  && fixtures.determinism.no_legal_target.includes("explicit expected fallback")
  && fixtures.determinism.external_outcomes.includes("abstract Adventure"),
"semantic fixture determinism/Adventure contract drift");
assert(fixtures.coverage_rule.reconciliation_coverage.includes("Goal 08/09"),
  "fixture coverage omits cross-goal reconciliation");

console.log(
  "Unknowable Domain normalized/authoring/fixture contracts verified " +
  "(65 files; 3 isolated workbooks; 24 semantic families; Goal 08/09 receipts).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function gitObjectExists(commit) {
  return spawnSync("git", ["cat-file", "-e", `${commit}^{commit}`], {
    cwd: root,
  }).status === 0;
}
function gitBlobSha256(commit, relative) {
  const bytes = execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 16 * 1024 * 1024,
  });
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
