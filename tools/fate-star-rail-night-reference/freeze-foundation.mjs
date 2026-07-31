#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = argument("--source-cache");
const check = args.includes("--check");
const output = path.join(root,
  "content-manifests/fate-star-rail-night-v1/foundation.json");

const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const resRoot = path.join(sourceCache, "StarRailRes");
const turnTreePaths = treePaths(turnRoot);
const required = [
  ...turnTreePaths.filter((entry) =>
    entry.startsWith("ExcelOutput/FateRin") && entry.endsWith(".json")),
  ...turnTreePaths.filter((entry) =>
    entry.startsWith("Config/Gameplays/Fate/") ||
      (entry.startsWith("Config/") && entry.includes("FateRin"))),
].sort(compareText);
const sourceFiles = required.map((relative) => receipt(turnRoot, relative));

const document = {
  schema_revision: "starclock.fate-star-rail-night-foundation.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P0-B1",
  snapshot: {
    game_version: "4.4",
    released_date: "2026-07-24",
    access_date: "2026-08-01",
    content_lane: "Experimental",
    target_lane: "Candidate",
    runtime_released: false,
  },
  launch: {
    base: "92febad080dd4cf9997718d64b3648fc198ab1f8",
    branch: "codex/goal19-fate-star-rail-night-reference",
    upstream: "origin/codex/goal19-fate-star-rail-night-reference",
    package_commit: "d3380a6b4a749968ad0d56a37f84df73b1d1bff4",
  },
  repositories: [repository(turnRoot), repository(resRoot)],
  source_seed: {
    membership_rule: "explicit-faterin-selector-and-transitive-reference-closure",
    seed_is_denominator: false,
    dedicated_table_count: sourceFiles.filter(({ path: relative }) =>
      relative.startsWith("ExcelOutput/FateRin")).length,
    focused_config_count: sourceFiles.filter(({ path: relative }) =>
      relative.startsWith("Config/")).length,
    files: sourceFiles,
    canonical_sha256: digest(`${JSON.stringify(sourceFiles)}\n`),
  },
  isolation: {
    owned_roots: [
      "content-manifests/fate-star-rail-night-v1/",
      "content-reference/fate-star-rail-night-v1/",
      "config/fate-star-rail-night/",
      "config/fate-star-rail-night-generated/",
      "tools/fate-star-rail-night-reference/",
      "evidence/fate-star-rail-night-reference-v1/",
    ],
    protected_runtime_roots: [
      "crates/", "config/generated/", "config/universe-generated/",
    ],
    adjacent_exclusions: [
      "Currency Wars Fate Bonds", "Config/Activity/RtBattle",
      "story prose", "account rewards", "presentation assets",
    ],
  },
  contracts: {
    authoritative_authoring: "xlsx-via-python-openpyxl",
    schema_export_authority: "sora-cli-0.3.0",
    json_runtime_loading: false,
    runtime_implementation: false,
    zero_requires_selector_closure: true,
    provenance_required: true,
  },
};

const serialized = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(output), `missing ${path.relative(root, output)}`);
  assert(fs.readFileSync(output, "utf8") === serialized,
    "Goal 19 foundation drift");
  console.log(`Goal 19 foundation verified (${sourceFiles.length} source seeds).`);
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, serialized);
  console.log(`Wrote ${path.relative(root, output)} (${sourceFiles.length} source seeds).`);
}

function repository(repositoryRoot) {
  assert(fs.existsSync(path.join(repositoryRoot, ".git")),
    `missing source repository ${repositoryRoot}`);
  assert(git(repositoryRoot, ["status", "--porcelain"]) === "",
    `source repository is dirty ${repositoryRoot}`);
  assert(git(repositoryRoot, ["symbolic-ref", "--quiet", "--short", "HEAD"], true)
    === "", `source repository is not detached ${repositoryRoot}`);
  return {
    cache_name: path.basename(repositoryRoot),
    revision: git(repositoryRoot, ["rev-parse", "HEAD"]),
    tree: git(repositoryRoot, ["show", "-s", "--format=%T", "HEAD"]),
    remote: git(repositoryRoot, ["remote", "get-url", "origin"]),
  };
}

function treePaths(repositoryRoot) {
  return git(repositoryRoot, ["ls-tree", "-r", "--name-only", "HEAD"])
    .split("\n").filter(Boolean);
}

function receipt(repositoryRoot, relative) {
  const bytes = fs.readFileSync(path.join(repositoryRoot, relative));
  return { path: relative, bytes: bytes.length, sha256: digest(bytes) };
}

function git(cwd, command, allowFailure = false) {
  try {
    return execFileSync("git", ["-C", cwd, ...command], {
      encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
      maxBuffer: 64 * 1024 * 1024,
    }).trim();
  } catch (error) {
    if (allowFailure) return "";
    throw error;
  }
}

function argument(name) {
  const index = args.indexOf(name);
  assert(index !== -1 && args[index + 1], `${name} requires a value`);
  return path.resolve(args[index + 1]);
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
