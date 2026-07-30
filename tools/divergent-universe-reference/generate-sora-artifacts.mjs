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
const project = path.join(root, "config/divergent-universe/project.toml");
const target = path.join(root, "config/divergent-universe-generated");
const sora = locateSora();

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
execFileSync(process.env.STARCLOCK_PYTHON ?? "python3", [
  path.join(
    root,
    "tools/divergent-universe-reference/normalize_xlsx_archives.py",
  ),
  ...[
    "DivergentUniverse.xlsx",
    "DivergentUniverseBindings.xlsx",
    "DivergentUniverseReview.xlsx",
  ].map((file) => path.join(target, "templates", file)),
], { cwd: root, stdio: "inherit" });
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
console.log(
  "Generated isolated Divergent Universe schema lock, three Excel templates " +
  "and Rust reader with Sora 0.3.0.",
);

function locateSora() {
  const policy = JSON.parse(fs.readFileSync(
    path.join(root, "policy/sora-toolchain.json"),
    "utf8",
  ));
  const candidates = [
    path.join(root, policy.install_root, "bin/sora"),
    path.join(
      "/Users/mikai/CLionProjects/starclock",
      policy.install_root,
      "bin/sora",
    ),
  ];
  const result = candidates.find((candidate) => fs.existsSync(candidate));
  if (!result) throw new Error("Sora 0.3.0 executable is unavailable");
  return result;
}

function run(arguments_) {
  execFileSync(sora, arguments_, { cwd: root, stdio: "inherit" });
}
