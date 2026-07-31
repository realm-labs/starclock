#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const python = option("--python") ?? "python3";
const toolRoot = path.join(root, "tools/fate-star-rail-night-reference");
const requireClean = process.argv.includes("--require-clean");

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${name} requires a value`);
  return value;
}

function node(script, args = []) {
  execFileSync(process.execPath, [path.join(toolRoot, script), ...args], {
    cwd: root,
    stdio: "inherit",
  });
}

node("assemble.mjs", ["--check"]);
node("audit-reference-pack.mjs", [root]);
node("verify-semantic-fixtures.mjs", [root]);
node("verify-peer-reconciliation.mjs");
execFileSync(python, [
  path.join(toolRoot, "verify-workbooks.py"),
  "--root", root,
  "--directory", path.join(root, "config/fate-star-rail-night/data"),
  "--templates", path.join(root, "config/fate-star-rail-night-generated/templates"),
], { cwd: root, stdio: "inherit" });
node("verify-sora-release.mjs", ["--root", root, "--python", python]);

if (requireClean) {
  const status = execFileSync(
    "git",
    ["status", "--porcelain", "--untracked-files=all"],
    { cwd: root, encoding: "utf8" },
  );
  if (status !== "") throw new Error(`Candidate verification drift:\n${status}`);
}

console.log(
  "Goal 19 Candidate verified: 1,904 obligations, 2,018 normalized records, "
    + "58 fixtures, four byte-stable workbooks, 48 Sora tables, 5,936 rows "
    + "and zero runtime profiles.",
);
