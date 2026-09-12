import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { buildDispositionArtifacts, targetDisposition } from "./generate-dispositions.mjs";
import { personaSourceReview } from "./persona-disposition.mjs";

const read = (file) => JSON.parse(fs.readFileSync(new URL(
  `../../content-reference/divergent-universe-v1/${file}.json`, import.meta.url), "utf8"));
const coverage = read("coverage");
const persona = read("persona-source-obligations");
const byId = new Map(persona.map((row) => [row.id, row]));
const sourceCoverage = coverage.filter((row) => row.manifest_category === "persona_source_obligations");

test("all current coverage IDs reach the runtime ledger exactly once without Persona credit", () => {
  const runtime = buildDispositionArtifacts()["runtime-dispositions.json"];
  assert.equal(runtime.obligations.length, 6762);
  assert.deepEqual(runtime.obligations.map((row) => row.obligation_id).sort(),
    coverage.map((row) => row.id).sort());
  const sources = runtime.obligations.filter((row) => row.source_review);
  assert.equal(sources.length, 547);
  assert.equal(sources.filter((row) => row.source_review.selector_proof === "CurrentLayerReference").length, 78);
  for (const row of sources) {
    assert.equal(row.target_disposition, "PendingSourceReview");
    assert.equal(row.runtime_status, "Pending");
    assert.equal(row.catalog_status, "SourceOnly");
    assert.equal(row.source_review.runtime_admission, false);
    const source = byId.get(row.normalized_record_ids[0]);
    assert.deepEqual(row.source_review.source_key, source.source_key);
    assert.deepEqual(row.source_review.parent_sources, source.parent_sources);
  }
  assert.deepEqual(runtime.summary.runtime_status, { Pending: 6578, Terminal: 184 });
  const reopened = runtime.obligations.filter((row) => row.target_disposition === "PendingDispositionReview");
  assert.equal(reopened.length, 4);
  assert.ok(reopened.every((row) => ["equation_keywords", "equation_keyword_params"].includes(row.manifest_category)));
});

test("source review rejects invented admission, broken provenance and missing joins", () => {
  for (const proof of ["CurrentLayerReference", "PendingSelectorProof"]) {
    const source = persona.find((row) => row.selector_proof === proof);
    const entry = sourceCoverage.find((row) => row.normalized_record_ids.includes(source.id));
    assert.ok(personaSourceReview(entry, [source]));
    for (const mutate of [
      (c, r) => { c.state = "DataReady"; r.coverage_state = "DataReady"; },
      (c, r) => { r.runtime_disposition = "ExactIntegrated"; },
      (c, r) => { r.selector_proof = "PrefixMatch"; },
      (c, r) => { r.source_key = []; },
      (c, r) => { r.parent_sources = proof === "CurrentLayerReference" ? [] : ["invented"]; },
      (c, r) => { c.source_evidence_sha256 = "0".repeat(64); },
      (c, r) => { c.source_locator = "invented"; },
      (c, r) => { c.ownership = "Shared"; r.ownership = "Shared"; },
    ]) {
      const c = structuredClone(entry), r = structuredClone(source);
      mutate(c, r);
      assert.throws(() => personaSourceReview(c, [r]));
    }
    assert.throws(() => personaSourceReview(entry, []));
    assert.throws(() => personaSourceReview(entry, [source, source]));
  }
});

test("unknown dispositions reject and unsupported policy/exclusion labels remain pending", () => {
  const row = { id: "test", manifest_category: "equation_keywords", ownership: "DivergentUniverse" };
  assert.throws(() => targetDisposition({ ...row, disposition: "Unknown" }, ["DivergentUniverse"], []));
  assert.throws(() => targetDisposition({ ...row, disposition: "Unknown" }, ["Excluded"], []));
  assert.equal(targetDisposition({ ...row, disposition: "NormalizedPolicyOrExclusionBoundary" },
    ["DivergentUniverse"], []), "PendingDispositionReview");
});
