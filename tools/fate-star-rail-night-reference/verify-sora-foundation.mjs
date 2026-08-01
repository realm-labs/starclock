#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { relative, resolve } from "node:path";
import { TABLES, WORKBOOKS } from "./sora-model.mjs";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const python = valueAfter("--python") ?? "python3";
const generated = resolve(root, "config/fate-star-rail-night-generated");
assert(TABLES.length === 48, `expected 48 tables, got ${TABLES.length}`);
assert(new Set(TABLES.map(({ sheet }) => sheet)).size === 48, "sheet identity drift");
assert(new Set(TABLES.map(({ workbook }) => WORKBOOKS[workbook])).size === 4, "workbook denominator drift");
assert(relativeFiles(resolve(generated, "templates")).length === 4, "template denominator drift");
const readers = relativeFiles(resolve(generated, "readers/rust"));
assert(readers.length === 50 && readers.includes("mod.rs") && readers.includes("runtime.rs"), "reader denominator drift");
const scratch = mkdtempSync(resolve(root, ".cache/g19-sora-foundation-"));
try {
  const fresh = resolve(scratch, "generated");
  execFileSync(process.execPath, [
    resolve(root, "tools/fate-star-rail-night-reference/generate-sora-foundation.mjs"),
    "--root", root,
    "--output", fresh,
    "--python", python,
  ], { cwd: root, stdio: "inherit" });
  assert(equalTree(generated, fresh), "Sora foundation regeneration drift");
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
console.log(`Verified Fate Sora foundation: 48 tables, 4 templates, 50 reader files, tree ${treeDigest(generated)}.`);

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
  const leftFiles = relativeFiles(left);
  const rightFiles = relativeFiles(right);
  return JSON.stringify(leftFiles) === JSON.stringify(rightFiles)
    && leftFiles.every((file) => readFileSync(resolve(left, file)).equals(readFileSync(resolve(right, file))));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function treeDigest(directory) {
  return sha256(relativeFiles(directory).map((file) => `${file}\0${sha256(readFileSync(resolve(directory, file)))}`).join("\n"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
