import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
assert(process.argv.length === 2, "usage: node tools/release/run-clean-checkout.mjs");
const temporaryRoot = path.resolve(os.tmpdir());
const cacheRoot = path.join(temporaryRoot, "scg01");
const toolSeed = path.join(root, ".cache", "goal01-clean-tool-cache");
const archive = path.join(root, ".cache", "goal01-clean-checkout.tar");
assert(path.dirname(cacheRoot) === temporaryRoot && path.basename(cacheRoot) === "scg01", "clean-checkout path escaped the exact temporary target");
assert(path.relative(root, archive).replaceAll("\\", "/") === ".cache/goal01-clean-checkout.tar", "clean-checkout archive escaped repository cache");
assert(path.relative(root, toolSeed).replaceAll("\\", "/") === ".cache/goal01-clean-tool-cache", "clean tool-cache path escaped repository cache");
const unstaged = capture("git", ["diff", "--name-only"]).trim();
assert(unstaged.length === 0, `unstaged changes cannot enter clean acceptance: ${unstaged}`);
const tree = capture("git", ["write-tree"]).trim();
assert(/^[0-9a-f]{40}$/.test(tree), "failed to create staged Git tree");

const installedTools = path.join(cacheRoot, ".cache", "tools");
if (fs.existsSync(installedTools)) {
  fs.rmSync(toolSeed, { recursive: true, force: true });
  fs.cpSync(installedTools, toolSeed, { recursive: true });
}
fs.rmSync(cacheRoot, { recursive: true, force: true });
fs.rmSync(archive, { force: true });
fs.mkdirSync(cacheRoot, { recursive: true });
run("git", ["archive", "--format=tar", `--output=${archive}`, tree], root);
run("tar", ["-xf", archive, "-C", cacheRoot], root);
fs.rmSync(archive, { force: true });
run("git", ["init", "--quiet"], cacheRoot);
run("git", ["add", "--all"], cacheRoot);
run("git", ["-c", "user.name=Starclock Acceptance", "-c", "user.email=acceptance@invalid", "commit", "--quiet", "-m", "clean acceptance snapshot"], cacheRoot);

const sourceDownload = path.join(root, ".cache/tools/downloads/sora-cli-0.3.0.crate");
if (fs.existsSync(toolSeed)) {
  fs.cpSync(toolSeed, path.join(cacheRoot, ".cache", "tools"), { recursive: true });
} else if (fs.existsSync(sourceDownload)) {
  const targetDownload = path.join(cacheRoot, ".cache/tools/downloads/sora-cli-0.3.0.crate");
  fs.mkdirSync(path.dirname(targetDownload), { recursive: true });
  fs.copyFileSync(sourceDownload, targetDownload);
}
run("node", ["tools/sora/install.mjs"], cacheRoot);
run("node", ["tools/repository-check/run.mjs"], cacheRoot);
console.log(`Clean-checkout acceptance passed for staged tree ${tree} (no inherited build or source cache).`);

function capture(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}: ${result.stderr}`);
  return result.stdout;
}
function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}`);
}
function assert(condition, message) { if (!condition) throw new Error(message); }
