#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import path from "node:path";

const root = process.cwd();
const node = process.execPath;
function run(command, args, extra = {}) {
  execFileSync(command, args, { cwd: root, stdio: "inherit",
    maxBuffer: 512 * 1024 * 1024, ...extra });
}
const source = path.join(root, ".cache/pure-fiction/turnbasedgamedata");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const actualRevision = execFileSync("git", ["-C", source, "rev-parse", "HEAD"],
  { encoding: "utf8" }).trim();
if (actualRevision !== revision) throw new Error("Pure Fiction source revision drift");
if (execFileSync("git", ["-C", source, "status", "--porcelain"],
  { encoding: "utf8" }).trim()) throw new Error("Pure Fiction source cache is dirty");
for (const script of ["inventory.mjs", "manifest.mjs", "contracts.mjs"])
  run(node, [`tools/pure-fiction-reference/${script}`, "--check"]);
for (const batch of ["G15-P1-B1", "G15-P1-B2", "G15-P1-B3", "G15-P1-B4",
  "G15-P1-B5", "G15-P1-B6", "G15-P1-B7", "G15-P2-B2", "G15-P2-B3",
  "G15-P2-B4", "G15-P2-B5", "G15-P2-B6"])
  run(node, ["tools/pure-fiction-reference/build-pack.mjs", `--batch=${batch}`, "--check"]);
run(node, ["tools/pure-fiction-reference/verify-pack.mjs"]);
run(node, ["tools/pure-fiction-reference/execute-semantic-fixtures.mjs"]);
run(node, ["tools/pure-fiction-reference/audit-release.mjs"]);
run(node, ["tools/pure-fiction-reference/verify-authoring.mjs"]);
console.log("Pure Fiction release verification passed: pinned sources, 796 obligations, "
  + "6,014 normalized rows, 37 Sora tables, 6,810 workbook rows and 18 fixtures.");
