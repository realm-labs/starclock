#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
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

const INITIAL_HP_HASH = "1912261755023964838";
const SQUAD_HP_RULE_HASH = "6971100623138337968";
const GAMEPLAY_RULES_HASH = "7693488975416237801";
const UNLIMITED_NODE_HASH = "5626677263404827289";
const FINAL_BOSS_RECOVERY_HASH = "4983101780975847570";
const NODE_COUNTDOWN_MODIFIER_HASH = "7940111314490605947";

const maximumHpPolicy = await context.policyRef(
  "squad-hp-initial-maximum",
  "Treat the released initial 100 Squad HP as both current and maximum HP until a released initialization field distinguishes them.",
  "Replace when a released initialization record or reproducible observation independently publishes initial maximum Squad HP.",
);
const actionValuePolicy = await context.policyRef(
  "action-value-node-configuration",
  "Finite initial action value and non-victory Squad HP loss are node/difficulty-authored values; no global number is invented.",
  "Replace each placeholder with an exact reachable node, difficulty, StageConfig or ability-program field.",
);
const projectionPolicy = await context.policyRef(
  "battle-result-action-value-projection",
  "Remaining node action value is captured for battle-finalization contributions and then discarded rather than carried as a run resource.",
  "Replace when released state/config evidence publishes the exact action-value finalization and carry boundary.",
);
const sameBoundaryPolicy = await context.policyRef(
  "victory-timeout-same-boundary-order",
  "When the last enemy is defeated on the same resolution boundary that exhausts the limit, determine victory before applying timeout loss; then project Squad HP and test zero.",
  "Replace when a released ability/state program or reproducible observation proves the same-boundary precedence.",
);

const squadHpRules = [{
  ...context.envelope({
    id: "currency-wars.squad-hp.global",
    kind: "CurrencyWarsSquadHpRules",
    nameEn: "Squad HP",
    nameZh: "小队生命值",
    summaryEn:
      "A match starts with 100 Squad HP; victory preserves it, non-victory applies configured loss, and zero ends the match.",
    summaryZh:
      "对局以 100 点小队生命值开始；胜利时保持不变，未胜利时扣除配置值，归零则结束对局。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...context.bilingualTextRefs(INITIAL_HP_HASH),
      ...context.bilingualTextRefs(SQUAD_HP_RULE_HASH),
      ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
      ...context.bilingualTextRefs(FINAL_BOSS_RECOVERY_HASH),
      maximumHpPolicy,
      actionValuePolicy,
    ],
    tags: ["activity-state", "project-policy", "squad-hp"],
  }),
  initial_hp: "100",
  minimum_hp: "0",
  maximum_hp: {
    initial_value: "100",
    mutation: "ContentDefinedIncreaseOrRecovery",
    resolution: "ProjectPolicy",
  },
  loss_rules: [
    {
      trigger: "NodeNonVictory",
      amount: "ConfiguredByNodeOrDifficulty",
      operation: "SubtractThenClampToMinimum",
      resolution: "ExactTriggerPolicyBoundAmount",
    },
  ],
  recovery_rules: [
    {
      trigger: "NodeVictory",
      amount: "0",
      operation: "PreserveSquadHp",
      resolution: "ExactPublicText",
    },
    {
      trigger: "ContentContribution",
      amount: "ConfiguredByContent",
      operation: "RestoreOrIncreaseMaximumAsAuthored",
      resolution: "DeferredToOwningContentBatch",
    },
  ],
  runtime_lowered: false,
}];
outputs.set("squad-hp-rules.json", squadHpRules);

const actionValueLimits = [
  {
    ...context.envelope({
      id: "currency-wars.action-value.finite-node",
      kind: "CurrencyWarsActionValueLimits",
      nameEn: "Finite Node Action Value",
      nameZh: "有限节点行动值",
      summaryEn:
        "A finite combat Node must defeat all enemies before its configured action-value limit expires.",
      summaryZh:
        "有限战斗节点必须在配置的行动值耗尽前消灭全部敌人。",
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
        ...context.bilingualTextRefs(NODE_COUNTDOWN_MODIFIER_HASH),
        actionValuePolicy,
      ],
      tags: ["action-value", "finite", "project-policy"],
    }),
    limit_kind: "FiniteNodeConfigured",
    initial_value: "ConfiguredByNodeOrDifficulty",
    decrement_rules: [
      {
        trigger: "CombatTimelineProgress",
        amount: "ElapsedAuthoritativeActionValue",
        resolution: "ProjectPolicy",
      },
      {
        trigger: "CharacterLethalRescue",
        amount: "ConfiguredByBattleContribution",
        resolution: "DeferredToP1B4",
      },
    ],
    timeout_boundary: {
      condition: "LimitExhaustedBeforeAllEnemiesDefeated",
      battle_outcome: "NonVictory",
      squad_hp_projection:
        "ApplyConfiguredNodeOrDifficultyLoss",
    },
    runtime_lowered: false,
  },
  {
    ...context.envelope({
      id: "currency-wars.action-value.unlimited-low-difficulty",
      kind: "CurrencyWarsActionValueLimits",
      nameEn: "Unlimited Low-Difficulty Node",
      nameZh: "无限行动值低难节点",
      summaryEn:
        "The released low-difficulty reward Node has unlimited action value and cannot time out through this limit.",
      summaryZh:
        "已发布的低难奖励节点拥有无限行动值，不会因该限制而超时。",
      evidenceQuality: "ExactPublicText",
      sourceRefs: context.bilingualTextRefs(UNLIMITED_NODE_HASH),
      tags: ["action-value", "low-difficulty", "unlimited"],
    }),
    limit_kind: "Unlimited",
    initial_value: "Infinite",
    decrement_rules: [],
    timeout_boundary: {
      condition: "UnreachableForActionValueLimit",
      battle_outcome: "NotApplicable",
      squad_hp_projection: "None",
    },
    runtime_lowered: false,
  },
];
outputs.set("action-value-limits.json", actionValueLimits);

