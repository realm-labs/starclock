#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function ordered(rows) {
  return rows.sort((left, right) => compare(left.id, right.id));
}
function normalize(value) {
  if (value === undefined || value === null) return "";
  if (Array.isArray(value)) return value.map(normalize);
  if (typeof value === "number") return decimal(value);
  if (typeof value !== "object") return value;
  if (Object.keys(value).length === 1 && Object.hasOwn(value, "Value"))
    return decimal(value);
  return Object.fromEntries(Object.entries(value)
    .map(([key, entry]) => [key, normalize(entry)]));
}
function textRefs(...references) {
  const hashes = [...new Set(references
    .map((reference) => reference?.Hash === undefined
      ? ""
      : String(reference.Hash))
    .filter(Boolean))];
  return hashes.flatMap((hash) => context.bilingualTextRefs(hash));
}
function display(reference, locale, fallback) {
  return context.text(reference, locale) || fallback;
}
function envelope(entry, {
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  textFields = [],
  tags = [],
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    sourceRefs: [context.sourceRef(entry), ...textRefs(...textFields)],
    tags: ["event", "gridfight", ...tags],
  });
}

const occurrences = [];
const variants = [];
const choices = [];
const prayFinishLinks = new Map();

const prayQuests = await context.table("GridFightPrayQuest");
for (const entry of prayQuests) {
  const row = entry.row;
  const id = String(row.ID);
  const choiceId = `currency-wars.occurrence-choice.pray.${id}`;
  const hasFinishWay = row.FinishWayID !== undefined;
  const variantId = hasFinishWay
    ? `currency-wars.occurrence-variant.pray-finish.${row.FinishWayID}`
    : `currency-wars.occurrence.pray.${id}`;
  if (hasFinishWay) {
    const links = prayFinishLinks.get(String(row.FinishWayID)) ?? [];
    links.push({
      occurrence_id: `currency-wars.occurrence.pray.${id}`,
      choice_id: choiceId,
    });
    prayFinishLinks.set(String(row.FinishWayID), links);
  }
  occurrences.push({
    ...envelope(entry, {
      id: `currency-wars.occurrence.pray.${id}`,
      kind: "CurrencyWarsOccurrence",
      nameEn: display(row.PrayTitle, "en", `Pray event ${id}`),
      nameZh: display(row.PrayTitle, "zh_cn", `祈愿事件 ${id}`),
      summaryEn:
        `Pray event ${id} has type ${row.PrayType}, ${hasFinishWay ? `finish condition ${row.FinishWayID}` : "an immediate outcome"} and one deterministic accept/finish outcome row.`,
      summaryZh:
        `祈愿事件 ${id} 的类型为 ${row.PrayType}，${hasFinishWay ? `完成条件为 ${row.FinishWayID}` : "使用即时结果"}，并具有一条确定的接受/完成结果。`,
      textFields: [row.PrayTitle, row.PrayDesc, row.PrayPriceDesc],
      tags: ["pray", String(row.PrayType).toLowerCase()],
    }),
    source_id: id,
    variant_ids: hasFinishWay ? [variantId] : [],
    unlock_rules: [{ pray_type: row.PrayType }],
    choice_ids: [choiceId],
  });
  choices.push({
    ...envelope(entry, {
      id: choiceId,
      kind: "CurrencyWarsOccurrenceChoice",
      nameEn: `Pray event ${id} choice`,
      nameZh: `祈愿事件 ${id} 选择`,
      summaryEn:
        `Accepting Pray event ${id} applies authored accept bonus ${row.AcceptBonus ?? "none"} and finish bonus ${row.FinishBonus ?? "none"}.`,
      summaryZh:
        `接受祈愿事件 ${id} 时应用已编写的接受加成 ${row.AcceptBonus ?? "无"} 与完成加成 ${row.FinishBonus ?? "无"}。`,
      textFields: [row.PrayTitle, row.PrayPriceDesc],
      tags: ["choice", "pray"],
    }),
    source_id: id,
    variant_id: variantId,
    ordinal: "0",
    conditions: {
      pray_type: row.PrayType,
      finish_way_id: String(row.FinishWayID ?? ""),
    },
    costs: row.AcceptBonus === undefined
      ? []
      : [{ kind: "AcceptBonus", id: String(row.AcceptBonus) }],
    ordered_outcomes: [
      ...(row.AcceptBonus === undefined
        ? []
        : [{ operation: "ApplyAcceptBonus", bonus_id: String(row.AcceptBonus) }]),
      ...(row.FinishBonus === undefined
        ? []
        : [{ operation: "ApplyFinishBonus", bonus_id: String(row.FinishBonus) }]),
    ],
  });
}

