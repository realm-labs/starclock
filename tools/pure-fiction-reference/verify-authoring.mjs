#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const root = process.cwd();
const sora = path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
const python = path.join(root, ".cache/python/bin/python");
const project = path.join(root, "config/pure-fiction/project.toml");
const data = path.join(root, "config/pure-fiction/data");
const generated = path.join(root, "config/pure-fiction-generated");
function run(command, args, extra = {}) {
  execFileSync(command, args, { cwd: root, stdio: "inherit",
    maxBuffer: 256 * 1024 * 1024, ...extra });
}
async function compare(left, right, label) {
  const [a, b] = await Promise.all([readFile(left), readFile(right)]);
  if (!a.equals(b)) throw new Error(`${label} generation drift`);
}
async function files(directory) {
  return (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
}

for (const batch of ["G15-P3-B1", "G15-P3-B2", "G15-P3-B3", "G15-P3-B4"])
  run(process.execPath, ["tools/pure-fiction-reference/generate-sora-schema.mjs",
    `--batch=${batch}`, "--check"]);
run(python, ["tools/pure-fiction-reference/verify_workbooks.py"]);
const temporary = await mkdtemp(path.join(os.tmpdir(), "starclock-g15-authoring-"));
const workbooks = path.join(temporary, "workbooks");
run(python, ["tools/pure-fiction-reference/author_workbooks.py",
  "--output", workbooks]);
for (const name of ["PureFiction.xlsx", "PureFictionBindings.xlsx",
  "PureFictionReview.xlsx"])
  await compare(path.join(data, name), path.join(workbooks, name), name);
run(sora, ["check", "--project", project, "--serial"]);
run(sora, ["schema-lock", "--project", project, "--out",
  path.join(temporary, "schema.lock"), "--serial"]);
run(sora, ["excel-template", "--project", project, "--out",
  path.join(temporary, "templates"), "--serial"]);
run(python, ["tools/pure-fiction-reference/normalize_templates.py",
  path.join(temporary, "templates")]);
run(sora, ["gen", "--target", "rust", "--project", project, "--out",
  path.join(temporary, "readers/rust"), "--format-code", "never", "--serial"]);
run(sora, ["export", "--format", "binary", "--project", project,
  "--data-root", data, "--out", path.join(temporary, "config.sora"),
  "--compression", "zstd", "--compression-level", "9", "--serial"]);
run(sora, ["export", "--format", "json-debug", "--project", project,
  "--data-root", data, "--out", path.join(temporary, "debug-json"), "--serial"]);
await compare(path.join(generated, "schema.lock"),
  path.join(temporary, "schema.lock"), "schema.lock");
await compare(path.join(generated, "config.sora"),
  path.join(temporary, "config.sora"), "config.sora");
for (const directory of ["templates", "debug-json", "readers/rust"])
  for (const name of await files(path.join(generated, directory)))
    await compare(path.join(generated, directory, name),
      path.join(temporary, directory, name), `${directory}/${name}`);
run("cargo", ["run", "--locked", "--manifest-path",
  "tools/pure-fiction-reference/reader-loader/Cargo.toml", "--",
  "config/pure-fiction-generated/config.sora"], {
  env: { ...process.env,
    CARGO_TARGET_DIR: path.join(root, ".cache/pure-fiction-reader-target") },
});
console.log("Pure Fiction authoring verified: byte-identical workbooks, "
  + "37 Sora tables/readers/debug exports, binary bundle and isolated load.");
