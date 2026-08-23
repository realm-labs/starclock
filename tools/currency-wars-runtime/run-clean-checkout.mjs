#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const target = process.argv[2] ?? hostTarget();
assert(process.argv.length <= 3,
  "usage: node tools/currency-wars-runtime/run-clean-checkout.mjs [native-target]");

const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-currency-wars-runtime-clean-"),
);
const checkout = path.join(temporary, "checkout");
fs.mkdirSync(checkout);
try {
  copyCurrentTree(checkout);
  run("git", ["init", "--quiet"], checkout);
  run("git", ["add", "--all"], checkout);
  const tree = capture("git", ["write-tree"], checkout);
  const commitEnvironment = {
    ...process.env,
    GIT_AUTHOR_DATE: "2026-08-24T00:00:00Z",
    GIT_COMMITTER_DATE: "2026-08-24T00:00:00Z",
  };
  run("git", [
    "-c", "user.name=Starclock Acceptance",
    "-c", "user.email=acceptance@invalid",
    "commit", "--quiet", "-m", "Goal 21 clean acceptance snapshot",
  ], checkout, commitEnvironment);
  seedSora(checkout);
  const environment = {
    ...process.env,
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: path.join(checkout, "target"),
    PYTHONDONTWRITEBYTECODE: "1",
    STARCLOCK_PYTHON: process.env.STARCLOCK_PYTHON
      ?? "/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/"
        + "dependencies/python/bin/python3",
  };
  const commands = [
    ["node", "tools/currency-wars-runtime/generate-dispositions.mjs", "--check"],
    ["node", "tools/currency-wars-runtime/generate-coverage-and-release.mjs", "--check"],
    ["node", "tools/currency-wars-runtime/generate-verification-scaffold.mjs", "--check"],
    ["node", "tools/currency-wars-runtime/verify-coverage-and-release.mjs"],
    ["node", "tools/currency-wars-runtime/verify-verification-scaffold.mjs"],
    ["node", "tools/dependency-policy/verify.mjs"],
    ["node", "tools/workspace/verify-dependencies.mjs"],
    ["node", "tools/repository-check/verify-source-policy.mjs"],
    ["node", "tools/repository-check/verify-native-handlers.mjs"],
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    ["cargo", "test", "--workspace"],
    ["cargo", "test", "-p", "starclock-test-kit", "--features", "exhaustive",
      "--test", "exhaustive_suite"],
    ["cargo", "test", "--release", "-p", "starclock-ai", "--test",
      "currency_wars_matrix", "--", "--ignored", "--exact",
      "generated_legal_matrix_completes_real_battles_and_fresh_replay"],
    ["node", "tools/currency-wars-runtime/native-evidence.mjs", "--target", target,
      "--check"],
  ];
  for (const [command, ...args] of commands)
    run(command, args, checkout, environment);
  assert(capture("git", ["status", "--porcelain"], checkout) === "",
    "clean-checkout acceptance modified tracked files");
  console.log(
    `Currency Wars runtime clean-checkout acceptance passed for tree ${tree} `
      + `on ${target} (fresh build target and no inherited source cache).`,
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function copyCurrentTree(destination) {
  const files = captureBytes("git", [
    "ls-files", "-z", "--cached", "--others", "--exclude-standard",
  ], root).toString("utf8").split("\0").filter(Boolean);
  for (const relative of files) {
    const source = path.join(root, relative);
    if (!fs.existsSync(source)) continue;
    const targetPath = path.join(destination, relative);
    fs.mkdirSync(path.dirname(targetPath), { recursive: true });
    fs.cpSync(source, targetPath, { dereference: false, recursive: true });
  }
}

function seedSora(destinationRoot) {
  const policy = JSON.parse(fs.readFileSync(
    path.join(root, "policy/sora-toolchain.json"),
    "utf8",
  ));
  const relative = path.join(policy.install_root, "bin", "sora");
  const source = path.join(root, relative);
  assert(fs.existsSync(source), `pinned Sora ${policy.version} executable is unavailable`);
  const destination = path.join(destinationRoot, relative);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(source, destination);
  fs.chmodSync(destination, 0o755);
}

function hostTarget() {
  if (process.platform === "darwin" && process.arch === "arm64")
    return "aarch64-apple-darwin";
  if (process.platform === "linux" && process.arch === "x64")
    return "x86_64-unknown-linux-gnu";
  if (process.platform === "win32" && process.arch === "x64")
    return "x86_64-pc-windows-msvc";
  throw new Error(`unsupported clean-checkout host ${process.platform}/${process.arch}`);
}

function capture(command, args, cwd) {
  return captureBytes(command, args, cwd).toString("utf8").trim();
}

function captureBytes(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: null });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command} ${args.join(" ")} failed: ${result.stderr.toString("utf8")}`);
  return result.stdout;
}

function run(command, args, cwd, env = process.env) {
  console.log(`\n==> ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command} ${args.join(" ")} exited with ${result.status}`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
