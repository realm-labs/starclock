#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const output = path.join(
  root,
  "content-manifests",
  "swarm-disaster-v1",
  "source-inventory.json",
);
const sources = [
  {
    id: "turnbasedgamedata",
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    root: path.join(root, ".cache/content-reference/turnbasedgamedata"),
  },
  {
    id: "starrailres",
    repository: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    root: path.join(root, ".cache/content-reference/StarRailRes"),
  },
];

const excludedDlcTables = new Set([
  "RogueDLCEndGameReward.json",
  "RogueDLCMainStory.json",
  "RogueDLCMainStoryBranch.json",
  "RogueDLCMainStoryReward.json",
  "RogueDLCSubStory.json",
  "RogueDLCSubStoryGroup.json",
]);
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
const otherModePrefix =
  /^(ActivityRogue|RogueEndless|RogueMagic|RoguePersona|RogueTourn)/u;
const directSwarmAbility =
  /^Config\/ConfigAbility\/Level\/Level_(?:RogueBuff_Ability_DLC1(?:_Other)?|RogueDLC_Ability)(?:\.layout)?\.json$/u;
const starRailResPaths = new Set([
  "info.json",
  ...["cn", "en"].flatMap((locale) =>
    ["blessings", "blocks", "curios", "events"].map(
      (family) => `index_new/${locale}/simulated_${family}.json`,
    )),
]);

function git(source, gitArgs, encoding = "utf8") {
  return execFileSync("git", ["-C", source.root, ...gitArgs], {
    encoding,
    maxBuffer: 128 * 1024 * 1024,
  });
}

function selected(sourceId, relativePath) {
  if (sourceId === "starrailres") return starRailResPaths.has(relativePath);
  if (relativePath === "ExcelOutput/StageConfig.json") return true;
  if (/^TextMap\/TextMap(?:EN|CHS)\.json$/u.test(relativePath)) return true;
  if (/^Config\/Gameplays\/RogueDLC\/.*\.json$/u.test(relativePath)) return true;
  if (relativePath.startsWith("ExcelOutput/")) {
    const name = path.posix.basename(relativePath);
    return (
      /^Rogue.*\.json$/u.test(name) ||
      /^(ActivityRogue.*|ConstValueRogue|FinishWayRogue|GuideRogue.*|ScheduleDataRogue)\.json$/u
        .test(name)
    );
  }
  return (
    /^Config\/ConfigAbility\/BattleEvent\/.*Rogue.*\.json$/u.test(relativePath) ||
    /^Config\/ConfigAbility\/Level\/Level_.*Rogue.*\.json$/u.test(relativePath) ||
    /^Config\/Level\/Rogue(?:\/|Dialogue\/).*\.json$/u.test(relativePath)
  );
}

