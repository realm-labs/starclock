#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const root = path.resolve(".");
const generated = path.join(root, "config/memory-of-chaos-generated");
const data = path.join(root, "config/memory-of-chaos/data");
const templates = path.join(generated, "templates");
const project = path.join(root, "config/memory-of-chaos/project.toml");
const sora = process.env.STARCLOCK_SORA_BIN
  ?? path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
const python = process.env.STARCLOCK_PYTHON
  ?? path.join(root, ".cache/python/bin/python");
function assert(condition, message) { if (!condition) throw new Error(message); }
function run(executable, args, options = {}) {
  return execFileSync(executable, args, { cwd: root, stdio: "inherit", ...options });
}
async function files(directory) {
  const output = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) output.push(...await files(target));
    else output.push(target);
  }
  return output.sort((left, right) => left.localeCompare(right, "en"));
}
async function compareTrees(leftRoot, rightRoot) {
  const left = await files(leftRoot);
  const right = await files(rightRoot);
  const leftNames = left.map((file) => path.relative(leftRoot, file));
  const rightNames = right.map((file) => path.relative(rightRoot, file));
  assert(JSON.stringify(leftNames) === JSON.stringify(rightNames), "export file-list drift");
  for (let index = 0; index < left.length; index++) {
    assert((await readFile(left[index])).equals(await readFile(right[index])),
      `${leftNames[index]} export drift`);
  }
}
async function treeDigest(directory) {
  const hash = createHash("sha256");
  for (const file of await files(directory)) {
    hash.update(path.relative(directory, file));
    hash.update("\0");
    hash.update(await readFile(file));
    hash.update("\0");
  }
  return hash.digest("hex");
}

assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim() === "sora 0.3.0",
  "pinned Sora version drift");
run(process.execPath, ["tools/memory-of-chaos-reference/finalize-pack.mjs", "--check"]);
run(process.execPath, ["tools/memory-of-chaos-reference/generate-sora-schema.mjs", "--phase=4", "--check"]);
run(process.execPath, ["tools/memory-of-chaos-reference/verify-sora-generated.mjs"], {
  env: { ...process.env, STARCLOCK_SORA_BIN: sora },
});
run(python, ["tools/memory-of-chaos-reference/verify-workbooks.py", "--root", root,
  "--directory", data, "--templates", templates]);

const temporary = await mkdtemp(path.join(os.tmpdir(), "starclock-g17-release-"));
const regeneratedData = path.join(temporary, "data");
run(python, ["tools/memory-of-chaos-reference/author-workbooks.py", "--root", root,
  "--output", regeneratedData, "--templates", templates]);
for (const workbook of ["MemoryOfChaos.xlsx", "MemoryOfChaosBindings.xlsx", "MemoryOfChaosReview.xlsx"]) {
  assert((await readFile(path.join(data, workbook))).equals(await readFile(path.join(regeneratedData, workbook))),
    `${workbook} generation drift`);
}
const regeneratedBundle = path.join(temporary, "config.sora");
const regeneratedDebug = path.join(temporary, "debug-json");
run(sora, ["export", "--serial", "--format", "binary", "--project", project,
  "--data-root", data, "--out", regeneratedBundle]);
run(sora, ["export", "--serial", "--format", "json-debug", "--project", project,
  "--data-root", data, "--out", regeneratedDebug]);
assert((await readFile(path.join(generated, "config.sora"))).equals(await readFile(regeneratedBundle)),
  "binary export drift");
await compareTrees(path.join(generated, "debug-json"), regeneratedDebug);
const debugFiles = await files(path.join(generated, "debug-json"));
assert(debugFiles.length === 27, "debug export table count drift");
run("cargo", ["run", "--locked", "--manifest-path",
  "tools/memory-of-chaos-reference/reader-loader/Cargo.toml", "--",
  path.join(generated, "config.sora")], {
  env: { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/goal17-reader-target") },
});
const review = JSON.parse(await readFile(path.join(root,
  "evidence/memory-of-chaos-reference-v1/workbook-review/visual-review.json"), "utf8"));
assert(review.visual_disposition === "PassedHumanInspection"
  && review.sheet_count === 27 && review.rendered_band_count === 81
  && review.all_schema_columns_rendered === true
  && review.severe_visual_defect_count === 0,
"workbook visual review incomplete");
const bundleBytes = await readFile(path.join(generated, "config.sora"));
console.log(`Goal 17 Sora release verified: bundle=${createHash("sha256").update(bundleBytes).digest("hex")} bytes=${bundleBytes.length}, debug=${await treeDigest(path.join(generated, "debug-json"))}, 27 tables/1521 rows, 27 sheets/81 bands.`);
