#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { root } from "./source.mjs";

const sora = path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
const python = path.join(root, ".cache/g18-venv/bin/python");
const project = path.join(root, "config/apocalyptic-shadow/project.toml");
const data = path.join(root, "config/apocalyptic-shadow/data");
const generated = path.join(root, "config/apocalyptic-shadow-generated");
function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit",
    maxBuffer: 256 * 1024 * 1024 });
}
async function compare(left, right, label) {
  const [a, b] = await Promise.all([readFile(left), readFile(right)]);
  if (!a.equals(b)) throw new Error(`${label} generation drift`);
}
async function files(directory) {
  return (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
}

run("node", ["tools/apocalyptic-shadow-reference/generate-sora-schema.mjs",
  "--check"]);
run(python, ["tools/apocalyptic-shadow-reference/verify_workbooks.py"]);
const temporary = await mkdtemp(path.join(os.tmpdir(), "starclock-g18-authoring-"));
const workbooks = path.join(temporary, "workbooks");
run(python, ["tools/apocalyptic-shadow-reference/author_workbooks.py",
  "--output", workbooks]);
for (const name of ["ApocalypticShadow.xlsx", "ApocalypticShadowBindings.xlsx",
  "ApocalypticShadowReview.xlsx"])
  await compare(path.join(data, name), path.join(workbooks, name), name);
run(sora, ["check", "--project", project]);
run(sora, ["schema-lock", "--project", project, "--out",
  path.join(temporary, "schema.lock")]);
run(sora, ["excel-template", "--project", project, "--out",
  path.join(temporary, "templates")]);
run(sora, ["gen", "--target", "rust", "--project", project, "--out",
  path.join(temporary, "readers/rust"), "--format-code", "never"]);
run(sora, ["export", "--format", "binary", "--project", project,
  "--data-root", data, "--out", path.join(temporary, "config.sora"),
  "--compression", "zstd", "--compression-level", "9"]);
run(sora, ["export", "--format", "json-debug", "--project", project,
  "--data-root", data, "--out", path.join(temporary, "debug-json")]);
await compare(path.join(generated, "schema.lock"),
  path.join(temporary, "schema.lock"), "schema.lock");
await compare(path.join(generated, "config.sora"),
  path.join(temporary, "config.sora"), "config.sora");
for (const name of await files(path.join(generated, "debug-json")))
  await compare(path.join(generated, "debug-json", name),
    path.join(temporary, "debug-json", name), name);
for (const name of await files(path.join(generated, "readers/rust")))
  await compare(path.join(generated, "readers/rust", name),
    path.join(temporary, "readers/rust", name), name);
console.log("Apocalyptic Shadow authoring verified: deterministic workbooks, "
  + "35 Sora tables/readers/debug exports and binary bundle.");
