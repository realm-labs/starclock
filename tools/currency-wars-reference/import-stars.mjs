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
const outputs = new Map();

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

const roleStars = await context.table("GridFightRoleStar");
const rankAttachments = await context.table("GridFightRankAttachment");
const servantStars = await context.table("GridFightServantStar");
if (roleStars.length !== 266 || rankAttachments.length !== 1596
  || servantStars.length !== 29)
  throw new Error("GridFight star table closure drift");
const attachmentsByRoleStar = Object.groupBy(
  rankAttachments,
  ({ row }) => `${row.RoleID}:${row.Star}`,
);
if (Object.keys(attachmentsByRoleStar).length !== 266
  || Object.values(attachmentsByRoleStar).some((entries) => entries.length !== 6))
  throw new Error("GridFight RankAttachment role/star closure drift");
const copyCount = { 1: "1", 2: "3", 3: "9", 4: "27" };
const publicStarRefs = context.bilingualTextRefs("7693488975416237801");

const states = roleStars.map((entry) => {
  const key = `${entry.row.ID}:${entry.row.Star}`;
  const attachments = attachmentsByRoleStar[key];
  return {
    ...context.envelope({
      id: `currency-wars.star-state.role.${entry.row.ID}.${entry.row.Star}`,
      kind: "CurrencyWarsStarState",
      nameEn: `Role ${entry.row.ID} star ${entry.row.Star}`,
      nameZh: `角色 ${entry.row.ID} ${entry.row.Star} 星`,
      summaryEn:
        `Role ${entry.row.ID} star ${entry.row.Star} binds battle event ${entry.row.BEID}, ${attachments.length} rank attachments and ${entry.row.GeneralPropertyModifyList.length} property modifiers.`,
      summaryZh:
        `角色 ${entry.row.ID} 的 ${entry.row.Star} 星状态绑定战斗事件 ${entry.row.BEID}、${attachments.length} 个位阶附件与 ${entry.row.GeneralPropertyModifyList.length} 个属性修改。`,
      sourceRefs: [
        context.sourceRef(entry),
        ...attachments.map((attachment) => context.sourceRef(attachment)),
        ...publicStarRefs,
      ],
      tags: ["gridfight", "role-star", `star-${entry.row.Star}`],
    }),
    source_id: `${entry.row.ID}:${entry.row.Star}`,
    avatar_id: String(entry.row.ID),
    star_level: entry.row.Star,
    copy_count: copyCount[entry.row.Star],
    scaling_refs: attachments.map(({ row }) =>
      `gridfight-rank-attachment:${row.RoleID}:${row.Rank}:${row.Star}`),
    rank_attachments: attachments.map(({ row }) => ({
      rank: row.Rank,
      property_modifiers: normalize(row.GeneralPropertyModifyList),
    })),
    battle_event_id: String(entry.row.BEID ?? ""),
    skill_override_source_ids: (entry.row.SkillOverrideSrc ?? []).map(String),
    skill_override_destination_ids:
      (entry.row.SkillOverrideDest ?? []).map(String),
    front_skill_ids: (entry.row.FrontShowSkillIDList ?? []).map(String),
    back_execution_skill_ids: (entry.row.BESkillIDList ?? []).map(String),
    back_skill_ids: (entry.row.BackShowSkillIDList ?? []).map(String),
    back_ability_name: entry.row.BackAbilityName ?? "",
    config_path: entry.row.JsonOverrideConfig ?? "",
    ai_path: entry.row.AIPath ?? "",
    property_modifiers: normalize(entry.row.GeneralPropertyModifyList),
    front_power_base: decimal(entry.row.FrontPowerBase ?? 0),
    back_power_base: decimal(entry.row.BackPowerBase ?? 0),
    luck_chance: decimal(entry.row.LuckChance ?? 0),
    luck_damage: decimal(entry.row.LuckDamage ?? 0),
    extra_heal_base: decimal(entry.row.ExtraHealBase ?? 0),
    extra_shield_base: decimal(entry.row.ExtraShieldBase ?? 0),
  };
});
for (const entry of servantStars)
  states.push({
    ...context.envelope({
      id:
        `currency-wars.star-state.servant.${entry.row.ID}.${entry.row.ServantID}.${entry.row.Star}`,
      kind: "CurrencyWarsStarState",
      nameEn:
        `Role ${entry.row.ID} servant ${entry.row.ServantID} star ${entry.row.Star}`,
      nameZh:
        `角色 ${entry.row.ID} 从者 ${entry.row.ServantID} ${entry.row.Star} 星`,
      summaryEn:
        `Servant ${entry.row.ServantID} at star ${entry.row.Star} publishes explicit config, AI, skill overrides and HP/Speed inheritance fields.`,
      summaryZh:
        `从者 ${entry.row.ServantID} 的 ${entry.row.Star} 星状态发布明确的配置、AI、技能覆盖与生命/速度继承字段。`,
      sourceRefs: [context.sourceRef(entry), ...publicStarRefs],
      tags: ["gridfight", "servant-star", `star-${entry.row.Star}`],
    }),
    source_id: `${entry.row.ID}:${entry.row.ServantID}:${entry.row.Star}`,
    avatar_id: String(entry.row.ID),
    servant_id: String(entry.row.ServantID),
    star_level: entry.row.Star,
    copy_count: copyCount[entry.row.Star],
    scaling_refs: [],
    rank_attachments: [],
    battle_event_id: "",
    skill_override_source_ids: (entry.row.SkillOverrideSrc ?? []).map(String),
    skill_override_destination_ids:
      (entry.row.SkillOverrideDest ?? []).map(String),
    front_skill_ids: [],
    back_execution_skill_ids: [],
    back_skill_ids: (entry.row.ServantShowSkiilIDList ?? []).map(String),
    back_ability_name: "",
    config_path: entry.row.JsonOverrideConfig ?? "",
    ai_path: entry.row.AIPath ?? "",
    property_modifiers: [],
    hp_base: String(entry.row.HPBase ?? ""),
    hp_inherit: String(entry.row.HPInherit ?? ""),
    hp_skill_id: String(entry.row.HPSkill ?? ""),
    speed_base: String(entry.row.SpeedBase ?? ""),
    speed_inherit: String(entry.row.SpeedInherit ?? ""),
    speed_skill_id: String(entry.row.SpeedSkill ?? ""),
  });
