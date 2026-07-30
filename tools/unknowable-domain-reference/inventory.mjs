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
  "unknowable-domain-v1",
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
    root: path.join(root, ".cache/content-reference/turnbasedgamedata"),
  },
  {
    id: "starrailres",
    repository: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    root: path.join(root, ".cache/content-reference/StarRailRes"),
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
const magicPresentationTables = new Set([
  "RogueMagicConstClient.json",
  "RogueMagicContentDisplay.json",
  "RogueMagicMiracleDisplay.json",
  "RogueMagicMiscDisplay.json",
  "RogueMagicRoomMark.json",
  "RogueMagicScepterDisplay.json",
  "RogueMagicStory.json",
  "RogueMagicUnitDisplay.json",
]);
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

function additionalModeEntry(relativePath) {
  return (
    relativePath === "ExcelOutput/StageConfig.json" ||
    /^TextMap\/TextMap(?:EN|CHS)\.json$/u.test(relativePath) ||
    /^Config\/ConfigAdventureModifier\/AdventureModifier_Rogue_RogueMagic(?:\.layout)?\.json$/u
      .test(relativePath) ||
    /^Config\/ConfigCharacter\/BattleEvent\/Avatar_RogueMagic_.*\.json$/u
      .test(relativePath) ||
    /^Config\/Level\/GroupTemplateGraph\/03_Rogue\/RogueMagic260\/.*\.json$/u
      .test(relativePath) ||
    /^Config\/Level\/Maze\/MazeRogue\/Rogue260\/.*\.json$/u.test(relativePath)
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
  if (/^Config\/ConfigAdventureModifier\/AdventureModifier_Rogue_RogueMagic/u
    .test(relativePath)) {
    return {
      family: "unknowable_adventure_modifier_evidence",
      selected_by: "Unknowable Domain-named Adventure outcome modifier",
    };
  }
  if (/^Config\/ConfigCharacter\/BattleEvent\/Avatar_RogueMagic_/u
    .test(relativePath)) {
    return {
      family: "unknowable_battle_event_candidate",
      selected_by: "Unknowable Domain-named Scepter battle-event configuration",
    };
  }
  if (/^Config\/Level\/GroupTemplateGraph\/03_Rogue\/RogueMagic260\//u
    .test(relativePath)) {
    return {
      family: "unknowable_service_graph_candidate",
      selected_by: "Unknowable Domain workbench, reforge or shop group graph",
    };
  }
  if (/^Config\/Level\/Maze\/MazeRogue\/Rogue260\//u.test(relativePath)) {
    return {
      family: relativePath.includes("MissionTalk")
        ? "unknowable_presentation_locator"
        : "unknowable_maze_graph_candidate",
      selected_by: relativePath.includes("MissionTalk")
        ? "mode final-round talk event retained only as a mechanical stage locator"
        : "Unknowable Domain maze door or group graph",
    };
  }
  if (/^Config\/ConfigAbility\/Level\/Level_RogueMagic_/u.test(relativePath)) {
    return {
      family: "unknowable_mechanic_evidence",
      selected_by: "Unknowable Domain-named released ability program",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueMagicPower\//u.test(relativePath)) {
    return {
      family: "unknowable_progression_graph_candidate",
      selected_by: "Unknowable Domain power and progression graph",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueNPC\/RogueNPC_260\//u.test(relativePath)) {
    return {
      family: "unknowable_npc_graph_candidate",
      selected_by: "Rogue260 NPC graph requiring room and service reachability review",
    };
  }
  if (relativePath.startsWith("Config/ConfigAbility/")) {
    if (/Nous/u.test(relativePath)) {
      return {
        family: "gold_and_gears_mechanic_exclusion_evidence",
        selected_by: "Gold and Gears-named ability retained to prove exclusion",
      };
    }
    if (/DLC1|RogueDLC/u.test(relativePath)) {
      return {
        family: "swarm_disaster_mechanic_exclusion_evidence",
        selected_by: "Swarm Disaster-named ability retained to prove exclusion",
      };
    }
    if (/RogueTourn/u.test(relativePath)) {
      return {
        family: "divergent_universe_mechanic_exclusion_evidence",
        selected_by: "Divergent Universe-named ability retained to prove exclusion",
      };
    }
    return {
      family: "shared_mechanic_evidence_candidate",
      selected_by: "shared Rogue ability requiring row-level reachability review",
    };
  }
  if (relativePath.startsWith("Config/Level/Rogue")) {
    return {
      family: relativePath.includes("/RogueDialogue/")
        ? "shared_occurrence_graph_candidate"
        : "shared_level_graph_candidate",
      selected_by: "shared Rogue graph requiring explicit transitive reachability review",
    };
  }

  const name = path.posix.basename(relativePath);
  if (/^RogueMagic.*\.json$/u.test(name)) {
    return magicPresentationTables.has(name)
      ? {
        family: "unknowable_presentation_locator",
        selected_by: "mode display/story table retained only for names and mechanical locators",
      }
      : {
        family: "unknowable_structured_candidate",
        selected_by: "RogueMagic table requiring row-level ownership and reachability review",
      };
  }
  if (/^RogueNous.*\.json$/u.test(name)) {
    return {
      family: "gold_and_gears_structured_exclusion_evidence",
      selected_by: "RogueNous table retained to prove Gold and Gears exclusion",
    };
  }
  if (/^RogueDLC.*\.json$/u.test(name)) {
    return {
      family: "swarm_disaster_structured_exclusion_evidence",
      selected_by: "RogueDLC table retained to prove Swarm Disaster/shared-framework boundary",
    };
  }
  if (/^RogueTourn.*\.json$/u.test(name)) {
    return {
      family: "divergent_universe_structured_exclusion_evidence",
      selected_by: "RogueTourn table retained to prove Divergent Universe exclusion",
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
    selected_by: "generic Rogue table requiring Unknowable Domain reachability review",
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
  schema_revision: "starclock.unknowable-domain-source-inventory.v1",
  goal_id: "unknowable-domain-reference-v1",
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
      "all 2,646 Goal 03 source files remain available for stable shared-ID and reachability closure",
    structured:
      "all RogueMagic tables plus inherited shared Rogue tables; named other modes remain exclusion evidence",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual StarRailRes simulated-universe indexes",
    mechanics:
      "all inherited Rogue ability/level/dialogue files plus direct RogueMagic battle-event, service and maze configuration entries",
    encounters:
      "complete StageConfig and inherited enemy/monster definitions retained for later row-level wave closure",
    denominator_rule:
      "file closure only; no content-row denominator, ownership or reachability is implied before G10-P0-B3",
  },
  classification_policy: {
    unknowable_structured_candidate:
      "RogueMagic structured table requiring row-level ownership and reachability proof",
    unknowable_mechanic_evidence:
      "Unknowable Domain-named released ability program",
    unknowable_battle_event_candidate:
      "Scepter battle-event configuration requiring identity and lifecycle closure",
    unknowable_adventure_modifier_evidence:
      "mode-named Adventure modifier retained as abstract outcome evidence",
    unknowable_service_graph_candidate:
      "mode workbench, reforge or shop group graph",
    unknowable_maze_graph_candidate: "mode maze door or group graph",
    unknowable_npc_graph_candidate:
      "Rogue260 NPC graph requiring room and service reachability review",
    unknowable_progression_graph_candidate: "mode power and progression graph",
    unknowable_presentation_locator:
      "mode display/story/talk source retained only for bilingual names or mechanical locators",
    shared_structured_candidate:
      "generic Rogue table requiring Unknowable Domain row-level reachability proof",
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
      "RogueDLC source retained to prove Swarm Disaster/shared-framework exclusion",
    swarm_disaster_mechanic_exclusion_evidence:
      "Swarm Disaster-named ability retained to prove exclusion",
    divergent_universe_structured_exclusion_evidence:
      "RogueTourn source retained to prove Divergent Universe exclusion",
    divergent_universe_mechanic_exclusion_evidence:
      "Divergent Universe-named ability retained to prove exclusion",
    other_mode_exclusion_evidence:
      "explicit other-mode table retained to prove ownership exclusion",
    presentation_account_exclusion_evidence:
      "shared presentation/account table retained only to prove exclusion",
  },
  closure: {
    inherited_goal03_files: inheritedPaths.size,
    turnbasedgamedata_additions: additions.length,
    rogue_magic_tables: records.filter(({ path: sourcePath }) =>
      /^ExcelOutput\/RogueMagic[^/]*\.json$/u.test(sourcePath)).length,
    direct_ability_files: count("unknowable_mechanic_evidence"),
    battle_event_files: count("unknowable_battle_event_candidate"),
    service_graph_files: count("unknowable_service_graph_candidate"),
    maze_graph_files: count("unknowable_maze_graph_candidate"),
    npc_graph_files: count("unknowable_npc_graph_candidate"),
    named_other_mode_exclusion_files: records.filter(({ family }) =>
      family.includes("_exclusion_evidence")
      && family !== "presentation_account_exclusion_evidence").length,
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    unclassified_selected_files: 0,
  },
  counts,
  records,
};
if (payload.closure.rogue_magic_tables !== 32)
  throw new Error(`RogueMagic source closure drift: ${payload.closure.rogue_magic_tables}`);
if (payload.closure.turnbasedgamedata_additions !== 29)
  throw new Error(
    `turnbasedgamedata addition closure drift: ${payload.closure.turnbasedgamedata_additions}`,
  );
if (payload.closure.direct_ability_files !== 16)
  throw new Error(
    `Unknowable ability source closure drift: ${payload.closure.direct_ability_files}`,
  );
if (payload.closure.battle_event_files !== 14)
  throw new Error(
    `Unknowable battle-event closure drift: ${payload.closure.battle_event_files}`,
  );
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Unknowable Domain source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Unknowable Domain source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.rogue_magic_tables} RogueMagic; ` +
  `${payload.closure.direct_ability_files} direct ability; ` +
  `${payload.closure.battle_event_files} battle event).`,
);
