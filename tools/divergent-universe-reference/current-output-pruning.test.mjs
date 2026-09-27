import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { sha256 } from "./lib/common.mjs";
import { pruneObsoletePackFiles } from "./current-output-pruning.mjs";

async function fixture(run) {
  const base = path.resolve(os.tmpdir());
  const root = await fs.mkdtemp(path.join(base, "starclock-owned-pack-"));
  const pack = path.join(root, "content-reference/divergent-universe-v1");
  try {
    await fs.mkdir(pack, { recursive: true });
    const bytes = Buffer.from("[]\n");
    const names = ["current.json", "obsolete-a.json", "obsolete-b.json"];
    for (const file of [...names, "unindexed.json"])
      await fs.writeFile(path.join(pack, file), bytes);
    await fs.writeFile(path.join(pack, "pack-index.json"), JSON.stringify([{
      file_digests: names.map((file) => ({ file, bytes: bytes.length, sha256: sha256(bytes) })),
    }]));
    await run({ root, pack, contracts: [{ file: "current.json" }, { file: "pack-index.json" }] });
  } finally {
    assert.equal(path.dirname(root), base);
    assert.ok(path.basename(root).startsWith("starclock-owned-pack-"));
    await fs.rm(root, { recursive: true, force: true });
  }
}

test("prunes only unchanged index-owned outputs absent from the current contract", () =>
  fixture(async ({ root, pack, contracts }) => {
    assert.deepEqual(await pruneObsoletePackFiles(root, contracts, false),
      ["obsolete-a.json", "obsolete-b.json"]);
    assert.deepEqual((await fs.readdir(pack)).sort(),
      ["current.json", "pack-index.json", "unindexed.json"]);
  }));

test("one edited output rejects before removing any target", () =>
  fixture(async ({ root, pack, contracts }) => {
    await fs.writeFile(path.join(pack, "obsolete-b.json"), "edited\n");
    const before = (await fs.readdir(pack)).sort();
    await assert.rejects(pruneObsoletePackFiles(root, contracts, false), /edited generated output/u);
    assert.deepEqual((await fs.readdir(pack)).sort(), before);
  }));

test("check mode reports obsolete output without changing the tree", () =>
  fixture(async ({ root, pack, contracts }) => {
    const before = (await fs.readdir(pack)).sort();
    await assert.rejects(pruneObsoletePackFiles(root, contracts, true), /obsolete generated output/u);
    assert.deepEqual((await fs.readdir(pack)).sort(), before);
  }));

test("unsafe indexed paths reject before removing any target", () =>
  fixture(async ({ root, pack, contracts }) => {
    const indexPath = path.join(pack, "pack-index.json");
    const index = JSON.parse(await fs.readFile(indexPath, "utf8"));
    index[0].file_digests.push({ file: "../outside.json", bytes: 0, sha256: "" });
    await fs.writeFile(indexPath, JSON.stringify(index));
    const before = (await fs.readdir(pack)).sort();
    await assert.rejects(pruneObsoletePackFiles(root, contracts, false), /unsafe indexed output/u);
    assert.deepEqual((await fs.readdir(pack)).sort(), before);
  }));
