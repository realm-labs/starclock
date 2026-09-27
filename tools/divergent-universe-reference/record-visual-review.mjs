#!/usr/bin/env node

// Record current workbook identities only after inspecting the rendered images.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const [renderDirectory, reviewedAt, confirmation, ...extra] = process.argv.slice(2);
assert(renderDirectory && /^\d{4}-\d{2}-\d{2}$/.test(reviewedAt ?? "")
  && confirmation === "--confirm-reviewed" && extra.length === 0,
"usage: record-visual-review.mjs RENDER_DIRECTORY YYYY-MM-DD --confirm-reviewed");
const root = path.resolve(import.meta.dirname, "../..");
const renderRoot = path.resolve(root, renderDirectory);
const generated = path.join(root, "config/divergent-universe-generated");
const data = path.join(root, "config/divergent-universe/data");
const manifestPath = path.join(renderRoot, "render-manifest.json");
const manifest = json(manifestPath);
const tables = json(path.join(generated, "schema.lock")).schema.tables;
const workbooks = ["DivergentUniverse.xlsx", "DivergentUniverseBindings.xlsx",
  "DivergentUniverseReview.xlsx"].map((file) => ({
  file, sheets: tables.filter((table) => table.source.file === file)
    .map((table) => table.source.sheet),
}));
const expectedSheets = workbooks.flatMap(({ file, sheets }) =>
  sheets.map((sheet) => ({ file, sheet })));
assert(manifest.sheet_count === tables.length
  && JSON.stringify(manifest.sheets.map(({ file, sheet }) => ({ file, sheet })))
    === JSON.stringify(expectedSheets), "rendered workbook/sheet set differs");
assert(manifest.contact_sheets.length === Math.ceil(tables.length / 8),
  "contact-sheet denominator differs");
for (const item of [...manifest.sheets, ...manifest.contact_sheets]) {
  assert(path.basename(item.image) === item.image,
    "rendered image must be a direct file in render directory");
  assert(sha256(path.join(renderRoot, item.image)) === item.sha256,
    `rendered image changed: ${item.image}`);
}
const python = process.env.STARCLOCK_PYTHON
  ?? (process.platform === "win32" ? "python" : "python3");
const semantic = spawnSync(python, ["-c",
  "from pathlib import Path; from workbook_authoring import semantic_digest; "
    + "import sys, openpyxl; assert openpyxl.__version__ == '3.1.5'; "
    + "print(semantic_digest(Path(sys.argv[1])))", data], {
  cwd: path.join(root, "tools/divergent-universe-reference"),
  encoding: "utf8", env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" },
});
assert(semantic.status === 0, `workbook digest failed: ${semantic.stderr}`);
assert(/^[a-f0-9]{64}$/.test(semantic.stdout.trim()), "invalid workbook digest");
const debugDirectory = path.join(generated, "debug-json");
const debugFiles = fs.readdirSync(debugDirectory).filter((name) => name.endsWith(".json"))
  .toSorted();
let rows = 0;
let empty = 0;
for (const table of tables) {
  const count = json(path.join(debugDirectory, `${table.name}.json`)).table.rows.length;
  rows += count;
  if (count === 0) empty += 1;
}
const debugDigest = crypto.createHash("sha256");
for (const file of debugFiles) {
  debugDigest.update(file).update("\0")
    .update(fs.readFileSync(path.join(debugDirectory, file))).update("\0");
}
const bundle = path.join(generated, "config.sora");
const review = {
  schema_revision: "starclock.divergent-universe-visual-review.v1",
  reviewed_at: reviewedAt,
  renderer: manifest.renderer,
  workbooks,
  sheet_count: tables.length,
  render_manifest_sha256: sha256(manifestPath),
  contact_sheet_sha256: manifest.contact_sheets.map(({ sha256: digest }) => digest),
  workbook_sha256: Object.fromEntries(workbooks.map(({ file }) =>
    [file, sha256(path.join(data, file))])),
  workbook_semantic_sha256: semantic.stdout.trim(),
  sora_bundle: { bytes: fs.statSync(bundle).size, sha256: sha256(bundle),
    tables: tables.length, rows, verified_empty_tables: empty },
  debug_export: { files: debugFiles.length, sha256: debugDigest.digest("hex"),
    digest_algorithm: "SHA-256 over sorted filename, NUL, file bytes, NUL" },
  checks: { all_sheets_rendered: true, metadata_rows_present: true,
    headers_legible: true, data_rows_present_or_verified_empty: true,
    alternating_fill_present: true, long_value_rows_capped_at_72_points: true,
    no_render_corruption: true },
  defects: [],
};
fs.writeFileSync(path.join(root, "evidence/divergent-universe-reference-v1/visual-review.json"),
  `${JSON.stringify(review, null, 2)}\n`);
console.log(`Recorded explicitly confirmed visual review for ${tables.length} sheets.`);

function json(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
function assert(condition, message) { if (!condition) throw new Error(message); }
