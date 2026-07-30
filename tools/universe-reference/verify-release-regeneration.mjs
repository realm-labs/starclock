#!/usr/bin/env node

import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const policy = JSON.parse(fs.readFileSync(
  path.join(root, "policy/release-snapshots.json"),
  "utf8",
));
const snapshot = policy.goals.find(
  ({ goal_id: goalId }) => goalId === "standard-universe-reference-v1",
);
assert(snapshot, "Goal 03 release snapshot is not registered");
assert(
  git(["show", "-s", "--format=%T", snapshot.completion_commit]).trim()
    === snapshot.completion_tree,
  "Goal 03 completion tree drift",
);

const sourceCache = path.join(
  root,
  ".cache/content-reference/turnbasedgamedata",
);
assert(fs.existsSync(sourceCache), "Standard Universe source cache is missing");
const temporaryRoot = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-goal03-regeneration-"),
);

try {
  const tracked = git([
    "ls-tree",
    "-r",
    "--name-only",
    snapshot.completion_commit,
    "--",
    "tools/universe-reference",
    "content-reference/v4.4",
    "content-reference/standard-universe-v1",
  ]).split(/\r?\n/u).filter((relative) =>
    relative.endsWith(".mjs")
      || relative.startsWith("content-reference/v4.4/")
      || relative.startsWith("content-reference/standard-universe-v1/"));
  assert(tracked.length > 0, "Goal 03 regeneration snapshot is empty");
  for (const relative of tracked) {
    const target = path.join(temporaryRoot, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(
      target,
      execFileSync(
        "git",
        ["show", `${snapshot.completion_commit}:${relative}`],
        { cwd: root, maxBuffer: 64 * 1024 * 1024 },
      ),
    );
  }

  const cacheParent = path.join(
    temporaryRoot,
    ".cache/content-reference",
  );
  fs.mkdirSync(cacheParent, { recursive: true });
  fs.symlinkSync(
    sourceCache,
    path.join(cacheParent, "turnbasedgamedata"),
    process.platform === "win32" ? "junction" : "dir",
  );

  const result = spawnSync(
    process.execPath,
    [
      path.join(
        temporaryRoot,
        "tools/universe-reference/bootstrap.mjs",
      ),
      temporaryRoot,
      "--check",
    ],
    {
      cwd: temporaryRoot,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  );
  assert(
    result.status === 0,
    "Goal 03 release-snapshot regeneration failed:\n"
      + `${result.stdout ?? ""}${result.stderr ?? ""}`,
  );
} finally {
  fs.rmSync(temporaryRoot, { recursive: true, force: true });
}

console.log(
  "Standard Universe Goal 03 release snapshot regenerated exactly from "
    + `${snapshot.completion_commit}.`,
);

function git(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
