#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import path from "node:path";

const root = path.resolve(".");
const cleanCheckout = process.argv.includes("--clean-checkout");
const sourceCache = process.env.STARCLOCK_SOURCE_CACHE
  ?? "/Users/mikai/.codex/source-caches/goal17-memory-of-chaos";
const sora = process.env.STARCLOCK_SORA_BIN
  ?? "/Users/mikai/CLionProjects/starclock/.cache/tools/sora-cli-0.3.0/bin/sora";
const python = process.env.STARCLOCK_PYTHON
  ?? path.join(root, ".cache/python/bin/python");
const environment = {
  ...process.env,
  STARCLOCK_SOURCE_CACHE: sourceCache,
  STARCLOCK_SORA_BIN: sora,
  STARCLOCK_PYTHON: python,
  CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR
    ?? path.join(root, ".cache/goal17-reader-target"),
};

function run(executable, args) {
  execFileSync(executable, args, { cwd: root, env: environment, stdio: "inherit" });
}
function node(script, ...args) { run(process.execPath, [script, ...args]); }

if (!cleanCheckout) node("tools/memory-of-chaos-reference/foundation.mjs", "--check");
node("tools/memory-of-chaos-reference/inventory.mjs", "--check");
node("tools/memory-of-chaos-reference/manifest.mjs", "--check");
node("tools/memory-of-chaos-reference/contracts.mjs", "--check");

for (const importer of [
  "import-flow.mjs",
  "import-participants.mjs",
  "import-clocks.mjs",
  "import-objectives.mjs",
  "import-turbulence.mjs",
  "import-resources.mjs",
  "import-pools.mjs",
  "import-events.mjs",
  "import-encounters.mjs",
  "import-enemies.mjs",
]) {
  node(`tools/memory-of-chaos-reference/${importer}`, "--check");
}
node("tools/memory-of-chaos-reference/finalize-pack.mjs", "--check");
node("tools/memory-of-chaos-reference/audit-release.mjs", "--check");
node("tools/memory-of-chaos-reference/execute-semantic-fixtures.mjs", "--check");
node("tools/memory-of-chaos-reference/verify-sora-release.mjs");
node("tools/dependency-policy/verify.mjs");
node("tools/workspace/verify-dependencies.mjs");

console.log(`Goal 17 Candidate release verified from ${cleanCheckout ? "clean prospective tree" : "goal worktree"}: source, pack, fixtures, Sora, reader and dependency boundaries pass.`);
