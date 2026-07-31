#!/usr/bin/env node

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { basename, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [rootArg, workbookArg, outputArg, tileArg, moduleArg] = process.argv.slice(2);
if (!moduleArg) throw new Error("usage: visual-review-workbooks.mjs <root> <workbooks> <output> <tiles> <node-modules>");
const root = resolve(rootArg);
const workbookRoot = resolve(workbookArg);
const outputRoot = resolve(outputArg);
const tileRoot = resolve(tileArg);
const runtimeRequire = createRequire(resolve(moduleArg, "package.json"));
const artifactTool = await import(pathToFileURL(runtimeRequire.resolve("@oai/artifact-tool")));
const sharp = (await import(pathToFileURL(runtimeRequire.resolve("sharp")))).default;
const { FileBlob, SpreadsheetFile } = artifactTool;
const contract = JSON.parse(readFileSync(resolve(root,
  "content-manifests/memory-of-chaos-v1/authoring-contract.json")));
const lock = JSON.parse(readFileSync(resolve(root,
  "config/memory-of-chaos-generated/schema.lock")));
const bindings = Object.entries(contract.normalized_family_bindings)
  .sort((left, right) => left[1].order - right[1].order);
const filesByWorkbook = new Map();
for (const [file, binding] of bindings) {
  const files = filesByWorkbook.get(binding.workbook) ?? [];
  files.push(file);
  filesByWorkbook.set(binding.workbook, files);
}
const tableBySheet = new Map(lock.schema.tables.map((table) =>
  [`${table.source.file}\u001f${table.source.sheet}`, table]));
for (const workbook of filesByWorkbook.keys()) {
  const target = resolve(outputRoot, `${basename(workbook, ".xlsx")}-contact.png`);
  if (existsSync(target)) throw new Error(`refusing to overwrite ${target}`);
}
mkdirSync(outputRoot, { recursive: true });
mkdirSync(tileRoot, { recursive: true });
const records = [];
const tileWidth = 420;
const tileHeight = 270;
const contactColumns = 4;
const bandWidth = 8;
for (const [workbookName, files] of filesByWorkbook) {
  const workbook = await SpreadsheetFile.importXlsx(await FileBlob.load(resolve(workbookRoot, workbookName)));
  const tiles = [];
  for (const file of files) {
    const binding = contract.normalized_family_bindings[file];
    const sheet = file.replace(/\.json$/u, "").split("-")
      .map((part) => `${part[0].toUpperCase()}${part.slice(1)}`).join("").slice(0, 31);
    const table = tableBySheet.get(`${workbookName}\u001f${sheet}`);
    if (!table) throw new Error(`${workbookName}/${sheet}: schema table missing`);
    const rowCount = JSON.parse(readFileSync(resolve(root,
      "content-reference/memory-of-chaos-v1", file))).records.length;
    const columnCount = table.fields.length + 1;
    let rendered = 0;
    for (let start = 0; start < columnCount; start += bandWidth) {
      const end = Math.min(columnCount - 1, start + bandWidth - 1);
      const range = `${columnName(start)}1:${columnName(end)}${Math.min(14, rowCount + 7)}`;
      const preview = await workbook.render({ sheetName: sheet, range, scale: 1, format: "png" });
      const previewBytes = new Uint8Array(await preview.arrayBuffer());
      const sequence = records.length + 1;
      const tilePath = resolve(tileRoot,
        `${String(sequence).padStart(3, "0")}-${basename(workbookName, ".xlsx")}-${sheet}-${columnName(start)}-${columnName(end)}.png`);
      const label = `${sheet} ${columnName(start)}:${columnName(end)}`
        .replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
      const tile = await sharp(previewBytes)
        .resize({ width: tileWidth, height: tileHeight - 30, fit: "contain", background: "#ffffff" })
        .extend({ top: 30, background: "#ffffff" })
        .composite([{ input: Buffer.from(`<svg width="${tileWidth}" height="30"><rect width="100%" height="100%" fill="#172554"/><text x="8" y="20" fill="#fff" font-size="12" font-family="Arial">${label}</text></svg>`), top: 0, left: 0 }])
        .png().toBuffer();
      writeFileSync(tilePath, tile);
      records.push({ workbook: workbookName, file, sheet, range,
        first_column_ordinal: start, last_column_ordinal: end,
        row_count: rowCount, schema_column_count: columnCount,
        sha256: createHash("sha256").update(tile).digest("hex") });
      tiles.push(tilePath);
      rendered += end - start + 1;
    }
    if (rendered !== columnCount) throw new Error(`${workbookName}/${sheet}: render gap`);
  }
  const contactRows = Math.ceil(tiles.length / contactColumns);
  await sharp({ create: { width: tileWidth * contactColumns,
    height: tileHeight * contactRows, channels: 3, background: "#DBEAFE" } })
    .composite(tiles.map((input, index) => ({ input,
      left: (index % contactColumns) * tileWidth,
      top: Math.floor(index / contactColumns) * tileHeight })))
    .png().toFile(resolve(outputRoot, `${basename(workbookName, ".xlsx")}-contact.png`));
}
writeFileSync(resolve(outputRoot, "visual-review.json"), `${JSON.stringify({
  schema_revision: "starclock.memory-of-chaos-workbook-review.v1",
  artifact_tool: "2.8.6+",
  render_rows: "metadata 1-7 plus up to seven authored rows",
  band_width: bandWidth,
  sheet_count: new Set(records.map((row) => `${row.workbook}/${row.sheet}`)).size,
  rendered_band_count: records.length,
  all_schema_columns_rendered: true,
  visual_disposition: "PendingHumanInspection",
  bands: records,
}, null, 2)}\n`);
console.log(`Rendered ${new Set(records.map((row) => `${row.workbook}/${row.sheet}`)).size} sheets across ${records.length} column bands.`);

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
