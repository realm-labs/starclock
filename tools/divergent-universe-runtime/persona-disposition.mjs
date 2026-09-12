// Reference admission is separate from runtime ownership and executable behavior.
export function personaSourceReview(coverage, normalized) {
  if (coverage.manifest_category !== "persona_source_obligations") return null;
  const row = normalized[0];
  const proven = row?.selector_proof === "CurrentLayerReference";
  const pending = row?.selector_proof === "PendingSelectorProof";
  const state = proven ? "Researched" : "Cataloged";
  const ownership = proven ? "DivergentUniverse" : "SharedCandidate";
  if (normalized.length !== 1 || (!proven && !pending)
      || row.kind !== "DivergentUniversePersonaSourceObligation"
      || row.runtime_disposition !== "Unimplemented"
      || coverage.disposition !== "NormalizedSourceObligationOnly"
      || coverage.state !== state || coverage.coverage_state !== state
      || row.coverage_state !== state
      || coverage.ownership !== ownership || row.ownership !== ownership
      || coverage.source_locator !== row.source_locator
      || !row.source_key || typeof row.source_key !== "object"
      || Array.isArray(row.source_key) || Object.keys(row.source_key).length === 0
      || !Array.isArray(row.parent_sources)
      || (row.parent_sources.length > 0) !== proven
      || !row.source_refs.some((source) => source.sha256 === coverage.source_evidence_sha256))
    throw new Error(`invalid Persona source-only obligation: ${coverage.id}`);
  return {
    selector_proof: row.selector_proof,
    source_key: row.source_key,
    parent_sources: row.parent_sources,
    runtime_admission: false,
    semantic_family_assignment: "PendingSelectorAndBehaviorReview",
    required_evidence: proven
      ? "Lower the reached room/position semantics and prove production execution before selecting a terminal disposition."
      : "Prove an exact released selector or reviewed row exclusion before assigning runtime membership or behavior.",
  };
}
