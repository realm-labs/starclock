#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  record,
  source,
  sourceRecordId,
  structuredRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const category = "clock_and_resource_contracts";
const mazeRows = await source("ExcelOutput/ChallengeMazeConfig.json");
const activeStages = mazeRows.filter(({ ID }) => ID >= 5201 && ID <= 5212)
  .sort((left, right) => left.ID - right.ID);
assert(activeStages.length === 12
  && activeStages.every(({ ChallengeCountDown }) => ChallengeCountDown === 30),
"ordinary countdown source drift");
const countdownEvidence = activeStages.map(({ ID }) => structuredRef(
  "ordinary_stages",
  `stage-${ID}`,
  `Exact ChallengeCountDown=30 on active ordinary stage ${ID}.`,
));

function clockPolicy({
  obligation,
  id,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  fields,
  knownFacts,
  selectedBehavior,
  rejectedAlternatives,
  rationale,
  fixture,
  replacementCondition,
  extraEvidence = [],
  evidenceQuality = "ProjectPolicy",
  tags,
}) {
  return record({
    id,
    kind: "ClockRule",
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId(category, obligation)],
    evidence: [
      structuredRef(category, obligation, `Non-shrinking ${obligation} clock obligation.`, "PolicyBoundary"),
      ...extraEvidence,
    ],
    tags,
    fields: {
      evidence_quality: evidenceQuality,
      mechanism_quality: "PolicyBoundary",
      ...fields,
      approximations: [approximation({
        knownFacts,
        selectedBehavior,
        rejectedAlternatives,
        rationale,
        fixtures: [fixture],
        confidence: "Medium",
        replacementCondition,
      })],
    },
  });
}

