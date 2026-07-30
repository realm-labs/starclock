#!/usr/bin/env node

import { createHash } from "node:crypto";
import {
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { basename, resolve } from "node:path";
import { FileBlob, SpreadsheetFile } from "@oai/artifact-tool";
import sharp from "sharp";

const root = resolve(process.argv[2] ?? process.cwd());
const workbookRoot = resolve(process.argv[3] ?? "");
const outputRoot = resolve(process.argv[4] ?? "");
if (!process.argv[3] || !process.argv[4]) {
  throw new Error(
    "usage: visual-review-workbooks.mjs <root> <workbooks> <output>",
  );
}

const lock = JSON.parse(
  readFileSync(resolve(root, "config/currency-wars-generated/schema.lock")),
);
const tablesByWorkbook = new Map();
for (const table of lock.schema.tables) {
  const sheets = tablesByWorkbook.get(table.source.file) ?? [];
  sheets.push(table.source.sheet);
  tablesByWorkbook.set(table.source.file, sheets);
}

const tileWidth = 400;
const tileHeight = 250;
const columns = 4;
const renderRange = "A1:H14";
const records = [];
mkdirSync(outputRoot, { recursive: true });

for (const [workbookName, sheetNames] of tablesByWorkbook) {
  const input = await FileBlob.load(resolve(workbookRoot, workbookName));
  const workbook = await SpreadsheetFile.importXlsx(input);
  const workbookStem = basename(workbookName, ".xlsx");
  const tilePaths = [];
  for (const [index, sheetName] of sheetNames.entries()) {
    const preview = await workbook.render({
      sheetName,
      range: renderRange,
      scale: 1,
      format: "png",
    });
    const previewBytes = new Uint8Array(await preview.arrayBuffer());
    const safeSheet = sheetName.replaceAll(/[^A-Za-z0-9_-]/g, "_");
    const tilePath = resolve(
      outputRoot,
      `${workbookStem}-${String(index + 1).padStart(3, "0")}-${safeSheet}.png`,
    );
    const label = `${workbookName} / ${sheetName}`;
    const escapedLabel = label
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;");
    const tile = await sharp(previewBytes)
      .resize({
        width: tileWidth,
        height: tileHeight - 28,
        fit: "contain",
        background: "#ffffff",
      })
      .extend({
        top: 28,
        background: "#ffffff",
      })
      .composite([
        {
          input: Buffer.from(
            `<svg width="${tileWidth}" height="28">`
              + '<rect width="100%" height="100%" fill="#17365D"/>'
              + `<text x="8" y="19" fill="#FFFFFF" font-size="12"`
              + ` font-family="Arial, sans-serif">${escapedLabel}</text></svg>`,
          ),
          top: 0,
          left: 0,
        },
      ])
      .png()
      .toBuffer();
    writeFileSync(tilePath, tile);
    const metadata = await sharp(tile).metadata();
    records.push({
      workbook: workbookName,
      sheet: sheetName,
      range: renderRange,
      sha256: createHash("sha256").update(tile).digest("hex"),
      width: metadata.width,
      height: metadata.height,
    });
    tilePaths.push(tilePath);
  }

  const rows = Math.ceil(tilePaths.length / columns);
  const contact = sharp({
    create: {
      width: tileWidth * columns,
      height: tileHeight * rows,
      channels: 3,
      background: "#D9E2F3",
    },
  });
  const composites = tilePaths.map((path, index) => ({
    input: path,
    left: (index % columns) * tileWidth,
    top: Math.floor(index / columns) * tileHeight,
  }));
  await contact
    .composite(composites)
    .png()
    .toFile(resolve(outputRoot, `${workbookStem}-contact.png`));
}

writeFileSync(
  resolve(outputRoot, "visual-review.json"),
  `${JSON.stringify(
    {
      schema_fingerprint: lock.fingerprint,
      render_range: renderRange,
      sheet_count: records.length,
      sheets: records,
    },
    null,
    2,
  )}\n`,
);
console.log(`Rendered ${records.length} Currency Wars sheets with artifact-tool.`);
