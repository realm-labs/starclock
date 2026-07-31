#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { revision, root, sourceJson } from "./source.mjs";

const check = process.argv.includes("--check");
const output = path.join(
  root,
  "content-manifests/apocalyptic-shadow-v1/content-manifest.json",
);
const inventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/apocalyptic-shadow-v1/source-inventory.json",
)));
const fileDigest = new Map(inventory.files.map((row) =>
  [row.source_path, row.sha256]));

function rows(value) {
  return Array.isArray(value) ? value : Object.values(value);
}

function sha(value) {
  return createHash("sha256").update(value).digest("hex");
}

function record(id, sourcePath, locator, ownership = "ApocalypticShadow",
  evidenceQuality = "ExactStructured", note = "") {
  const digest = fileDigest.get(sourcePath)
    ?? sha(JSON.stringify({ sourcePath, revision }));
  return {
    id: String(id),
    ownership,
    disposition: "DataReady",
    source_path: sourcePath,
    row_locator: String(locator),
    evidence_sha256: sha(`${digest}#${locator}`),
    evidence_quality: evidenceQuality,
    note,
  };
}

const schedulePath = "ExcelOutput/ScheduleDataChallengeBoss.json";
const groupPath = "ExcelOutput/ChallengeBossGroupConfig.json";
const groupExtraPath = "ExcelOutput/ChallengeBossGroupExtra.json";
const mazePath = "ExcelOutput/ChallengeBossMazeConfig.json";
const mazeExtraPath = "ExcelOutput/ChallengeBossMazeExtra.json";
const tiercePath = "ExcelOutput/ChallengeBossMazeTierce.json";
const targetPath = "ExcelOutput/ChallengeBossTargetConfig.json";
const buffPath = "ExcelOutput/MazeBuff.json";
const monsterPath = "ExcelOutput/MonsterConfig.json";
const templatePath = "ExcelOutput/MonsterTemplateConfig.json";
const skillPath = "ExcelOutput/MonsterSkillConfig.json";

const schedule = rows(await sourceJson(schedulePath)).find((row) => row.ID === 203019);
const activeGroup = rows(await sourceJson(groupPath)).find((row) =>
  row.GroupID === 3019);
const groupExtra = rows(await sourceJson(groupExtraPath)).find((row) =>
  row.GroupID === 3019);
const stages = rows(await sourceJson(mazePath)).filter((row) =>
  row.GroupID === 3019).sort((a, b) => a.ID - b.ID);
const stageExtras = rows(await sourceJson(mazeExtraPath)).filter((row) =>
  row.ID >= 30191 && row.ID <= 30194).sort((a, b) => a.ID - b.ID);
const tierce = rows(await sourceJson(tiercePath)).find((row) =>
  Object.values(row).some((value) => value === 30195));
if (!schedule || !activeGroup || !groupExtra || stages.length !== 4
  || stageExtras.length !== 4 || !tierce) {
  throw new Error("active Version 4.4 selector drift");
}

const targetIds = new Set([3001, 3002, 3003, 5001, 5002, 5003]);
const targets = rows(await sourceJson(targetPath)).filter((row) =>
  targetIds.has(row.ID));
const selectedBuffIds = new Set([
  activeGroup.MazeBuffID,
  ...stages.map((row) => row.MazeBuffID),
  ...Object.entries(groupExtra)
    .filter(([key, value]) => /^BuffList/.test(key) && Array.isArray(value))
    .flatMap(([, value]) => value),
]);
const buffs = rows(await sourceJson(buffPath)).filter((row) =>
  selectedBuffIds.has(row.ID));

const ordinaryMonsterIds = stageExtras.flatMap((row) =>
  Object.entries(row).filter(([key, value]) =>
    /^MonsterID/.test(key) && Number.isInteger(value) && value > 0)
    .map(([, value]) => value));
const tierceMonsterId = 3003015;
const monsterIds = new Set([...ordinaryMonsterIds, tierceMonsterId]);
const monsters = rows(await sourceJson(monsterPath)).filter((row) =>
  monsterIds.has(row.MonsterID));
