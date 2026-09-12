import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// Explicit reviewed file closure. This is candidate retention, not prefix-based
// membership. Row reachability comes only from the independently verified audit.
export const personaTables = new Set([
  "RoguePersonaConstClient.json", "RoguePersonaConstCommon.json",
  "RoguePersonaLayerRoom.json", "RoguePersonaRoomAttribute.json",
  "RoguePersonaRoomCompType.json", "RoguePersonaRoomComposition.json",
  "RoguePersonaRoomPreset.json", "RoguePersonaStyle.json",
  "RoguePersonaStyleGift.json", "RoguePersonaTalent.json",
  "RoguePersonaTalentGroup.json",
]);

export function personaObligations(root, sourceRoot, inventory) {
  const report = JSON.parse(fs.readFileSync(path.join(root,
    "content-manifests/divergent-universe-runtime-v1/persona-reachability.json"), "utf8"));
  if (report.source_revision !== "fd978d6ef09f941fba644c731ab54abd6f7c3568"
      || report.summary.source_tables !== personaTables.size
      || report.rows.length !== 547)
    throw new Error("Persona audit source closure drift");
  const sourceRecords = new Map(inventory.records
    .filter(({ repository }) => repository === "turnbasedgamedata")
    .map((record) => [record.path, record]));
  for (const [name, digest] of Object.entries(report.source_file_sha256)) {
    const file = `ExcelOutput/${name}.json`;
    const actual = crypto.createHash("sha256")
      .update(fs.readFileSync(path.join(sourceRoot, file))).digest("hex");
    if (actual !== digest || sourceRecords.get(file)?.sha256 !== digest)
      throw new Error(`Persona audit source bytes mismatch: ${file}`);
  }
  return report.rows.map((row) => {
    const [file] = row.source.split("#");
    const name = path.basename(file);
    if (!personaTables.has(name)
        || !["CurrentLayerReference", "PendingSelectorProof"].includes(row.reachability)
        || !/^[0-9a-f]{64}$/u.test(row.row_sha256))
      throw new Error(`invalid Persona source obligation: ${row.source}`);
    const proven = row.reachability === "CurrentLayerReference";
    return {
      id: `${name.replace(/\.json$/u, "")}:${Object.values(row.source_key).join(":")}`,
      source: row.source,
      evidence_sha256: row.row_sha256,
      evidence_hash_codec: report.row_hash_codec,
      evidence_quality: "ExactStructured",
      ownership: proven ? "DivergentUniverse" : "SharedCandidate",
      reachability: proven ? "TransitiveReference" : "SourceObligation",
      selector_proof: row.reachability,
      source_key: row.source_key,
      parent_sources: row.parent_sources,
      interpretation: proven
        ? "Reviewed obfuscated current area-layer-preset-composition joins; no room execution is implied."
        : "Retained source obligation; exact current selector and membership still require proof. Not an exclusion or runtime candidate.",
      runtime_disposition: "Unimplemented",
    };
  });
}
