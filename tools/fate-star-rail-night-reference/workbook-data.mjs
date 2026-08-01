#!/usr/bin/env node

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  COMMON_FIELDS,
  TABLES,
  canonicalRow,
  rowsForTable,
  workbookFor,
} from "./sora-model.mjs";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const output = resolve(valueAfter("--output"));
const workbooks = {};
for (const definition of TABLES) {
  const workbook = workbookFor(definition);
  workbooks[workbook] ??= [];
  workbooks[workbook].push({
    sheet: definition.sheet,
    rows: rowsForTable(root, definition).map(canonicalRow),
  });
}
const payload = {
  columns: ["id", ...COMMON_FIELDS.map(([name]) => name)],
  workbooks,
};
mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, `${JSON.stringify(payload)}\n`);
console.log(`Wrote ${TABLES.length} workbook tables to ${output}.`);

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1]) throw new Error(`${flag} requires a value`);
  return arguments_[index + 1];
}
