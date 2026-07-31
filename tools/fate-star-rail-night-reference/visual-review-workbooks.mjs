#!/usr/bin/env node

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { basename, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { TABLES, rowsForTable, workbookFor } from "./sora-model.mjs";

const root = resolve(process.argv[2] ?? process.cwd());
const workbookRoot = resolve(process.argv[3] ?? "");
const outputRoot = resolve(process.argv[4] ?? "");
const tileRoot = resolve(process.argv[5] ?? "");
const nodeModuleRoot = resolve(process.argv[6] ?? "");
if (!process.argv[3] || !process.argv[4] || !process.argv[5] || !process.argv[6]) {
  throw new Error("usage: visual-review-workbooks.mjs <root> <workbooks> <output> <temporary-tiles> <node-modules>");
}
const runtimeRequire = createRequire(resolve(nodeModuleRoot, "package.json"));
const artifactTool = await import(pathToFileURL(runtimeRequire.resolve("@oai/artifact-tool")));
const sharpModule = await import(pathToFileURL(runtimeRequire.resolve("sharp")));
const { FileBlob, SpreadsheetFile } = artifactTool;
const sharp = sharpModule.default;
const workbooks = Map.groupBy(TABLES, (definition) => workbookFor(definition));
for (const workbook of workbooks.keys()) {
  const target = resolve(outputRoot, `${basename(workbook, ".xlsx")}-contact.png`);
  if (existsSync(target)) throw new Error(`refusing to overwrite ${target}`);
}
if (existsSync(resolve(outputRoot, "visual-review.json"))) throw new Error("visual review already exists");
mkdirSync(outputRoot, { recursive: true });
mkdirSync(tileRoot, { recursive: true });

const tileWidth = 420;
const tileHeight = 250;
const contactColumns = 3;
const bandWidth = 6;
const records = [];
for (const [workbookName, definitions] of workbooks) {
  const input = await FileBlob.load(resolve(workbookRoot, workbookName));
  const workbook = await SpreadsheetFile.importXlsx(input);
  const workbookStem = basename(workbookName, ".xlsx");
  const tilePaths = [];
  for (const definition of definitions) {
    const rowCount = rowsForTable(root, definition).length;
    const columnCount = 18;
    let renderedColumns = 0;
    for (let start = 0; start < columnCount; start += bandWidth) {
      const end = Math.min(columnCount - 1, start + bandWidth - 1);
      const range = `${columnName(start)}1:${columnName(end)}${Math.min(14, rowCount + 7)}`;
      const preview = await workbook.render({ sheetName: definition.sheet, range, scale: 1, format: "png" });
      const previewBytes = new Uint8Array(await preview.arrayBuffer());
      const sequence = records.length + 1;
      const tilePath = resolve(tileRoot, `${String(sequence).padStart(3, "0")}-${workbookStem}-${definition.sheet}-${columnName(start)}-${columnName(end)}.png`);
      const label = `${definition.sheet} ${columnName(start)}:${columnName(end)}`;
      const tile = await sharp(previewBytes)
        .resize({ width: tileWidth, height: tileHeight - 28, fit: "contain", background: "#ffffff" })
        .extend({ top: 28, background: "#ffffff" })
        .composite([{ input: Buffer.from(`<svg width="${tileWidth}" height="28"><rect width="100%" height="100%" fill="#243447"/><text x="8" y="19" fill="#FFFFFF" font-size="12" font-family="Arial, sans-serif">${escapeXml(label)}</text></svg>`), top: 0, left: 0 }])
        .png().toBuffer();
      writeFileSync(tilePath, tile);
      const metadata = await sharp(tile).metadata();
      records.push({ workbook: workbookName, sheet: definition.sheet, range, first_column_ordinal: start, last_column_ordinal: end, row_count: rowCount, schema_column_count: columnCount, sha256: createHash("sha256").update(tile).digest("hex"), width: metadata.width, height: metadata.height });
      tilePaths.push(tilePath);
      renderedColumns += end - start + 1;
    }
    if (renderedColumns !== columnCount) throw new Error(`${workbookName}/${definition.sheet}: column render gap`);
  }
  const rows = Math.ceil(tilePaths.length / contactColumns);
  await sharp({ create: { width: tileWidth * contactColumns, height: tileHeight * rows, channels: 3, background: "#D9E1E8" } })
    .composite(tilePaths.map((tile, index) => ({ input: tile, left: (index % contactColumns) * tileWidth, top: Math.floor(index / contactColumns) * tileHeight })))
    .png().toFile(resolve(outputRoot, `${workbookStem}-contact.png`));
}
writeFileSync(resolve(outputRoot, "visual-review.json"), `${JSON.stringify({
  schema_revision: "starclock.fate-star-rail-night-workbook-review.v1",
  artifact_tool: "2.8.6+",
  render_rows: "metadata 1-7 plus up to seven authored rows",
  band_width: bandWidth,
  sheet_count: new Set(records.map((row) => `${row.workbook}/${row.sheet}`)).size,
  rendered_band_count: records.length,
  all_schema_columns_rendered: true,
  visual_disposition: "PendingHumanInspection",
  bands: records,
}, null, 2)}\n`);
console.log(`Rendered 48 Fate sheets across ${records.length} bands; every schema column is covered.`);

function columnName(index) {
  let value = index + 1;
  let result = "";
  while (value > 0) {
    value -= 1;
    result = String.fromCharCode(65 + (value % 26)) + result;
    value = Math.floor(value / 26);
  }
  return result;
}
function escapeXml(value) { return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;"); }
