import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const source = path.join(root, ".cache", "content-reference", "turnbasedgamedata");
const output = path.join(
  root,
  "content-manifests",
  "standard-universe-v1",
  "source-inventory.json",
);
const expectedRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function git(args) {
  return execFileSync("git", ["-C", source, ...args], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function isRogueEvidence(relativePath) {
  if (relativePath.startsWith("ExcelOutput/")) {
    const name = path.posix.basename(relativePath);
    return (
      /^Rogue.*\.json$/u.test(name) ||
      /^(ActivityRogue.*|ConstValueRogue|FinishWayRogue|GuideRogue.*|ScheduleDataRogue)\.json$/u.test(
        name,
      )
    );
  }
  return (
    /^Config\/ConfigAbility\/BattleEvent\/.*Rogue.*\.json$/u.test(relativePath) ||
    /^Config\/ConfigAbility\/Level\/Level_.*Rogue.*\.json$/u.test(relativePath)
  );
}

const standardTables = new Set([
  "ConstValueRogue.json",
  "FinishWayRogue.json",
  "RogueActivityResidentConfig.json",
  "RogueAdventureRoom.json",
  "RogueAeon.json",
  "RogueAeonDisplay.json",
  "RogueAeonLevelConfig.json",
  "RogueAeonListConfig.json",
  "RogueAreaConfig.json",
  "RogueBonus.json",
  "RogueBuff.json",
  "RogueBuffGroup.json",
  "RogueBuffHint.json",
  "RogueBuffType.json",
  "RogueDialogueOption.json",
  "RogueDialogueOptionDisplay.json",
  "RogueEventSpecialOption.json",
  "RogueManager.json",
  "RogueMap.json",
  "RogueMazeBuff.json",
  "RogueMiracle.json",
  "RogueMiracleDisplay.json",
  "RogueMiracleEffect.json",
  "RogueMiracleEffectDisplay.json",
  "RogueMiracleGroup.json",
  "RogueMonster.json",
  "RogueMonsterEliteDropItem.json",
  "RogueMonsterGroup.json",
  "RogueNPC.json",
  "RogueRoom.json",
  "RogueRoomType.json",
  "RogueShop.json",
  "RogueTalent.json",
  "RogueUnlockConfig.json",
  "ScheduleDataRogue.json",
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

function classify(relativePath) {
  if (relativePath.startsWith("Config/ConfigAbility/")) {
    return "mechanic_evidence";
  }
  const name = path.posix.basename(relativePath);
  if (standardTables.has(name)) return "standard_candidate";
  if (presentationTables.has(name)) return "presentation_or_account";
  if (
    /^(ActivityRogue|RogueDLC|RogueEndless|RogueMagic|RogueNous|RoguePersona|RogueTourn)/u.test(
      name,
    )
  ) {
    return "other_mode";
  }
  return "shared_requires_reachability";
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

const revision = git(["rev-parse", "HEAD"]).trim();
if (revision !== expectedRevision) {
  throw new Error(`source revision mismatch: ${revision}`);
}

const tracked = git(["ls-tree", "-r", "--name-only", "HEAD"])
  .split(/\r?\n/u)
  .filter(Boolean)
  .filter(isRogueEvidence)
  .sort(compareText);

const records = [];
for (const relativePath of tracked) {
  const absolutePath = path.join(source, ...relativePath.split("/"));
  const bytes = await readFile(absolutePath);
  records.push({
    path: relativePath,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.length,
    family: classify(relativePath),
  });
}

const counts = Object.fromEntries(
  [...new Set(records.map((record) => record.family))]
    .sort(compareText)
    .map((family) => [family, records.filter((record) => record.family === family).length]),
);
const payload = {
  schema: "starclock.standard-universe-source-inventory.v1",
  source: {
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision,
  },
  classification_policy: {
    standard_candidate:
      "base table required to derive the main-world denominator",
    shared_requires_reachability:
      "shared Rogue row/table requiring main-world pool proof",
    other_mode:
      "table owned by a DLC, tournament, event or another universe family",
    presentation_or_account:
      "dialogue, guide, handbook, score/reward or other excluded presentation/account data",
    mechanic_evidence:
      "ability program used only to review exact battle-visible mechanics",
  },
  counts,
  records,
};

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = await readFile(output, "utf8");
  if (committed !== encoded) {
    throw new Error("standard universe source inventory has generated drift");
  }
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(
  `Standard universe source inventory ${check ? "verified" : "generated"}: ${records.length} files (${revision}).`,
);
