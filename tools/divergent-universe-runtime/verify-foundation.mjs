#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildFoundation } from "./generate-foundation.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceCache = sourceCacheArgument(process.argv.slice(2));
const foundation = buildFoundation();

run("node", ["tools/divergent-universe-runtime/generate-foundation.mjs", "--check"]);
for (const source of foundation.source_snapshot.repositories) {
  const repository = path.join(sourceCache, source.id === "starrailres"
    ? "StarRailRes" : source.id);
  assert(fs.statSync(path.join(repository, ".git"), { throwIfNoEntry: false }),
    `source cache is missing ${source.id}`);
  assert(git(repository, ["rev-parse", "HEAD"]).trim() === source.revision,
    `source revision drift for ${source.id}`);
  assert(git(repository, ["remote", "get-url", "origin"]).trim() === source.repository,
    `source remote drift for ${source.id}`);
  assert(git(repository, ["status", "--porcelain"]).trim() === "",
    `source cache has local changes for ${source.id}`);
  assert(spawnSync("git", ["symbolic-ref", "-q", "HEAD"], {
    cwd: repository,
    encoding: "utf8",
  }).status !== 0, `source cache must be detached for ${source.id}`);
  runGit(repository, ["fsck", "--connectivity-only", "--no-dangling"]);
}
verifySourceInventory(sourceCache, foundation.denominators);

run("node", ["tools/divergent-universe-reference/verify-sora-migration.mjs"]);
assert(foundation.promoted_reference.status === "CandidateReferenceOnly"
  && foundation.promoted_reference.runtime_execution_credit === false
  && foundation.promoted_reference.full_playability_proven === false,
"reference input validation must not imply execution or full playability");
assert(foundation.reference_fixture_summary.runtime_executions === 0,
  "reference fixtures must not claim runtime execution");
run(process.env.STARCLOCK_PYTHON ?? (process.platform === "win32" ? "python" : "python3"), [
  "tools/divergent-universe-runtime/persona_reachability.py",
  "--source-cache", path.join(sourceCache, "turnbasedgamedata"),
  "--check", "--require-promoted",
]);
console.log("Divergent Universe current foundation verified "
  + "(6,762 obligations; 81 families; 28,732 rows; 547 source-only Persona obligations; "
  + "pinned revisions and inventory paths, not a full source-blob rehash; no execution credit).");

function sourceCacheArgument(args) {
  const index = args.indexOf("--source-cache");
  if (index === -1) return path.join(root, ".cache/content-reference");
  assert(args[index + 1] !== undefined, "--source-cache requires a path");
  assert(args.length === 2, "unsupported foundation verifier arguments");
  return path.resolve(args[index + 1]);
}

function verifySourceInventory(cache, denominators) {
  const inventory = JSON.parse(fs.readFileSync(path.join(root,
    "content-manifests/divergent-universe-v1/source-inventory.json"), "utf8"));
  const pathsByRepository = new Map();
  for (const source of foundation.source_snapshot.repositories) {
    const repository = path.join(cache, source.id === "starrailres"
      ? "StarRailRes" : source.id);
    pathsByRepository.set(source.id, new Set(git(repository,
      ["ls-tree", "-r", "--name-only", "HEAD"]).split(/\r?\n/u).filter(Boolean)));
  }
  const counts = {};
  for (const record of inventory.records) {
    assert(pathsByRepository.get(record.repository)?.has(record.path),
      `source inventory path is absent at pinned revision: ${record.repository}:${record.path}`);
    counts[record.repository] = (counts[record.repository] ?? 0) + 1;
  }
  assert(inventory.records.length === denominators.source_files,
    "source inventory total differs from frozen denominator");
  for (const [repository, expected] of Object.entries(
    denominators.source_files_by_repository))
    assert(counts[repository] === expected,
      `source inventory repository count drift for ${repository}`);
}

function run(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8" });
}

function git(cwd, args) {
  return execFileSync("git", args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function runGit(cwd, args) {
  execFileSync("git", args, { cwd, stdio: "inherit" });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