const battleResultProjections = [
  {
    ...context.envelope({
      id: "currency-wars.battle-result.victory",
      kind: "CurrencyWarsBattleResultProjections",
      nameEn: "Node Victory Projection",
      nameZh: "节点胜利投影",
      summaryEn:
        "Victory at a Battle, Encounter or Boss Node preserves Squad HP and continues or completes the run.",
      summaryZh:
        "战斗、遭遇或首领节点胜利时保持小队生命值，并继续或完成对局。",
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...context.bilingualTextRefs(SQUAD_HP_RULE_HASH),
        projectionPolicy,
      ],
      tags: ["battle-result", "project-policy", "victory"],
    }),
    battle_outcome: "Victory",
    squad_hp_projection: "PreserveBeforeContentContributions",
    action_value_projection: "CaptureForFinalizationThenDiscard",
    run_disposition: "ContinueUnlessFinalBossCompletesRun",
    runtime_lowered: false,
  },
  {
    ...context.envelope({
      id: "currency-wars.battle-result.non-victory",
      kind: "CurrencyWarsBattleResultProjections",
      nameEn: "Node Non-Victory Projection",
      nameZh: "节点未胜利投影",
      summaryEn:
        "A finite Node timeout is non-victory: apply its configured Squad HP loss, then continue only if HP remains above zero.",
      summaryZh:
        "有限节点超时属于未胜利：先扣除配置的小队生命值，仅在生命值仍高于零时继续。",
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...context.bilingualTextRefs(SQUAD_HP_RULE_HASH),
        ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
        actionValuePolicy,
        projectionPolicy,
      ],
      tags: ["battle-result", "non-victory", "project-policy"],
    }),
    battle_outcome: "NonVictory",
    squad_hp_projection:
      "SubtractConfiguredLossClampToZeroThenEvaluateRun",
    action_value_projection: "CaptureExhaustedLimitThenDiscard",
    run_disposition: "FailAtZeroOtherwiseContinue",
    runtime_lowered: false,
  },
];
outputs.set("battle-result-projections.json", battleResultProjections);

const runFailureRules = [{
  ...context.envelope({
    id: "currency-wars.run-failure.squad-hp-zero",
    kind: "CurrencyWarsRunFailureRules",
    nameEn: "Zero Squad HP Run Failure",
    nameZh: "小队生命值归零对局失败",
    summaryEn:
      "After battle projection, zero Squad HP ends the match; same-boundary victory precedence is an explicit replaceable policy.",
    summaryZh:
      "战斗投影后，小队生命值归零会结束对局；同边界胜利优先级是明确且可替换的项目策略。",
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [
      ...context.bilingualTextRefs(SQUAD_HP_RULE_HASH),
      ...context.bilingualTextRefs(GAMEPLAY_RULES_HASH),
      sameBoundaryPolicy,
    ],
    tags: ["project-policy", "run-failure", "same-boundary"],
  }),
  failure_condition: "ProjectedSquadHpEqualsZero",
  same_boundary_order: [
    "DetermineVictoryBeforeTimeoutLoss",
    "ProjectVictoryOrConfiguredNonVictoryLoss",
    "ClampSquadHpToZero",
    "FailRunAtZeroOtherwiseContinue",
  ],
  terminal_disposition: "FailedRun",
  alternatives_rejected_pending_evidence: [
    "TimeoutPrecedesLastEnemyDefeat",
    "RunFailureCheckPrecedesBattleResultProjection",
  ],
  runtime_lowered: false,
}];
outputs.set("run-failure-rules.json", runFailureRules);

await writeOrCheck(context, outputs, check);
console.log(
  `Currency Wars Squad boundary ${check ? "verified" : "generated"}: ` +
  `${squadHpRules.length} Squad HP rule, ` +
  `${actionValueLimits.length} action-value limits, ` +
  `${battleResultProjections.length} battle-result projections and ` +
  `${runFailureRules.length} run-failure rule.`,
);
