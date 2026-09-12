#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  process.argv[2]
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const reauthorWorkbooks = process.argv.slice(3).includes("--reauthor-workbooks");
if (process.argv.slice(3).some((argument) =>
  argument !== "--reauthor-workbooks"))
  throw new Error("usage: generate-sora-artifacts.mjs [ROOT] [--reauthor-workbooks]");
const project = path.join(root, "config/divergent-universe-project.toml");
const target = path.join(root, "config/divergent-universe-generated");
const dataRoot = path.join(root, "config/divergent-universe/data");
const workbookScratch = path.join(
  root,
  ".cache/divergent-universe-workbook-migration",
);
const sora = locateSora();
assert(path.relative(root, target).replaceAll("\\", "/")
  === "config/divergent-universe-generated", "unsafe generated target");
assert(path.relative(root, dataRoot).replaceAll("\\", "/")
  === "config/divergent-universe/data", "unsafe workbook target");
assert(path.relative(root, workbookScratch).replaceAll("\\", "/")
  === ".cache/divergent-universe-workbook-migration",
"unsafe workbook scratch target");

const workbooks = [
  "DivergentUniverse.xlsx",
  "DivergentUniverseBindings.xlsx",
  "DivergentUniverseReview.xlsx",
];
const python = process.env.STARCLOCK_PYTHON
  ?? (process.platform === "win32" ? "python" : "python3");

fs.rmSync(target, { recursive: true, force: true });
fs.mkdirSync(target, { recursive: true });
run([
  "--serial",
  "schema-lock",
  "--project",
  project,
  "--out",
  path.join(target, "schema.lock"),
]);
run([
  "--serial",
  "excel-template",
  "--project",
  project,
  "--out",
  path.join(target, "templates"),
]);
run([
  "--serial",
  "gen",
  "--target",
  "rust",
  "--project",
  project,
  "--out",
  path.join(target, "reader"),
  "--format-code",
  "never",
]);
execFileSync(
  python,
  [
    path.join(
      root,
      "tools/divergent-universe-reference/normalize_xlsx_archives.py",
    ),
    ...workbooks.map((file) => path.join(target, "templates", file)),
  ],
  { cwd: root, stdio: "inherit" },
);
if (reauthorWorkbooks) {
  fs.rmSync(workbookScratch, { recursive: true, force: true });
  const scratchData = path.join(workbookScratch, "data");
  execFileSync(python, [
    path.join(root, "tools/divergent-universe-reference/author_workbooks.py"),
    root,
    "--output",
    scratchData,
  ], {
    cwd: root,
    env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" },
    stdio: "inherit",
  });
  fs.mkdirSync(dataRoot, { recursive: true });
  for (const workbook of workbooks)
    fs.copyFileSync(
      path.join(scratchData, workbook),
      path.join(dataRoot, workbook),
    );
  fs.rmSync(workbookScratch, { recursive: true, force: true });
}
run(["--serial", "build", "--project", project]);
execFileSync(python, [
  path.join(
    root,
    "tools/divergent-universe-reference/normalize_xlsx_archives.py",
  ),
  ...workbooks.map((file) => path.join(target, "templates", file)),
], { cwd: root, stdio: "inherit" });
console.log(
  "Generated isolated Divergent Universe schema lock, three Excel templates, " +
  "Rust readers, debug export and binary bundle with Sora 0.6.1.",
);

function locateSora() {
  const policy = JSON.parse(fs.readFileSync(
    path.join(root, "policy/sora-toolchain.json"),
    "utf8",
  ));
  assert(policy.version === "0.6.1", "Divergent Universe requires Sora 0.6.1");
  const binary = process.platform === "win32" ? "sora.exe" : "sora";
  const result = path.join(root, policy.install_root, "bin", binary);
  if (!fs.existsSync(result))
    throw new Error(`Sora ${policy.version} executable is unavailable`);
  return result;
}

function run(arguments_) {
  execFileSync(sora, arguments_, { cwd: root, stdio: "inherit" });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
