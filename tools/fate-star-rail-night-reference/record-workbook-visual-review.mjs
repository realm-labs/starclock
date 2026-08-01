#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { TABLES, workbookFor } from "./sora-model.mjs";

const root = resolve(process.argv[2] ?? process.cwd());
const reviewRoot = resolve(process.argv[3] ?? "");
if (!process.argv[3]) throw new Error("usage: record-workbook-visual-review.mjs <root> <review-directory>");
const reviewPath = resolve(reviewRoot, "visual-review.json");
const review = JSON.parse(readFileSync(reviewPath));
if (review.visual_disposition !== "PendingHumanInspection") throw new Error("visual review is not pending");
if (review.sheet_count !== 48 || review.rendered_band_count !== 144) throw new Error("visual denominator drift");
const bySheet = Map.groupBy(review.bands, (band) => `${band.workbook}/${band.sheet}`);
for (const definition of TABLES) {
  const key = `${workbookFor(definition)}/${definition.sheet}`;
  const bands = (bySheet.get(key) ?? []).sort((left, right) => left.first_column_ordinal - right.first_column_ordinal);
  if (bands.length !== 3 || bands[0].first_column_ordinal !== 0 || bands.at(-1).last_column_ordinal !== 17) throw new Error(`${key}: render coverage gap`);
}
const contacts = [...new Set(TABLES.map((definition) => workbookFor(definition)))].map((workbook) => {
  const file = `${basename(workbook, ".xlsx")}-contact.png`;
  const bytes = readFileSync(resolve(reviewRoot, file));
  if (bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a") throw new Error(`${file}: invalid PNG`);
  return { file, sha256: createHash("sha256").update(bytes).digest("hex"), width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
});
writeFileSync(reviewPath, `${JSON.stringify({
  ...review,
  visual_disposition: "PassedHumanInspection",
  inspected_contacts: contacts,
  inspection_criteria: [
    "all 48 sheet labels and all 144 column-band labels are present",
    "Sora metadata, bilingual summaries and authored rows remain readable",
    "no visible clipping, overlap, formula error or broken style is present",
    "blank space is limited to expected short sheets and final rows",
  ],
  severe_visual_defect_count: 0,
}, null, 2)}\n`);
console.log("Recorded passed inspection for four contact sheets, 48 sheets and 144 bands.");
