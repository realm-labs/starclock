#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { relative, resolve } from "node:path";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const python = resolve(valueAfter("--python"));
const generated = resolve(
  root,
  "config/galactic-baseballer-generated",
);
const project = resolve(root, "config/galactic-baseballer/project.toml");
const policy = json(resolve(root, "policy/sora-toolchain.json"));
const sora = resolve(root, policy.install_root, "bin/sora");
const schema = json(resolve(
  root,
  "content-manifests/galactic-baseballer-v1/normalized-schema.json",
));
const authoring = json(resolve(
  root,
  "content-manifests/galactic-baseballer-v1/authoring-contract.json",
));

const version = execFileSync(sora, ["--version"], {
  cwd: root,
  encoding: "utf8",
}).trim();
assert(version === `sora ${policy.version}`, `Sora version drift: ${version}`);
execFileSync(process.execPath, [
  "tools/galactic-baseballer-reference/generate-sora-schema.mjs",
  "--check",
  "--root",
  root,
], { cwd: root, stdio: "inherit" });
execFileSync(sora, [
  "--serial",
  "check",
  "--project",
  project,
], { cwd: root, stdio: "inherit" });
execFileSync(python, [
  "tools/galactic-baseballer-reference/verify-workbooks.py",
  "--root",
  root,
  "--directory",
  resolve(root, "config/galactic-baseballer/data"),
  "--templates",
  resolve(generated, "templates"),
], { cwd: root, stdio: "inherit" });

assert(
  JSON.stringify(readdirSync(generated).sort())
    === JSON.stringify(["schema.lock", "templates"]),
  "P3-B3 generated root contains a reader or export owned by P3-B4",
);
const lock = json(resolve(generated, "schema.lock"));
assert(
  lock.package === "starclock_galactic_baseballer_reference"
    && lock.schema.package === lock.package,
  "isolated Sora package drift",
);
assert(
  lock.schema.tables.length === schema.files.length,
  `expected ${schema.files.length} Sora tables`,
);
const expectedSources = new Map(authoring.workbooks.flatMap((workbook) =>
  workbook.normalized_files.map((file) => [
    `${workbook.file}/${sheetName(file)}`,
    file,
  ])));
const actualSources = new Set();
for (const table of lock.schema.tables) {
  const key = `${table.source.file}/${table.source.sheet}`;
  assert(expectedSources.has(key), `${table.name}: unexpected source ${key}`);
  assert(!actualSources.has(key), `${table.name}: duplicate source ${key}`);
  actualSources.add(key);
  assert(table.name === `Gb${pascal(expectedSources.get(key))}`,
    `${key}: table name drift`);
  assert(table.mode === "Map" && table.key === "id",
    `${table.name}: map/private-key contract drift`);
  assert(table.fields[0].name === "id"
    && table.fields[1].name === "stable_key",
  `${table.name}: private/stable key order drift`);
  const stableIndex = table.indexes.find(({ name }) =>
    name === "by_stable_key");
  assert(stableIndex?.unique === true,
    `${table.name}: unique stable-key index missing`);
}
assert(
  actualSources.size === expectedSources.size,
  "not every normalized file has exactly one Sora source",
);

const expectedSheets = Object.fromEntries(authoring.workbooks.map((workbook) =>
  [workbook.file, workbook.normalized_files.length]));
const templates = readdirSync(resolve(generated, "templates")).sort();
assert(
  JSON.stringify(templates)
    === JSON.stringify(Object.keys(expectedSheets).sort()),
  "Sora template workbook partition drift",
);
for (const [file, count] of Object.entries(expectedSheets)) {
  const xml = execFileSync("unzip", [
    "-p",
    resolve(generated, "templates", file),
    "xl/workbook.xml",
  ], { encoding: "utf8" });
  assert([...xml.matchAll(/<sheet[\s>]/gu)].length === count,
    `${file}: template sheet count drift`);
}

const scratch = mkdtempSync(resolve(root, ".cache/goal16-sora-foundation-"));
try {
  const fresh = resolve(scratch, "foundation");
  execFileSync(process.execPath, [
    "tools/galactic-baseballer-reference/generate-sora-foundation.mjs",
    "--root",
    root,
    "--output",
    fresh,
    "--python",
    python,
  ], { cwd: root, stdio: "inherit" });
  assert(equalTree(generated, fresh),
    "Sora schema lock/template regeneration drift");
} finally {
  rmSync(scratch, { recursive: true, force: true });
}

console.log(
  `Goal 16 Sora foundation verified: ${lock.schema.tables.length} tables, `
  + `${templates.length} templates, schema ${sha256(readFileSync(resolve(
    generated,
    "schema.lock",
  )))} and tree ${treeDigest(generated)}.`,
);

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1] || arguments_[index + 1].startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return arguments_[index + 1];
}

function json(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

function pascal(value) {
  return value.replace(/\.json$/u, "").split(/[^a-zA-Z0-9]+/u)
    .filter(Boolean)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join("");
}

function sheetName(file) {
  return pascal(file).slice(0, 31);
}

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const target = resolve(directory, entry.name);
      return entry.isDirectory() ? walk(target) : [target];
    })
    .sort();
}

function relativeFiles(directory) {
  return walk(directory).map((file) =>
    relative(directory, file).replaceAll("\\", "/"));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function equalTree(left, right) {
  const leftFiles = relativeFiles(left);
  const rightFiles = relativeFiles(right);
  return JSON.stringify(leftFiles) === JSON.stringify(rightFiles)
    && leftFiles.every((file) =>
      readFileSync(resolve(left, file))
        .equals(readFileSync(resolve(right, file))));
}

function treeDigest(directory) {
  return sha256(relativeFiles(directory).map((file) =>
    `${file}\0${sha256(readFileSync(resolve(directory, file)))}`)
  .join("\n"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
