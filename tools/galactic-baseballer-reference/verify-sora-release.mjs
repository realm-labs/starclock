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
const reference = resolve(
  root,
  "content-reference/galactic-baseballer-v1",
);
const schema = json(resolve(
  root,
  "content-manifests/galactic-baseballer-v1/normalized-schema.json",
));

execFileSync(process.execPath, [
  "tools/galactic-baseballer-reference/verify-sora-foundation.mjs",
  "--allow-release",
  "--root",
  root,
  "--python",
  python,
], { cwd: root, stdio: "inherit" });

const expected = new Map(schema.files.map(({ file }) => {
  const rows = json(resolve(reference, file));
  return [`Gb${pascal(file)}`, rows];
}));
const debugRoot = resolve(generated, "debug-json");
const debugFiles = readdirSync(debugRoot).sort();
assert(
  debugFiles.length === expected.size
    && debugFiles.every((file) => file.endsWith(".json")),
  "debug export file denominator drift",
);
let rowCount = 0;
for (const file of debugFiles) {
  const table = file.replace(/\.json$/u, "");
  const expectedRows = expected.get(table);
  assert(expectedRows, `${file}: unexpected debug table`);
  const payload = json(resolve(debugRoot, file));
  assert(payload.table.name === table, `${file}: table identity drift`);
  assert(
    payload.table.rows.length === expectedRows.length,
    `${file}: row denominator drift`,
  );
  for (const [index, row] of payload.table.rows.entries()) {
    assert(
      row.values.id.Integer === index + 1,
      `${file}/row ${index + 1}: private key drift`,
    );
    assert(
      row.values.stable_key.String === expectedRows[index].id,
      `${file}/row ${index + 1}: stable identity/order drift`,
    );
  }
  rowCount += payload.table.rows.length;
}
assert(rowCount === 10_615, `expected 10615 debug rows, got ${rowCount}`);

const readers = relativeFiles(resolve(generated, "readers/rust"));
assert(
  readers.length === expected.size + 2
    && readers.includes("mod.rs")
    && readers.includes("runtime.rs"),
  "generated Rust reader file denominator drift",
);

const scratch = mkdtempSync(resolve(root, ".cache/goal16-sora-release-"));
try {
  const fresh = resolve(scratch, "release");
  execFileSync(process.execPath, [
    "tools/galactic-baseballer-reference/generate-sora-release.mjs",
    "--root",
    root,
    "--output",
    fresh,
    "--python",
    python,
  ], { cwd: root, stdio: "inherit" });
  assert(equalTree(generated, fresh),
    "Sora reader/export regeneration drift");
} finally {
  rmSync(scratch, { recursive: true, force: true });
}

execFileSync("cargo", [
  "run",
  "--quiet",
  "--locked",
  "--manifest-path",
  "tools/galactic-baseballer-reference/reader-loader/Cargo.toml",
  "--",
  resolve(generated, "config.sora"),
], {
  cwd: root,
  env: {
    ...process.env,
    CARGO_TARGET_DIR: resolve(
      root,
      ".cache/galactic-baseballer-bundle-loader-target",
    ),
  },
  stdio: "inherit",
});

const bundle = readFileSync(resolve(generated, "config.sora"));
console.log(
  `Goal 16 Sora release verified: ${expected.size} tables, ${rowCount} rows, `
  + `${readers.length} Rust reader files, bundle ${sha256(bundle)} and `
  + `tree ${treeDigest(generated)}.`,
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
