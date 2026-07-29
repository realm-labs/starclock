#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  GAME_VERSION,
  SOURCE_REVISION,
  canonical,
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const manifestPath =
  "content-manifests/currency-wars-v1/content-manifest.json";
const schemaPath =
  "content-manifests/currency-wars-v1/normalized-schema.json";
const fixturePath =
  "content-manifests/currency-wars-v1/fixture-contract.json";
const manifest = json(path.join(root, manifestPath));
const schema = json(path.join(root, schemaPath));
const fixtureContract = json(path.join(root, fixturePath));
const manifestSha = sha256(fs.readFileSync(path.join(root, manifestPath)));

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function splitSource(source, fallbackLocator) {
  const match = /^(.*\.json)#([0-9]+)$/u.exec(source);
  if (match) return { sourcePath: match[1], locator: match[2] };
  return { sourcePath: source, locator: fallbackLocator ?? "file" };
}
function manifestRef(category, record) {
  const { sourcePath, locator } = splitSource(record.source, record.id);
  const upstream = sourcePath.startsWith("ExcelOutput/")
    || sourcePath.startsWith("Config/");
  const policy = record.evidence_quality === "ProjectPolicy";
  return {
    source_id: `source.goal12.manifest.${slug(category)}.${slug(record.id)}`,
    repository: upstream
      ? "https://gitlab.com/Dimbreath/turnbasedgamedata.git"
      : "starclock",
    revision: upstream ? SOURCE_REVISION : manifestSha,
    path: sourcePath,
    locator,
    sha256: record.evidence_sha256,
    access_date: ACCESS_DATE,
    game_version: GAME_VERSION,
    evidence_quality: record.evidence_quality,
    mechanism_quality: policy
      ? "PolicyBound"
      : upstream ? "DirectStructured" : "GeneratedContract",
    ...(policy ? {
      note:
        "This row records an explicit reference-pack policy, not an observed runtime fact.",
      replacement_condition:
        "Replace only when released structured data or a reproducible observation supplies the missing join or ordering.",
    } : {}),
  };
}
function sourceKey(ref) {
  return canonical([
    ref.repository,
    ref.revision,
    ref.path,
    ref.locator,
    ref.sha256,
    ref.evidence_quality,
  ]);
}
function sourceStableId(ref) {
  return `currency-wars.source.${sha256(sourceKey(ref)).slice(0, 32)}`;
}
function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  sourceRefs,
  ownership = "CurrencyWars",
  coverageState = "DataReady",
  evidenceQuality = "ExactStructured",
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs,
    ownership,
    coverageState,
    evidenceQuality,
    tags,
  });
}

const phase2Files = new Set(schema.files
  .filter(({ phase }) => phase === "P2-B6")
  .map(({ file }) => file));
const existingRows = [];
const sourceToNormalized = new Map();
const allRefs = new Map();
for (const contract of schema.files) {
  if (phase2Files.has(contract.file)) continue;
  const target = path.join(context.outputRoot, contract.file);
  if (!fs.existsSync(target)) continue;
  const rows = json(target);
  for (const row of rows) {
    existingRows.push({ file: contract.file, row });
    for (const ref of row.source_refs ?? []) {
      allRefs.set(sourceKey(ref), ref);
      const ids = sourceToNormalized.get(sourceKey(ref)) ?? [];
      ids.push(row.id);
      sourceToNormalized.set(sourceKey(ref), ids);
    }
  }
}

const obligationRefs = new Map();
for (const [category, value] of Object.entries(manifest.categories))
  for (const record of value.records) {
    const ref = manifestRef(category, record);
    allRefs.set(sourceKey(ref), ref);
    obligationRefs.set(`${category}\0${record.id}`, ref);
  }
const policyRef = await context.policyRef(
  "phase-2-pack-policy",
  "The reference pack preserves exact source programs and explicit policy gaps without lowering runtime behavior.",
  "Replace a policy field only when released structured data or a reproducible observation provides the missing operation, join or ordering.",
);
allRefs.set(sourceKey(policyRef), policyRef);

