#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
assert(
  process.argv.length === 2,
  "usage: node tools/swarm-disaster-reference/run-clean-checkout.mjs",
);
const unstaged = capture("git", ["diff", "--name-only"]);
const untracked = capture("git", [
  "ls-files",
  "--others",
  "--exclude-standard",
]);
assert(
  unstaged === "" && untracked === "",
  "stage every proposed Goal 09 file before clean-checkout acceptance",
);
const tree = capture("git", ["write-tree"]);
const sourceCommit = capture("git", ["rev-parse", "HEAD"]);
assert(/^[0-9a-f]{40}$/u.test(tree), "staged tree identity is invalid");

const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-swarm-clean-"),
);
const checkout = path.join(temporary, "checkout");
const archive = path.join(temporary, "tree.tar");
fs.mkdirSync(checkout);
try {
  run("git", [
    "archive",
    "--format=tar",
    `--output=${archive}`,
    tree,
  ], root);
  run("tar", ["-xf", archive, "-C", checkout], root);
  run("git", ["init", "--quiet"], checkout);
  run("git", [
    "fetch",
    "--quiet",
    "--no-tags",
    root,
    sourceCommit,
  ], checkout);
  run("git", [
    "fetch",
    "--quiet",
    "--no-tags",
    root,
    "457d05f0e3a7b6fe3abb7e8f142f96fa271f5ecd",
  ], checkout);
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
    "Goal 09 clean acceptance snapshot",
  ], checkout);

  seedSora(checkout);
  const environment = {
    ...process.env,
    CARGO_TARGET_DIR: path.join(checkout, "target"),
    CARGO_INCREMENTAL: "0",
  };
  run(
    "node",
    ["tools/swarm-disaster-reference/run-acceptance.mjs", checkout],
    checkout,
    environment,
  );
  assert(
    capture("git", ["status", "--porcelain"], checkout) === "",
    "clean-checkout acceptance modified tracked files",
  );
  console.log(
    `Swarm Disaster clean-checkout acceptance passed for staged tree ${tree} ` +
    "(fresh build target; no inherited source cache).",
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function seedSora(targetRoot) {
  const policy = JSON.parse(
    fs.readFileSync(path.join(root, "policy/sora-toolchain.json"), "utf8"),
  );
  const relative = path.join(policy.install_root, "bin", "sora");
  const candidates = [
    path.join(root, relative),
    ...capture("git", ["worktree", "list", "--porcelain"])
      .split(/\r?\n/u)
      .filter((line) => line.startsWith("worktree "))
      .map((line) => path.join(line.slice("worktree ".length), relative)),
  ];
  const executable = candidates.find((candidate) => fs.existsSync(candidate));
  assert(executable, "pinned Sora 0.3.0 executable is unavailable");
  const destination = path.join(targetRoot, relative);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(executable, destination);
  fs.chmodSync(destination, 0o755);
}

function capture(command, arguments_, cwd = root) {
  const result = spawnSync(command, arguments_, {
    cwd,
    encoding: "utf8",
  });
  if (result.error) throw result.error;
  assert(
    result.status === 0,
    `${command} ${arguments_.join(" ")} failed: ${result.stderr}`,
  );
  return result.stdout.trim();
}

function run(command, arguments_, cwd, env = process.env) {
  console.log(`\n==> ${command} ${arguments_.join(" ")}`);
  const result = spawnSync(command, arguments_, {
    cwd,
    env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command} ${arguments_.join(" ")} exited with ${result.status}`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
