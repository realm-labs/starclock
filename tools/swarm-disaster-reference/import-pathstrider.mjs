#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function common(values) {
  return context.envelope(values);
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}
function modeHint(finish, detailEn, detailZh) {
  const detail = `${detailEn} ${detailZh}`;
  if (finish.row.FinishType.includes("Nous")
    || /Gold and Gears|黄金与机械/iu.test(detail))
    return "GoldAndGears";
  if (/Swarm Disaster|寰宇蝗灾/iu.test(detail))
    return "SwarmDisaster";
  return "UnresolvedSharedDlc";
}
function enabledForSwarm(hint) {
  return hint === "SwarmDisaster";
}
function renderParameters(text, parameters) {
  return text.replace(/#([1-9][0-9]*)\[i\]/gu, (match, ordinal) => {
    const value = parameters[Number(ordinal) - 1];
    return value === undefined ? match : decimal(value);
  });
}

const objectivePolicy = await context.policyRef(
  "pathstrider-objectives",
  "Treat each cabinet QuestID as an external quest-completion condition. Commit its cabinet point adjustments and outgoing cabinet unlocks once, after the accepted Activity operation reports quest completion.",
  "Replace the external quest condition and commit boundary if released quest evaluator data exposes the authoritative progress event and ordering.",
);
const unlockPolicy = await context.policyRef(
  "pathstrider-unlocks",
  "Evaluate a released FinishWay after an accepted Activity operation, set its unlock flag once, and never infer a Swarm consumer from the shared DLC unlock row alone. Only explicit Swarm text enables a row before a later exact consumer binding.",
  "Replace evaluation timing, revocation or mode applicability when released engine evidence and exact consumers establish those semantics.",
);
const chapterPolicy = await context.policyRef(
  "mechanical-chapter-locators",
  "Use released layer, Communing dimension and point threshold only as a mechanical chapter-availability locator. Do not replay story presentation or infer a bonus payload from IsBonusUnlock.",
  "Replace the locator-only consequence when released structured data binds a chapter to an exact simulation-visible payload.",
);

const cabinetEntries = await context.table("RogueDLCAeonCabinet");
const objectives = cabinetEntries.map((cabinet) => {
  const cabinetId = cabinet.row.CabinetID;
  const questId = cabinet.row.QuestID;
  const name = localized(
    cabinet.row.CabinetName,
    `Pathstrider Objective ${cabinetId}`,
    `行者之道目标 ${cabinetId}`,
  );
  const rawDescription = localized(
    cabinet.row.CabinetMissionDesc,
    `Complete external quest ${questId}.`,
    `完成外部任务 ${questId}。`,
  );
  const descriptionParameters = (cabinet.row.DescParam ?? []).map(decimal);
  const description = {
    en: renderParameters(rawDescription.en, descriptionParameters),
    zh: renderParameters(rawDescription.zh, descriptionParameters),
  };
  return {
    ...common({
      id: `swarm-disaster.pathstrider-objective.${cabinetId}`,
      kind: "PathstriderObjective",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(cabinet), objectivePolicy],
      tags: ["pathstrider-objective", "external-quest", "project-policy"],
    }),
    cabinet_id: `swarm-disaster.pathstrider-cabinet.${cabinetId}`,
    quest_id: String(questId),
    finish_condition_id:
      `swarm-disaster.external-quest-condition.${questId}`,
    progress_policy: {
      source: "ExternalQuestCompletion",
      comparison: "Completed",
      description_parameters: descriptionParameters,
      update_boundary: "AfterAcceptedActivityOperation",
      once_scope: `PathstriderQuest:${questId}`,
    },
    unlock_ids: (cabinet.row.UnlockCabinetID ?? []).map((id) =>
      `swarm-disaster.pathstrider-cabinet.${id}`),
  };
});
outputs.set("pathstrider-objectives.json", ordered(objectives));

