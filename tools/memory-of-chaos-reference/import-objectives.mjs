#!/usr/bin/env node

import {
  approximation,
  assert,
  assertSource,
  digest,
  normalizedFile,
  publicRef,
  record,
  source,
  sourceRecordId,
  structuredRef,
  textRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const [targets, textZh, textEn] = await Promise.all([
  source("ExcelOutput/ChallengeTargetConfig.json"),
  source("TextMap/TextMapCHS.json"),
  source("TextMap/TextMapEN.json"),
]);
const ids = [251, 252, 253, 601, 602, 603];
const selected = targets.filter(({ ID }) => ids.includes(ID))
  .sort((left, right) => left.ID - right.ID);
assert(selected.length === 6, "objective source closure drift");
const stageGuide = publicRef(
  "hoyolab:19664810:stage-stars",
  "https://www.hoyolab.com/article/19664810",
  "So How Does it Work",
  "Released public guide evidence describes up to three stars per stage: no downed characters plus two remaining-cycle conditions.",
  "IdentityCrossCheck",
  "1.1 stable-family cross-check",
);
const cumulativeGuide = publicRef(
  "hoyolab:18067001:cumulative-objectives",
  "https://www.hoyolab.com/article/18067001",
  "Challenge Scoring",
  "Released public guide evidence states that Forgotten Hall stage objectives can be achieved cumulatively across multiple attempts.",
  "IdentityCrossCheck",
  "1.0 stable-family cross-check",
);

function translated(hash) {
  const zh = textZh[String(hash)];
  const en = textEn[String(hash)];
  assert(typeof zh === "string" && typeof en === "string", `missing target text ${hash}`);
  return { hash: String(hash), zh, en };
}

const objectiveRecords = selected.map((target) => {
  const ordinary = target.ID < 600;
  const survival = target.ChallengeTargetType === "DEAD_AVATAR";
  const name = translated(target.ChallengeTargetName.Hash);
  const threshold = target.ChallengeTargetParam1 ?? 0;
  const manifestId = `target-${target.ID}`;
  const domain = ordinary ? "OrdinaryStage" : "TierceExtension";
  const conditionEn = survival
    ? "complete without a downed allied combat form"
    : `complete with at least ${threshold} remaining cycles`;
  const conditionZh = survival
    ? "通关且没有我方战斗形态倒下"
    : `通关时至少剩余${threshold}轮`;
  return record({
    id: `objective.${target.ID}`,
    kind: "Objective",
    nameEn: survival ? "No downed characters" : `At least ${threshold} cycles remaining`,
    nameZh: survival ? "无角色倒下" : `至少剩余${threshold}轮`,
    summaryEn: `One-star ${domain} objective: ${conditionEn}.`,
    summaryZh: `${domain === "OrdinaryStage" ? "常规关卡" : "扩展关"}一星目标：${conditionZh}。`,
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("objectives", manifestId)],
    evidence: [
      structuredRef("objectives", manifestId, "Exact target type, parameter and active selector membership."),
      textRef("zh_cn", name.hash, name.zh),
      textRef("en", name.hash, name.en),
      stageGuide,
      cumulativeGuide,
    ],
    tags: [ordinary ? "ordinary" : "tierce", survival ? "survival" : "remaining-cycles", "star"],
    fields: {
      upstream_target_id: target.ID,
      applies_to: domain,
      target_type: survival ? "NoDownedCombatForm" : "RemainingCyclesAtLeast",
      threshold: survival ? 0 : threshold,
      threshold_encoding: survival ? "OmittedSourceFieldDefaultsToZero" : "ExplicitSourceInteger",
      completion_required: true,
      evaluation_boundary: "AfterAllRequiredNodeVictoriesBeforeStageSettlementProjectPolicy",
      survival_scope: survival ? `${domain}AllRequiredBattlesProjectPolicy` : null,
      contribution_stars: 1,
      aggregation_scope: `${domain}BestObjectiveRecordWithinSeasonProjectPolicy`,
      cumulative_across_completed_attempts: true,
      failed_or_abandoned_attempt_can_satisfy: false,
      account_reward_payload_included: false,
      evidence_quality: "ExactStructuredThresholdWithProjectPolicyTiming",
      mechanism_quality: "PolicyBoundary",
      approximations: [approximation({
        knownFacts: `Target ${target.ID} exactly selects ${target.ChallengeTargetType}${survival ? "" : ` with parameter ${threshold}`}; stable-family public guidance identifies survival/cycle stars and cumulative objective progress.`,
        selectedBehavior: `Evaluate ${conditionEn} only after ${domain} completion, then latch this one-star objective independently into the season-stage best record.`,
        rejectedAlternatives: [
          "evaluate on an incomplete or failed attempt",
          "require all three objectives in one attempt",
          "sum duplicate completions of the same objective",
        ],
        rationale: "Independent monotonic objective receipts preserve cumulative progress without importing account reward payloads.",
        fixtures: ["fixture.objective-star-aggregation.cumulative-independent-objectives"],
        confidence: "Medium",
        replacementCondition: "Replace timing, survival scope or cumulative behavior with a released Version 4.4 objective settlement trace.",
      })],
      source_display_text: { en: name.en, zh_cn: name.zh },
    },
  });
});

const claimed = objectiveRecords.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = ids.map((id) => sourceRecordId("objectives", `target-${id}`)).sort();
assert(claimed.length === new Set(claimed).size, "objective obligations must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected), "objective obligation coverage drift");
const output = normalizedFile("objectives.json", "Objective", objectiveRecords);
await writeCanonical("objectives.json", output, check);
const objectiveDigest = digest(output);
await writeText(
  "evidence/memory-of-chaos-reference-v1/objective-audit.md",
  `# Goal 17 objective and star audit

- Objective obligations: 6/6, each claimed exactly once
- Ordinary thresholds: remaining cycles >=10, >=20; no downed combat form
- Tierce thresholds: remaining cycles >=15, >=30; no downed combat form
- Star contribution: one per independently satisfied objective
- Aggregation: monotonic best objective receipts across completed attempts (ProjectPolicy)
- Failed/abandoned attempt contribution: none
- Reward payloads: excluded
- Normalized objective digest: \`${objectiveDigest}\`
- Runtime executable rows: 0

IDs, target kinds and numeric thresholds are ExactStructured. Evaluation timing,
cross-node survival scope and cumulative best-record aggregation use stable-family
public cross-checks plus explicit Version 4.4 ProjectPolicy replacement conditions.
`,
  check,
);
console.log(`Goal 17 objectives ${check ? "verified" : "generated"}: 6/6 targets, ordinary 10/20 and Tierce 15/30.`);
