#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const through = valueAfter("--through") ?? "P3-B4";
const toolPolicy = JSON.parse(fs.readFileSync(path.join(
  root,
  "policy/sora-toolchain.json",
), "utf8"));
const sora = path.join(root, toolPolicy.install_root, "bin", "sora");
assert(fs.existsSync(sora),
  `Sora ${toolPolicy.version} is not installed; run ${toolPolicy.install_command}`);
const version = execFileSync(sora, ["--version"], {
  cwd: root,
  encoding: "utf8",
}).trim();
assert(version === `sora ${toolPolicy.version}`,
  `expected Sora ${toolPolicy.version}, got ${version}`);
execFileSync(process.execPath, [
  "tools/currency-wars-reference/generate-sora-schema.mjs",
  "--check",
  "--root",
  root,
  "--through",
  through,
], { cwd: root, stdio: "inherit" });
execFileSync(sora, [
  "--serial",
  "check",
  "--project",
  "config/currency-wars-project.toml",
], { cwd: root, stdio: "inherit" });

const expected = {
  "P3-B1": 22,
  "P3-B2": 63,
  "P3-B3": 103,
  "P3-B4": 111,
}[through];
const schemaFiles = {
  "P3-B1": ["core.toml"],
  "P3-B2": ["core.toml", "systems.toml"],
  "P3-B3": ["core.toml", "systems.toml", "content.toml"],
  "P3-B4": ["core.toml", "systems.toml", "content.toml", "audit.toml"],
}[through];
const text = schemaFiles.map((file) => fs.readFileSync(path.join(
  root,
  "config/currency-wars/schema",
  file,
), "utf8")).join("\n");
const tables = [...text.matchAll(/^\[\[tables\]\]$/gmu)].length;
const names = [...text.matchAll(/^name = "(CurrencyWars[^"]+)"$/gmu)]
  .map((match) => match[1]);
const sheets = [...text.matchAll(/^sheet = "([^"]+)"$/gmu)]
  .map((match) => match[1]);
assert(tables === expected && new Set(names).size === expected,
  `expected ${expected} unique tables, got ${tables}`);
assert(sheets.length === expected
  && sheets.every((name) => name.length <= 31),
"sheet name contract drift");
assert(!text.includes("runtime_lowered = true")
  && !text.includes("config/generated"),
"schema isolation/runtime boundary drift");

console.log(
  `Currency Wars Sora schema verified through ${through}: ` +
  `${expected} isolated tables.`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
