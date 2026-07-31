#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { relative, resolve } from "node:path";
import { TABLES, canonicalRow, rowsForTable } from "./sora-model.mjs";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const python = valueAfter("--python") ?? "python3";
const generated = resolve(root, "config/fate-star-rail-night-generated");
const expected = new Map(TABLES.map((definition) => [
  `Fsn${definition.sheet}`,
  rowsForTable(root, definition).map(canonicalRow),
]));
const debugRoot = resolve(generated, "debug-json");
const debugFiles = readdirSync(debugRoot).filter((file) => file.endsWith(".json")).sort();
assert(debugFiles.length === expected.size, "debug table denominator drift");
let rowCount = 0;
for (const file of debugFiles) {
  const table = file.replace(/\.json$/u, "");
  const expectedRows = expected.get(table);
  assert(expectedRows, `${file}: unexpected debug table`);
  const payload = JSON.parse(readFileSync(resolve(debugRoot, file), "utf8"));
  assert(payload.table.name === table, `${file}: table identity drift`);
  assert(payload.table.rows.length === expectedRows.length, `${file}: row denominator drift`);
  for (const [index, row] of payload.table.rows.entries()) {
    assert(row.values.id.Integer === index + 1, `${file}/row ${index + 1}: private key drift`);
    assert(row.values.stable_key.String === expectedRows[index].stable_key, `${file}/row ${index + 1}: stable key drift`);
  }
  rowCount += payload.table.rows.length;
}
assert(rowCount === 5_934, `expected 5934 rows, got ${rowCount}`);
const scratch = mkdtempSync(resolve(root, ".cache/g19-sora-release-"));
try {
  const fresh = resolve(scratch, "release");
  execFileSync(process.execPath, [
    resolve(root, "tools/fate-star-rail-night-reference/generate-sora-release.mjs"),
    "--root", root,
    "--output", fresh,
    "--python", python,
  ], { cwd: root, stdio: "inherit" });
  assert(equalTree(generated, fresh), "Sora reader/export regeneration drift");
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
execFileSync("cargo", ["run", "--quiet", "--locked", "--manifest-path", "tools/fate-star-rail-night-reference/reader-loader/Cargo.toml", "--", resolve(generated, "config.sora")], {
  cwd: root,
  env: { ...process.env, CARGO_TARGET_DIR: resolve(root, ".cache/fate-star-rail-night-bundle-loader-target") },
  stdio: "inherit",
});
console.log(`Verified Fate Sora release: 48 tables, ${rowCount} rows, bundle ${sha256(readFileSync(resolve(generated, "config.sora")))} and tree ${treeDigest(generated)}.`);

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1]) throw new Error(`${flag} requires a value`);
  return arguments_[index + 1];
}
function walk(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = resolve(directory, entry.name);
    return entry.isDirectory() ? walk(target) : [target];
  }).sort();
}
function relativeFiles(directory) {
  return walk(directory).map((file) => relative(directory, file).replaceAll("\\", "/"));
}
function equalTree(left, right) {
  const files = relativeFiles(left);
  return JSON.stringify(files) === JSON.stringify(relativeFiles(right)) && files.every((file) => readFileSync(resolve(left, file)).equals(readFileSync(resolve(right, file))));
}
function sha256(value) { return createHash("sha256").update(value).digest("hex"); }
function treeDigest(directory) { return sha256(relativeFiles(directory).map((file) => `${file}\0${sha256(readFileSync(resolve(directory, file)))}`).join("\n")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
