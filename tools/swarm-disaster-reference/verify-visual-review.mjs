#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  process.argv[2]
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const evidence = json(
  "evidence/swarm-disaster-reference-v1/visual-review.json",
);
const lock = json("config/swarm-disaster-generated/schema.lock");
const dataRoot = path.join(root, "config", "swarm-disaster", "data");
const expectedWorkbookOrder = [
  "SwarmDisaster.xlsx",
  "SwarmDisasterProgression.xlsx",
  "SwarmDisasterContent.xlsx",
  "SwarmDisasterEvidence.xlsx",
];

assert(
  evidence.schema_revision === "starclock.swarm-disaster-visual-review.v1",
  "visual-review schema revision differs",
);
assert(evidence.reviewed_at === "2026-07-29", "review date differs");
assert(
  evidence.renderer?.name === "@oai/artifact-tool"
    && evidence.renderer.version === "2.8.6"
    && evidence.renderer.range_policy
      === "rows 1-12 across every schema field column",
  "renderer contract differs",
);
assert(
  evidence.schema_lock_sha256
    === sha256(path.join(root, "config/swarm-disaster-generated/schema.lock")),
  "visual review is not bound to the committed schema lock",
);

const expected = [];
for (const table of lock.schema.tables) {
  const source = table.source;
  assert(source?.format === "xlsx", `${table.name} is not XLSX-backed`);
  let workbook = expected.find((entry) => entry.file === source.file);
  if (!workbook) {
    workbook = { file: source.file, sheets: [] };
    expected.push(workbook);
  }
  workbook.sheets.push(source.sheet);
}
assert(
  JSON.stringify(expected.map((entry) => entry.file))
    === JSON.stringify(expectedWorkbookOrder),
  "schema workbook order differs",
);
assert(
  evidence.workbooks.length === expected.length,
  "visual-review workbook count differs",
);
for (let index = 0; index < expected.length; index += 1) {
  const actual = evidence.workbooks[index];
  const wanted = expected[index];
  assert(actual.file === wanted.file, `workbook ${index + 1} order differs`);
  assert(
    JSON.stringify(actual.sheets) === JSON.stringify(wanted.sheets),
    `${actual.file} visual-review sheet order differs`,
  );
  assert(
    actual.sha256 === sha256(path.join(dataRoot, actual.file)),
    `${actual.file} visual-review digest differs`,
  );
}

const sheetCount = evidence.workbooks.reduce(
  (sum, workbook) => sum + workbook.sheets.length,
  0,
);
assert(
  evidence.sheet_count === 65 && sheetCount === evidence.sheet_count,
  "visual-review sheet denominator differs",
);
assert(isSha256(evidence.render_manifest_sha256),
  "render manifest digest is invalid");
assert(
  Array.isArray(evidence.contact_sheets)
    && evidence.contact_sheets.length === 10,
  "contact-sheet evidence must contain ten records",
);
let expectedFirst = 1;
for (const contact of evidence.contact_sheets) {
  assert(
    contact.first_render === expectedFirst
      && contact.last_render >= contact.first_render
      && contact.last_render <= evidence.sheet_count
      && isSha256(contact.sha256),
    "contact-sheet range or digest is invalid",
  );
  expectedFirst = contact.last_render + 1;
}
assert(
  expectedFirst === evidence.sheet_count + 1,
  "contact-sheet ranges do not cover every rendered sheet exactly once",
);

const booleanChecks = [
  "all_sheets_rendered",
  "all_schema_columns_rendered",
  "metadata_rows_present",
  "headers_legible",
  "data_rows_present",
  "alternating_fill_present",
  "long_value_rows_capped",
];
for (const name of booleanChecks)
  assert(evidence.checks?.[name] === true, `${name} is not attested`);
assert(
  evidence.checks?.formula_error_matches === 0
    && evidence.checks.overlap_or_clipping_defects === 0,
  "visual review records unresolved formula or layout defects",
);
assert(
  Array.isArray(evidence.corrective_actions)
    && evidence.corrective_actions.length === 2,
  "visual corrective-action record differs",
);
assert(
  Array.isArray(evidence.defects) && evidence.defects.length === 0,
  "visual review contains unresolved defects",
);

console.log(
  `Swarm Disaster visual review verified (${evidence.workbooks.length} ` +
  `workbooks, ${evidence.sheet_count} sheets, 10 contact sheets, no defects).`,
);

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/u.test(value);
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