const records = [
  clockPolicy({
    obligation: "ordinary-cycle-budget",
    id: "clock.ordinary-stage-budget",
    nameEn: "Ordinary stage cycle budget",
    nameZh: "常规关卡轮次预算",
    summaryEn: "Every active ordinary stage declares an initial remaining-cycle value of 30.",
    summaryZh: "全部当前常规关卡均声明30轮初始剩余轮次。",
    fields: {
      value_domain: "RemainingCycles",
      initial_value: 30,
      lower_bound: 0,
      ownership_scope: "OrdinaryStageSectionProjectPolicy",
      reset_on_accepted_stage_start: true,
      tierce_value: "DefinedSeparately",
    },
    knownFacts: "All twelve active ChallengeMazeConfig rows carry ChallengeCountDown=30; the source rows do not encode Activity clock ownership.",
    selectedBehavior: "Create one Section-owned remaining-cycle slot with initial value 30 for each accepted ordinary-stage attempt.",
    rejectedAlternatives: ["independent 30-cycle clocks per node", "a hidden universal mode constant", "inherit 30 for Tierce"],
    rationale: "The normative Memory profile requires stage-owned carry, while keeping the exact value snapshot-authored.",
    fixture: "fixture.cycle-node-wave-carry.stage-owned-budget",
    replacementCondition: "Replace ownership with a released Version 4.4 clock trace or decoded challenge activity program if it differs.",
    extraEvidence: countdownEvidence,
    evidenceQuality: "ExactStructuredValueWithProjectPolicyScope",
    tags: ["countdown", "ordinary", "section-clock"],
  }),
  clockPolicy({
    obligation: "first-cycle-av-window",
    id: "clock.first-cycle-av-window",
    nameEn: "First-cycle Action Value window",
    nameZh: "首轮行动值窗口",
    summaryEn: "Each fresh node battle uses a 150-AV first cycle followed by 100-AV cycles as an explicit reference policy.",
    summaryZh: "每个新节点战斗按明确参考策略使用150行动值首轮，后续轮次为100行动值。",
    fields: {
      first_cycle_action_value: "150",
      later_cycle_action_value: "100",
      authoritative_decimal_encoding: "CanonicalString",
      selection_scope: "FreshNodeBattleTimeline",
      source_selector_found: false,
      parity_claim: false,
    },
    knownFacts: "The challenge architecture records the commonly observed 150/100 cycle preset but requires each profile to select it; no active Version 4.4 row or program in the frozen closure explicitly selects the numeric preset.",
    selectedBehavior: "Select 150 AV for the first cycle of each fresh node battle and 100 AV afterward as a deterministic Candidate reference policy.",
    rejectedAlternatives: ["100 AV for every cycle", "carry a partial AV window across nodes", "leave the battle scheduler without a window"],
    rationale: "The explicit preset is implementation-ready while the parity limitation and stronger-evidence trigger remain visible.",
    fixture: "fixture.cycle-first-av-window.150-then-100",
    replacementCondition: "Replace with a released Version 4.4 StageConfig/clock selector or reproducible action-gauge trace.",
    tags: ["action-value", "first-cycle", "project-policy"],
  }),
  clockPolicy({
    obligation: "cycle-tick-boundary",
    id: "clock.cycle-tick-boundary",
    nameEn: "Cycle tick boundary",
    nameZh: "轮次计数边界",
    summaryEn: "A cycle ticks after its AV window and reactions finish, before the next cycle's start rules.",
    summaryZh: "行动值窗口及其反应处理完成后轮次计数，再执行下一轮开始规则。",
    fields: {
      ordered_boundary: [
        "DrainAcceptedActionAndBoundedReactions",
        "EmitCycleEnded",
        "DecrementRemainingCycles",
        "EvaluateExpiry",
        "InitializeNextCycleActionValue",
        "EmitCycleStarted",
        "ExecuteCycleStartRules",
      ],
      consuming_actions: "SchedulerActionValueOnly",
      interrupts_consume_extra_cycle: false,
    },
    knownFacts: "Memory Turbulence is described and programmed at cycle start; released rows do not encode its order relative to Activity clock decrement and reaction draining.",
    selectedBehavior: "Commit the decrement after prior-cycle reactions drain and before any next-cycle Turbulence callback.",
    rejectedAlternatives: ["decrement during an action", "execute Turbulence before expiry", "let interrupts decrement cycles directly"],
    rationale: "A single typed boundary preserves command/event ordering and prevents expired clocks from producing another cycle-start effect.",
    fixture: "fixture.cycle-node-wave-carry.tick-before-cycle-start",
    replacementCondition: "Replace ordering with a released event trace that identifies clock tick, expiry and OnPhase1 order.",
    tags: ["cycle-end", "cycle-start", "tick"],
  }),
  clockPolicy({
    obligation: "node-cycle-carry",
    id: "clock.node-carry",
    nameEn: "Node-to-node cycle carry",
    nameZh: "节点间轮次携带",
    summaryEn: "Node 2 receives Node 1's finalized remaining-cycle integer but starts a fresh battle timeline.",
    summaryZh: "节点2接收节点1结算后的剩余轮次整数，但创建全新的战斗时间线。",
    fields: {
      from_node_index: 1,
      to_node_index: 2,
      carried_value: "RemainingCyclesAfterNode1Victory",
      carried_partial_action_value: false,
      node2_first_cycle_action_value: "150",
      battle_state_carry: "None",
    },
    knownFacts: "The normative Memory profile owns one stage clock across nodes and creates a fresh battle/timeline for Node 2; released Version 4.4 rows do not expose partial-window carry.",
    selectedBehavior: "Carry only the finalized integer remaining-cycle slot, then initialize Node 2 with the explicit first-cycle AV window.",
    rejectedAlternatives: ["reset Node 2 to 30", "carry Node 1 partial AV", "carry Node 1 battle state"],
    rationale: "Integer-only Activity projection keeps battle timelines independent and makes the unresolved client boundary explicit.",
    fixture: "fixture.cycle-node-wave-carry.node2-fresh-window",
    replacementCondition: "Replace partial-window behavior with a released cross-node gauge/countdown trace.",
    tags: ["carry", "node-1", "node-2"],
  }),
  clockPolicy({
    obligation: "wave-cycle-carry",
    id: "clock.wave-carry",
    nameEn: "Wave cycle carry",
    nameZh: "波次间轮次携带",
    summaryEn: "A wave transition preserves the current remaining cycles and current cycle AV position.",
    summaryZh: "波次转移保持当前剩余轮次和本轮行动值位置。",
    fields: {
      reset_remaining_cycles_on_wave: false,
      reset_cycle_action_value_on_wave: false,
      transition_order: "AfterActionAfterBoundedReactions",
      wave_start_rules_after_transition: true,
    },
    knownFacts: "All selected StageConfig rows declare two waves; the generic wave lifecycle preserves surviving allied state unless authored otherwise, and no selected row declares a clock reset.",
    selectedBehavior: "Carry both the Section cycle value and current per-battle AV position through an ordinary wave transition.",
    rejectedAlternatives: ["grant a new 150-AV cycle per wave", "decrement once solely because a wave ended", "reset the stage budget"],
    rationale: "Fail-closed no-reset semantics avoid inventing time and match the absence of a selected reset operation.",
    fixture: "fixture.cycle-node-wave-carry.wave-no-reset",
    replacementCondition: "Replace with a released StageCommonTemplate branch or wave transition trace selecting a reset.",
    tags: ["carry", "wave", "no-reset"],
  }),
  clockPolicy({
    obligation: "expiry-failure-order",
    id: "clock.expiry-failure",
    nameEn: "Clock expiry and failure",
    nameZh: "轮次耗尽与失败",
    summaryEn: "Reaching zero after a cycle tick fails the attempt before another cycle-start rule or transition can run.",
    summaryZh: "轮次计数后到达零时，在下一轮开始规则或转移执行前判定尝试失败。",
    fields: {
      expiry_predicate: "RemainingCyclesEqualsZeroAfterTick",
      expiry_order: [
        "FinalizePreviousCycleEvents",
        "EmitClockExpired",
        "EmitNodeFailed",
        "CloseStageAttempt",
        "TeardownBattle",
      ],
      allow_cycle_start_effect_after_expiry: false,
      allow_wave_or_node_transition_after_expiry: false,
      retry_reset_value: 30,
    },
    knownFacts: "ChallengeCountDown is 30 and the challenge design states that a failed node terminates the attempt; the source closure does not expose the exact zero-boundary callback order.",
    selectedBehavior: "Expire on the tick that produces zero, before cycle-start effects, wave transitions or Node 2 entry.",
    rejectedAlternatives: ["allow a full zero-labelled cycle", "run Turbulence after expiry", "transition nodes and then fail"],
    rationale: "Fail-before-new-work ordering is deterministic and prevents post-expiry mutations while remaining replaceable by stronger traces.",
    fixture: "fixture.cycle-node-wave-carry.expiry-before-start-rules",
    replacementCondition: "Replace with a released zero-cycle trace including the last action, reactions, Turbulence and failure UI boundary.",
    tags: ["expiry", "failure", "zero"],
  }),
];

