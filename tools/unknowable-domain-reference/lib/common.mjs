import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

export const SOURCE_REVISION = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
export const ACCESS_DATE = "2026-07-22";
export const GAME_VERSION = "4.4";
export const ROW_SCHEMA = "starclock.unknowable-domain-row.v1";

export function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object")
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

export function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

export function cleanText(value) {
  return String(value ?? "")
    .normalize("NFC")
    .replace(/<unbreak>/gu, "")
    .replace(/<\/unbreak>/gu, "")
    .replace(/<color=[^>]+>/gu, "")
    .replace(/<\/color>/gu, "")
    .replace(/<nobr>/gu, "")
    .replace(/<\/nobr>/gu, "")
    .replace(/<[^>]+>/gu, "")
    .replace(/\\n/gu, " ")
    .replace(/\s+/gu, " ")
    .trim();
}

export function decimal(value) {
  if (value === undefined || value === null) return "";
  if (typeof value === "object" && "Value" in value) return decimal(value.Value);
  const text = String(value);
  if (!/^-?(0|[1-9][0-9]*)(\.[0-9]+)?$/u.test(text)) return text;
  if (!text.includes(".")) return text === "-0" ? "0" : text;
  const canonicalText = text.replace(/0+$/u, "").replace(/\.$/u, "");
  return canonicalText === "-0" ? "0" : canonicalText;
}

export function slug(value) {
  return String(value)
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-|-$/gu, "") || "unnamed";
}

export async function createContext(root) {
  const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
  const outputRoot = path.join(root, "content-reference/unknowable-domain-v1");
  const readSource = async (relative) => {
    const raw = await fs.readFile(path.join(sourceRoot, relative), "utf8");
    return JSON.parse(raw.replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"'));
  };
  const textEn = await readSource("TextMap/TextMapEN.json");
  const textZh = await readSource("TextMap/TextMapCHS.json");

  async function table(name) {
    const sourcePath = `ExcelOutput/${name}.json`;
    const rows = await readSource(sourcePath);
    if (!Array.isArray(rows)) throw new Error(`expected source array ${sourcePath}`);
    return rows.map((row, index) => ({
      sourcePath,
      locator: String(index),
      row,
    }));
  }

  function text(reference, locale) {
    const hash = reference?.Hash === undefined ? "" : String(reference.Hash);
    return cleanText((locale === "zh_cn" ? textZh : textEn)[hash] ?? "");
  }

  function sourceRef(entry, evidenceQuality = "ExactStructured", extra = {}) {
    return {
      source_id: `source.goal10.${slug(entry.sourcePath)}.${slug(entry.locator)}`,
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision: SOURCE_REVISION,
      path: entry.sourcePath,
      locator: entry.locator,
      sha256: sha256(canonical(entry.row)),
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: evidenceQuality,
      mechanism_quality: evidenceQuality === "ExactStructured"
        ? "DirectStructured"
        : "PolicyBound",
      ...extra,
    };
  }

  async function policyRef(locator, note, replacementCondition) {
    const relative =
      "content-manifests/unknowable-domain-v1/normalized-schema.json";
    const bytes = await fs.readFile(path.join(root, relative));
    return {
      source_id: `source.goal10.project-policy.${slug(locator)}`,
      repository: "starclock",
      revision: "starclock.unknowable-domain-normalized-schema.v1",
      path: relative,
      locator,
      sha256: sha256(bytes),
      access_date: ACCESS_DATE,
      game_version: GAME_VERSION,
      evidence_quality: "ProjectPolicy",
      mechanism_quality: "PolicyBound",
      note,
      replacement_condition: replacementCondition,
    };
  }

  function envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    ownership = "UnknowableDomain",
    coverageState = "DataReady",
    evidenceQuality = "ExactStructured",
    sourceRefs,
    tags = [],
  }) {
    const row = {
      id,
      schema_revision: ROW_SCHEMA,
      kind,
      name_en: cleanText(nameEn),
      name_zh_cn: cleanText(nameZh),
      summary_en: cleanText(summaryEn),
      summary_zh_cn: cleanText(summaryZh),
      ownership,
      coverage_state: coverageState,
      evidence_quality: evidenceQuality,
      source_refs: sourceRefs,
      tags: [...new Set(tags)].sort(),
    };
    for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
      if (!row[field]) throw new Error(`${id} has empty ${field}`);
    if (!Array.isArray(sourceRefs) || sourceRefs.length === 0)
      throw new Error(`${id} has no source reference`);
    return row;
  }

  return {
    root,
    sourceRoot,
    outputRoot,
    readSource,
    table,
    text,
    sourceRef,
    policyRef,
    envelope,
  };
}

export async function writeOrCheck(context, outputs, check) {
  const entries = [...outputs.entries()].sort(([left], [right]) =>
    left.localeCompare(right));
  await fs.mkdir(context.outputRoot, { recursive: true });
  for (const [name, value] of entries) {
    const encoded = `${JSON.stringify(value, null, 2)}\n`;
    const target = path.join(context.outputRoot, name);
    if (check) {
      const committed = await fs.readFile(target, "utf8");
      if (committed !== encoded) throw new Error(`${name} has generated drift`);
    } else {
      await fs.writeFile(target, encoded, "utf8");
    }
  }
}