const finishEntries = await context.table("RogueDLCFinishWay");
const finishById = new Map(finishEntries.map((entry) => [entry.row.ID, entry]));
const unlockEntries = await context.table("RogueDLCUnlock");
const unlocksByFinish = new Map();
for (const unlock of unlockEntries) {
  const finishId = unlock.row.UnlockFinishWay;
  if (!unlocksByFinish.has(finishId)) unlocksByFinish.set(finishId, []);
  unlocksByFinish.get(finishId).push(unlock);
}

const finishConditions = finishEntries.map((finish) => {
  const finishId = finish.row.ID;
  const linkedUnlocks = unlocksByFinish.get(finishId) ?? [];
  const linkedHints = linkedUnlocks.map((unlock) => modeHint(
    finish,
    context.text(unlock.row.RogueUnlockDetail, "en"),
    context.text(unlock.row.RogueUnlockDetail, "zh_cn"),
  ));
  const hint = linkedHints.includes("SwarmDisaster")
    ? "SwarmDisaster"
    : linkedHints.includes("GoldAndGears")
      ? "GoldAndGears"
      : finish.row.FinishType.includes("Nous")
        ? "GoldAndGears"
        : "UnresolvedSharedDlc";
  return {
    ...common({
      id: `swarm-disaster.pathstrider-finish-condition.${finishId}`,
      kind: "PathstriderFinishCondition",
      nameEn: `DLC Finish Condition ${finishId}`,
      nameZh: `DLC 完成条件 ${finishId}`,
      summaryEn:
        `Apply ${finish.row.ParamType} to ${finish.row.FinishType} progress and require ${finish.row.Progress}.`,
      summaryZh:
        `对 ${finish.row.FinishType} 进度应用 ${finish.row.ParamType}，要求达到 ${finish.row.Progress}。`,
      sourceRefs: [
        context.sourceRef(finish),
        ...linkedUnlocks.map((unlock) => context.sourceRef(unlock)),
      ],
      tags: [
        "pathstrider-finish-condition",
        slug(finish.row.FinishType),
        slug(hint),
      ],
    }),
    source_id: String(finishId),
    finish_type: finish.row.FinishType,
    comparison: finish.row.ParamType,
    parameters: {
      integer: finish.row.ParamInt1 === undefined
        ? ""
        : String(finish.row.ParamInt1),
      text: finish.row.ParamStr1 ?? "",
      integers: (finish.row.ParamIntList ?? []).map(String),
      items: (finish.row.ParamItemList ?? []).map((item) =>
        typeof item === "object"
          ? Object.fromEntries(Object.entries(item).map(([key, value]) =>
            [key, decimal(value)]))
          : decimal(item)),
    },
    target_progress: String(finish.row.Progress),
    unlock_ids: linkedUnlocks.map(({ row }) =>
      `swarm-disaster.pathstrider-unlock.${row.RogueUnlockID}`),
    mode_hint: hint,
    enabled_for_swarm_compilation: enabledForSwarm(hint),
  };
});
outputs.set(
  "pathstrider-finish-conditions.json",
  ordered(finishConditions),
);

