#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  publicRef,
  record,
  sourceRecordId,
  structuredRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const category = "participant_and_attempt_contracts";
const guide = publicRef(
  "hoyolab:19664810:two-teams",
  "https://www.hoyolab.com/article/19664810",
  "How is it Unlocked / So How Does it Work",
  "Released public guide evidence identifies Memory of Chaos as the harder Forgotten Hall branch and states that two teams are required.",
  "IdentityCrossCheck",
  "1.1 stable-family cross-check",
);

function resolvedPolicy({
  obligation,
  id,
  kind,
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
  tags,
}) {
  return record({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId(category, obligation)],
    evidence: [
      structuredRef(category, obligation, `Non-shrinking ${obligation} semantic obligation.`, "PolicyBoundary"),
      ...extraEvidence,
    ],
    tags,
    fields: {
      ...fields,
      evidence_quality: extraEvidence.length > 0 ? "ExactPublicTextAndProjectPolicy" : "ProjectPolicy",
      mechanism_quality: "PolicyBoundary",
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

const participantRecords = [
  resolvedPolicy({
    obligation: "participant-policy",
    id: "participant-policy.ordinary-stage",
    kind: "ParticipantPolicy",
    nameEn: "Ordinary stage participants",
    nameZh: "常规关卡参战策略",
    summaryEn: "Two disjoint team slots are selected before an ordinary stage attempt begins.",
    summaryZh: "常规关卡尝试开始前选择两支互不重叠的队伍。",
    fields: {
      scope: "OrdinaryStageAttempt",
      selection_boundary: "BeforeAcceptedStageStart",
      team_slot_ids: ["team-slot.ordinary.1", "team-slot.ordinary.2"],
      uniqueness_scope: "Section",
      substitutions_after_start: "Rejected",
      tierce_scope: "UnresolvedNotInherited",
    },
    knownFacts: "Every active ordinary stage declares StageNum=2 and two node selector families; released public guidance says Memory of Chaos requires two teams.",
    selectedBehavior: "Capture two disjoint teams atomically before accepting Node 1; do not project this ordinary-stage policy onto Tierce.",
    rejectedAlternatives: ["select Node 2 team after Node 1", "reuse one combat form across both slots", "inherit ordinary policy for Tierce"],
    rationale: "The generic challenge design fixes a stage configuration before Node 1 and section-scoped uniqueness prevents ambiguous ownership.",
    fixture: "fixture.participant-uniqueness.accept-two-disjoint-teams",
    replacementCondition: "Replace policy fields with a released lineup transaction trace or decoded participant program.",
    extraEvidence: [guide],
    tags: ["ordinary", "participant", "section-scope"],
  }),
  resolvedPolicy({
    obligation: "ordinary-team-slots",
    id: "team-slots.ordinary",
    kind: "TeamSlotPolicy",
    nameEn: "Two ordinary team slots",
    nameZh: "两支常规队伍槽位",
    summaryEn: "Node 1 and Node 2 each bind one distinct ordered team slot.",
    summaryZh: "节点1与节点2各绑定一个不同的有序队伍槽位。",
    fields: {
      slots: [
        { id: "team-slot.ordinary.1", node_index: 1, min_members: 1, max_members: 4 },
        { id: "team-slot.ordinary.2", node_index: 2, min_members: 1, max_members: 4 },
      ],
      order_semantics: "NodeIndexOrder",
      cross_slot_overlap: "Rejected",
    },
    knownFacts: "The active rows contain two ordered node selector sets, and released guidance requires two teams.",
    selectedBehavior: "Bind exactly one 1-to-4-member team to each node index.",
    rejectedAlternatives: ["one shared team", "unordered team selection", "more than four deployed members"],
    rationale: "This is the smallest deterministic projection of two ordered challenge nodes onto generic team slots.",
    fixture: "fixture.participant-uniqueness.team-slot-cardinality",
    replacementCondition: "Replace cardinality and ordering with a released lineup schema if it differs.",
    extraEvidence: [guide],
    tags: ["node-1", "node-2", "team-slot"],
  }),
  resolvedPolicy({
    obligation: "combat-form-uniqueness",
    id: "uniqueness.combat-form",
    kind: "UniquenessPolicy",
    nameEn: "Combat-form uniqueness",
    nameZh: "战斗形态唯一性",
    summaryEn: "A combat form may occupy only one ordinary-stage team slot in an accepted attempt.",
    summaryZh: "同一战斗形态在一次已接受的常规关卡尝试中只能占用一个队伍槽位。",
    fields: {
      identity_key: "ResolvedCombatFormId",
      scope: "OrdinaryStageSection",
      duplicate_result: "RejectStartByteIdentical",
      base_character_aliasing: "Unavailable",
    },
    knownFacts: "Two different teams are required; released rows do not expose the client uniqueness-key implementation.",
    selectedBehavior: "Reject a repeated resolved combat-form identity across the two team slots before mutation.",
    rejectedAlternatives: ["allow duplicates", "deduplicate silently", "key only by display character name"],
    rationale: "Resolved combat-form identity is the lowest stable project identity and preserves rejected-command atomicity.",
    fixture: "fixture.participant-uniqueness.reject-duplicate-form",
    replacementCondition: "Replace the identity key with a released lineup validator trace that distinguishes base character and alternate forms.",
    tags: ["combat-form", "rejected-start", "uniqueness"],
  }),
  resolvedPolicy({
    obligation: "loadout-instance-lock",
    id: "loadout-lock.ordinary-stage",
    kind: "LoadoutLockPolicy",
    nameEn: "Stage loadout lock",
    nameZh: "关卡配装锁定",
    summaryEn: "Accepted ordinary-stage attempts freeze resolved forms, Light Cones and concrete Relic instances for both nodes.",
    summaryZh: "常规关卡尝试被接受时，冻结两节点的已解析形态、光锥和具体遗器实例。",
    fields: {
      capture_boundary: "AcceptedStageStartBeforeNode1",
      locked_components: ["CombatForm", "LightConeInstance", "RelicInstanceSet"],
      mutation_after_start: "Rejected",
      snapshot_identity: "CanonicalResolvedLoadoutDigest",
      account_inventory_mutation: "OutsideReferenceScope",
    },
    knownFacts: "The normative challenge design fixes loadouts and the configuration digest before Node 1; released Memory rows do not encode equipment-instance mutation behavior.",
    selectedBehavior: "Capture immutable resolved loadout digests for both teams at accepted stage start and reject in-attempt substitution.",
    rejectedAlternatives: ["read live account equipment during Node 2", "allow same Relic instance in both teams", "silently refresh snapshots"],
    rationale: "Immutable loadouts make replay/config identity stable and keep combat independent from account inventory.",
    fixture: "fixture.loadout-lock-retry.reject-after-start-mutation",
    replacementCondition: "Replace with released equipment lock/retry traces and instance ownership semantics.",
    tags: ["light-cone", "loadout", "relic", "snapshot"],
  }),
];

const attemptRecords = [
  resolvedPolicy({
    obligation: "attempt-retry-reset",
    id: "attempt-rule.retry-reset",
    kind: "AttemptRule",
    nameEn: "Attempt, retry and reset",
    nameZh: "尝试、重试与重置",
    summaryEn: "An accepted stage start creates one attempt; failure or abandonment closes it without partial completion.",
    summaryZh: "已接受的关卡开始命令创建一次尝试；失败或放弃会结束尝试且不产生部分通关。",
    fields: {
      accepted_start_effect: "CreateAttemptAndImmutableSnapshots",
      rejected_start_effect: "ByteIdentical",
      failure_effect: "CloseAttemptWithAuditResult",
      abandonment_effect: "CloseAttemptNoCompletion",
      retry_granularity: "WholeOrdinaryStageAttemptProjectPolicy",
      retry_snapshot_source: "FreshAcceptedStart",
      previous_partial_node_results: "Discarded",
    },
    knownFacts: "The challenge design requires an attempt boundary and states that a failed node terminates the attempt; released rows do not expose current client retry shortcuts.",
    selectedBehavior: "Model retry as a new whole-stage attempt with fresh accepted-start snapshots and no partial completion reuse.",
    rejectedAlternatives: ["resume a failed mutable battle", "publish Node 1 as stage completion", "reuse live mutable loadouts"],
    rationale: "This fail-closed policy avoids claiming single-node retry parity until released traces prove the shortcut and its snapshot rules.",
    fixture: "fixture.loadout-lock-retry.new-attempt-after-failure",
    replacementCondition: "Replace retry granularity with a released Version 4.4 retry trace, especially any Node 2-only restart behavior.",
    tags: ["attempt", "failure", "retry"],
  }),
  resolvedPolicy({
    obligation: "node-transition-lock",
    id: "attempt-rule.node-transition",
    kind: "TransitionRule",
    nameEn: "Ordinary node transition",
    nameZh: "常规节点转移",
    summaryEn: "Node 2 may start only after Node 1 victory and uses its pre-locked second team in a fresh battle state.",
    summaryZh: "仅节点1胜利后可进入节点2，并以预先锁定的第二队创建全新战斗状态。",
    fields: {
      from_node_index: 1,
      accepted_result: "Victory",
      to_node_index: 2,
      participant_slot: "team-slot.ordinary.2",
      loadout_mutation_between_nodes: "Rejected",
      battle_state_carry: "None",
      section_slot_carry: ["ClockProjectionDeferredToClockRules", "ObjectiveAuditState"],
    },
    knownFacts: "Active stages select two ordered nodes; the challenge design declares fresh per-node battle state and pre-start loadout locking.",
    selectedBehavior: "Require Node 1 victory, preserve only typed section projections, and instantiate Node 2 from the locked second-team snapshot.",
    rejectedAlternatives: ["enter Node 2 after failure", "carry HP/Energy/effects between unrelated teams", "edit team 2 between nodes"],
    rationale: "Typed transition projections preserve battle/activity ownership and deterministic replay.",
    fixture: "fixture.loadout-lock-retry.node-transition-lock",
    replacementCondition: "Replace with a released transition trace if Version 4.4 permits lineup mutation or additional carry slots.",
    tags: ["node-1", "node-2", "transition"],
  }),
];

const outputs = [
  ["participant-policies.json", normalizedFile("participant-policies.json", "ParticipantPolicy", participantRecords)],
  ["attempt-rules.json", normalizedFile("attempt-rules.json", "AttemptOrTransitionRule", attemptRecords)],
];
const claimed = outputs.flatMap(([, value]) => value.records)
  .flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = [
  "attempt-retry-reset",
  "combat-form-uniqueness",
  "loadout-instance-lock",
  "node-transition-lock",
  "ordinary-team-slots",
  "participant-policy",
].map((id) => sourceRecordId(category, id)).sort();
assert(claimed.length === new Set(claimed).size,
  "participant/attempt obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected),
  "participant/attempt obligation coverage drift");
for (const [file, value] of outputs) await writeCanonical(file, value, check);
const participantDigest = digest(outputs.map(([file, value]) => ({ file, value })));
await writeText(
  "evidence/memory-of-chaos-reference-v1/participant-attempt-audit.md",
  `# Goal 17 participant and attempt audit

- Frozen obligations: 6/6, each claimed exactly once
- Ordinary team slots: 2
- Uniqueness scope: ordinary stage Section
- Locked loadout components: combat form, Light Cone instance, Relic instance set
- Rejected start mutation: byte-identical
- Tierce participant semantics: unresolved and not inherited
- Retry granularity: explicit whole-stage ProjectPolicy, not an observed parity claim
- Normalized participant/attempt digest: \`${participantDigest}\`
- Runtime executable rows: 0

Released public evidence supports the two-team boundary. Instance uniqueness,
snapshot timing, retry granularity and transition atomicity remain explicit
project policies with replacement conditions and semantic fixture IDs.
`,
  check,
);
console.log(`Goal 17 participants ${check ? "verified" : "generated"}: 6/6 obligations, two ordinary team slots.`);
