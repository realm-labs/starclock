#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference"));
const output = path.join(
  root,
  "content-manifests",
  "divergent-universe-v1",
  "source-inventory.json",
);
const standardInventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "standard-universe-v1",
  "source-inventory.json",
), "utf8"));
const inheritedPaths = new Set(standardInventory.records.map(
  ({ path: sourcePath }) => sourcePath,
));
const sources = [
  {
    id: "turnbasedgamedata",
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    root: path.join(sourceCache, "turnbasedgamedata"),
  },
  {
    id: "starrailres",
    repository: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    root: path.join(sourceCache, "StarRailRes"),
  },
];

const presentationTables = new Set([
  "ActivityRewardRogueEndless.json",
  "GuideRogueData.json",
  "GuideRogueTab.json",
  "RogueAeonStoryConfig.json",
  "RogueCommonDialogue.json",
  "RogueCommonModeTitle.json",
  "RogueDialogueDynamicDisplay.json",
  "RogueGuideActivityPanelData.json",
  "RogueHandBookEvent.json",
  "RogueHandBookEventType.json",
  "RogueHandbookMiracle.json",
  "RogueHandbookMiracleType.json",
  "RogueHandbookType.json",
  "RogueHint.json",
  "RogueImage.json",
  "RogueScoreReward.json",
  "RogueTalkNameColor.json",
  "RogueTalkNameConfig.json",
]);
const tournPresentationTables = new Set([
  "RogueTournCollection.json",
  "RogueTournCollectionConfig.json",
  "RogueTournConstClient.json",
  "RogueTournContentDisplay.json",
  "RogueTournExhibition.json",
  "RogueTournExhibitionConfig.json",
  "RogueTournExpReward.json",
  "RogueTournExpScore.json",
  "RogueTournExpScore_Index_ScoreExpID.json",
  "RogueTournFormulaAeonIcon.json",
  "RogueTournFormulaDisplay.json",
  "RogueTournHandBookEvent.json",
  "RogueTournHandbookMiracle.json",
  "RogueTournHexDisplay.json",
  "RogueTournMiracleDisplay.json",
  "RogueTournMiscDisplay.json",
  "RogueTournRecordShowcase.json",
  "RogueTournWeeklyDisplay.json",
]);
const tournTestTables = new Set([
  "RogueTournMiracleGroupTest.json",
  "RogueTournMiracleTest.json",
]);
const starRailResPaths = new Set([
  "info.json",
  ...["cn", "en"].flatMap((locale) =>
    ["blessings", "blocks", "curios", "events"].map(
      (family) => `index_new/${locale}/simulated_${family}.json`,
    )),
]);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}
function git(source, gitArgs, encoding = "utf8") {
  return execFileSync("git", [
    "-c",
    "http.version=HTTP/1.1",
    "-C",
    source.root,
    ...gitArgs,
  ], {
    encoding,
    maxBuffer: 128 * 1024 * 1024,
  });
}
function additionalModeEntry(relativePath) {
  return (
    relativePath === "ExcelOutput/StageConfig.json" ||
    /^TextMap\/TextMap(?:EN|CHS)\.json$/u.test(relativePath) ||
    /^Config\/ConfigAdventureModifier\/AdventureModifier_Rogue_Tourn1\.json$/u
      .test(relativePath) ||
    /^Config\/Level\/GroupTemplateGraph\/03_Rogue\/RogueTourn230\/.*\.json$/u
      .test(relativePath) ||
    /^Config\/Level\/Maze\/MazeRogue\/RogueTourn\/.*\.json$/u
      .test(relativePath)
  );
}
function selected(sourceId, relativePath) {
  if (sourceId === "starrailres") return starRailResPaths.has(relativePath);
  return inheritedPaths.has(relativePath) || additionalModeEntry(relativePath);
}
function classify(sourceId, relativePath) {
  if (sourceId === "starrailres") {
    return {
      family: "public_index_cross_check",
      selected_by: "bilingual released-resource index for shared identity review",
    };
  }
  if (/^TextMap\//u.test(relativePath)) {
    return {
      family: "localized_text_evidence",
      selected_by: "complete pinned EN/CHS TextMap for referenced hash closure",
    };
  }
  if (relativePath === "ExcelOutput/StageConfig.json") {
    return {
      family: "encounter_stage_evidence",
      selected_by: "complete pinned StageConfig for encounter and wave closure",
    };
  }
  if (relativePath
    === "Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json") {
    return {
      family: "divergent_adventure_modifier_evidence",
      selected_by: "Divergent Universe-named Adventure outcome modifier",
    };
  }
  if (/^Config\/Level\/GroupTemplateGraph\/03_Rogue\/RogueTourn230\//u
    .test(relativePath)) {
    return {
      family: "divergent_service_graph_candidate",
      selected_by: "Divergent Universe secret gamble or Curio-composition graph",
    };
  }
  if (/^Config\/Level\/Maze\/MazeRogue\/RogueTourn\//u.test(relativePath)) {
    return relativePath.includes("Adv")
      ? {
        family: "divergent_adventure_graph_candidate",
        selected_by: "Divergent Universe abstract Adventure room graph",
      }
      : {
        family: "divergent_maze_graph_candidate",
        selected_by: "Divergent Universe door, boss, monster or room graph",
      };
  }
  if (/^Config\/ConfigAbility\/Level\/Level_RogueBuff_Ability_(?:Tourn1|HEX_S[13])(?:\.layout)?\.json$/u
    .test(relativePath)) {
    return {
      family: "divergent_mechanic_evidence",
      selected_by: "Divergent Universe direct released ability/layout program",
    };
  }
  if (relativePath.startsWith("Config/ConfigAbility/")) {
    if (/Nous/u.test(relativePath)) {
      return {
        family: "gold_and_gears_mechanic_exclusion_evidence",
        selected_by: "Gold and Gears-named ability retained to prove exclusion",
      };
    }
    if (/RogueMagic/u.test(relativePath)) {
      return {
        family: "unknowable_domain_mechanic_exclusion_evidence",
        selected_by: "Unknowable Domain-named ability retained to prove exclusion",
      };
    }
    if (/DLC1|RogueDLC/u.test(relativePath)) {
      return {
        family: "swarm_disaster_mechanic_exclusion_evidence",
        selected_by: "Swarm Disaster-named ability retained to prove exclusion",
      };
    }
    return {
      family: "shared_mechanic_evidence_candidate",
      selected_by: "shared Rogue ability requiring row-level reachability review",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueDialogue\/RogueEventTourn[12]\//u
    .test(relativePath)) {
    return {
      family: "divergent_occurrence_graph_candidate",
      selected_by: "Divergent Universe occurrence action/option graph",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueNPC\/RogueNPC_(?:230|310|330|380|410)\//u
    .test(relativePath)) {
    return {
      family: "divergent_npc_graph_candidate",
      selected_by: "candidate NPC graph from an explicitly named Divergent module seed",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueModifier\/RogueTournGodMode\//u
    .test(relativePath)) {
    return {
      family: "divergent_test_exclusion_evidence",
      selected_by: "mode GodMode fixture retained to prove fail-closed exclusion",
    };
  }
  if (relativePath.startsWith("Config/Level/Rogue")) {
    if (/Tourn/u.test(relativePath)) {
      return {
        family: "divergent_level_graph_candidate",
        selected_by: "Divergent Universe-named level graph requiring reference closure",
      };
    }
    return {
      family: relativePath.includes("/RogueDialogue/")
        ? "shared_occurrence_graph_candidate"
        : "shared_level_graph_candidate",
      selected_by: "shared Rogue graph requiring explicit transitive reachability review",
    };
  }

  const name = path.posix.basename(relativePath);
  if (/^RogueTourn.*\.json$/u.test(name)) {
    if (tournTestTables.has(name)) {
      return {
        family: "divergent_test_exclusion_evidence",
        selected_by: "mode test table retained to prove fail-closed exclusion",
      };
    }
    return tournPresentationTables.has(name)
      ? {
        family: "divergent_presentation_account_locator",
        selected_by: "mode display, handbook or account table retained only as a locator",
      }
      : {
        family: "divergent_structured_candidate",
        selected_by: "RogueTourn table requiring row-level module and ownership review",
      };
  }
  if (/^RogueNous.*\.json$/u.test(name)) {
    return {
      family: "gold_and_gears_structured_exclusion_evidence",
      selected_by: "RogueNous table retained to prove Gold and Gears exclusion",
    };
  }
  if (/^RogueMagic.*\.json$/u.test(name)) {
    return {
      family: "unknowable_domain_structured_exclusion_evidence",
      selected_by: "RogueMagic table retained to prove Unknowable Domain exclusion",
    };
  }
  if (/^RogueDLC.*\.json$/u.test(name)) {
    return {
      family: "swarm_disaster_structured_exclusion_evidence",
      selected_by: "RogueDLC table retained to prove Swarm Disaster/framework boundary",
    };
  }
  if (/^(ActivityRogue|RogueEndless|RoguePersona)/u.test(name)) {
    return {
      family: "other_mode_exclusion_evidence",
      selected_by: "explicit other-mode table retained to prove ownership exclusion",
    };
  }
  if (presentationTables.has(name)) {
    return {
      family: "presentation_account_exclusion_evidence",
      selected_by: "shared presentation/account table retained only to prove exclusion",
    };
  }
  return {
    family: "shared_structured_candidate",
    selected_by: "generic Rogue table requiring Divergent row-level reachability review",
  };
}
function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

if (standardInventory.records.length !== 2646)
  throw new Error("Goal 03 source inventory denominator drift");

const records = [];
for (const source of sources) {
  const revision = git(source, ["rev-parse", "HEAD"]).trim();
  if (revision !== source.revision)
    throw new Error(`source revision mismatch for ${source.id}: ${revision}`);
  if (git(source, ["status", "--porcelain"]).trim())
    throw new Error(`source cache has local changes: ${source.id}`);
  const tracked = git(source, ["ls-tree", "-r", "--name-only", "HEAD"])
    .split(/\r?\n/u)
    .filter(Boolean)
    .filter((relativePath) => selected(source.id, relativePath))
    .sort(compareText);
  for (const relativePath of tracked) {
    const bytes = git(source, ["cat-file", "blob", `HEAD:${relativePath}`], null);
    records.push({
      repository: source.id,
      path: relativePath,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      bytes: bytes.length,
      ...classify(source.id, relativePath),
    });
  }
}
records.sort((left, right) =>
  compareText(`${left.repository}/${left.path}`, `${right.repository}/${right.path}`));

const families = [...new Set(records.map(({ family }) => family))].sort(compareText);
const counts = {
  total: records.length,
  by_repository: Object.fromEntries(sources.map(({ id }) => [
    id,
    records.filter(({ repository }) => repository === id).length,
  ])),
  by_family: Object.fromEntries(families.map((family) => [
    family,
    records.filter((record) => record.family === family).length,
  ])),
};
const count = (family) => counts.by_family[family] ?? 0;
const additions = records.filter(({ repository, path: sourcePath }) =>
  repository === "turnbasedgamedata" && !inheritedPaths.has(sourcePath));
const payload = {
  schema_revision: "starclock.divergent-universe-source-inventory.v1",
  goal_id: "divergent-universe-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    repositories: sources.map(({ id, repository, revision }) => ({
      id,
      repository,
      revision,
    })),
    hash_basis: "raw Git blob bytes at the pinned revision",
  },
  selection_contract: {
    inherited:
      "all 2,646 Goal 03 source files remain available for shared-ID, enemy, ability and reachability closure",
    structured:
      "all 64 RogueTourn tables plus inherited shared Rogue tables; named other modes remain exclusion evidence",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual StarRailRes simulated-universe indexes",
    mechanics:
      "all inherited Rogue ability/level/dialogue files plus direct Tourn/Hex programs and focused service, maze and Adventure entries",
    encounters:
      "complete StageConfig and inherited enemy/monster definitions retained for later row-level wave closure",
    denominator_rule:
      "file closure only; no content-row denominator, ownership or reachability is implied before G11-P0-B3",
  },
  classification_policy: {
    divergent_structured_candidate:
      "RogueTourn structured table requiring row-level module and ownership proof",
    divergent_mechanic_evidence:
      "Divergent Universe direct released ability/layout program",
    divergent_adventure_modifier_evidence:
      "mode-named Adventure modifier retained as abstract outcome evidence",
    divergent_service_graph_candidate:
      "mode secret gamble or Curio-composition service graph",
    divergent_adventure_graph_candidate:
      "mode Adventure room graph retained for abstract outcome closure",
    divergent_maze_graph_candidate:
      "mode door, boss, monster or room graph",
    divergent_occurrence_graph_candidate:
      "mode occurrence action/option graph",
    divergent_npc_graph_candidate:
      "candidate NPC graph from an explicitly named module seed",
    divergent_level_graph_candidate:
      "other mode-named level graph requiring transitive closure",
    divergent_presentation_account_locator:
      "mode display, handbook or account source retained only as a locator",
    divergent_test_exclusion_evidence:
      "mode test table retained to prove fail-closed exclusion",
    shared_structured_candidate:
      "generic Rogue table requiring Divergent row-level reachability proof",
    shared_mechanic_evidence_candidate:
      "shared Rogue ability requiring row-level reachability proof",
    shared_occurrence_graph_candidate:
      "shared Rogue dialogue graph requiring occurrence reachability proof",
    shared_level_graph_candidate:
      "shared Rogue graph requiring room, NPC or service reachability proof",
    encounter_stage_evidence: "complete StageConfig retained for exact wave closure",
    localized_text_evidence: "complete bilingual TextMap retained for hash resolution",
    public_index_cross_check: "released-resource bilingual index used for identity review",
    gold_and_gears_structured_exclusion_evidence:
      "RogueNous source retained to prove Gold and Gears exclusion",
    gold_and_gears_mechanic_exclusion_evidence:
      "Gold and Gears-named ability retained to prove exclusion",
    swarm_disaster_structured_exclusion_evidence:
      "RogueDLC source retained to prove Swarm Disaster/framework exclusion",
    swarm_disaster_mechanic_exclusion_evidence:
      "Swarm Disaster-named ability retained to prove exclusion",
    unknowable_domain_structured_exclusion_evidence:
      "RogueMagic source retained to prove Unknowable Domain exclusion",
    unknowable_domain_mechanic_exclusion_evidence:
      "Unknowable Domain-named ability retained to prove exclusion",
    other_mode_exclusion_evidence:
      "explicit other-mode table retained to prove ownership exclusion",
    presentation_account_exclusion_evidence:
      "shared presentation/account table retained only to prove exclusion",
  },
  closure: {
    inherited_goal03_files: inheritedPaths.size,
    turnbasedgamedata_additions: additions.length,
    rogue_tourn_tables: records.filter(({ path: sourcePath }) =>
      /^ExcelOutput\/RogueTourn[^/]*\.json$/u.test(sourcePath)).length,
    direct_ability_and_layout_files: count("divergent_mechanic_evidence"),
    occurrence_graph_files: count("divergent_occurrence_graph_candidate"),
    npc_graph_files: count("divergent_npc_graph_candidate"),
    service_graph_files: count("divergent_service_graph_candidate"),
    adventure_graph_files: count("divergent_adventure_graph_candidate"),
    maze_graph_files: count("divergent_maze_graph_candidate"),
    named_other_mode_exclusion_files: records.filter(({ family }) =>
      family.includes("_exclusion_evidence")
      && family !== "presentation_account_exclusion_evidence"
      && family !== "divergent_test_exclusion_evidence").length,
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    unclassified_selected_files: 0,
  },
  counts,
  records,
};
if (payload.closure.rogue_tourn_tables !== 64)
  throw new Error(`RogueTourn source closure drift: ${payload.closure.rogue_tourn_tables}`);
if (payload.closure.turnbasedgamedata_additions !== 29)
  throw new Error(
    `turnbasedgamedata addition closure drift: ${payload.closure.turnbasedgamedata_additions}`,
  );
if (payload.closure.direct_ability_and_layout_files !== 6)
  throw new Error(
    `Divergent ability source closure drift: ${payload.closure.direct_ability_and_layout_files}`,
  );
if (payload.closure.occurrence_graph_files !== 478
  || payload.closure.npc_graph_files !== 159)
  throw new Error("Divergent occurrence/NPC graph closure drift");
if (payload.closure.service_graph_files !== 3
  || payload.closure.adventure_graph_files !== 13
  || payload.closure.maze_graph_files !== 9)
  throw new Error("Divergent focused service/maze graph closure drift");
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Divergent Universe source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Divergent Universe source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.rogue_tourn_tables} RogueTourn; ` +
  `${payload.closure.direct_ability_and_layout_files} direct ability/layout; ` +
  `${payload.closure.occurrence_graph_files} occurrence graphs).`,
);