const unlocks = unlockEntries.map((unlock) => {
  const unlockId = unlock.row.RogueUnlockID;
  const finish = finishById.get(unlock.row.UnlockFinishWay);
  if (!finish)
    throw new Error(`unlock ${unlockId} references missing FinishWay`);
  const detail = localized(
    unlock.row.RogueUnlockDetail,
    `Satisfy DLC finish condition ${unlock.row.UnlockFinishWay}.`,
    `满足 DLC 完成条件 ${unlock.row.UnlockFinishWay}。`,
  );
  const hint = modeHint(finish, detail.en, detail.zh);
  const enabled = enabledForSwarm(hint);
  return {
    ...common({
      id: `swarm-disaster.pathstrider-unlock.${unlockId}`,
      kind: "PathstriderUnlock",
      nameEn: `DLC Unlock ${unlockId}`,
      nameZh: `DLC 解锁 ${unlockId}`,
      summaryEn: detail.en,
      summaryZh: detail.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(unlock),
        context.sourceRef(finish),
        unlockPolicy,
      ],
      tags: [
        "pathstrider-unlock",
        slug(hint),
        "project-policy",
        ...(enabled ? ["swarm-enabled"] : ["fail-closed"]),
      ],
    }),
    source_id: String(unlockId),
    finish_condition_id:
      `swarm-disaster.pathstrider-finish-condition.${finish.row.ID}`,
    unlock_consequence: {
      operation: "SetDlcUnlockFlag",
      unlock_flag_id: `swarm-disaster.dlc-unlock-flag.${unlockId}`,
      once_scope: `DlcUnlock:${unlockId}`,
      revocable: false,
      enabled_for_swarm_compilation: enabled,
    },
    evaluation_boundary: "AfterAcceptedActivityOperation",
    mode_hint: hint,
  };
});
outputs.set("pathstrider-unlocks.json", ordered(unlocks));

const dimensionEntries = await context.table("RogueDLCAeonDimension");
const dimensionIds = new Set(dimensionEntries.map(({ row }) =>
  row.AeonDimensionID));
const chapterEntries = await context.table("RogueDLCMainStory");
const chapters = chapterEntries.map((chapter) => {
  const chapterId = chapter.row.MainStoryID;
  const dimensionId = chapter.row.UnlockAeonDimension;
  if (dimensionId !== undefined && !dimensionIds.has(dimensionId))
    throw new Error(`chapter ${chapterId} references missing dimension`);
  const name = localized(
    chapter.row.MainStoryName,
    `Mechanical Chapter ${chapterId}`,
    `机械章节 ${chapterId}`,
  );
  const bonus = Boolean(chapter.row.IsBonusUnlock);
  return {
    ...common({
      id: `swarm-disaster.mechanical-chapter.${chapterId}`,
      kind: "MechanicalChapterLocator",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: dimensionId === undefined
        ? `Chapter ${chapterId} has no released Communing threshold.`
        : `Chapter ${chapterId} becomes available on plane ${chapter.row.Layer} at Communing dimension ${dimensionId} threshold ${chapter.row.UnlockPoint}.`,
      summaryZh: dimensionId === undefined
        ? `章节 ${chapterId} 没有已发布的觐见维度阈值。`
        : `章节 ${chapterId} 在第 ${chapter.row.Layer} 位面、觐见维度 ${dimensionId} 达到 ${chapter.row.UnlockPoint} 时可用。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(chapter), chapterPolicy],
      tags: [
        "mechanical-chapter-locator",
        bonus ? "bonus-declared" : "chapter-availability",
        "project-policy",
      ],
    }),
    source_id: String(chapterId),
    layer: chapter.row.Layer === undefined ? "" : String(chapter.row.Layer),
    dimension_id: dimensionId === undefined
      ? ""
      : `swarm-disaster.communing-dimension.${dimensionId}`,
    point_threshold: chapter.row.UnlockPoint === undefined
      ? ""
      : String(chapter.row.UnlockPoint),
    mechanical_unlock: {
      operation: "MakeMechanicalChapterAvailable",
      chapter_id: `swarm-disaster.mechanical-chapter.${chapterId}`,
      bonus_declared: bonus,
      bonus_payload: "",
      presentation_toast_type: chapter.row.MainStoryToastType ?? "",
      simulation_payload_status: bonus
        ? "UnresolvedFailClosed"
        : "ChapterAvailabilityOnly",
    },
  };
});
outputs.set(
  "mechanical-chapter-locators.json",
  ordered(chapters, ["layer", "id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Pathstrider ${check ? "verified" : "generated"}: ` +
  `${objectives.length} objectives, ${finishConditions.length} finish ` +
  `conditions, ${unlocks.length} unlocks and ${chapters.length} chapters.`,
);