const sources = ordered([...allRefs.values()].map((ref) => ({
  ...envelope({
    id: sourceStableId(ref),
    kind: "CurrencyWarsSource",
    nameEn: `${ref.path} at ${ref.locator}`,
    nameZh: `${ref.path} 定位 ${ref.locator}`,
    summaryEn:
      `Auditable Version 4.4 source receipt for ${ref.path} at ${ref.locator}.`,
    summaryZh:
      `Version 4.4 可审计来源回执：${ref.path}，定位 ${ref.locator}。`,
    sourceRefs: [ref],
    ownership: "Shared",
    evidenceQuality: ref.evidence_quality,
    tags: ["provenance", "source"],
  }),
  repository: ref.repository,
  revision: ref.revision,
  path: ref.path,
  locator: ref.locator,
  sha256: ref.sha256,
  mechanism_quality: ref.mechanism_quality,
})));

function mechanicFamily(sourcePath) {
  if (sourcePath.startsWith("Config/")) {
    const parts = sourcePath.split("/");
    return `ConfigurationProgram:${parts[1] ?? "GridFight"}`;
  }
  return `StructuredTable:${path.basename(sourcePath, ".json")}`;
}
function mechanicScope(sourcePath) {
  if (/Battle|Buff|Skill|Monster|Enemy|Ability/u.test(sourcePath))
    return "BattleVisibleOrBattleBoundary";
  return "CrossBattleActivity";
}

const mechanicRecords = manifest.categories.mechanic_rules.records
  .filter(({ ownership }) => ownership !== "EvidenceOnly");
const mechanicSourceFiles = [];
const mechanicRules = [];
const mechanicIdsByManifest = new Map();
for (const record of mechanicRecords) {
  const ref = obligationRefs.get(`mechanic_rules\0${record.id}`);
  const token = sha256(record.id).slice(0, 24);
  const sourceId = `currency-wars.mechanic-source.${token}`;
  const ruleId = `currency-wars.mechanic-rule.${token}`;
  const family = mechanicFamily(ref.path);
  mechanicSourceFiles.push({
    ...envelope({
      id: sourceId,
      kind: "CurrencyWarsMechanicSourceFile",
      nameEn: `Mechanic source ${record.id}`,
      nameZh: `机制来源 ${record.id}`,
      summaryEn:
        `Exact ${family} obligation is preserved for audit and later typed lowering.`,
      summaryZh:
        `精确保留 ${family} 义务，用于审计与后续类型化 lowering。`,
      sourceRefs: [ref],
      tags: ["mechanic", "source-program"],
    }),
    source_path: ref.path,
    source_sha256: ref.sha256,
    mechanic_family: family,
    disposition: "ExactSourceProgramPreservedNoRuntimeLowering",
  });
  mechanicRules.push({
    ...envelope({
      id: ruleId,
      kind: "CurrencyWarsMechanicRule",
      nameEn: `Reference rule ${record.id}`,
      nameZh: `参考规则 ${record.id}`,
      summaryEn:
        "The exact source contribution is retained as a reference-only operation boundary; runtime behavior is intentionally not lowered by this goal.",
      summaryZh:
        "精确保留来源贡献作为仅供参考的操作边界；本目标明确不进行运行时 lowering。",
      sourceRefs: [ref],
      tags: ["mechanic", "reference-only", "runtime-excluded"],
    }),
    scope: mechanicScope(ref.path),
    trigger: ref.path.startsWith("Config/")
      ? "AuthoredConfigurationProgram"
      : "AuthoredStructuredContribution",
    ordered_operations: [{
      kind: "PreserveExactSourceContribution",
      source_id: sourceStableId(ref),
      interpretation: "DeferredToLaterRuntimeGoal",
    }],
    state_lifecycle: "ReferenceOnlyExactSourceBoundary",
    runtime_lowered: false,
  });
  mechanicIdsByManifest.set(record.id, [sourceId, ruleId]);
}

