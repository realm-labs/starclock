#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  mkdtemp,
  readFile,
  readdir,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const root = path.resolve(".");
const sora = path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
const project = path.join(root, "config/anomaly-arbitration/project.toml");
const dataRoot = path.join(root, "config/anomaly-arbitration/data");
const generated = path.join(root, "config/anomaly-arbitration-generated");
const loader = path.join(
  root,
  "tools/anomaly-arbitration-reference/reader-loader/Cargo.toml",
);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function fileNames(directory) {
  return (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
}

const temporary = await mkdtemp(
  path.join(os.tmpdir(), "starclock-g13-sora-release-"),
);
const bundle = path.join(temporary, "config.sora");
const debug = path.join(temporary, "debug-json");
run(sora, ["check", "--project", project]);
run(sora, [
  "export",
  "--format", "binary",
  "--project", project,
  "--data-root", dataRoot,
  "--out", bundle,
  "--compression", "zstd",
  "--compression-level", "9",
]);
run(sora, [
  "export",
  "--format", "json-debug",
  "--project", project,
  "--data-root", dataRoot,
  "--out", debug,
]);
const committedBundle = await readFile(path.join(generated, "config.sora"));
const regeneratedBundle = await readFile(bundle);
assert(
  committedBundle.equals(regeneratedBundle),
  "binary Sora export drift",
);
const expectedDebug = path.join(generated, "debug-json");
const expectedNames = await fileNames(expectedDebug);
const actualNames = await fileNames(debug);
assert(
  JSON.stringify(expectedNames) === JSON.stringify(actualNames),
  "debug export file-set drift",
);
assert(actualNames.length === 37, "debug export must contain 37 tables");
let rowCount = 0;
for (const name of actualNames) {
  const expected = await readFile(path.join(expectedDebug, name));
  const actual = await readFile(path.join(debug, name));
  assert(expected.equals(actual), `${name}: debug export drift`);
  const payload = JSON.parse(actual);
  rowCount += payload.table.rows.length;
}
assert(rowCount === 2103, `expected 2103 debug rows, got ${rowCount}`);
run("cargo", [
  "run",
  "--quiet",
  "--locked",
  "--manifest-path", loader,
  "--",
  bundle,
]);
console.log(
  "Anomaly Arbitration Sora release verified: "
    + `37 tables, ${rowCount} rows, bundle=${digest(committedBundle)}.`,
);
