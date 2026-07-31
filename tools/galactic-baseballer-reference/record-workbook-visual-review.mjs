#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const reviewRoot = resolve(process.argv[3] ?? "");
if (!process.argv[3]) {
  throw new Error(
    "usage: record-workbook-visual-review.mjs <root> <review-directory>",
  );
}

const contract = JSON.parse(readFileSync(resolve(
  root,
  "content-manifests/galactic-baseballer-v1/authoring-contract.json",
)));
const schema = JSON.parse(readFileSync(resolve(
  root,
  "content-manifests/galactic-baseballer-v1/normalized-schema.json",
)));
const expectedFiles = new Map(schema.files.map((row) => [row.file, row]));
const reviewPath = resolve(reviewRoot, "visual-review.json");
const review = JSON.parse(readFileSync(reviewPath));

if (review.visual_disposition !== "PendingHumanInspection") {
  throw new Error(
    `visual review is not pending: ${review.visual_disposition}`,
  );
}
if (review.sheet_count !== expectedFiles.size) {
  throw new Error(
    `expected ${expectedFiles.size} rendered sheets, got ${review.sheet_count}`,
  );
}

const bandsByFile = new Map();
for (const band of review.bands) {
  if (!expectedFiles.has(band.file)) {
    throw new Error(`unexpected rendered file ${band.file}`);
  }
  const bands = bandsByFile.get(band.file) ?? [];
  bands.push(band);
  bandsByFile.set(band.file, bands);
}
for (const [file] of expectedFiles) {
  const rows = JSON.parse(readFileSync(resolve(
    root,
    "content-reference/galactic-baseballer-v1",
    file,
  )));
  const authoredFields = new Set(rows.flatMap((row) => Object.keys(row)));
  authoredFields.delete("id");
  const expectedColumnCount = authoredFields.size + 3;
  const bands = bandsByFile.get(file) ?? [];
  bands.sort((left, right) =>
    left.first_column_ordinal - right.first_column_ordinal);
  if (bands.length === 0) throw new Error(`${file}: no rendered bands`);
  let nextColumn = 0;
  for (const band of bands) {
    if (band.first_column_ordinal !== nextColumn) {
      throw new Error(`${file}: render gap before column ${nextColumn}`);
    }
    if (band.schema_column_count !== expectedColumnCount) {
      throw new Error(`${file}: schema column count drift`);
    }
    nextColumn = band.last_column_ordinal + 1;
  }
  if (nextColumn !== expectedColumnCount) {
    throw new Error(`${file}: render stopped at column ${nextColumn}`);
  }
}

const contacts = [];
for (const workbook of contract.workbooks) {
  const file = `${basename(workbook.file, ".xlsx")}-contact.png`;
  const bytes = readFileSync(resolve(reviewRoot, file));
  const pngSignature = "89504e470d0a1a0a";
  if (
    bytes.length < 24
    || bytes.subarray(0, 8).toString("hex") !== pngSignature
  ) {
    throw new Error(`${file}: invalid contact sheet`);
  }
  const width = bytes.readUInt32BE(16);
  const height = bytes.readUInt32BE(20);
  if (!width || !height) throw new Error(`${file}: empty contact sheet`);
  contacts.push({
    file,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    width,
    height,
  });
}

const completed = {
  ...review,
  visual_disposition: "PassedHumanInspection",
  inspected_contacts: contacts,
  inspection_criteria: [
    "all sheet and column-band labels are present",
    "headers, metadata rows, and authored rows remain readable",
    "no visible clipping, overlap, formula error, or broken style is present",
    "blank space is limited to expected narrow final bands or short sheets",
  ],
  severe_visual_defect_count: 0,
};
writeFileSync(reviewPath, `${JSON.stringify(completed, null, 2)}\n`);
console.log(
  `Recorded passed human inspection for ${contacts.length} contact sheets, `
  + `${review.sheet_count} workbook sheets, and `
  + `${review.rendered_band_count} rendered bands.`,
);