const familyManifest = new Map(
  manifest.categories.semantic_fixtures.records.map((record) =>
    [record.id, record]),
);
const semanticFamilies = [];
const reviewFixtures = [];
const fixtureIdsByManifest = new Map();
for (const family of fixtureContract.required_families) {
  const record = familyManifest.get(family.id);
  if (!record) throw new Error(`missing fixture manifest row ${family.id}`);
  const ref = obligationRefs.get(`semantic_fixtures\0${record.id}`);
  const familyId = `currency-wars.fixture-family.${family.id}`;
  const fixtureId = `currency-wars.review-fixture.${family.id}.base`;
  semanticFamilies.push({
    ...envelope({
      id: familyId,
      kind: "CurrencyWarsSemanticFixtureFamily",
      nameEn: `Fixture family: ${family.id}`,
      nameZh: `语义夹具族：${family.id}`,
      summaryEn:
        `Reference fixture family covering ${family.must_cover.join(", ")}.`,
      summaryZh:
        `参考语义夹具族，覆盖：${family.must_cover.join("、")}。`,
      sourceRefs: [ref],
      evidenceQuality: "ProjectPolicy",
      tags: ["fixture", "semantic-review"],
    }),
    minimum_cases: String(family.minimum_cases),
    must_cover: family.must_cover,
  });
  reviewFixtures.push({
    ...envelope({
      id: fixtureId,
      kind: "CurrencyWarsReviewFixture",
      nameEn: `Base review fixture: ${family.id}`,
      nameZh: `基础审查夹具：${family.id}`,
      summaryEn:
        "A deterministic reference-only review case records the required facts and operation order without executing runtime behavior.",
      summaryZh:
        "确定性的仅参考审查用例记录必需事实与操作顺序，不执行运行时行为。",
      sourceRefs: [ref],
      evidenceQuality: "ProjectPolicy",
      tags: ["fixture", "reference-only"],
    }),
    family_id: familyId,
    source_record_ids: [familyId],
    preconditions: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      kind: "RequiredReviewFact",
      fact,
    })),
    input: {
      kind: "ReferenceReviewBoundary",
      deterministic_seed: "0",
      candidate_order: "StableIdAscending",
    },
    ordered_operations: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      kind: "AssertReferenceFact",
      fact,
    })),
    expected_facts: family.must_cover.map((fact, index) => ({
      ordinal: String(index),
      fact,
      disposition: "MustBeExactOrExplicitPolicy",
    })),
    evidence_refs: [sourceStableId(ref)],
  });
  fixtureIdsByManifest.set(record.id, [familyId, fixtureId]);
}

