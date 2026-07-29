#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-pools.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const rows = json(path.join(outputRoot, "pool-membership.json"));
assert(rows.length === 3044, "pool membership row count drift");
assert(new Set(rows.map((row) => row.id)).size === rows.length,
  "pool membership IDs are not unique");
assert(rows.every((row) =>
  row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.ownership === "DivergentUniverse"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured"
    && row.module_scope === "Tourn3/6002201"
    && row.runtime_lowered === false
    && row.source_refs.length > 0),
"pool membership envelope/boundary drift");

const byBasis = Object.fromEntries(
  [...Map.groupBy(rows, (row) => row.membership_basis)]
    .map(([basis, values]) => [basis, values.length])
    .sort(),
);
assert(JSON.stringify(byBasis) === JSON.stringify({
  DirectStableIdReference: 527,
  ExplicitTourn3ActiveTypeSelector: 8,
  ExplicitTourn3BlessingType: 414,
  TransitiveStableIdClosure: 2095,
}), "pool membership basis distribution drift");
assert(rows.filter((row) => row.edge_kind === "NestedGroup").length === 176,
  "nested subgroup edge count drift");
assert(rows.filter((row) => row.edge_kind === "TerminalLevel").length === 351,
  "direct terminal edge count drift");

const paths = json(path.join(outputRoot, "blessing-paths.json"));
const blessings = json(path.join(outputRoot, "blessings.json"));
const groups = json(path.join(outputRoot, "blessing-groups.json"));
const levels = json(path.join(outputRoot, "blessing-levels.json"));
const knownMembers = new Set([
  ...paths.map((row) => row.id),
  ...blessings.map((row) => row.id),
  ...groups.map((row) => row.id),
  ...levels.map((row) => row.id),
]);
assert(rows.every((row) => knownMembers.has(row.member_id)),
  "pool membership references an unknown member");
assert(groups.every((group) =>
  rows.some((row) =>
    row.pool_id === group.id
      && row.membership_basis === "TransitiveStableIdClosure")),
"Blessing group lacks terminal closure");

const goal08Blessings = JSON.parse(execFileSync("git", [
  "show",
  "c283c7f195dcfe05854f3b212df73444ee89255a:" +
    "content-reference/gold-and-gears-v1/blessings.json",
], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
}));
assert(goal08Blessings.length === 162
  && goal08Blessings.every((row) => row.ownership === "Shared"),
"Goal 08 shared Blessing checkpoint drift");
const sharedSourceIds = new Set(goal08Blessings.map((row) => row.source_id));
assert(blessings.every((row) => !sharedSourceIds.has(row.source_id)),
  "Divergent Universe Blessing incorrectly overlaps a shared source ID");
const sharedNames = new Set(goal08Blessings.map((row) => row.name_en));
assert(blessings.filter((row) => sharedNames.has(row.name_en)).length === 97,
  "same-name non-identity audit drift");

const digest = crypto.createHash("sha256")
  .update(fs.readFileSync(path.join(outputRoot, "pool-membership.json")))
  .digest("hex");
console.log(
  `Divergent Universe pools verified (${rows.length.toLocaleString("en-US")} ` +
  `rows; 527 direct edges; 2,095 terminal closures; digest ${digest}).`,
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

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