const templateIds = new Set(monsters.map((row) => row.MonsterTemplateID));
const templates = rows(await sourceJson(templatePath)).filter((row) =>
  templateIds.has(row.MonsterTemplateID));
const skillIds = new Set(monsters.flatMap((row) => row.SkillList ?? []));
const skills = rows(await sourceJson(skillPath)).filter((row) =>
  skillIds.has(row.SkillID));

const categories = {
  family: [record("apocalyptic-shadow-v1",
    "docs/18-standard-and-challenge-modes.md", "Apocalyptic Shadow",
    "ApocalypticShadow", "ProjectPolicy", "Stable family boundary only.")],
  periods: [record("period.203019", schedulePath, "ID=203019")],
  groups: [
    record("group.3019", groupPath, "GroupID=3019"),
    record("group-extra.3019", groupExtraPath, "GroupID=3019"),
  ],
  stages: [
    ...stages.map((row) => record(`stage.${row.ID}`, mazePath, `ID=${row.ID}`)),
    record("stage.30195", tiercePath, "selected-ID=30195"),
  ],
  nodes: [
    ...stages.flatMap((stage) => [1, 2].map((side) =>
      record(`node.${stage.ID}.${side}`, mazePath, `ID=${stage.ID}#side=${side}`))),
    record("node.30195.1", tiercePath, "selected-ID=30195#side=1"),
  ],
  targets: targets.map((row) => record(`target.${row.ID}`, targetPath,
    `ID=${row.ID}`)),
  buffs: buffs.map((row) => record(`buff.${row.ID}`, buffPath, `ID=${row.ID}`)),
  enemy_variants: monsters.map((row) => record(`enemy.${row.MonsterID}`,
    monsterPath, `MonsterID=${row.MonsterID}`, "Shared")),
  enemy_templates: templates.map((row) => record(
    `enemy-template.${row.MonsterTemplateID}`, templatePath,
    `MonsterTemplateID=${row.MonsterTemplateID}`, "Shared")),
  enemy_skills: skills.map((row) => record(`enemy-skill.${row.SkillID}`,
    skillPath, `SkillID=${row.SkillID}`, "Shared")),
  mechanic_programs: inventory.files
    .filter((row) => row.classification === "mechanic-program"
      && !row.source_path.endsWith(".layout.json")
      && row.source_path.startsWith("Config/"))
    .map((row) => record(`program.${path.basename(row.source_path, ".json")}`,
      row.source_path, "$", "ApocalypticShadow")),
  exact_zero_pools: [
    "blessings", "curios", "occurrences", "services", "currencies", "shops",
  ].map((pool) => record(`zero-pool.${pool}`,
    "docs/goals/18-apocalyptic-shadow-reference-data.md",
    `Included content#${pool}`, "ApocalypticShadow", "ProjectPolicy",
    "Generated selector proof required before Candidate freeze.")),
};
for (const records of Object.values(categories)) records.sort((a, b) =>
  a.id.localeCompare(b.id));
const categoryCounts = Object.fromEntries(Object.entries(categories).map(
  ([name, records]) => [name, records.length]));
const all = Object.values(categories).flat();
const document = {
  schema_revision: "starclock.apocalyptic-shadow-content-manifest.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  game_version: "4.4",
  source_revision: revision,
  active_selector: {
    schedule_id: 203019,
    group_id: 3019,
    ordinary_stage_ids: stages.map((row) => row.ID),
    tierce_id: 30195,
    excluded_later_group_id: 3020,
  },
  counts: {
    records: all.length,
    categories: categoryCounts,
    ownership: {
      ApocalypticShadow: all.filter((row) =>
        row.ownership === "ApocalypticShadow").length,
      Shared: all.filter((row) => row.ownership === "Shared").length,
    },
  },
  categories,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  const existing = await readFile(output, "utf8").catch(() => "");
  if (existing !== bytes) throw new Error("content manifest drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, bytes);
}
console.log(`Apocalyptic Shadow manifest: ${all.length} obligations; `
  + `${document.counts.ownership.ApocalypticShadow} owned, `
  + `${document.counts.ownership.Shared} shared.`);