const gapDefinitions = [
  ["gambit-route-membership", "route.gambit_membership",
    "GridFight routes and both released Gambit names are exact, but no released field joins them.",
    "Keep route-to-Gambit membership policy-bound and do not infer it from order.",
    ["infer from order", "merge both Gambits"],
    "A released Division/route-to-Gambit selector or reproducible observation is published."],
  ["cross-node-carry-reset", "flow.carry_reset",
    "StageRoute and NodeTemplate topology is exact; cross-Node mutation operations are not published.",
    "Preserve fields and require fixtures to declare carry/reset assumptions.",
    ["carry everything", "reset everything"],
    "A released operation program or reproducible transition trace publishes the lifecycle."],
  ["squad-boundary-order", "squad_hp.same_boundary_order",
    "Victory preservation, non-victory loss and zero-HP failure are exact; simultaneous precedence is not.",
    "Use the reviewed victory-first policy only in reference fixtures.",
    ["timeout first", "simultaneous merge"],
    "A reproducible last-enemy and timeout same-boundary observation is available."],
  ["offer-sampling-order", "economy.offer_sampling_order",
    "Offer pools, weights, prices and refresh cost are exact; sampling without replacement order is not.",
    "Sort candidates by stable ID before deterministic weighted review.",
    ["source enumeration order", "unordered sampling"],
    "A released selector program or reproducible seeded offer trace publishes the order."],
  ["position-and-rescue-order", "position.automatic_technique_rescue",
    "Position contributions and defeat-energy ratio are exact; omitted enums and lethal rescue order are incomplete.",
    "Keep omitted positions dual-candidate and review rescue/countdown as explicit policy.",
    ["decode omitted enum", "invent rescue HP"],
    "Released enum metadata or reproducible lethal-boundary observations become available."],
  ["bond-simultaneous-recompute", "bond.simultaneous_recompute",
    "Membership, thresholds and contributions are exact; simultaneous roster ordering is not.",
    "Apply ordered roster mutations, then one deterministic Bond recomputation in fixtures.",
    ["recompute after each unordered mutation", "use hash iteration"],
    "A released operation program or reproducible simultaneous mutation trace is available."],
  ["maximum-star-overflow", "star.maximum_overflow",
    "Authored star states and legal next-state joins are exact; maximum-star duplicate overflow is not.",
    "Retain overflow as an explicit fixture policy without inventing a reward.",
    ["discard silently", "grant inferred currency"],
    "Released overflow data or a reproducible maximum-star purchase observation is available."],
  ["role-build-join", "build.role_to_shared_build",
    "Mode role identities and shared build sources are exact; their role-level join is absent.",
    "Keep shared builds fail closed and preserve account state unchanged.",
    ["join by name", "join by numeric adjacency"],
    "A released role-to-build selector or reproducible owned/trial substitution observation is available."],
  ["investment-operation-order", "investment.operation_order",
    "All direct Augment, Portal, Orb, Projection, Talent and enhancement rows are exact.",
    "Preserve configuration programs and declare offer/activation order in fixtures only.",
    ["infer from file order", "collapse source families"],
    "Released program semantics or reproducible same-boundary activation traces are available."],
  ["gold-coin-structured-id", "economy.gold_coin_id",
    "Released bilingual text proves Gold Coin mechanics; generic resource rows do not identify it.",
    "Use the stable project ID and keep the upstream resource locator unresolved.",
    ["select by numeric ID", "select by icon"],
    "A released structured field explicitly binds Gold Coin to a resource record."],
  ["camp-boss-identity", "encounter.boss_identity",
    "Ten Camps identify BossBattleArea and Camp-wide monster candidates, but not the exact boss.",
    "Retain every Camp candidate and reject inferred boss narrowing.",
    ["match names", "match numeric ranges"],
    "A released BattleArea-to-GridFightMonster join or reproducible boss observation is available."],
  ["configuration-program-semantics", "mechanic.configuration_program",
    "All direct mechanic rows and 984 GridFight configuration files are hash-frozen.",
    "Preserve exact program boundaries without runtime lowering or inferred operations.",
    ["execute untyped JSON", "translate names into handlers"],
    "A later runtime goal supplies reviewed typed lowering with semantic fixture evidence."],
];
const researchGaps = [];
for (const [token, field, knownFacts, selectedPolicy, alternatives,
  replacementCondition] of gapDefinitions) {
  researchGaps.push({
    ...envelope({
      id: `currency-wars.research-gap.${token}`,
      kind: "CurrencyWarsResearchGap",
      nameEn: `Research gap: ${token}`,
      nameZh: `研究缺口：${token}`,
      summaryEn: `${knownFacts} ${selectedPolicy}`,
      summaryZh: `已知事实与选定策略：${knownFacts} ${selectedPolicy}`,
      sourceRefs: [policyRef],
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      tags: ["nonblocking", "research-gap"],
    }),
    field,
    known_facts: [knownFacts],
    selected_policy: selectedPolicy,
    alternatives,
    replacement_condition: replacementCondition,
  });
}

const coverage = [];
for (const [category, value] of Object.entries(manifest.categories))
  for (const record of value.records) {
    const ref = obligationRefs.get(`${category}\0${record.id}`);
    const ids = new Set([
      sourceStableId(ref),
      ...(sourceToNormalized.get(sourceKey(ref)) ?? []),
      ...(category === "mechanic_rules"
        ? mechanicIdsByManifest.get(record.id) ?? []
        : []),
      ...(category === "semantic_fixtures"
        ? fixtureIdsByManifest.get(record.id) ?? []
        : []),
    ]);
    const excluded = record.ownership === "EvidenceOnly";
    coverage.push({
      ...envelope({
        id:
          `currency-wars.coverage.${slug(category)}.${sha256(record.id).slice(0, 24)}`,
        kind: "CurrencyWarsCoverage",
        nameEn: `Coverage ${category}: ${record.id}`,
        nameZh: `覆盖 ${category}：${record.id}`,
        summaryEn: excluded
          ? "The frozen evidence-only obligation is explicitly excluded and cannot promote content."
          : "The frozen obligation resolves to auditable normalized source and semantic records.",
        summaryZh: excluded
          ? "冻结的仅证据义务已明确排除，不能提升为内容。"
          : "冻结义务已解析到可审计的规范化来源与语义记录。",
        sourceRefs: [ref],
        evidenceQuality: record.evidence_quality,
        tags: ["coverage", excluded ? "excluded" : "data-ready"],
      }),
      manifest_category: category,
      manifest_record_id: record.id,
      normalized_record_ids: [...ids].sort(compare),
      state: excluded ? "Excluded" : "DataReady",
    });
  }

