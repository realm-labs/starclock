import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

const root = resolve(process.env.STARCLOCK_PF_SOURCE ?? ".cache/pure-fiction/turnbasedgamedata");
const output = resolve("content-manifests/pure-fiction-v1/source-inventory.json");
const check = process.argv.includes("--check");

if (!existsSync(join(root, "ExcelOutput/ChallengeStoryGroupConfig.json"))) {
  throw new Error(`Pure Fiction source cache is incomplete: ${root}`);
}

const exactFiles = [
  "ExcelOutput/ChallengeStoryGroupConfig.json",
  "ExcelOutput/ChallengeStoryGroupExtra.json",
  "ExcelOutput/ChallengeStoryMazeConfig.json",
  "ExcelOutput/ChallengeStoryMazeExtra.json",
  "ExcelOutput/ChallengeStoryMazeTierce.json",
  "ExcelOutput/ChallengeStoryRewardLine.json",
  "ExcelOutput/ChallengeStoryTargetConfig.json",
  "ExcelOutput/ChallengeStoryTheme.json",
  "ExcelOutput/ScheduleDataChallengeStory.json",
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/MapEntrance.json",
  "ExcelOutput/MapEntranceGroup.json",
  "ExcelOutput/MapEntranceUnlock.json",
  "ExcelOutput/MappingInfo.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
  "Config/ConfigAbility/Level/Level_MazeChallengeBuff_Ability.json",
  "Config/ConfigAbility/StageBattleEventAbility.json",
  "Config/Level/StageCommonTemplate.json"
];

function walk(directory) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  });
}

const adjacent = readdirSync(join(root, "ExcelOutput"))
  .filter((name) => /^(Challenge|ConstValueChallenge).*\.json$/.test(name))
  .map((name) => `ExcelOutput/${name}`);
const fantastic = walk(join(root, "Config"))
  .map((path) => relative(root, path))
  .filter((path) => /FantasticStory.*\.json$/.test(path));
const enemyPrograms = walk(join(root, "Config/Character/Monster"))
  .map((path) => relative(root, path))
  .filter((path) => path.endsWith(".json"));
const stagePrograms = walk(join(root, "Config/Level/Stage"))
  .map((path) => relative(root, path))
  .filter((path) => path.endsWith(".json"));

const selected = [...new Set([...exactFiles, ...adjacent, ...fantastic, ...enemyPrograms, ...stagePrograms])]
  .sort((left, right) => left.localeCompare(right));
const records = selected.map((path) => {
  const bytes = readFileSync(join(root, path));
  let classification = "shared_closure_candidate";
  if (/ExcelOutput\/ChallengeStory|ScheduleDataChallengeStory/.test(path)) classification = "pure_fiction_table";
  else if (/FantasticStory/.test(path)) classification = "pure_fiction_program_candidate";
  else if (/ExcelOutput\/(Challenge(?!Story)|ConstValueChallenge)/.test(path)) classification = "adjacent_challenge_exclusion_candidate";
  else if (/TextMap/.test(path)) classification = "bilingual_identity_evidence";
  else if (/Config\/Character\/Monster/.test(path)) classification = "enemy_program_closure_candidate";
  else if (/Config\/Level\/Stage/.test(path)) classification = "stage_program_closure_candidate";
  return {
    path,
    classification,
    size: statSync(join(root, path)).size,
    sha256: createHash("sha256").update(bytes).digest("hex")
  };
});
const counts = Object.fromEntries([...new Set(records.map((row) => row.classification))]
  .sort()
  .map((classification) => [classification, records.filter((row) => row.classification === classification).length]));
const inventory = {
  schema_revision: "pure-fiction-source-inventory-v1",
  snapshot: "Version 4.4",
  source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  records,
  counts,
  record_count: records.length
};
const canonical = `${JSON.stringify(inventory, null, 2)}\n`;
if (check) {
  if (readFileSync(output, "utf8") !== canonical) throw new Error("Pure Fiction inventory drift");
} else {
  writeFileSync(output, canonical);
}
console.log(`Pure Fiction inventory verified: ${records.length} files`);