const expected = [
  "cycle-tick-boundary",
  "expiry-failure-order",
  "first-cycle-av-window",
  "node-cycle-carry",
  "ordinary-cycle-budget",
  "wave-cycle-carry",
].map((id) => sourceRecordId(category, id)).sort();
const claimed = records.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
assert(claimed.length === new Set(claimed).size, "clock obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected), "clock obligation coverage drift");
const output = normalizedFile("clock-rules.json", "ClockRule", records);
await writeCanonical("clock-rules.json", output, check);
const clockDigest = digest(output);
await writeText(
  "evidence/memory-of-chaos-reference-v1/clock-audit.md",
  `# Goal 17 clock audit

- Clock obligations: 6/6, each claimed exactly once
- Exact ordinary countdown values: 12/12 rows declare 30
- Ordinary ownership: one Section clock (ProjectPolicy)
- First/later cycle Action Value: 150 / 100 (ProjectPolicy; no active selector found)
- Node carry: integer remaining cycles only; Node 2 starts a fresh timeline
- Wave carry: no cycle or AV reset
- Expiry: zero after tick fails before cycle-start effects
- Tierce clock composition: deferred and not inherited
- Normalized clock digest: \`${clockDigest}\`
- Runtime executable rows: 0

Only the per-stage value 30 is ExactStructured. Ownership, AV windows,
partial-window carry and boundary ordering are explicit Candidate reference
policies with rejected alternatives, fixture IDs and replacement conditions.
`,
  check,
);
console.log(`Goal 17 clocks ${check ? "verified" : "generated"}: 6/6 obligations, 12 exact countdown rows.`);