const finishWays = await context.table("GridFightPrayQuestFinishWay");
for (const entry of finishWays) {
  const row = entry.row;
  const id = String(row.ID);
  const links = prayFinishLinks.get(id) ?? [];
  variants.push({
    ...envelope(entry, {
      id: `currency-wars.occurrence-variant.pray-finish.${id}`,
      kind: "CurrencyWarsOccurrenceVariant",
      nameEn: `Pray finish condition ${id}`,
      nameZh: `祈愿完成条件 ${id}`,
      summaryEn:
        `Pray finish condition ${id} uses ${row.FinishType}, progress ${row.Progress} and explicit typed parameters.`,
      summaryZh:
        `祈愿完成条件 ${id} 使用 ${row.FinishType}，进度要求为 ${row.Progress}，并保留明确的类型化参数。`,
      tags: ["finish-condition", "pray"],
    }),
    source_id: id,
    occurrence_id: links[0]?.occurrence_id
      ?? `currency-wars.occurrence.pray.${id}`,
    graph_path: "",
    entry_conditions: {
      finish_type: row.FinishType,
      parameter_type: row.ParamType,
      integer_1: String(row.ParamInt1),
      string_1: row.ParamStr1,
      integer_list: row.ParamIntList.map(String),
      item_list: normalize(row.ParamItemList),
      progress: String(row.Progress),
      backtracks: row.IsBackTrack,
    },
    choice_ids: links.map(({ choice_id: choiceId }) => choiceId),
    occurrence_ids: links.map(({ occurrence_id: occurrenceId }) => occurrenceId),
  });
}

const presents = await context.table("GridFightPresentConfig");
for (const entry of presents) {
  const row = entry.row;
  const id = String(row.ID);
  const choiceId = `currency-wars.occurrence-choice.present.${id}`;
  occurrences.push({
    ...envelope(entry, {
      id: `currency-wars.occurrence.present.${id}`,
      kind: "CurrencyWarsOccurrence",
      nameEn: display(row.PresentName, "en", `Present ${id}`),
      nameZh: display(row.PresentName, "zh_cn", `礼物 ${id}`),
      summaryEn:
        `Present ${id} applies bonus ${row.BonusID} for ${row.ShortenType} result shortening.`,
      summaryZh:
        `礼物 ${id} 在 ${row.ShortenType} 结果缩短规则下应用加成 ${row.BonusID}。`,
      textFields: [row.PresentName, row.PresentDesc],
      tags: ["present", String(row.ShortenType).toLowerCase()],
    }),
    source_id: id,
    variant_ids: [],
    unlock_rules: [{ result_shortening: row.ShortenType }],
    choice_ids: [choiceId],
  });
  choices.push({
    ...envelope(entry, {
      id: choiceId,
      kind: "CurrencyWarsOccurrenceChoice",
      nameEn: `Present ${id} outcome`,
      nameZh: `礼物 ${id} 结果`,
      summaryEn:
        `Present ${id} deterministically applies bonus ${row.BonusID} at the ${row.ShortenType} result boundary.`,
      summaryZh:
        `礼物 ${id} 在 ${row.ShortenType} 结果边界确定性应用加成 ${row.BonusID}。`,
      textFields: [row.PresentName],
      tags: ["choice", "present"],
    }),
    source_id: id,
    variant_id: `currency-wars.occurrence.present.${id}`,
    ordinal: "0",
    conditions: { result_shortening: row.ShortenType },
    costs: [],
    ordered_outcomes: [
      { operation: "ApplyBonus", bonus_id: String(row.BonusID) },
    ],
  });
}

const tutorialTasks = await context.table("GridFightTutorialTask");
for (const entry of tutorialTasks) {
  const row = entry.row;
  const id = String(row.TaskID);
  const occurrenceId = `currency-wars.occurrence.tutorial-task.${id}`;
  const variantId = `currency-wars.occurrence-variant.tutorial-task.${id}`;
  occurrences.push({
    ...envelope(entry, {
      id: occurrenceId,
      kind: "CurrencyWarsOccurrence",
      nameEn: `Mechanical tutorial task ${id}`,
      nameZh: `机制教程任务 ${id}`,
      summaryEn:
        `Mechanical tutorial task ${id} enters one mode-owned level graph; no dialogue or presentation is retained.`,
      summaryZh:
        `机制教程任务 ${id} 进入一条玩法专属关卡图；不保留对话或演出内容。`,
      tags: ["mechanical-tutorial", "task"],
    }),
    source_id: id,
    variant_ids: [variantId],
    unlock_rules: [{ task_id: id }],
    choice_ids: [],
  });
  variants.push({
    ...envelope(entry, {
      id: variantId,
      kind: "CurrencyWarsOccurrenceVariant",
      nameEn: `Mechanical tutorial graph ${id}`,
      nameZh: `机制教程图 ${id}`,
      summaryEn:
        `Tutorial task ${id} resolves through ${row.LevelGraphPath}; graph operations are deferred to the mechanic-program batch.`,
      summaryZh:
        `教程任务 ${id} 通过 ${row.LevelGraphPath} 解析；图操作留待机制程序批次处理。`,
      tags: ["mechanical-tutorial", "variant"],
    }),
    source_id: id,
    occurrence_id: occurrenceId,
    graph_path: row.LevelGraphPath,
    entry_conditions: { task_id: id },
    choice_ids: [],
  });
}

const outputs = new Map([
  ["occurrences.json", ordered(occurrences)],
  ["occurrence-variants.json", ordered(variants)],
  ["occurrence-choices.json", ordered(choices)],
]);
await writeOrCheck(context, outputs, check);
if (occurrences.length !== 167 || variants.length !== 150
  || choices.length !== 90)
  throw new Error("GridFight occurrence closure drift");
console.log(
  `Currency Wars events ${check ? "verified" : "generated"}: ` +
  `${occurrences.length} occurrences, ${variants.length} variants and ` +
  `${choices.length} choices; four AssistantMessage rows excluded.`,
);
