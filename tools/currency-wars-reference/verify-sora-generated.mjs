#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { sha256 } from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json(path.join(root, "policy/sora-toolchain.json"));
const sora = path.join(root, policy.install_root, "bin", "sora");
const project = "config/currency-wars-project.toml";
const generated = path.join(root, "config/currency-wars-generated");
const scratch = path.join(root, ".cache/currency-wars-sora-verify");
assert(path.relative(root, scratch).replaceAll("\\", "/")
  === ".cache/currency-wars-sora-verify", "unsafe scratch path");
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === `sora ${policy.version}`, "Sora version drift");

execFileSync(process.execPath, [
  "tools/currency-wars-reference/verify-sora-schema.mjs",
  "--through",
  "P3-B4",
], { cwd: root, stdio: "inherit" });
fs.rmSync(scratch, { recursive: true, force: true });
fs.mkdirSync(scratch, { recursive: true });
run(["--serial", "schema-lock", "--project", project,
  "--out", path.join(scratch, "schema.lock")]);
run(["--serial", "excel-template", "--project", project,
  "--out", path.join(scratch, "templates")]);
run(["--serial", "gen", "--target", "rust", "--project", project,
  "--out", path.join(scratch, "rust"), "--format-code", "never"]);

assert(equalFile(
  path.join(generated, "schema.lock"),
  path.join(scratch, "schema.lock"),
), "schema lock regeneration drift");
assert(equalTree(
  path.join(generated, "rust"),
  path.join(scratch, "rust"),
), "generated reader regeneration drift");

const expectedWorkbooks = {
  "CurrencyWars.xlsx": 63,
  "CurrencyWarsBindings.xlsx": 38,
  "CurrencyWarsReview.xlsx": 10,
};
const committedWorkbooks = fs.readdirSync(path.join(generated, "templates"))
  .filter((file) => file.endsWith(".xlsx")).sort();
const freshWorkbooks = fs.readdirSync(path.join(scratch, "templates"))
  .filter((file) => file.endsWith(".xlsx")).sort();
assert(JSON.stringify(committedWorkbooks)
  === JSON.stringify(Object.keys(expectedWorkbooks).sort())
  && JSON.stringify(freshWorkbooks) === JSON.stringify(committedWorkbooks),
"Sora template workbook partition drift");
for (const [file, expectedSheets] of Object.entries(expectedWorkbooks)) {
  assert(sheetCount(path.join(generated, "templates", file)) === expectedSheets
    && sheetCount(path.join(scratch, "templates", file)) === expectedSheets,
  `${file} sheet partition drift`);
}

const lock = json(path.join(generated, "schema.lock"));
assert(lock.schema.project_id === "starclock_currency_wars_reference"
  && lock.schema.contract_id === "starclock_currency_wars_reference/default"
  && lock.schema.view === "default"
  && lock.schema.tables.length === 111,
"schema lock project/view/table drift");
const rustFiles = walk(path.join(generated, "rust"));
assert(rustFiles.filter((file) => file.endsWith(".rs")).length === 113,
  "generated Rust reader file count drift");
const schemaDigest = sha256(fs.readFileSync(path.join(
  generated,
  "schema.lock",
)));
const readerDigest = treeDigest(path.join(generated, "rust"));
console.log(
  `Currency Wars generated Sora surface verified (111 tables; ` +
  `63/38/10 sheets; schema ${schemaDigest}; reader ${readerDigest}).`,
);

function run(arguments_) {
  execFileSync(sora, arguments_, { cwd: root, stdio: "inherit" });
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function sheetCount(file) {
  const xml = execFileSync("unzip", ["-p", file, "xl/workbook.xml"], {
    encoding: "utf8",
  });
  return [...xml.matchAll(/<sheet[\s>]/gu)].length;
}
function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const target = path.join(directory, entry.name);
      return entry.isDirectory() ? walk(target) : [target];
    })
    .sort();
}
function relativeFiles(directory) {
  return walk(directory).map((file) =>
    path.relative(directory, file).replaceAll("\\", "/"));
}
function equalFile(left, right) {
  return fs.readFileSync(left).equals(fs.readFileSync(right));
}
function equalTree(left, right) {
  const leftFiles = relativeFiles(left);
  const rightFiles = relativeFiles(right);
  return JSON.stringify(leftFiles) === JSON.stringify(rightFiles)
    && leftFiles.every((file) => equalFile(
      path.join(left, file),
      path.join(right, file),
    ));
}
function treeDigest(directory) {
  return sha256(relativeFiles(directory).map((file) =>
    `${file}\0${sha256(fs.readFileSync(path.join(directory, file)))}`)
  .join("\n"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
