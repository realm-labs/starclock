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

function ordered(rows) {
  return rows.sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}

const avatarEntries = await context.table("AvatarConfig");
const avatarById = new Map(
  avatarEntries.map((entry) => [String(entry.row.AvatarID), entry]),
);
const eligibilityEntries = await context.table("RogueTournBuildRefAvatar");
const mappingEntries = await context.table("RogueTournAvatar");
const roleEntries = await context.table("RogueTournRole");
const mazeBuffEntries = await context.table("RogueMazeBuff");
const mazeBuffById = new Map(
  mazeBuffEntries.map((entry) => [String(entry.row.ID), entry]),
);
const mappingByAvatar = new Map(
  mappingEntries.map((entry) => [String(entry.row.AvatarID), entry]),
);
const roleByAvatar = new Map(
  roleEntries.map((entry) => [String(entry.row.AvatarID), entry]),
);
const eligibilityByAvatar = new Map(
  eligibilityEntries.map((entry) => [String(entry.row.AvatarID), entry]),
);

function avatarName(avatarId) {
  const avatar = avatarById.get(String(avatarId));
  if (!avatar) return {
    en: `Unresolved Released Avatar Locator ${avatarId}`,
    zh: `未解析的已发布角色定位符 ${avatarId}`,
  };
  return {
    en: context.text(avatar.row.AvatarName, "en")
      || `Avatar ${avatarId}`,
    zh: context.text(avatar.row.AvatarName, "zh_cn")
      || `角色 ${avatarId}`,
  };
}

const eligibilityRows = eligibilityEntries.map((entry) => {
  const avatarId = String(entry.row.AvatarID);
  const name = avatarName(avatarId);
  const avatar = avatarById.get(avatarId);
  return {
    ...context.envelope({
      id: `divergent-universe.mapping-eligibility.${avatarId}`,
      kind: "DivergentUniverseArithmeticMappingEligibility",
      nameEn: `${name.en} Arithmetic Mapping Eligibility`,
      nameZh: `${name.zh} 数值映射资格`,
      summaryEn:
        `${name.en} is present in the released build-reference ordering at weight ${entry.row.SortWeight}.`,
      summaryZh:
        `${name.zh} 位于已发布构筑参考顺序中，排序权重为 ${entry.row.SortWeight}。`,
      sourceRefs: [
        context.sourceRef(entry),
        ...(avatar ? [context.sourceRef(avatar)] : []),
      ],
      tags: ["arithmetic-mapping", "eligibility"],
    }),
    avatar_id: avatarId,
    sort_weight: entry.row.SortWeight,
    eligibility: "ExplicitBuildReferenceCatalog",
    has_special_avatar_mapping: mappingByAvatar.has(avatarId),
    has_role_buff: roleByAvatar.has(avatarId),
    account_comparison_policy:
      "Apply only when the corresponding released below-threshold condition is true.",
  };
});
outputs.set("arithmetic-mapping-eligibility.json", ordered(eligibilityRows));

const policyRef = await context.policyRef(
  "arithmetic-mapping-build-values",
  "The released tables expose opaque SpecialAvatar and role-buff IDs but do not publish per-avatar temporary Trace, Light Cone or Relic loadouts.",
  "Replace per-avatar Unspecified fields when released mapping-info rows or reproducible inspection identifies the exact temporary build.",
);
const allAvatarIds = [...new Set([
  ...mappingByAvatar.keys(),
  ...roleByAvatar.keys(),
])].sort();
const buildRows = allAvatarIds.map((avatarId) => {
  const mapping = mappingByAvatar.get(avatarId);
  const role = roleByAvatar.get(avatarId);
  const roleBuff = role
    ? mazeBuffById.get(String(role.row.BuffID))
    : undefined;
  if (role && !roleBuff)
    throw new Error(`role buff ${role.row.BuffID} does not resolve`);
  const avatar = avatarById.get(avatarId);
  const name = avatarName(avatarId);
  const exactRefs = [
    ...(mapping ? [context.sourceRef(mapping)] : []),
    ...(role ? [context.sourceRef(role)] : []),
    ...(roleBuff ? [context.sourceRef(roleBuff)] : []),
    ...(avatar ? [context.sourceRef(avatar)] : []),
  ];
  const publicIdentity = Boolean(avatar);
  return {
    ...context.envelope({
      id: `divergent-universe.mapping-build.${avatarId}`,
      kind: "DivergentUniverseArithmeticMappingBuild",
      nameEn: `${name.en} Temporary Mapping Build`,
      nameZh: `${name.zh} 临时映射构筑`,
      summaryEn: publicIdentity
        ? `${name.en} binds released special-avatar and role-buff locators where present; exact temporary equipment remains unpublished.`
        : `Source locator ${avatarId} has no released AvatarConfig identity and is retained without a character claim.`,
      summaryZh: publicIdentity
        ? `${name.zh} 绑定已发布的特殊角色与角色增益定位符（若存在）；精确临时装备尚未公开。`
        : `源定位符 ${avatarId} 没有已发布 AvatarConfig 身份，仅保留定位记录而不声明角色。`,
      coverageState: publicIdentity ? "DataReady" : "Cataloged",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [...exactRefs, policyRef],
      tags: [
        "arithmetic-mapping",
        "temporary-build",
        ...(publicIdentity ? [] : ["unresolved-public-identity"]),
      ],
    }),
    avatar_id: avatarId,
    public_identity_resolution: publicIdentity
      ? "ResolvedAvatarConfig"
      : "MissingReleasedAvatarConfig",
    eligible_catalog_entry: eligibilityByAvatar.has(avatarId),
    special_avatar_id: mapping ? String(mapping.row.SpecialAvatarID) : "",
    role_buff_id: role ? String(role.row.BuffID) : "",
    role_buff_binding_key: roleBuff?.row.InBattleBindingKey ?? "",
    role_buff_modifier_name: roleBuff?.row.ModifierName ?? "",
    role_buff_parameters: (roleBuff?.row.ParamList ?? []).map(decimal),
    level: "EquilibriumLevelCapWhenBelow",
    trace_state: "ActivateOrRaiseWhenInactiveOrBelowRequirement",
    light_cone: "UnspecifiedConditionAndTemporaryIdentity",
    relics: "ReplaceWhenTotalEnhancementBelowRequirement",
    exact_temporary_loadout: "Unspecified",
    runtime_lowered: false,
  };
});
outputs.set("arithmetic-mapping-builds.json", ordered(buildRows));

