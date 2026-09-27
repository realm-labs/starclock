import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

test("pre-existing loader output rejects without deleting another task's files", () => {
  const base = path.resolve(os.tmpdir());
  const root = fs.mkdtempSync(path.join(base, "starclock-release-cleanup-"));
  try {
    const policy = path.join(root, "policy");
    fs.mkdirSync(policy);
    fs.writeFileSync(path.join(policy, "sora-toolchain.json"), JSON.stringify({
      version: "0.6.1", install_root: ".cache/test-sora",
    }));
    const bin = path.join(root, ".cache/test-sora/bin");
    fs.mkdirSync(bin, { recursive: true });
    // The guard must reject before any authoring tool or Cargo is invoked.
    fs.writeFileSync(path.join(bin, process.platform === "win32" ? "sora.exe" : "sora"), "");
    const existing = path.join(root,
      "tools/divergent-universe-reference/bundle-loader/src/generated");
    fs.mkdirSync(existing, { recursive: true });
    fs.writeFileSync(path.join(existing, "owned-by-another-task.txt"), "preserve\n");
    const result = spawnSync(process.execPath,
      [path.join(import.meta.dirname, "verify-sora-release.mjs"), root], {
        encoding: "utf8",
      });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /ephemeral generated-reader directory already exists/u);
    assert.equal(fs.readFileSync(path.join(existing, "owned-by-another-task.txt"), "utf8"),
      "preserve\n");
    assert.deepEqual(fs.readdirSync(path.join(root, ".cache/divergent-universe-release-check")), []);
  } finally {
    assert.equal(path.dirname(root), base);
    assert.ok(path.basename(root).startsWith("starclock-release-cleanup-"));
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("existing-build verification refuses paths outside its owned cache boundary", () => {
  const base = path.resolve(os.tmpdir());
  const root = fs.mkdtempSync(path.join(base, "starclock-release-cleanup-"));
  try {
    fs.mkdirSync(path.join(root, "policy"));
    fs.writeFileSync(path.join(root, "policy/sora-toolchain.json"), JSON.stringify({
      version: "0.6.1", install_root: ".cache/test-sora",
    }));
    const bin = path.join(root, ".cache/test-sora/bin");
    fs.mkdirSync(bin, { recursive: true });
    fs.writeFileSync(path.join(bin, process.platform === "win32" ? "sora.exe" : "sora"), "");
    const outside = path.join(root, "outside");
    fs.mkdirSync(outside);
    fs.writeFileSync(path.join(outside, "preserve.txt"), "preserve\n");
    const result = spawnSync(process.execPath,
      [path.join(import.meta.dirname, "verify-sora-release.mjs"), root,
        "--verify-existing", outside], { encoding: "utf8" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unsafe release-check build directory/u);
    assert.equal(fs.readFileSync(path.join(outside, "preserve.txt"), "utf8"), "preserve\n");
  } finally {
    assert.equal(path.dirname(root), base);
    assert.ok(path.basename(root).startsWith("starclock-release-cleanup-"));
    fs.rmSync(root, { recursive: true, force: true });
  }
});
