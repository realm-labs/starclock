#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { createContext, writeOrCheck } from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map([
  ["curios.json", []],
  ["curio-states.json", []],
  ["curio-groups.json", []],
  ["curio-lifecycle-rules.json", []],
  ["hex-states.json", []],
  ["hex-eligibility.json", []],
]);

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars Curio/Hex closure ${check ? "verified" : "generated"}: ` +
  "zero reachable Curios, Miracles and Hex states across six normalized files.",
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
