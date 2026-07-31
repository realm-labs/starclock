#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const arguments_ = process.argv.slice(2);
const root = resolve(valueAfter("--root") ?? process.cwd());
const output = resolve(valueAfter("--output"));
const python = valueAfter("--python") ?? "python3";
const policy = JSON.parse(readFileSync(resolve(root, "policy/sora-toolchain.json"), "utf8"));
const sora = resolve(root, policy.install_root, "bin/sora");
const project = resolve(root, "config/fate-star-rail-night/project.toml");
if (existsSync(output)) throw new Error(`refusing to overwrite Sora foundation target ${output}`);
if (!existsSync(sora)) throw new Error(`Sora ${policy.version} missing; run ${policy.install_command}`);
const version = execFileSync(sora, ["--version"], { cwd: root, encoding: "utf8" }).trim();
if (version !== `sora ${policy.version}`) throw new Error(`expected sora ${policy.version}, got ${version}`);
mkdirSync(output, { recursive: true });
run(["--serial", "schema-lock", "--project", project, "--out", resolve(output, "schema.lock")]);
run(["--serial", "excel-template", "--project", project, "--out", resolve(output, "templates")]);
execFileSync(python, [
  resolve(root, "tools/fate-star-rail-night-reference/normalize-sora-templates.py"),
  resolve(output, "templates"),
], { cwd: root, env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" }, stdio: "inherit" });
run(["--serial", "gen", "--target", "rust", "--project", project, "--out", resolve(output, "readers/rust"), "--format-code", "never"]);
console.log(`Generated Fate Sora ${policy.version} lock, templates and isolated readers at ${output}.`);

function run(commandArguments) {
  execFileSync(sora, commandArguments, { cwd: root, stdio: "inherit" });
}

function valueAfter(flag) {
  const index = arguments_.indexOf(flag);
  if (index < 0) return undefined;
  if (!arguments_[index + 1]) throw new Error(`${flag} requires a value`);
  return arguments_[index + 1];
}
