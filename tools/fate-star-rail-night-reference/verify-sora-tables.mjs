#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { rowsForTable, tablesThrough } from "./sora-model.mjs";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const batch = valueAfter("--batch") ?? "G19-P3-B1";
const tables = tablesThrough(batch);
const seen = new Set();
let rows = 0;
for (const definition of tables) {
  const values = rowsForTable(root, definition);
  if (values.length === 0) throw new Error(`${definition.sheet}: no rows`);
  for (const row of values) {
    if (seen.has(row.stable_id)) throw new Error(`duplicate stable ID ${row.stable_id}`);
    seen.add(row.stable_id);
  }
  rows += values.length;
}
execFileSync(process.execPath, [
  resolve(root, "tools/fate-star-rail-night-reference/generate-sora-schema.mjs"),
  "--root", root,
  "--batch", batch,
  "--check",
], { cwd: root, stdio: "inherit" });
console.log(`Verified ${tables.length} non-empty exact-once Fate Sora tables with ${rows} rows through ${batch}.`);

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1]) throw new Error(`${flag} requires a value`);
  return arguments_[index + 1];
}
