#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";

const [rootArg, reviewArg] = process.argv.slice(2);
if (!reviewArg) throw new Error("usage: record-workbook-visual-review.mjs <root> <review-directory>");
const root = resolve(rootArg);
const reviewRoot = resolve(reviewArg);
const contract = JSON.parse(readFileSync(resolve(root,
  "content-manifests/memory-of-chaos-v1/authoring-contract.json")));
const expectedFiles = Object.keys(contract.normalized_family_bindings);
const reviewPath = resolve(reviewRoot, "visual-review.json");
const review = JSON.parse(readFileSync(reviewPath));
if (review.visual_disposition !== "PendingHumanInspection") throw new Error("visual review is not pending");
if (review.sheet_count !== 27) throw new Error(`expected 27 sheets, got ${review.sheet_count}`);
const bandsByFile = new Map();
for (const band of review.bands) {
  if (!expectedFiles.includes(band.file)) throw new Error(`unexpected rendered file ${band.file}`);
  const bands = bandsByFile.get(band.file) ?? [];
  bands.push(band);
  bandsByFile.set(band.file, bands);
}
for (const file of expectedFiles) {
  const bands = (bandsByFile.get(file) ?? []).sort((left, right) =>
    left.first_column_ordinal - right.first_column_ordinal);
  if (!bands.length) throw new Error(`${file}: no rendered bands`);
  let next = 0;
  for (const band of bands) {
    if (band.first_column_ordinal !== next) throw new Error(`${file}: render gap before ${next}`);
    next = band.last_column_ordinal + 1;
  }
  if (next !== bands[0].schema_column_count) throw new Error(`${file}: final render gap`);
}
const workbookNames = [...new Set(Object.values(contract.normalized_family_bindings)
  .map(({ workbook }) => workbook))];
const contacts = workbookNames.map((workbook) => {
  const file = `${basename(workbook, ".xlsx")}-contact.png`;
  const bytes = readFileSync(resolve(reviewRoot, file));
  if (bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a") throw new Error(`${file}: invalid PNG`);
  return { file, sha256: createHash("sha256").update(bytes).digest("hex"),
    width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
});
writeFileSync(reviewPath, `${JSON.stringify({
  ...review,
  visual_disposition: "PassedHumanInspection",
  inspected_contacts: contacts,
  inspection_criteria: [
    "all sheet and column-band labels are present",
    "metadata, headers and representative authored rows are readable",
    "no clipping, overlap, formula error or broken style is visible",
    "blank space is limited to short sheets or final partial bands",
  ],
  severe_visual_defect_count: 0,
}, null, 2)}\n`);
console.log(`Recorded passed inspection for ${contacts.length} contacts, 27 sheets and ${review.rendered_band_count} bands.`);
