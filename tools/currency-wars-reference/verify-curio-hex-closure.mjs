#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/currency-wars-reference/import-curio-hex-closure.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const files = [
  "curios.json",
  "curio-states.json",
  "curio-groups.json",
  "curio-lifecycle-rules.json",
  "hex-states.json",
  "hex-eligibility.json",
];
const outputRoot = path.join(root, "content-reference/currency-wars-v1");
for (const file of files)
  assert(Array.isArray(json(path.join(outputRoot, file)))
    && json(path.join(outputRoot, file)).length === 0,
  `${file} proven-empty drift`);

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
const category = manifest.categories.curios_miracles_hex_states;
assert(category.count === 0 && category.records.length === 0,
  "Curio/Miracle/Hex manifest zero denominator drift");

const inventory = json(path.join(
  root,
  "content-manifests/currency-wars-v1/source-inventory.json",
));
const directTables = inventory.records
  .map(({ path: sourcePath }) => sourcePath)
  .filter((sourcePath) =>
    /^ExcelOutput\/GridFight[^/]*\.json$/u.test(sourcePath));
const directConfigs = manifest.categories.mechanic_rules.records
  .map(({ source }) => source)
  .filter((sourcePath) =>
    /^Config\/.*GridFight.*\.json$/u.test(sourcePath));
assert(directTables.length === 153 && directConfigs.length === 984,
  "GridFight source closure size drift");

for (const sourcePath of [...directTables, ...directConfigs]) {
  const bytes = fs.readFileSync(path.join(sourceRoot, sourcePath), "utf8");
  assert(!/\b(?:Curio|Miracle)\b|Rogue(?:Miracle|Curio)/iu.test(bytes),
    `${sourcePath} contains an unresolved Curio/Miracle reference`);
  if (sourcePath !== "ExcelOutput/GridFightAugment.json")
    assert(!/"[^"]*\bHex\b[^"]*"/iu.test(bytes),
      `${sourcePath} contains an unresolved Hex reference`);
}
const augmentBytes = fs.readFileSync(path.join(
  sourceRoot,
  "ExcelOutput/GridFightAugment.json",
), "utf8");
const strippedAugment = augmentBytes
  .replace(/"HexName"/gu, "\"AugmentName\"")
  .replace(/"HexDesc"/gu, "\"AugmentDesc\"");
assert(!/"[^"]*\bHex\b[^"]*"/iu.test(strippedAugment),
  "GridFightAugment has a Hex reference beyond legacy name/description keys");

const digest = crypto.createHash("sha256");
for (const file of [...files].sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars Curio/Hex closure verified (0 manifest obligations; ` +
  `153 direct tables; 984 configs; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
