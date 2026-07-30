#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? path.join(root, ".cache/galactic-baseballer-source"));
const python = path.resolve(option("--python"));
const allowDetached = process.argv.includes("--allow-detached");
const requireClean = process.argv.includes("--require-clean");
const toolRoot = path.join("tools", "galactic-baseballer-reference");

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function run(script, args = []) {
  execFileSync(process.execPath, [path.join(toolRoot, script), ...args], {
    cwd: root,
    stdio: "inherit",
  });
}

function runPython(script, args) {
  execFileSync(python, [path.join(toolRoot, script), ...args], {
    cwd: root,
    stdio: "inherit",
  });
}

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const target = path.join(directory, entry.name);
      return entry.isDirectory() ? walk(target) : [target];
    })
    .sort();
}

function equalTrees(left, right) {
  const leftFiles = walk(left).map((file) =>
    path.relative(left, file).replaceAll("\\", "/"));
  const rightFiles = walk(right).map((file) =>
    path.relative(right, file).replaceAll("\\", "/"));
  return JSON.stringify(leftFiles) === JSON.stringify(rightFiles)
    && leftFiles.every((file) =>
      readFileSync(path.join(left, file))
        .equals(readFileSync(path.join(right, file))));
}

const sourceArgs = ["--source-cache", sourceCache];
run("verify-contracts.mjs");
run("verify-inventory.mjs", sourceArgs);
run("verify-manifest.mjs", sourceArgs);

run("normalize-departure.mjs", sourceArgs);
run("verify-departure-profile.mjs", sourceArgs);
run("normalize-departure-arsenal.mjs", sourceArgs);
run("verify-departure-arsenal.mjs", sourceArgs);
run("normalize-departure-growth.mjs", sourceArgs);
run("verify-departure-growth.mjs", sourceArgs);
run("normalize-departure-encounters.mjs", sourceArgs);
run("verify-departure-encounters.mjs", sourceArgs);
run("normalize-departure-progression.mjs", sourceArgs);
run("normalize-departure-fixtures.mjs");
run("verify-departure-fixtures.mjs");

run("verify-demon-profile.mjs", sourceArgs);
run("verify-demon-arsenal.mjs", sourceArgs);
run("verify-demon-progression.mjs", sourceArgs);
run("verify-demon-encounters.mjs", sourceArgs);

run("assemble-reference-pack.mjs");
run("assemble-reference-pack.mjs", ["--check"]);
run("verify-reference-pack.mjs", sourceArgs);
run("execute-semantic-fixtures.mjs", ["--check"]);
run("audit-reference-boundaries.mjs", [
  "--check",
  ...(allowDetached ? ["--allow-detached"] : []),
]);

runPython("verify-workbooks.py", [
  "--root",
  root,
  "--directory",
  path.join(root, "config", "galactic-baseballer", "data"),
  "--templates",
  path.join(root, "config", "galactic-baseballer-generated", "templates"),
]);
const workbookScratch = mkdtempSync(
  path.join(root, ".cache", "goal16-candidate-workbooks-"),
);
try {
  const fresh = path.join(workbookScratch, "workbooks");
  runPython("author-workbooks.py", [
    "--root",
    root,
    "--output",
    fresh,
    "--templates",
    path.join(root, "config", "galactic-baseballer-generated", "templates"),
  ]);
  if (!equalTrees(
    path.join(root, "config", "galactic-baseballer", "data"),
    fresh,
  )) {
    throw new Error("Candidate workbook double-generation drift");
  }
} finally {
  rmSync(workbookScratch, { recursive: true, force: true });
}
run("verify-sora-release.mjs", ["--root", root, "--python", python]);

for (const repository of ["turnbasedgamedata", "StarRailRes"]) {
  const status = execFileSync(
    "git",
    ["-C", path.join(sourceCache, repository), "status", "--porcelain"],
    { cwd: root, encoding: "utf8" },
  );
  if (status !== "") throw new Error(`source cache became dirty: ${repository}`);
}
if (requireClean) {
  const status = execFileSync(
    "git",
    ["status", "--porcelain", "--untracked-files=all"],
    { cwd: root, encoding: "utf8" },
  );
  if (status !== "") throw new Error(`Candidate regeneration drift:\n${status}`);
}
console.log(
  "Goal 16 Candidate verified: fixed sources, 2 profiles, 2232 obligations, "
    + "20 semantic families, 4 byte-stable workbooks, 40 Sora tables and "
    + "10615 independently loaded rows.",
);
