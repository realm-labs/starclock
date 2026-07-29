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
  "currency-wars-v1",
  "source-inventory.json",
);
const standardInventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "standard-universe-v1",
  "source-inventory.json",
), "utf8"));
const sourceCorrection = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "currency-wars-v1",
  "source-correction.json",
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
const personaPresentationTables = new Set([
  "RoguePersonaConstClient.json",
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
const buildTables =
  /^RogueUpgradeAvatar(?:Const|Equipment|SubRelic|SubType|SubValue)?\.json$/u;
const directS3Ability =
  /^Config\/ConfigAbility\/Level\/Level_RogueBuff_Ability_(?:Ability|HEX|Miracle|Recipe)_S3(?:\.layout)?\.json$/u;
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
  return execFileSync("git", ["-C", source.root, ...gitArgs], {
    encoding,
    maxBuffer: 128 * 1024 * 1024,
  });
}
function additionalModeEntry(relativePath) {
  return (
    /^ExcelOutput\/GuideRogue(?:Data|Tab)\.json$/u.test(relativePath) ||
    /GridFight/iu.test(relativePath) ||
    relativePath === "ExcelOutput/StageConfig.json" ||
    /^TextMap\/TextMap(?:EN|CHS)\.json$/u.test(relativePath) ||
    /^Config\/ConfigAdventureModifier\/AdventureModifier_Rogue_(?:S3|Tourn1)\.json$/u
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
  if (/^ExcelOutput\/GuideRogue(?:Data|Tab)\.json$/u.test(relativePath)) {
    return {
      family: "currency_wars_identity_selector",
      selected_by:
        "released GuideType GridFight and Currency Wars guide-entry selector",
    };
  }
  if (/^ExcelOutput\/GridFight.*\.json$/u.test(relativePath)) {
    return {
      family: "currency_wars_gridfight_table",
      selected_by:
        "direct GridFight table under the released Currency Wars GuideType selector",
    };
  }
  if (relativePath.startsWith("Config/") && /GridFight/iu.test(relativePath)) {
    return {
      family: "currency_wars_gridfight_config",
      selected_by:
        "direct GridFight configuration under the released Currency Wars GuideType selector",
    };
  }
  if (relativePath === "ExcelOutput/StageConfig.json") {
    return {
      family: "encounter_stage_evidence",
      selected_by: "complete pinned StageConfig for encounter and wave closure",
    };
  }
  if (relativePath
    === "Config/ConfigAdventureModifier/AdventureModifier_Rogue_S3.json") {
    return {
      family: "divergent_universe_mechanic_boundary_evidence",
      selected_by:
        "superseded Tourn S3 Adventure modifier retained as Divergent Universe boundary evidence",
    };
  }
  if (relativePath
    === "Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json") {
    return {
      family: "divergent_universe_modifier_boundary_evidence",
      selected_by: "Tourn1 Adventure modifier retained for explicit Tourn3 exclusion review",
    };
  }
  if (/^Config\/Level\/GroupTemplateGraph\/03_Rogue\/RogueTourn230\//u
    .test(relativePath)) {
    return {
      family: "divergent_universe_service_boundary_evidence",
      selected_by:
        "Tourn service graph retained only for Divergent Universe boundary reconciliation",
    };
  }
  if (/^Config\/Level\/Maze\/MazeRogue\/RogueTourn\//u.test(relativePath)) {
    if (relativePath.endsWith("/RogueTournS3_Group_Base.json")) {
      return {
        family: "divergent_universe_mechanic_boundary_evidence",
        selected_by:
          "superseded Tourn S3 graph retained as Divergent Universe boundary evidence",
      };
    }
    if (relativePath.endsWith("/RogueTournS2_Group_BaseElite.json")) {
      return {
        family: "divergent_universe_maze_boundary_evidence",
        selected_by: "S2 Tourn graph retained for explicit Tourn3 exclusion review",
      };
    }
    if (relativePath.includes("_Adv")) {
      return {
        family: "divergent_universe_adventure_boundary_evidence",
        selected_by:
          "Tourn Adventure graph retained only for Divergent Universe boundary reconciliation",
      };
    }
    return {
      family: "divergent_universe_maze_boundary_evidence",
      selected_by:
        "Tourn maze graph retained only for Divergent Universe boundary reconciliation",
    };
  }
  if (directS3Ability.test(relativePath)) {
    return {
      family: "divergent_universe_mechanic_boundary_evidence",
      selected_by:
        "superseded Tourn S3 ability retained as Divergent Universe boundary evidence",
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
    if (/Tourn1|HEX_S1/u.test(relativePath)) {
      return {
        family: "divergent_universe_mechanic_boundary_evidence",
        selected_by: "Tourn1/S1 ability retained for explicit Tourn3 exclusion review",
      };
    }
    return {
      family: "shared_mechanic_evidence_candidate",
      selected_by: "shared Rogue ability requiring row-level reachability review",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueDialogue\/RogueEventTourn/u
    .test(relativePath)) {
    return {
      family: "divergent_universe_occurrence_boundary_evidence",
      selected_by:
        "Tourn occurrence graph retained only for Divergent Universe boundary reconciliation",
    };
  }
  if (/^Config\/Level\/Rogue\/RogueNPC\//u.test(relativePath)) {
    return {
      family: "shared_npc_graph_candidate",
      selected_by: "shared Rogue NPC graph requiring Tourn3 room/service reachability proof",
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
  if (/^RoguePersona.*\.json$/u.test(name)) {
    return personaPresentationTables.has(name)
      ? {
        family: "divergent_universe_presentation_boundary_evidence",
        selected_by:
          "Persona client table retained only for Divergent Universe boundary reconciliation",
      }
      : {
        family: "divergent_universe_structured_boundary_evidence",
        selected_by:
          "Persona table retained only for Divergent Universe boundary reconciliation",
      };
  }
  if (/^RogueTourn.*\.json$/u.test(name)) {
    if (tournTestTables.has(name)) {
      return {
        family: "tourn_test_exclusion_evidence",
        selected_by: "Tourn test table retained to prove fail-closed exclusion",
      };
    }
    return tournPresentationTables.has(name)
      ? {
        family: "divergent_universe_presentation_boundary_evidence",
        selected_by:
          "Tourn presentation/account table retained only for Divergent Universe boundary reconciliation",
      }
      : {
        family: "divergent_universe_structured_boundary_evidence",
        selected_by:
          "Tourn table retained only for Divergent Universe boundary reconciliation",
      };
  }
  if (buildTables.test(name)) {
    return {
      family: "shared_build_mapping_candidate",
      selected_by: "shared build table requiring explicit Currency Wars avatar mapping proof",
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
  if (/^(ActivityRogue|RogueEndless)/u.test(name)) {
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
  if (/^(Avatar|Equipment|Relic|Monster|Stage|Rogue)/u.test(name)) {
    return {
      family: "shared_structured_candidate",
      selected_by:
        "shared build, Rogue or enemy table requiring GridFight-originating reachability proof",
    };
  }
  return {
    family: "inherited_reference_evidence",
    selected_by: "inherited Goal 03 source retained for stable-ID and evidence closure",
  };
}
function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function containsValue(value, expected) {
  if (value === expected) return true;
  if (Array.isArray(value))
    return value.some((entry) => containsValue(entry, expected));
  if (value !== null && typeof value === "object")
    return Object.values(value).some((entry) => containsValue(entry, expected));
  return false;
}
function hasNamedThreeSelector(row) {
  if (row === null || typeof row !== "object" || Array.isArray(row)) return false;
  return Object.entries(row).some(([key, value]) =>
    /^(?:MainTournID|TournID|TournMode)$/u.test(key)
      && (value === 3 || value === "3" || value === "Tourn3"));
}
function tableAudit(source, relativePath) {
  const bytes = git(source, ["cat-file", "blob", `HEAD:${relativePath}`], null);
  const parsed = JSON.parse(bytes.toString("utf8"));
  const rows = Array.isArray(parsed) ? parsed : Object.values(parsed);
  const indexes = [];
  for (const [index, row] of rows.entries())
    if (containsValue(row, "Tourn3")
      || containsValue(row, 6002201)
      || hasNamedThreeSelector(row))
      indexes.push(index);
  return {
    path: relativePath,
    rows: rows.length,
    direct_tourn3_row_indexes: indexes,
    direct_tourn3_rows: indexes.length,
  };
}

if (standardInventory.records.length !== 2646)
  throw new Error("Goal 03 source inventory denominator drift");

const records = [];
const audits = [];
const gridFightAudits = [];
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
    if (source.id === "turnbasedgamedata"
      && /^ExcelOutput\/Rogue(?:Persona|Tourn)[^/]*\.json$/u.test(relativePath))
      audits.push(tableAudit(source, relativePath));
    if (source.id === "turnbasedgamedata"
      && /^ExcelOutput\/GridFight.*\.json$/u.test(relativePath)) {
      const audit = tableAudit(source, relativePath);
      gridFightAudits.push({
        path: audit.path,
        rows: audit.rows,
      });
    }
  }
}
records.sort((left, right) =>
  compareText(`${left.repository}/${left.path}`, `${right.repository}/${right.path}`));
audits.sort((left, right) => compareText(left.path, right.path));
gridFightAudits.sort((left, right) => compareText(left.path, right.path));

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
  schema_revision: "starclock.currency-wars-source-inventory.v1",
  goal_id: "currency-wars-reference-v1",
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
      "all 2,646 Goal 03 source files remain available for shared-ID, build, enemy, ability and reachability closure",
    structured:
      "all 153 GridFight tables are direct Currency Wars sources; all 11 RoguePersona and 64 RogueTourn tables remain only for superseded-selector and other-mode reconciliation",
    text:
      "complete pinned English and Simplified Chinese TextMaps plus bilingual StarRailRes simulated-universe indexes",
    mechanics:
      "all 984 GridFight config paths are direct Currency Wars sources; inherited Rogue/Tourn programs remain only as shared or other-mode evidence",
    encounters:
      "complete StageConfig and inherited monster/enemy definitions retained for later row-level wave closure",
    row_audit:
      "all 153 GridFight tables record top-level row counts; the 75 Persona/Tourn tables retain their historical selector audit only for exclusion reconciliation",
    denominator_rule:
      "file closure and selector audit only; no content-row denominator, ownership or reachability is implied before corrective G12-P0-B5",
  },
  classification_policy: {
    currency_wars_identity_selector:
      "released GuideType GridFight and Currency Wars guide-entry selector",
    currency_wars_gridfight_table:
      "direct GridFight structured table selected by released Currency Wars identity",
    currency_wars_gridfight_config:
      "direct GridFight config selected by released Currency Wars identity",
    divergent_universe_structured_boundary_evidence:
      "Persona/Tourn structured table retained only for Divergent Universe boundary reconciliation",
    divergent_universe_presentation_boundary_evidence:
      "Persona/Tourn presentation source retained only for Divergent Universe boundary reconciliation",
    divergent_universe_service_boundary_evidence:
      "Tourn service graph retained only for Divergent Universe boundary reconciliation",
    divergent_universe_adventure_boundary_evidence:
      "Tourn Adventure graph retained only for Divergent Universe boundary reconciliation",
    divergent_universe_occurrence_boundary_evidence:
      "Tourn occurrence graph retained only for Divergent Universe boundary reconciliation",
    tourn_test_exclusion_evidence:
      "Tourn test table retained to prove fail-closed exclusion",
    shared_build_mapping_candidate:
      "shared build table requiring explicit Currency Wars avatar mapping proof",
    shared_structured_candidate:
      "shared build, Rogue or enemy table requiring GridFight-originating reachability proof",
    shared_mechanic_evidence_candidate:
      "shared Rogue ability requiring row-level reachability proof",
    shared_occurrence_graph_candidate:
      "shared Rogue occurrence graph requiring row-level reachability proof",
    shared_npc_graph_candidate:
      "shared Rogue NPC graph requiring room/service reachability proof",
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
    divergent_universe_mechanic_boundary_evidence:
      "Tourn1/S1 ability retained for explicit Tourn3 exclusion review",
    divergent_universe_modifier_boundary_evidence:
      "Tourn1 Adventure modifier retained for explicit Tourn3 exclusion review",
    divergent_universe_maze_boundary_evidence:
      "S2 Tourn maze graph retained for explicit Tourn3 exclusion review",
    other_mode_exclusion_evidence:
      "explicit non-Currency-Wars source retained to prove ownership exclusion",
    presentation_account_exclusion_evidence:
      "shared presentation/account source retained only to prove exclusion",
    inherited_reference_evidence:
      "inherited Goal 03 source retained for stable-ID and evidence closure",
  },
  closure: {
    inherited_goal03_files: inheritedPaths.size,
    turnbasedgamedata_additions: additions.length,
    persona_tables: audits.filter(({ path: sourcePath }) =>
      sourcePath.includes("/RoguePersona")).length,
    tourn_tables: audits.filter(({ path: sourcePath }) =>
      sourcePath.includes("/RogueTourn")).length,
    audited_structured_rows: audits.reduce((sum, { rows }) => sum + rows, 0),
    direct_tourn3_rows: audits.reduce((sum, row) =>
      sum + row.direct_tourn3_rows, 0),
    gridfight_files: records.filter(({ path: sourcePath }) =>
      /GridFight/iu.test(sourcePath)).length,
    gridfight_tables: count("currency_wars_gridfight_table"),
    gridfight_config_files: count("currency_wars_gridfight_config"),
    gridfight_structured_rows:
      gridFightAudits.reduce((sum, { rows }) => sum + rows, 0),
    tourn_maze_graph_files: records.filter(({ path: sourcePath }) =>
      sourcePath.startsWith("Config/Level/Maze/MazeRogue/RogueTourn/")).length,
    tourn_service_graph_files:
      count("divergent_universe_service_boundary_evidence"),
    shared_build_tables: count("shared_build_mapping_candidate"),
    named_other_mode_exclusion_files: records.filter(({ family }) =>
      family.includes("_exclusion_evidence")
      && family !== "presentation_account_exclusion_evidence"
      && family !== "tourn_test_exclusion_evidence").length,
    text_and_public_index_files:
      count("localized_text_evidence") + count("public_index_cross_check"),
    unclassified_selected_files: 0,
  },
  structured_table_audit: audits,
  gridfight_table_audit: gridFightAudits,
  source_correction: {
    path: "content-manifests/currency-wars-v1/source-correction.json",
    sha256: createHash("sha256")
      .update(await readFile(path.join(
        root,
        "content-manifests",
        "currency-wars-v1",
        "source-correction.json",
      )))
      .digest("hex"),
    guide_type: sourceCorrection.authoritative_selector.guide_type,
  },
  counts,
  records,
};

if (payload.closure.persona_tables !== 11 || payload.closure.tourn_tables !== 64)
  throw new Error("Persona/Tourn source table closure drift");
if (payload.closure.turnbasedgamedata_additions !== 1167)
  throw new Error(
    `turnbasedgamedata addition closure drift: ${payload.closure.turnbasedgamedata_additions}`,
  );
if (payload.closure.gridfight_files !== 1137
  || payload.closure.gridfight_tables !== 153
  || payload.closure.gridfight_config_files !== 984)
  throw new Error("Currency Wars GridFight source closure drift");
if (payload.closure.tourn_maze_graph_files !== 22
  || payload.closure.tourn_service_graph_files !== 3)
  throw new Error("superseded Tourn graph reconciliation closure drift");
if (payload.closure.shared_build_tables !== 6)
  throw new Error("Currency Wars shared build-table closure drift");
if (counts.by_repository.starrailres !== starRailResPaths.size)
  throw new Error("StarRailRes focused index closure drift");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded)
    throw new Error("Currency Wars source inventory has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Currency Wars source inventory ${check ? "verified" : "generated"}: ` +
  `${records.length} files (${payload.closure.gridfight_tables} GridFight ` +
  `tables, ${payload.closure.gridfight_config_files} GridFight configs and ` +
  `${payload.closure.gridfight_structured_rows} GridFight rows).`,
);
