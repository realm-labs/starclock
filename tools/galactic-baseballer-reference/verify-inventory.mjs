#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const args = process.argv.slice(2);
const sourceCacheIndex = args.indexOf("--source-cache");
const sourceCacheArgs = sourceCacheIndex === -1
  ? []
  : ["--source-cache", requiredArgument(sourceCacheIndex)];

function requiredArgument(index) {
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error("--source-cache requires a path");
  return args[index + 1];
}

execFileSync("node", [
  "tools/galactic-baseballer-reference/inventory.mjs",
  "--check",
  ...sourceCacheArgs,
], { cwd: root, stdio: "inherit" });
execFileSync("node", [
  "tools/galactic-baseballer-reference/public-sources.mjs",
  "--check",
], { cwd: root, stdio: "inherit" });

console.log("Galactic Baseballer focused inventory acceptance passed.");
