#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
const profileId = "galactic-baseballer.demon-king.v3_3";

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-demon-profile.mjs",
  ),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));
const profile = await read("demon-profile.json");
const boundaries = await read("demon-release-boundaries.json");
const stages = await read("demon-stages.json");
const periods = await read("demon-stage-periods.json");
const differences = await read("demon-edition-differences.json");
const corrections = await read("demon-released-corrections.json");

assert(profile.length === 1, "Demon King profile count drift");
assert(
  profile[0].id === profileId
    && profile[0].released_version === "3.3"
    && profile[0].retained_baseline_version === "4.4"
    && profile[0].activity_module_id === "5003501"
    && profile[0].origin_stage_numeric_id === "424000"
    && profile[0].released_entry_requirement.minimum_trailblaze_level === 21
    && profile[0].released_entry_requirement.finalitys_vision_early_access
    && profile[0].does_not_replace_profile_ids.join(",")
      === "galactic-baseballer.departure.v2_2"
    && profile[0].runtime_enabled === false,
  "Demon King profile boundary drift",
);
assert(
  profile[0].structured_unlock_quest_locators.reward_unlock_quest_id
    === "6070206"
    && profile[0].structured_unlock_quest_locators.shop_unlock_quest_id
      === "6070210"
    && profile[0].structured_unlock_quest_locators
      .skip_origin_stage_unlock_quest_id === "6020139",
  "Demon King unlock locator drift",
);

assert(boundaries.length === 3, "Demon King release boundary count drift");
assert(
  boundaries.map(({ disposition }) => disposition).sort().join(",")
    === [
      "EvidenceOnly",
      "ReferenceOnlyPermanent",
      "ReferenceOnlyRetainedCorrection",
    ].join(","),
  "Demon King release boundary separation drift",
);

assert(stages.length === 7, "Demon King stage row count drift");
assert(
  stages.filter(({ stage_role: role }) => role === "Origin").length === 1
    && stages.filter(({ stage_role: role }) => role === "Challenge").length
      === 6,
  "Demon King origin/challenge stage separation drift",
);
assert(
  stages.map(({ name_en: name }) => name).join("|")
    === [
      "Initial Planet",
      "V612 - Volcanic Planet",
      "C996 - Cogwheel Planet",
      "F233 - Sugarfrost Planet",
      "M078 - Miniature Planet",
      "D007 - Blissdream Planet",
      "Demon King's Den",
    ].join("|"),
  "Demon King bilingual stage identity drift",
);
assert(
  stages.every(({ rating_thresholds: ratings }) =>
    ratings.map(({ rating }) => rating).join(",") === "C,B,A,S,SS"),
  "Demon King rating threshold order drift",
);
assert(periods.length === 56, "Demon King stage-period count drift");
assert(
  periods.every(({ unresolved_shared_stage: unresolved }) => !unresolved),
  "Demon King shared StageConfig closure drift",
);
assert(
  periods.filter(({ period_rank: rank }) => rank === "PeriodExtra").length
    === 1
    && periods.find(({ period_rank: rank }) => rank === "PeriodExtra")
      .source_numeric_id === "424999",
  "Demon King final period boundary drift",
);

const stageManifestIds = stages.flatMap(
  ({ manifest_record_ids: ids }) => ids,
);
const periodManifestIds = periods.flatMap(
  ({ manifest_record_ids: ids }) => ids,
);
assert(
  new Set(stageManifestIds).size === 7
    && new Set(periodManifestIds).size === 56,
  "Demon King stage manifest exact-once mapping drift",
);

assert(differences.length === 1, "edition difference index count drift");
assert(
  differences[0].constant_comparisons.length === 83,
  "constant comparison denominator drift",
);
assert(
  JSON.stringify(differences[0].relationship_counts)
    === JSON.stringify({
      SharedValueExplicitlyRepeated: 38,
      DemonKingChanged: 25,
      DemonKingAdded: 13,
      DepartureOnlyNotInherited: 7,
    }),
  "constant relationship counts drift",
);
assert(
  differences[0].constant_comparisons
    .filter(({ relationship }) => relationship !==
      "SharedValueExplicitlyRepeated").length === 45,
  "explicit edition-difference count drift",
);
assert(
  differences[0].stage_identity_policy.includes(
    "No cross-profile stage aliases",
  ),
  "stage non-alias policy drift",
);

assert(corrections.length === 3, "released correction count drift");
const mechanicalCorrections = corrections.filter(({ disposition }) =>
  disposition === "ReferenceOnlyReleasedCorrection");
assert(
  mechanicalCorrections.length === 2
    && mechanicalCorrections.every((row) =>
      row.unknown_fields.length >= 3
      && row.rejected_alternatives.length >= 2
      && row.affected_fixtures.length >= 2
      && row.replacement_condition.length > 40),
  "mechanical correction replacement boundary drift",
);
assert(
  corrections.some(({ id, disposition }) =>
    id.endsWith("boothill-ultimate-visual")
      && disposition === "EvidenceOnly"),
  "non-mechanical Version 3.4 correction disposition drift",
);

for (const row of [
  ...profile,
  ...boundaries,
  ...stages,
  ...periods,
  ...differences,
  ...corrections,
]) {
  assert(
    row.profile_ids.length === 1
      && row.profile_ids[0] === profileId
      && row.ownership === "DemonKing"
      && row.coverage_state === "Researched"
      && row.source_refs.length >= 1,
    `Demon King row envelope drift: ${row.id}`,
  );
}

console.log(
  "Demon King profile verified: 1 profile, 3 release boundaries, "
  + "7 stage rows (6 challenges), 56 periods, 45 explicit constant "
  + "differences and 3 released Version 3.4 corrections",
);
