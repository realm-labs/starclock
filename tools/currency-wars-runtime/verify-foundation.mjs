#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildFoundation } from "./generate-foundation.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceCache = sourceCacheArgument(process.argv.slice(2));
const foundation = buildFoundation();

run("node", ["tools/currency-wars-runtime/generate-foundation.mjs", "--check"]);
runGit(root, ["cat-file", "-e", `${foundation.launch_baseline.commit}^{commit}`]);
assert(git(root, ["show", "-s", "--format=%T", foundation.launch_baseline.commit]).trim()
  === foundation.launch_baseline.tree, "Goal 21 launch tree drift");
runGit(root, ["merge-base", "--is-ancestor", foundation.launch_baseline.commit, "HEAD"]);
assert(foundation.launch_baseline.worktree_clean_before_goal_changes === true,
  "Goal 21 clean-launch observation is missing");

for (const prerequisite of foundation.prerequisite_releases) {
  runGit(root, ["cat-file", "-e", `${prerequisite.commit}^{commit}`]);
  assert(git(root, ["show", "-s", "--format=%T", prerequisite.commit]).trim()
    === prerequisite.tree, `${prerequisite.goal} prerequisite tree drift`);
  runGit(root, ["merge-base", "--is-ancestor", prerequisite.commit,
    foundation.launch_baseline.commit]);
}

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

run("node", ["tools/currency-wars-reference/verify-sora-migration.mjs"]);
const validation = JSON.parse(run("cargo", [
  "run", "--quiet", "-p", "starclock-cli", "--",
  "currency-wars", "config", "validate", "--json",
]));
assert(validation.valid === true, "Currency Wars production catalog is invalid");
const expectedCatalog = {
  routes: foundation.denominators.routes,
  nodes: foundation.denominators.nodes,
  difficulties: foundation.denominators.difficulties,
  roles: foundation.denominators.roster_roles,
  bonds: foundation.denominators.bonds,
  investments: foundation.denominators.investment_identities,
  project_policies: foundation.denominators.policy_gaps,
};
for (const [field, expected] of Object.entries(expectedCatalog))
  assert(validation[field] === expected,
    `Currency Wars production catalog ${field} drift`);

assert(foundation.runtime_state.status === "PartialRuntimeSkeleton",
  "Goal 21 skeleton must remain partial at G21-P0-B2");

console.log(
  "Currency Wars runtime foundation verified "
    + "(baseline; sources; 19,250 obligations; 2,367 programs; partial skeleton).",
);

function sourceCacheArgument(args) {
  const index = args.indexOf("--source-cache");
  if (index === -1)
    return path.join(root, ".cache/content-reference");
  assert(args[index + 1] !== undefined, "--source-cache requires a path");
  return path.resolve(args[index + 1]);
}

function run(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8" });
}

function git(cwd, args) {
  return execFileSync("git", args, { cwd, encoding: "utf8" });
}

function runGit(cwd, args) {
  execFileSync("git", args, { cwd, stdio: "inherit" });
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
