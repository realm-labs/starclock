#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference"));
const correction = json(path.join(
  root,
  "content-manifests/currency-wars-v1/source-correction.json",
));
const repository = path.join(sourceCache, "turnbasedgamedata");

assert(correction.schema_revision
  === "starclock.currency-wars-source-correction.v1",
"unsupported source correction revision");
assert(correction.goal_id === "currency-wars-reference-v1"
  && correction.batch === "G12-P0-B5",
"source correction identity drift");
assert(correction.execution_batch_additions.join(",")
  === "G12-P0-B5,G12-P1-B10",
"source correction batch additions drift");
assert(correction.authoritative_selector.guide_type === "GridFight"
  && correction.authoritative_selector.guide_tab_id === 1003
  && correction.authoritative_selector.guide_data_id === 301,
"authoritative GridFight selector drift");
assert(correction.supersedes.selector.sub_mode === "TournRogue"
  && correction.supersedes.selector.tourn_mode === "Tourn3",
"superseded Tourn selector receipt drift");

const revision = correction.snapshot.revision;
assert(git(["rev-parse", "HEAD"]).trim() === revision,
"source correction cache revision drift");
const treePaths = git(["ls-tree", "-r", "--name-only", revision])
  .split(/\r?\n/u).filter(Boolean);
const gridFightPaths = treePaths.filter((entry) => /GridFight/iu.test(entry));
assert(gridFightPaths.length === 1137
  && gridFightPaths.filter((entry) =>
    /^ExcelOutput\/GridFight.*\.json$/u.test(entry)).length === 153
  && gridFightPaths.filter((entry) => entry.startsWith("Config/")).length
    === 984,
"pinned GridFight Git-tree closure drift");

for (const receipt of correction.authoritative_selector.source_records) {
  const rows = gitJson(receipt.path);
  const row = rows[Number(receipt.locator)];
  assert(row !== undefined, `missing selector row ${receipt.path}`);
  assert(digest(row) === receipt.sha256,
    `selector row digest drift ${receipt.path}`);
  if (receipt.path.endsWith("GuideRogueTab.json"))
    assert(row.ID === 1003 && row.GuideType === "GridFight",
      "GuideRogueTab GridFight selector drift");
  else
    assert(row.ID === 301 && row.TabID === 1003,
      "GuideRogueData Currency Wars selector drift");
}

const textEn = gitJson("TextMap/TextMapEN.json");
const textZh = gitJson("TextMap/TextMapCHS.json");
for (const hash of ["12766196657685680910", "11179573777653333385"]) {
  assert(textEn[hash] === "Currency Wars"
    && textZh[hash] === "货币战争",
  `Currency Wars bilingual selector text drift ${hash}`);
}

console.log(
  "Currency Wars source correction verified (GuideType GridFight; " +
  "153 tables, 984 configs, 1,137 exact pinned paths; Tourn3 superseded).",
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  assert(args[index + 1] && !args[index + 1].startsWith("--"),
    `${flag} requires a value`);
  return args[index + 1];
}
function git(gitArgs, encoding = "utf8") {
  return execFileSync("git", ["-C", repository, ...gitArgs], {
    encoding,
    maxBuffer: 128 * 1024 * 1024,
  });
}
function gitJson(sourcePath) {
  return JSON.parse(git(["show", `${revision}:${sourcePath}`]));
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function digest(value) {
  return crypto.createHash("sha256")
    .update(`${JSON.stringify(value)}\n`)
    .digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
