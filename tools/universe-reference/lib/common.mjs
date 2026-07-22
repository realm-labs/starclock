import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

export const SOURCE_REVISION = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
export const ACCESS_DATE = "2026-07-22";
export const GAME_VERSION = "4.4";

export function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

export function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

export function slug(value) {
  return String(value)
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-|-$/gu, "") || "unnamed";
}

export function cleanText(value) {
  return String(value ?? "")
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
  if (!text.includes(".")) return text;
  return text.replace(/0+$/u, "").replace(/\.$/u, "");
}

export async function createContext(root) {
  const sourceRoot = path.join(root, ".cache", "content-reference", "turnbasedgamedata");
  const outputRoot = path.join(root, "content-reference", "standard-universe-v1");
  const readSource = async (relative) => {
    const raw = await readFile(path.join(sourceRoot, ...relative.split("/")), "utf8");
    return JSON.parse(raw.replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"'));
  };
  const textEn = await readSource("TextMap/TextMapEN.json");
  const textZh = await readSource("TextMap/TextMapCHS.json");
  const evidence = new Map();

  async function table(name) {
    const relativePath = `ExcelOutput/${name}.json`;
    const data = await readSource(relativePath);
    return Object.entries(data).map(([sourceKey, row]) => ({ relativePath, sourceKey, row }));
  }

  function text(reference, locale) {
    const hash = reference?.Hash === undefined ? "" : String(reference.Hash);
    return cleanText((locale === "zh_cn" ? textZh : textEn)[hash] ?? "");
  }

  function provenance(entry, quality = "ExactStructured", note = "Released structured row.") {
    const id = `source.standard-su.${slug(entry.relativePath)}.${entry.sourceKey}`;
    if (!evidence.has(id)) {
      evidence.set(id, {
        id,
        source_kind: "ReleasedStructuredData",
        repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
        revision_or_access_date: SOURCE_REVISION,
        game_version: GAME_VERSION,
        relative_path_or_page: entry.relativePath,
        row_locator: entry.sourceKey,
        evidence_sha256: sha256(canonical(entry.row)),
        quality,
        license_note: "Research reference only; no assets or long descriptions redistributed.",
        note,
      });
    }
    return id;
  }

  function publicProvenance({ id, url, page, fact, quality = "ExactPublicText", note = "Public mechanics cross-check." }) {
    const stableId = `source.public.${slug(id)}`;
    if (!evidence.has(stableId)) {
      evidence.set(stableId, {
        id: stableId,
        source_kind: "PublicCrossCheck",
        repository_or_url: url,
        revision_or_access_date: ACCESS_DATE,
        game_version: GAME_VERSION,
        relative_path_or_page: page,
        row_locator: page,
        evidence_sha256: sha256(fact),
        quality,
        license_note: "Mechanic fact cross-check only; page prose is not redistributed.",
        note,
      });
    }
    return stableId;
  }

  function envelope({ id, nameEn, nameZh, summaryEn, summaryZh, entry, quality = "ExactStructured", mechanismQuality = quality, coverageState = "DataReady", modeOwner = "Standard", note = "", sourceIds = [] }) {
    return {
      id,
      enabled: true,
      mode_owner: modeOwner,
      name_en: cleanText(nameEn),
      name_zh_cn: cleanText(nameZh),
      summary_en: cleanText(summaryEn),
      summary_zh_cn: cleanText(summaryZh),
      quality,
      mechanism_quality: mechanismQuality,
      quality_overrides: [],
      coverage_state: coverageState,
      provenance_ids: entry ? [provenance(entry, quality)] : [],
      source_ids: sourceIds.map(String),
      note,
    };
  }

  async function writeJson(name, value) {
    await mkdir(outputRoot, { recursive: true });
    await writeFile(path.join(outputRoot, name), `${JSON.stringify(value, null, 2)}\n`, "utf8");
  }

  return { root, sourceRoot, outputRoot, table, text, provenance, publicProvenance, envelope, evidence, writeJson, readSource };
}

export async function writeOrCheck(ctx, outputs, check) {
  const entries = [...outputs.entries()].sort(([left], [right]) => left.localeCompare(right));
  for (const [name, value] of entries) {
    const encoded = `${JSON.stringify(value, null, 2)}\n`;
    const target = path.join(ctx.outputRoot, name);
    if (check) {
      if (await readFile(target, "utf8") !== encoded) throw new Error(`${name} has generated drift`);
    } else {
      await ctx.writeJson(name, value);
    }
  }
}
