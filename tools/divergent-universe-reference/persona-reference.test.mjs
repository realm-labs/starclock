import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { createContext } from "./lib/common.mjs";
import { personaReferenceRows } from "./persona-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifest = JSON.parse(fs.readFileSync(path.join(root,
  "content-manifests/divergent-universe-v1/content-manifest.json"), "utf8"));
const context = await createContext(root);
const records = manifest.categories.persona_source_obligations.records;

test("all Persona obligations preserve exact source identity without runtime admission", () => {
  const rows = personaReferenceRows(context, records);
  assert.equal(rows.length, 547);
  assert.equal(new Set(rows.map(({ id }) => id)).size, 547);
  assert.equal(rows.filter(({ coverage_state: state }) => state === "Researched").length, 78);
  assert.equal(rows.filter(({ coverage_state: state }) => state === "Cataloged").length, 469);
  const bySource = new Map(records.map((record) => [record.id, record]));
  for (const row of rows) {
    const source = bySource.get(row.source_id);
    assert.equal(row.runtime_disposition, "Unimplemented");
    assert.equal(row.source_refs[0].sha256, source.evidence_sha256);
    assert.equal(row.source_locator, source.source);
    assert.deepEqual(row.source_key, source.source_key);
    assert.deepEqual(row.parent_sources, source.parent_sources);
    assert.equal(row.ownership, source.ownership);
    assert.equal(row.source_refs[0].mechanism_quality, "SourceObligationOnly");
  }
});

test("unknown proofs and invented runtime or ownership promotion reject", () => {
  for (const [field, value] of [["selector_proof", "PrefixMatch"],
    ["runtime_disposition", "ExactIntegrated"], ["ownership", "Shared"]]) {
    const record = structuredClone(records[0]);
    record[field] = value;
    assert.throws(() => personaReferenceRows(context, [record]));
  }
});