const baseOutputs = new Map([
  ["coverage.json", ordered(coverage)],
  ["mechanic-rules.json", ordered(mechanicRules)],
  ["mechanic-source-files.json", ordered(mechanicSourceFiles)],
  ["research-gaps.json", ordered(researchGaps)],
  ["review-fixtures.json", ordered(reviewFixtures)],
  ["semantic-fixture-families.json", ordered(semanticFamilies)],
  ["sources.json", sources],
]);
await writeOrCheck(context, baseOutputs, check);

const presentFiles = schema.files.map(({ file }) => file)
  .filter((file) =>
    file !== "pack-index.json"
      && (baseOutputs.has(file)
        || file === "manifest.json"
        || fs.existsSync(path.join(context.outputRoot, file))));
const recordCounts = {};
for (const file of presentFiles) {
  if (file === "manifest.json") {
    recordCounts[file] = "1";
    continue;
  }
  recordCounts[file] = String(json(path.join(context.outputRoot, file)).length);
}
recordCounts["pack-index.json"] = "1";
const normalizedFiles = [...presentFiles, "pack-index.json"].sort(compare);
const packManifest = [{
  ...envelope({
    id: "currency-wars.manifest.v1",
    kind: "CurrencyWarsManifest",
    nameEn: "Currency Wars normalized manifest",
    nameZh: "货币战争规范化清单",
    summaryEn:
      "Canonical Version 4.4 normalized file membership and record counts.",
    summaryZh:
      "Version 4.4 规范化文件成员关系与记录计数。",
    sourceRefs: [policyRef],
    evidenceQuality: "ProjectPolicy",
    tags: ["manifest", "pack"],
  }),
  content_manifest_sha256: manifestSha,
  normalized_files: normalizedFiles,
  record_counts: Object.fromEntries(Object.entries(recordCounts)
    .sort(([left], [right]) => compare(left, right))),
}];
await writeOrCheck(context, new Map([["manifest.json", packManifest]]), check);

const fileDigests = [];
const stableIdIndex = [];
for (const file of presentFiles.sort(compare)) {
  const bytes = fs.readFileSync(path.join(context.outputRoot, file));
  const rows = JSON.parse(bytes);
  fileDigests.push({
    file,
    bytes: String(bytes.length),
    rows: String(rows.length),
    sha256: sha256(bytes),
  });
  for (const row of rows)
    stableIdIndex.push({ id: row.id, file });
}
stableIdIndex.sort((left, right) =>
  compare(left.id, right.id) || compare(left.file, right.file));
const packDigest = sha256(fileDigests
  .map(({ file, sha256: digest }) => `${file}\0${digest}`)
  .join("\n"));
const packIndex = [{
  ...envelope({
    id: "currency-wars.pack-index.v1",
    kind: "CurrencyWarsPackIndex",
    nameEn: "Currency Wars canonical pack index",
    nameZh: "货币战争规范包索引",
    summaryEn:
      "Canonical file digests and stable-ID locations for the Version 4.4 reference pack.",
    summaryZh:
      "Version 4.4 资料包的规范文件摘要与稳定 ID 定位。",
    sourceRefs: [policyRef],
    evidenceQuality: "ProjectPolicy",
    tags: ["index", "pack"],
  }),
  pack_digest: packDigest,
  file_digests: fileDigests,
  stable_id_index: stableIdIndex,
}];
await writeOrCheck(context, new Map([["pack-index.json", packIndex]]), check);

console.log(
  `Currency Wars pack ${check ? "verified" : "generated"}: ` +
  `${coverage.length} coverage rows, ${mechanicRules.length} mechanic rules, ` +
  `${reviewFixtures.length} fixture families, digest ${packDigest}.`,
);