const overviewHash = "3050660410227566581";
const publicRefs = [
  context.sourceRef(
    context.textEntry(overviewHash, "en"),
    "ExactPublicText",
  ),
  context.sourceRef(
    context.textEntry(overviewHash, "zh_cn"),
    "ExactPublicText",
  ),
];
const timingPolicy = await context.policyRef(
  "arithmetic-mapping-refresh",
  "Released text defines thresholds and mode-only scope but not evaluation timing when party or equipment changes during a run.",
  "Replace when released flow/config evidence or reproducible observations establish refresh checkpoints and simultaneous changes.",
);
const rules = [
  rule(
    "scope",
    "Mode-Only Mapping Scope",
    "玩法内映射范围",
    "Arithmetic Mapping enhancements apply only inside Divergent Universe.",
    "数值映射强化仅在差分宇宙内生效。",
    "ModeBoundary",
    "InsideDivergentUniverse",
    "ApplyTemporaryMappingState",
    "ExactPublicText",
    publicRefs,
  ),
  rule(
    "character-level",
    "Character Level Threshold",
    "角色等级阈值",
    "Raise a character below the current Equilibrium Level cap to that cap; preserve a character already at or above it.",
    "角色低于当前均衡等级上限时提升至上限；已达到或超过时保持原值。",
    "RunEntryOrRefresh",
    "CharacterLevelBelowEquilibriumCap",
    "RaiseCharacterToEquilibriumCap",
    "ExactPublicText",
    publicRefs,
  ),
  rule(
    "traces",
    "Trace Activation and Level Threshold",
    "行迹激活与等级阈值",
    "Activate unlocked inactive Traces or raise below-required Trace levels; preserve already sufficient Traces.",
    "激活已解锁但未激活的行迹，或提升低于要求的行迹等级；已满足要求时保持原值。",
    "RunEntryOrRefresh",
    "UnlockedTraceInactiveOrBelowRequirement",
    "ActivateOrRaiseTrace",
    "ExactPublicText",
    publicRefs,
  ),
  rule(
    "relics",
    "Relic Total Enhancement Threshold",
    "遗器总强化等级阈值",
    "Replace equipped Relics with compatible temporary Relics only when total Enhancement Level is below the current requirement.",
    "仅当已装备遗器总强化等级低于当前要求时，替换为适配的临时遗器。",
    "RunEntryOrRefresh",
    "RelicTotalEnhancementBelowRequirement",
    "ReplaceWithCompatibleTemporaryRelics",
    "ExactPublicText",
    publicRefs,
  ),
  rule(
    "light-cone",
    "Light Cone Mapping Boundary",
    "光锥映射边界",
    "Released text includes Light Cone levels in Arithmetic Mapping but does not publish the exact condition or temporary identity.",
    "已发布文本将光锥等级纳入数值映射，但未公布精确条件或临时光锥身份。",
    "Unspecified",
    "Unspecified",
    "Unspecified",
    "ApproximateFromReleasedText",
    [...publicRefs, policyRef],
  ),
  rule(
    "refresh",
    "Mapping Refresh Checkpoint",
    "映射刷新检查点",
    "Reference policy reevaluates mapping at run entry and accepted party changes; hidden mid-run equipment timing remains unavailable.",
    "资料策略在流程进入与已接受的队伍变更时重新评估映射；隐藏的局内装备时机仍不可用。",
    "RunEntryAndAcceptedPartyChange",
    "MappingInputChanged",
    "ReevaluateOnlyBelowThresholdFields",
    "ProjectPolicy",
    [timingPolicy],
  ),
  rule(
    "teardown",
    "Temporary Mapping Teardown",
    "临时映射拆除",
    "At run finalization, remove temporary mapping state without mutating account characters, Light Cones or Relics.",
    "流程结束时移除临时映射状态，不修改账号角色、光锥或遗器。",
    "RunFinalization",
    "LeavingDivergentUniverse",
    "RemoveTemporaryMappingState",
    "ProjectPolicy",
    [...publicRefs, timingPolicy],
  ),
];
outputs.set("arithmetic-mapping-rules.json", ordered(rules));

await writeOrCheck(context, outputs, check);
console.log(
  `Divergent Universe Arithmetic Mapping ${check ? "verified" : "generated"}: ` +
  `${eligibilityRows.length} eligibility, ${buildRows.length} build and ` +
  `${rules.length} lifecycle rule rows.`,
);

function rule(
  id,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  timing,
  condition,
  operation,
  evidenceQuality,
  sourceRefs,
) {
  return {
    ...context.envelope({
      id: `divergent-universe.mapping-rule.${id}`,
      kind: "DivergentUniverseArithmeticMappingRule",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      evidenceQuality,
      sourceRefs,
      tags: ["arithmetic-mapping", slug(evidenceQuality), "rule"],
    }),
    selection_timing: timing,
    condition,
    ordered_operations: [operation],
    stronger_build_rule: "PreserveWhenConditionIsFalse",
    account_mutation: false,
    runtime_lowered: false,
  };
}
