#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const output = resolve(valueAfter("--output"));
const python = valueAfter("--python") ?? "python3";
const policy = JSON.parse(readFileSync(resolve(root, "policy/sora-toolchain.json"), "utf8"));
const sora = resolve(root, policy.install_root, "bin/sora");
const project = resolve(root, "config/fate-star-rail-night/project.toml");
const data = resolve(root, "config/fate-star-rail-night/data");
if (existsSync(output)) throw new Error(`refusing to overwrite Sora release target ${output}`);
execFileSync(process.execPath, [
  resolve(root, "tools/fate-star-rail-night-reference/generate-sora-foundation.mjs"),
  "--root", root,
  "--output", output,
  "--python", python,
], { cwd: root, stdio: "inherit" });
run(["--serial", "export", "--format", "binary", "--project", project, "--data-root", data, "--out", resolve(output, "config.sora"), "--compression", "zstd", "--compression-level", "9"]);
run(["--serial", "export", "--format", "json-debug", "--project", project, "--data-root", data, "--out", resolve(output, "debug-json")]);
console.log(`Generated Fate Sora ${policy.version} readers, binary and debug exports at ${output}.`);

function run(commandArguments) {
  execFileSync(sora, commandArguments, { cwd: root, stdio: "inherit" });
}

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1]) throw new Error(`${flag} requires a value`);
  return arguments_[index + 1];
}
