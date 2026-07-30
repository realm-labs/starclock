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
      const a = left[field];
      const b = right[field];
      if (a < b) return -1;
      if (a > b) return 1;
    }
    return 0;
  });
}

const displays = await context.table("RogueAeonDisplay");
const displayById = new Map(displays.map((display) => [
  display.row.DisplayID,
  display,
]));
const faces = await context.table("RogueDLCAeonDiceSurface");
const faceIdsByDie = new Map();
for (const face of faces) {
  const dieId = face.row.AeonDiceID;
  if (!faceIdsByDie.has(dieId)) faceIdsByDie.set(dieId, []);
  faceIdsByDie.get(dieId).push({
    id: face.row.AeonSurfaceDiceID,
    sort: face.row.Sort,
  });
}
for (const values of faceIdsByDie.values())
  values.sort((left, right) => left.sort - right.sort || left.id - right.id);

const effectSlotPolicy = await context.policyRef(
  "audience-paths",
  "Lower EffectType1 as the run-start maze effect and EffectType3 as the persistent Path graph effect while preserving every authored parameter slot separately.",
  "Replace the slot-to-lifecycle mapping if released engine evidence identifies a different application boundary.",
);
const pathRows = await context.table("RogueDLCAeon");
const audiencePaths = pathRows.map((pathRow) => {
  const aeonId = pathRow.row.AeonID;
  const display = displayById.get(pathRow.row.RogueAeonDisplayID);
  if (!display) throw new Error(`missing RogueAeonDisplay ${aeonId}`);
  const name = localized(
    display.row.RogueAeonPathName2,
    `Path ${aeonId}`,
    `命途 ${aeonId}`,
  );
  const description = localized(
    pathRow.row.EffectDesc3,
    `Audience Path effect ${aeonId}.`,
    `觐见行迹效果 ${aeonId}。`,
  );
  const pathSlug = slug(name.en.replace(/^The /u, ""));
  return {
    ...common({
      id: `swarm-disaster.audience-path.${aeonId}`,
      kind: "AudiencePath",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(pathRow),
        context.sourceRef(display),
        effectSlotPolicy,
      ],
      tags: ["audience-path", pathSlug, "project-policy"],
    }),
    source_id: String(aeonId),
    sort: pathRow.row.Sort,
    path_id: `universe.path.${pathSlug}`,
    audience_die_id: `swarm-disaster.audience-die.${pathRow.row.AeonDiceID}`,
    unlock_id: pathRow.row.UnlockID === undefined
      ? ""
      : String(pathRow.row.UnlockID),
    unlock_policy: pathRow.row.UnlockID === undefined
      ? "AvailableWithoutAuthoredUnlock"
      : "RequireAuthoredUnlockId",
    initial_effects: [{
      order: 0,
      operation: pathRow.row.EffectType1,
      parameters: (pathRow.row.EffectParam1 ?? []).map(String),
      secondary_parameters: (pathRow.row.EffectParam2 ?? []).map(String),
      application_boundary: "RunStart",
    }],
    passive_effects: [{
      order: 0,
      operation: pathRow.row.EffectType3,
      parameters: (pathRow.row.EffectParam3 ?? []).map(String),
      secondary_parameters: (pathRow.row.EffectParam4 ?? []).map(String),
      application_boundary: "AcceptedActivityOperation",
    }],
    description_parameters: (pathRow.row.DescParam ?? []).map(decimal),
    rogue_buff_type: String(pathRow.row.RogueBuffType),
    battle_event_buff_group: String(pathRow.row.BattleEventBuffGroup),
    battle_event_enhance_buff_group:
      String(pathRow.row.BattleEventEnhanceBuffGroup),
    extra_effect_refs: (pathRow.row.ExtraEffect ?? []).map((id) =>
      `source-effect.${id}`),
  };
});
outputs.set("audience-paths.json", ordered(audiencePaths, ["sort", "id"]));

const pathByDieId = new Map(pathRows.map((pathRow) => [
  pathRow.row.AeonDiceID,
  pathRow,
]));
const dicePolicy = await context.policyRef(
  "audience-dice",
  "Retain authored face Sort ordering and reject an empty face set. Exact roll, reroll, cheat and no-legal-target behavior is supplied by the typed G09-P1-B5 rows.",
  "Replace only the provisional roll policy fields when G09-P1-B5 or stronger released engine evidence supplies the exact operation.",
);
const diceRows = await context.table("RogueDLCAeonDice");
const audienceDice = diceRows.map((dice) => {
  const dieId = dice.row.AeonDiceID;
  const pathRow = pathByDieId.get(dieId);
  if (!pathRow) throw new Error(`missing audience Path for die ${dieId}`);
  const display = displayById.get(pathRow.row.RogueAeonDisplayID);
  const pathName = localized(
    display.row.RogueAeonPathName2,
    `Path ${dieId}`,
    `命途 ${dieId}`,
  );
  const description = localized(
    dice.row.DiceShortDesc,
    `${pathName.en} Audience Die.`,
    `${pathName.zh}觐见行迹骰。`,
  );
  const startDescription = localized(
    dice.row.DiceStartEffectDesc,
    `${pathName.en} Audience Die start effect.`,
    `${pathName.zh}觐见行迹骰初始效果。`,
  );
  const pathSlug = slug(pathName.en.replace(/^The /u, ""));
  const faceIds = faceIdsByDie.get(dieId) ?? [];
  return {
    ...common({
      id: `swarm-disaster.audience-die.${dieId}`,
      kind: "AudienceDie",
      nameEn: `${pathName.en} Audience Die`,
      nameZh: `${pathName.zh}觐见行迹骰`,
      summaryEn: description.en,
      summaryZh: description.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(dice),
        context.sourceRef(pathRow),
        dicePolicy,
      ],
      tags: ["audience-die", pathSlug, "project-policy"],
    }),
    source_id: String(dieId),
    path_id: `universe.path.${pathSlug}`,
    audience_path_id: `swarm-disaster.audience-path.${pathRow.row.AeonID}`,
    face_ids: faceIds.map(({ id }) => `swarm-disaster.dice-face.${id}`),
    roll_policy: {
      candidate_order: "AuthoredSortThenStableFaceId",
      empty_face_set: "Reject",
      control_rule_source: "G09-P1-B5",
    },
    unlock_id: pathRow.row.UnlockID === undefined
      ? ""
      : String(pathRow.row.UnlockID),
    initial_effect_summary_en: startDescription.en,
    initial_effect_summary_zh_cn: startDescription.zh,
    initial_effect_parameters: (dice.row.StartDescParam ?? []).map(String),
    passive_description_parameters:
      (dice.row.DescParam ?? []).map(decimal),
    extra_effect_refs: (dice.row.ExtraEffect ?? []).map((id) =>
      `source-effect.${id}`),
  };
});
outputs.set("audience-dice.json", ordered(audienceDice));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Audience Dice ${check ? "verified" : "generated"}: ` +
  `${audiencePaths.length} Paths and ${audienceDice.length} dice with ` +
  `${faces.length} face references.`,
);
