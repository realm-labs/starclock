#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
assert(process.argv.length === 2,
  "usage: node tools/currency-wars-reference/run-clean-checkout.mjs");
assert(capture("git", ["diff", "--name-only"]) === ""
  && capture("git", ["ls-files", "--others", "--exclude-standard"]) === "",
"stage every proposed Goal 12 file before clean-checkout acceptance");
const tree = capture("git", ["write-tree"]);
const sourceCommit = capture("git", ["rev-parse", "HEAD"]);
assert(/^[0-9a-f]{40}$/u.test(tree), "staged tree identity is invalid");

const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-currency-wars-clean-"),
);
const checkout = path.join(temporary, "checkout");
const archive = path.join(temporary, "tree.tar");
fs.mkdirSync(checkout);
try {
  run("git", ["archive", "--format=tar", `--output=${archive}`, tree], root);
  run("tar", ["-xf", archive, "-C", checkout], root);
  run("git", ["init", "--quiet"], checkout);
  run("git", ["fetch", "--quiet", "--no-tags", root, sourceCommit], checkout);
  fetchCheckpoint(
    checkout,
    "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    "refs/goal12-checkpoints/goal08",
  );
  fetchCheckpoint(
    checkout,
    "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    "refs/remotes/origin/codex/goal09-swarm-disaster-reference",
  );
  fetchCheckpoint(
    checkout,
    "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    "refs/remotes/origin/codex/goal10-unknowable-domain-reference",
  );
  fetchCheckpoint(
    checkout,
    "3071d2c2fa7764c133931756769c9efe7f9dabd2",
    "refs/remotes/origin/codex/goal11-divergent-universe-reference",
  );
  run("git", ["reset", "--mixed", "--quiet", sourceCommit], checkout);
  run("git", ["add", "--all"], checkout);
  run("git", [
    "-c",
    "user.name=Starclock Acceptance",
    "-c",
    "user.email=acceptance@invalid",
    "commit",
    "--quiet",
    "-m",
    "Goal 12 clean acceptance snapshot",
  ], checkout);

  seedSora(checkout);
  const environment = {
    ...process.env,
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: path.join(checkout, "target"),
    PYTHONDONTWRITEBYTECODE: "1",
    STARCLOCK_PYTHON:
      process.env.STARCLOCK_PYTHON
      ?? "/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/" +
        "dependencies/python/bin/python3",
  };
  run("node", [
    "tools/currency-wars-reference/run-acceptance.mjs",
    checkout,
  ], checkout, environment);
  assert(capture("git", ["status", "--porcelain"], checkout) === "",
    "clean-checkout acceptance modified tracked files");
  console.log(
    `Currency Wars clean-checkout acceptance passed for staged tree ${tree} ` +
    "(fresh build target; no inherited source cache).",
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function fetchCheckpoint(checkout, commit, reference) {
  run("git", [
    "fetch",
    "--quiet",
    "--no-tags",
    root,
    `${commit}:${reference}`,
  ], checkout);
}
function seedSora(targetRoot) {
  const policy = JSON.parse(fs.readFileSync(path.join(
    root,
    "policy/sora-toolchain.json",
  ), "utf8"));
  const relative = path.join(policy.install_root, "bin", "sora");
  const candidates = [
    path.join(root, relative),
    ...capture("git", ["worktree", "list", "--porcelain"])
      .split(/\r?\n/u)
      .filter((line) => line.startsWith("worktree "))
      .map((line) => path.join(line.slice("worktree ".length), relative)),
  ];
  const executable = candidates.find((candidate) => fs.existsSync(candidate));
  assert(executable, `pinned Sora ${policy.version} executable is unavailable`);
  const destination = path.join(targetRoot, relative);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(executable, destination);
  fs.chmodSync(destination, 0o755);
}
function capture(command, commandArgs, cwd = root) {
  const result = spawnSync(command, commandArgs, { cwd, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command} ${commandArgs.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}
function run(command, commandArgs, cwd, env = process.env) {
  console.log(`\n==> ${command} ${commandArgs.join(" ")}`);
  const result = spawnSync(command, commandArgs, {
    cwd,
    env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command} ${commandArgs.join(" ")} exited with ${result.status}`);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
