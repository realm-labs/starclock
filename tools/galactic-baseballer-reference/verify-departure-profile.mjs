#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  return process.argv[index + 1];
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join("tools", "galactic-baseballer-reference", "normalize-departure.mjs"),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const profiles = await read("profiles.json");
const boundaries = await read("release-boundaries.json");
const stages = await read("stages.json");
const periods = await read("stage-periods.json");

assert(profiles.length === 1, "Departure profile count drift");
assert(
  profiles[0].id === "galactic-baseballer.departure.v2_2"
    && profiles[0].released_version === "2.2"
    && profiles[0].retained_baseline_version === "4.4"
    && profiles[0].entry_unlock_quest_id === "6070207"
    && profiles[0].runtime_enabled === false,
  "Departure profile boundary drift",
);
assert(
  boundaries.length === 2
    && boundaries.some(({ disposition }) =>
      disposition === "ReferenceOnlyPermanent")
    && boundaries.some(({ disposition }) => disposition === "EvidenceOnly"),
  "release boundary separation drift",
);
assert(stages.length === 6, "Departure stage count drift");
assert(periods.length === 57, "Departure period count drift");
assert(
  stages.map(({ name_en: name }) => name).join("|")
    === "Volcanic Planet|Cogwheel Planet|Sugarfrost Planet|Miniature Planet|Blissdream Planet|Eternal Black Hole",
  "Departure bilingual stage identity drift",
);
assert(
  stages.every(({ rating_thresholds: ratings }) =>
    ratings.map(({ rating }) => rating).join(",") === "C,B,A,S,SS"),
  "rating threshold order drift",
);
assert(
  periods.filter(({ unresolved_shared_stage: unresolved }) => unresolved)
    .map(({ source_numeric_id: id }) => id).join(",") === "3097,3098,3099",
  "legacy StageID boundary drift",
);
assert(
  periods.every(({ source_refs: refs, manifest_record_ids: ids }) =>
    refs.length >= 1 && ids.length === 1),
  "stage-period provenance drift",
);
const stageManifestIds = stages.flatMap(
  ({ manifest_record_ids: ids }) => ids,
);
const periodManifestIds = periods.flatMap(
  ({ manifest_record_ids: ids }) => ids,
);
assert(
  new Set(stageManifestIds).size === 6
    && new Set(periodManifestIds).size === 57,
  "Departure manifest exact-once mapping drift",
);
for (const row of [...profiles, ...boundaries, ...stages, ...periods]) {
  assert(
    row.profile_ids.length === 1
      && row.profile_ids[0] === "galactic-baseballer.departure.v2_2"
      && row.ownership === "Departure"
      && row.coverage_state === "Researched",
    `row envelope drift: ${row.id}`,
  );
}

console.log(
  "Departure profile verified: 1 profile, 2 release boundaries, "
  + "6 stages, 57 stage periods",
);
