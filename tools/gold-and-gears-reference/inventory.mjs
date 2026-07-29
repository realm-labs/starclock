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
  "gold-and-gears-v1",
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

const excludedNousTables = new Set([
  "RogueNousEndGameReward.json",
  "RogueNousMainStory.json",
  "RogueNousMissionReward.json",
  "RogueNousStoryDisplay.json",
  "RogueNousStoryReward.json",
  "RogueNousSubStory.json",
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
  /^(ActivityRogue|RogueDLC|RogueEndless|RogueMagic|RoguePersona|RogueTourn)/u;
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
  if (relativePath.startsWith("Config/ConfigAbility/")) {
    if (/Nous/u.test(relativePath)) {
      return {
        family: "gold_and_gears_mechanic_evidence",
        selected_by: "Gold and Gears-named released ability program",
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
  if (/^RogueNous.*\.json$/u.test(name)) {
    return excludedNousTables.has(name)
      ? {
        family: "gold_and_gears_exclusion_evidence",
        selected_by: "Gold and Gears story/account table retained only to prove exclusion",
      }
      : {
        family: "gold_and_gears_structured",
        selected_by: "Gold and Gears-owned RogueNous table",
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
      selected_by: "explicit non-Gold universe family retained to prove ownership exclusion",
    };
  }
  return {
    family: "shared_structured_candidate",
    selected_by: "generic Rogue table requiring Gold and Gears reachability review",
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
  schema_revision: "starclock.gold-and-gears-source-inventory.v1",
  goal_id: "gold-and-gears-reference-v1",
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
      "all Rogue/ActivityRogue tables plus StageConfig; explicit other-mode and presentation rows remain exclusion evidence",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual StarRailRes simulated-universe indexes",
    mechanics:
      "all sparse-cached Rogue BattleEvent/Level abilities and Rogue level/dialogue graphs; B3 must close row-level reachability",
    denominator_rule:
      "file closure only; no content-row denominator or ownership is implied before G08-P0-B3",
  },
  classification_policy: {
    gold_and_gears_structured: "mechanically relevant RogueNous source table",
    gold_and_gears_exclusion_evidence:
      "RogueNous story/account source retained only to prove exclusion",
    gold_and_gears_mechanic_evidence:
      "Gold and Gears-named ability program",
    shared_structured_candidate:
      "generic Rogue table requiring Gold and Gears row-level reachability proof",
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
      "explicit non-Gold universe table retained to prove ownership exclusion",
    presentation_account_exclusion_evidence:
      "shared presentation/account table retained only to prove exclusion",
  },
  closure: {
    rogue_nous_tables: count("gold_and_gears_structured")
      + count("gold_and_gears_exclusion_evidence"),
    direct_gold_mechanic_files: count("gold_and_gears_mechanic_evidence"),
    structured_and_exclusion_files: records.filter(({ path: relativePath }) =>
      relativePath.startsWith("ExcelOutput/")).length,
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    mechanic_and_level_candidates:
      count("shared_mechanic_evidence_candidate")
      + count("shared_occurrence_graph_candidate")
      + count("shared_level_graph_candidate"),
    unclassified_selected_files: 0,
  },
  counts,
  records,
};
if (payload.closure.rogue_nous_tables !== 21)
  throw new Error(`RogueNous source closure drift: ${payload.closure.rogue_nous_tables}`);
if (payload.closure.direct_gold_mechanic_files !== 2)
  throw new Error(
    `Gold mechanic source closure drift: ${payload.closure.direct_gold_mechanic_files}`,
  );
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Gold and Gears source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Gold and Gears source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.rogue_nous_tables} RogueNous; ` +
  `${payload.closure.direct_gold_mechanic_files} direct mechanic).`,
);
