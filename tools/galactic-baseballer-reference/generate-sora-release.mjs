#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const output = resolve(valueAfter("--output"));
const python = resolve(valueAfter("--python"));
const policy = JSON.parse(readFileSync(
  resolve(root, "policy/sora-toolchain.json"),
  "utf8",
));
const sora = resolve(root, policy.install_root, "bin/sora");
const project = resolve(root, "config/galactic-baseballer/project.toml");
const data = resolve(root, "config/galactic-baseballer/data");
if (existsSync(output)) {
  throw new Error(`refusing to overwrite Sora release target ${output}`);
}
const version = execFileSync(sora, ["--version"], {
  cwd: root,
  encoding: "utf8",
}).trim();
if (version !== `sora ${policy.version}`) {
  throw new Error(`expected sora ${policy.version}, got ${version}`);
}
execFileSync(process.execPath, [
  "tools/galactic-baseballer-reference/generate-sora-foundation.mjs",
  "--root",
  root,
  "--output",
  output,
  "--python",
  python,
], { cwd: root, stdio: "inherit" });
run([
  "--serial",
  "gen",
  "--target",
  "rust",
  "--project",
  project,
  "--out",
  resolve(output, "readers/rust"),
  "--format-code",
  "never",
]);
run([
  "--serial",
  "export",
  "--format",
  "binary",
  "--project",
  project,
  "--data-root",
  data,
  "--out",
  resolve(output, "config.sora"),
  "--compression",
  "zstd",
  "--compression-level",
  "9",
]);
run([
  "--serial",
  "export",
  "--format",
  "json-debug",
  "--project",
  project,
  "--data-root",
  data,
  "--out",
  resolve(output, "debug-json"),
]);
console.log(
  `Generated Goal 16 Sora ${policy.version} readers, binary and debug `
  + `exports at ${output}.`,
);

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0 || !arguments_[index + 1]) {
    throw new Error(`${flag} requires a value`);
  }
  return arguments_[index + 1];
}

function run(commandArguments) {
  execFileSync(sora, commandArguments, { cwd: root, stdio: "inherit" });
}
