#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
assert(process.argv.length === 2, "usage: node tools/goal20/run-clean-checkout.mjs");
assert(capture("git", ["diff", "--name-only"]) === "" &&
  capture("git", ["ls-files", "--others", "--exclude-standard"]) === "",
"stage every proposed Goal 20 file before clean-checkout acceptance");
const tree = capture("git", ["write-tree"]);
const sourceCommit = capture("git", ["rev-parse", "HEAD"]);
assert(/^[0-9a-f]{40}$/u.test(tree) && /^[0-9a-f]{40}$/u.test(sourceCommit),
  "Goal 20 clean-checkout Git identity is invalid");

const temporaryBase = process.platform === "win32" ? path.dirname(root) : os.tmpdir();
const temporary = fs.mkdtempSync(path.join(temporaryBase, ".g20-"));
const checkout = path.join(temporary, "checkout");
const archive = path.join(temporary, "staged.tar");
try {
  run("git", ["archive", "--format=tar", `--output=${archive}`, tree], root);
  run("git", ["clone", "--quiet", "--no-local", "--no-hardlinks", root, checkout], root);
  clearWorktree(checkout);
  run("tar", ["-xf", archive, "-C", checkout], root);
  run("git", ["add", "--all"], checkout);
  run("git", ["-c", "user.name=Starclock Acceptance", "-c",
    "user.email=acceptance@invalid", "commit", "--quiet", "-m",
    "Goal 20 clean acceptance snapshot"], checkout);
  seedSora(checkout);
  const environment = {
    ...process.env,
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: path.join(checkout, "target"),
    STARCLOCK_TEST_JOBS: "1",
    STARCLOCK_TEST_THREADS: "8",
  };
  run("node", ["tools/repository-check/run.mjs", "--full"], checkout, environment);
  run("node", ["tools/goal20/verify-release-audits.mjs", checkout], checkout, {
    ...environment,
    STARCLOCK_ARTIFACT_CHECK_ONLY: "1",
  });
  run("git", ["diff", "--check"], checkout, environment);
  assert(capture("git", ["status", "--porcelain"], checkout) === "",
    "Goal 20 clean-checkout acceptance modified tracked files");
  console.log(`Goal 20 clean-checkout acceptance passed for staged tree ${tree} (fresh target; no source cache).`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function clearWorktree(checkoutRoot) {
  for (const entry of fs.readdirSync(checkoutRoot)) {
    if (entry !== ".git") fs.rmSync(path.join(checkoutRoot, entry), { recursive: true, force: true });
  }
}

function seedSora(targetRoot) {
  const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/sora-toolchain.json"), "utf8"));
  const executableName = process.platform === "win32" ? "sora.exe" : "sora";
  const relative = path.join(policy.install_root, "bin", executableName);
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
  const archiveName = `${policy.package}-${policy.version}.crate`;
  const archive = path.join(root, ".cache", "tools", "downloads", archiveName);
  assert(fs.existsSync(archive), "pinned Sora 0.3.0 crate archive is unavailable");
  const archiveDestination = path.join(targetRoot, ".cache", "tools", "downloads", archiveName);
  fs.mkdirSync(path.dirname(archiveDestination), { recursive: true });
  fs.copyFileSync(archive, archiveDestination);
}
function capture(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}
function run(command, args, cwd, env = process.env) {
  console.log(`\n==> ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} ${args.join(" ")} exited with ${result.status}`);
}
function assert(condition, message) { if (!condition) throw new Error(message); }