function classify(sourceId, relativePath) {
  if (sourceId === "starrailres") {
    return {
      family: "public_index_cross_check",
      selected_by: "bilingual released-resource index for shared pool identity review",
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
      selected_by: "complete pinned StageConfig for encounter-wave closure",
    };
  }
  if (relativePath.startsWith("Config/Gameplays/RogueDLC/")) {
    if (relativePath.includes("/MapRepo160/")) {
      return {
        family: "gold_and_gears_topology_exclusion_evidence",
        selected_by: "MapRepo160 Gold topology retained to prove mode exclusion",
      };
    }
    return {
      family: "swarm_disaster_topology_candidate",
      selected_by:
        "non-MapRepo160 DLC chessboard configuration requiring ChessRogue reachability review",
    };
  }
  if (relativePath.startsWith("Config/ConfigAbility/")) {
    if (directSwarmAbility.test(relativePath)) {
      return {
        family: "swarm_disaster_mechanic_evidence",
        selected_by: "Swarm Disaster-named released ability program",
      };
    }
    if (/Nous/u.test(relativePath)) {
      return {
        family: "gold_and_gears_mechanic_exclusion_evidence",
        selected_by: "Gold and Gears-named ability retained to prove mode exclusion",
      };
    }
    return {
      family: "shared_mechanic_evidence_candidate",
      selected_by: "shared Rogue ability program requiring row-level reachability review",
    };
  }
  if (relativePath.startsWith("Config/Level/Rogue")) {
    return {
      family: relativePath.includes("/RogueDialogue/")
        ? "shared_occurrence_graph_candidate"
        : "shared_level_graph_candidate",
      selected_by: "shared Rogue level graph requiring NPC/room/occurrence reachability review",
    };
  }

  const name = path.posix.basename(relativePath);
  if (/^RogueDLC.*\.json$/u.test(name)) {
    return excludedDlcTables.has(name)
      ? {
        family: "swarm_story_account_exclusion_evidence",
        selected_by: "DLC story/account table retained only for mechanical unlock locators",
      }
      : {
        family: "swarm_disaster_structured_candidate",
        selected_by: "DLC framework table requiring ChessRogue row-level reachability review",
      };
  }
  if (/^RogueNous.*\.json$/u.test(name)) {
    return {
      family: "gold_and_gears_structured_exclusion_evidence",
      selected_by: "RogueNous table retained to prove Gold and Gears ownership exclusion",
    };
  }
  if (presentationTables.has(name)) {
    return {
      family: "presentation_account_exclusion_evidence",
      selected_by: "shared presentation/account table retained only to prove exclusion",
    };
  }
  if (otherModePrefix.test(name)) {
    return {
      family: "other_mode_exclusion_evidence",
      selected_by: "explicit non-Swarm universe family retained to prove ownership exclusion",
    };
  }
  return {
    family: "shared_structured_candidate",
    selected_by: "generic Rogue table requiring Swarm Disaster reachability review",
  };
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

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
const payload = {
  schema_revision: "starclock.swarm-disaster-source-inventory.v1",
  goal_id: "swarm-disaster-reference-v1",
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
    structured:
      "all Rogue/ActivityRogue tables plus StageConfig; explicit Gold, other-mode and presentation rows remain exclusion evidence",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual StarRailRes simulated-universe indexes",
    mechanics:
      "all sparse-cached Rogue BattleEvent/Level abilities and Rogue level/dialogue graphs; B3 must close row-level reachability",
    topology:
      "all DLC configs retained; non-MapRepo160 rows are Swarm candidates and MapRepo160 rows are Gold exclusion evidence",
    denominator_rule:
      "file closure only; no content-row denominator or ownership is implied before G09-P0-B3",
  },
  classification_policy: {
    swarm_disaster_structured_candidate:
      "DLC framework table requiring ChessRogue row-level reachability proof",
    swarm_story_account_exclusion_evidence:
      "DLC story/account source retained only for mechanical unlock locators",
    swarm_disaster_mechanic_evidence:
      "Swarm Disaster-named released ability program",
    swarm_disaster_topology_candidate:
      "non-MapRepo160 topology requiring ChessRogue reachability proof",
    gold_and_gears_structured_exclusion_evidence:
      "RogueNous source retained to prove Gold and Gears exclusion",
    gold_and_gears_mechanic_exclusion_evidence:
      "Gold and Gears-named ability retained to prove mode exclusion",
    gold_and_gears_topology_exclusion_evidence:
      "MapRepo160 topology retained to prove Gold and Gears exclusion",
    shared_structured_candidate:
      "generic Rogue table requiring Swarm Disaster row-level reachability proof",
    shared_mechanic_evidence_candidate:
      "shared Rogue ability program requiring row-level reachability proof",
    shared_occurrence_graph_candidate:
      "shared Rogue dialogue graph requiring occurrence reachability proof",
    shared_level_graph_candidate:
      "shared Rogue level graph requiring room/NPC reachability proof",
    encounter_stage_evidence: "complete StageConfig retained for exact wave closure",
    localized_text_evidence: "complete bilingual TextMap retained for hash resolution",
    public_index_cross_check: "released-resource bilingual index used for identity review",
    other_mode_exclusion_evidence:
      "explicit non-Swarm universe table retained to prove ownership exclusion",
    presentation_account_exclusion_evidence:
      "shared presentation/account table retained only to prove exclusion",
  },
  closure: {
    rogue_dlc_tables: records.filter(({ path: relativePath }) =>
      /^ExcelOutput\/RogueDLC[^/]*\.json$/u.test(relativePath)).length,
    rogue_nous_exclusion_tables: count("gold_and_gears_structured_exclusion_evidence"),
    direct_swarm_mechanic_files: count("swarm_disaster_mechanic_evidence"),
    direct_gold_mechanic_exclusion_files:
      count("gold_and_gears_mechanic_exclusion_evidence"),
    structured_and_exclusion_files: records.filter(({ path: relativePath }) =>
      relativePath.startsWith("ExcelOutput/")).length,
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    mechanic_and_level_candidates:
      count("shared_mechanic_evidence_candidate")
      + count("shared_occurrence_graph_candidate")
      + count("shared_level_graph_candidate"),
    swarm_topology_config_candidates: count("swarm_disaster_topology_candidate"),
    gold_topology_exclusion_files: count("gold_and_gears_topology_exclusion_evidence"),
    unclassified_selected_files: 0,
  },
  counts,
  records,
};
if (payload.closure.rogue_dlc_tables !== 32)
  throw new Error(`RogueDLC source closure drift: ${payload.closure.rogue_dlc_tables}`);
if (payload.closure.rogue_nous_exclusion_tables !== 21)
  throw new Error(
    `RogueNous exclusion closure drift: ${payload.closure.rogue_nous_exclusion_tables}`,
  );
if (payload.closure.direct_swarm_mechanic_files !== 6)
  throw new Error(
    `Swarm mechanic source closure drift: ${payload.closure.direct_swarm_mechanic_files}`,
  );
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Swarm Disaster source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Swarm Disaster source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.rogue_dlc_tables} RogueDLC; ` +
  `${payload.closure.direct_swarm_mechanic_files} direct mechanic; ` +
  `${payload.closure.swarm_topology_config_candidates} Swarm topology).`,
);
