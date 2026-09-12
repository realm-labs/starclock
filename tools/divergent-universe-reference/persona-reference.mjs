import { slug } from "./lib/common.mjs";

// Source-obligation projection only. Unproven rows are not runtime candidates.
export function personaReferenceRows(context, records) {
  return records.map((record) => {
    if (!["CurrentLayerReference", "PendingSelectorProof"].includes(record.selector_proof)
        || record.runtime_disposition !== "Unimplemented")
      throw new Error(`invalid source-only Persona disposition: ${record.id}`);
    const [sourcePath, locator] = record.source.split("#");
    const proven = record.selector_proof === "CurrentLayerReference";
    if (record.ownership !== (proven ? "DivergentUniverse" : "SharedCandidate"))
      throw new Error(`Persona ownership does not match selector proof: ${record.id}`);
    const sourceRef = {
      source_id: `source.du.persona.${slug(record.id)}`,
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
      path: sourcePath, locator, sha256: record.evidence_sha256,
      access_date: "2026-09-12", game_version: "4.4",
      evidence_quality: "ExactStructured", mechanism_quality: "SourceObligationOnly",
      note: `Row hash codec: ${record.evidence_hash_codec}. ${record.interpretation}`,
      replacement_condition: "Replace inferred field roles with released named selectors; preserve exact values and review every candidate independently.",
    };
    return {
      ...context.envelope({
        id: `divergent-universe.persona-source.${slug(record.id)}`,
        kind: "DivergentUniversePersonaSourceObligation",
        nameEn: `Persona Source Obligation ${record.id}`,
        nameZh: `Persona 源记录义务 ${record.id}`,
        summaryEn: proven
          ? "Current area-layer references reach this source row; executable room semantics remain unimplemented."
          : "Source row retained for individual selector proof; neither excluded nor admitted as runtime content.",
        summaryZh: proven
          ? "当前区域与层引用可达此源记录；区域可执行语义仍未实现。"
          : "保留此源记录以逐一核定选择规则；既未排除，也未准入运行时内容。",
        ownership: record.ownership,
        coverageState: proven ? "Researched" : "Cataloged",
        sourceRefs: [sourceRef],
        tags: ["persona", "source-obligation-only", proven ? "current-layer-reference" : "pending-selector-proof"],
      }),
      source_id: record.id, source_locator: record.source,
      source_key: record.source_key, selector_proof: record.selector_proof,
      parent_sources: record.parent_sources, interpretation: record.interpretation,
      runtime_disposition: "Unimplemented",
    };
  }).sort((left, right) => left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}