outputs.set("star-states.json", ordered(states));

const roleStarsById = Object.groupBy(roleStars, ({ row }) => String(row.ID));
const combinationRules = [];
for (const [roleId, entries] of Object.entries(roleStarsById)) {
  const byStar = new Map(entries.map((entry) => [entry.row.Star, entry]));
  for (const entry of entries) {
    const next = byStar.get(entry.row.Star + 1);
    if (!next) continue;
    combinationRules.push({
      ...context.envelope({
        id:
          `currency-wars.star-combination.role.${roleId}.${entry.row.Star}.to.${next.row.Star}`,
        kind: "CurrencyWarsStarCombinationRule",
        nameEn:
          `Role ${roleId}: star ${entry.row.Star} to ${next.row.Star}`,
        nameZh:
          `角色 ${roleId}：${entry.row.Star} 星升至 ${next.row.Star} 星`,
        summaryEn:
          `Three copies of role ${roleId} at star ${entry.row.Star} automatically combine into one star-${next.row.Star} state.`,
        summaryZh:
          `三个 ${entry.row.Star} 星角色 ${roleId} 自动合成为一个 ${next.row.Star} 星状态。`,
        evidenceQuality: "ExactPublicText",
        sourceRefs: [
          context.sourceRef(entry),
          context.sourceRef(next),
          ...publicStarRefs,
        ],
        tags: ["combine", "gridfight", "star-upgrade"],
      }),
      input_state:
        `currency-wars.star-state.role.${roleId}.${entry.row.Star}`,
      required_copies: 3,
      output_state:
        `currency-wars.star-state.role.${roleId}.${next.row.Star}`,
      overflow_rule: "Repeat while at least three equal-star copies remain.",
    });
  }
}
outputs.set("star-combination-rules.json", ordered(combinationRules));

const lifecyclePolicy = await context.policyRef(
  "star-lifecycle-order",
  "Released text proves automatic three-copy combination but does not publish simultaneous purchase/sale/replacement precedence.",
  "Replace with a released GridFight operation program or reproducible same-boundary observations.",
);
const lifecycleRules = [
  {
    id: "acquire",
    nameEn: "Acquire and combine",
    nameZh: "获得与合成",
    operation: "AcquireCopy",
    replacement: "Add the copy, then repeatedly combine legal equal-star triples.",
    sale: "No sale in this operation.",
    teardown: "Preserve the resulting star state in run scope.",
  },
  {
    id: "sell",
    nameEn: "Sell starred role",
    nameZh: "出售升星角色",
    operation: "SellRole",
    replacement: "Remove the selected role state.",
    sale: "Use the exact rarity/star sell price from roster-transactions.json.",
    teardown: "Remove role, star, position and contribution state together.",
  },
  {
    id: "terminal",
    nameEn: "Terminal star state",
    nameZh: "终端星级状态",
    operation: "AcquireAtMaximumStar",
    replacement: "Do not synthesize an unauthored higher star state.",
    sale: "Retain or sell using an explicit user decision.",
    teardown: "No implicit overflow conversion is claimed.",
  },
].map((rule) => ({
  ...context.envelope({
    id: `currency-wars.star-lifecycle.${rule.id}`,
    kind: "CurrencyWarsStarLifecycleRule",
    nameEn: rule.nameEn,
    nameZh: rule.nameZh,
    summaryEn:
      `${rule.nameEn} records the reference lifecycle boundary; same-boundary precedence remains policy-bound.`,
    summaryZh:
      `${rule.nameZh} 记录资料生命周期边界；同边界优先级仍为策略约束。`,
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [...publicStarRefs, lifecyclePolicy],
    tags: ["gridfight", "star-lifecycle", rule.id],
  }),
  operation: rule.operation,
  replacement_rule: rule.replacement,
  sale_rule: rule.sale,
  teardown: rule.teardown,
}));
outputs.set("star-lifecycle-rules.json", ordered(lifecycleRules));

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars stars ${check ? "verified" : "generated"}: ` +
  `${states.length} states, ${combinationRules.length} combinations and ` +
  `${lifecycleRules.length} lifecycle rules.`,
);
