#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const root = path.resolve(".");
const project = path.join(root, "config/memory-of-chaos/project.toml");
const generated = path.join(root, "config/memory-of-chaos-generated");
const sora = process.env.STARCLOCK_SORA_BIN
  ?? path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
function assert(condition, message) { if (!condition) throw new Error(message); }
function run(args) { execFileSync(sora, args, { cwd: root, stdio: "inherit" }); }
async function files(directory) {
  const output = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) output.push(...await files(target));
    else output.push(target);
  }
  return output.sort((left, right) => left.localeCompare(right, "en"));
}
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim() === "sora 0.3.0",
  "pinned Sora version drift");
run(["check", "--project", project]);
const temporary = await mkdtemp(path.join(os.tmpdir(), "starclock-g17-sora-"));
run(["--serial", "schema-lock", "--project", project, "--out", path.join(temporary, "schema.lock")]);
run(["--serial", "excel-template", "--project", project, "--out", path.join(temporary, "templates")]);
run(["--serial", "gen", "--target", "rust", "--project", project, "--out", path.join(temporary, "readers/rust"), "--format-code", "never"]);
const committedCore = (await files(generated)).filter((file) => {
  const relative = path.relative(generated, file);
  return relative === "schema.lock" || relative.startsWith("templates/") || relative.startsWith("readers/");
});
const regeneratedCore = await files(temporary);
assert(committedCore.length === regeneratedCore.length,
  `generated file count drift ${committedCore.length}/${regeneratedCore.length}`);
for (const committed of committedCore) {
  const relative = path.relative(generated, committed);
  const regenerated = path.join(temporary, relative);
  if (relative.startsWith("templates/")) {
    const committedMembers = zipMembers(committed);
    assert(JSON.stringify(committedMembers) === JSON.stringify(zipMembers(regenerated)),
      `${relative} ZIP member drift`);
    for (const member of committedMembers.filter((item) => item !== "docProps/core.xml")) {
      assert(zipMember(committed, member).equals(zipMember(regenerated, member)),
        `${relative}:${member} template drift`);
    }
  } else {
    assert((await readFile(committed)).equals(await readFile(regenerated)), `${relative} drift`);
  }
}
const lock = JSON.parse(await readFile(path.join(generated, "schema.lock"), "utf8"));
assert(lock.schema.tables.length === 27, "schema-lock table count drift");
const templates = (await files(path.join(generated, "templates"))).filter((file) => file.endsWith(".xlsx"));
assert(templates.length === 3, "template workbook count drift");
const readers = (await files(path.join(generated, "readers/rust"))).filter((file) => file.endsWith(".rs"));
assert(readers.length >= 27, "generated reader count drift");
const tree = createHash("sha256");
for (const file of committedCore) {
  tree.update(path.relative(generated, file));
  tree.update("\0");
  tree.update(await readFile(file));
  tree.update("\0");
}
console.log(`Goal 17 Sora generated artifacts verified: 27 tables, 3 templates, ${readers.length} Rust files, tree=${tree.digest("hex")}.`);

function zipMembers(workbook) {
  return execFileSync("unzip", ["-Z1", workbook], { encoding: "utf8" })
    .trim().split("\n").filter(Boolean).sort();
}
function zipMember(workbook, member) {
  return execFileSync("unzip", ["-p", workbook, member.replaceAll("[", "\\[").replaceAll("]", "\\]")], {
    encoding: null,
    maxBuffer: 64 * 1024 * 1024,
  });
}
