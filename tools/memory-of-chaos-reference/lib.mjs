import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

export const root = path.resolve(".");
export const sourceCache = path.resolve(
  process.env.STARCLOCK_SOURCE_CACHE
    ?? "/Users/mikai/.codex/source-caches/goal17-memory-of-chaos",
);
export const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
export const sourceRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
export const outputRoot = path.join(root, "content-reference/memory-of-chaos-v1");
export const manifest = losslessJson(await readFile(path.join(
  root,
  "content-manifests/memory-of-chaos-v1/content-manifest.json",
)));

export function assert(condition, message) {
  if (!condition) throw new Error(message);
}

export function assertSource() {
  const head = git(["rev-parse", "HEAD"]).trim();
  assert(head === sourceRevision, `source revision drift: ${head}`);
  assert(git(["status", "--porcelain"]).trim() === "", "source cache is dirty");
}

export function git(args) {
  return execFileSync("git", ["-C", sourceRoot, ...args], {
    encoding: "utf8",
    maxBuffer: 512 * 1024 * 1024,
  });
}

export async function source(relativePath) {
  return losslessJson(await readFile(path.join(sourceRoot, relativePath)));
}

export function losslessJson(bytes) {
  return JSON.parse(Buffer.from(bytes).toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  ));
}

export function digest(value) {
  return createHash("sha256")
    .update(typeof value === "string" ? value : JSON.stringify(value))
    .digest("hex");
}

export function manifestRecord(category, id) {
  const record = manifest.categories[category]?.records.find(
    (candidate) => candidate.id === id,
  );
  assert(record !== undefined, `missing manifest record ${category}:${id}`);
  return record;
}

export function sourceRecordId(category, id) {
  manifestRecord(category, id);
  return `${category}:${id}`;
}

export function structuredRef(category, id, note, mechanismQuality = "ExactRelationship") {
  const record = manifestRecord(category, id);
  return {
    id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: sourceRevision,
    game_version: "4.4",
    path_or_page: record.source_path,
    row_locator: record.row_locator,
    evidence_sha256: record.evidence_sha256,
    quality: record.evidence_quality,
    mechanism_quality: mechanismQuality,
    note,
  };
}

export function textRef(locale, hash, value) {
  const sourcePath = locale === "zh_cn"
    ? "TextMap/TextMapCHS.json"
    : "TextMap/TextMapEN.json";
  return {
    id: `turnbasedgamedata:${sourcePath}:Hash=${hash}`,
    repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: sourceRevision,
    game_version: "4.4",
    path_or_page: sourcePath,
    row_locator: `Hash=${hash}`,
    evidence_sha256: digest({ hash, value }),
    quality: "ExactStructured",
    mechanism_quality: "IdentityCrossCheck",
    note: `Exact released ${locale} display text for this identity.`,
  };
}

export function policyRef(id, note) {
  return {
    id: `starclock:memory-of-chaos-policy:${id}`,
    repository_or_url: "https://github.com/realm-labs/starclock.git",
    revision_or_access_date: "2026-08-01",
    game_version: "4.4",
    path_or_page: "docs/goals/17-memory-of-chaos-reference-data.md",
    row_locator: id,
    evidence_sha256: digest({ id, note }),
    quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
    note,
  };
}

export function approximation({
  knownFacts,
  selectedBehavior,
  rejectedAlternatives,
  rationale,
  fixtures,
  confidence,
  replacementCondition,
}) {
  return {
    known_facts: knownFacts,
    selected_behavior: selectedBehavior,
    rejected_alternatives: rejectedAlternatives,
    rationale,
    affected_fixture_ids: fixtures,
    confidence,
    replacement_condition: replacementCondition,
  };
}

export function record({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  ownership,
  sourceIds,
  evidence,
  tags = [],
  fields = {},
}) {
  return {
    id,
    schema_revision: "starclock.memory-of-chaos-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    game_version: "4.4",
    ownership,
    coverage_state: "DataReady",
    source_record_ids: [...sourceIds].sort(compareText),
    evidence_refs: evidence,
    tags: [...tags].sort(compareText),
    ...fields,
    runtime_executable: false,
  };
}

export function normalizedFile(file, recordKind, records) {
  return {
    schema_revision: "starclock.memory-of-chaos-normalized-file.v1",
    goal_id: "memory-of-chaos-reference-v1",
    profile: "memory-of-chaos-v1",
    file,
    record_kind: recordKind,
    records,
  };
}

export async function writeCanonical(relativePath, value, check) {
  const destination = path.join(outputRoot, relativePath);
  const bytes = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(destination, "utf8");
    assert(existing === bytes, `${relativePath} drift`);
  } else {
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, bytes);
  }
}

export async function writeText(relativePath, value, check) {
  const destination = path.join(root, relativePath);
  const bytes = value.endsWith("\n") ? value : `${value}\n`;
  if (check) {
    const existing = await readFile(destination, "utf8");
    assert(existing === bytes, `${relativePath} drift`);
  } else {
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, bytes);
  }
}

export function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
