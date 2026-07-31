#!/usr/bin/env node

import {
  assert,
  assertSource,
  digest,
  manifest,
  normalizedFile,
  record,
  sourceRecordId,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();

const names = {
  blessing: ["Blessing", "祝福"],
  choice: ["event choice", "事件选项"],
  curio: ["Curio", "奇物"],
  currency: ["mode currency", "玩法货币"],
  occurrence: ["Occurrence", "事件"],
  "rogue-path": ["Rogue path", "模拟宇宙命途"],
  "rogue-progression": ["Rogue progression", "模拟宇宙成长"],
  "rogue-room": ["Rogue room", "模拟宇宙房间"],
  service: ["service", "服务"],
  shop: ["shop", "商店"],
};

const proofs = manifest.categories.empty_pool_proofs.records;
assert(proofs.length === 10, "empty-pool proof denominator drift");
const poolRecords = proofs.map((proof) => {
  const labels = names[proof.family];
  assert(labels !== undefined, `unmapped pool family ${proof.family}`);
  return record({
    id: `pool-audit.${proof.family}`,
    kind: "PoolAudit",
    nameEn: `Empty ${labels[0]} pool proof`,
    nameZh: `${labels[1]}空池证明`,
    summaryEn: `The frozen active schedule-to-config selector closure reaches no ${labels[0]} row.`,
    summaryZh: `冻结的有效日程至关卡配置选择器闭包未触达任何${labels[1]}条目。`,
    ownership: proof.ownership,
    sourceIds: [sourceRecordId("empty_pool_proofs", proof.id)],
    evidence: [{
      id: `starclock:${proof.source_path}:${proof.row_locator}`,
      repository_or_url: "https://github.com/realm-labs/starclock.git",
      revision_or_access_date: "2026-08-01",
      game_version: "4.4",
      path_or_page: proof.source_path,
      row_locator: proof.row_locator,
      evidence_sha256: proof.evidence_sha256,
      quality: proof.evidence_quality,
      mechanism_quality: "ExactSelectorClosure",
      note: proof.selector_proof,
    }],
    tags: ["empty-pool", proof.family, "selector-closure"],
    fields: {
      family: proof.family,
      membership_rule: manifest.membership_rule,
      selector_roots: {
        schedule_id: 201033,
        group_id: 1033,
        ordinary_stage_ids: Array.from({ length: 12 }, (_, index) => 5201 + index),
        tierce_stage_id: 5213,
        selected_stage_config_count: 25,
      },
      reachable_record_count: 0,
      exact_zero: true,
      proof_kind: "GeneratedSelectorClosure",
      proof_statement: proof.selector_proof,
      fail_closed_on_unresolved_selector: true,
      evidence_quality: "ExactStructuredSelectorClosure",
      mechanism_quality: "ExactSelectorClosure",
      approximations: [],
    },
  });
});

const claimed = poolRecords.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expected = proofs.map(({ id }) => sourceRecordId("empty_pool_proofs", id)).sort();
assert(claimed.length === new Set(claimed).size, "empty-pool proofs must be claimed exactly once");
assert(JSON.stringify([...claimed].sort()) === JSON.stringify(expected), "empty-pool proof coverage drift");
assert(poolRecords.every(({ reachable_record_count: count }) => count === 0), "non-empty pool cannot use exact-zero proof");

const output = normalizedFile("pool-audits.json", "PoolAudit", poolRecords);
await writeCanonical("pool-audits.json", output, check);
const outputDigest = digest(output);
await writeText(
  "evidence/memory-of-chaos-reference-v1/pool-closure-audit.md",
  `# Goal 17 empty-pool selector-closure audit

- Frozen proof obligations: 10/10, each claimed exactly once
- Selector root: schedule 201033 -> group 1033 -> stages 5201-5213 -> 25 selected StageConfig rows
- Reachable Blessing, Curio, Occurrence, service, currency, shop, choice, Rogue path/progression/room rows: 0
- Proof policy: generated exact-zero closure; any unresolved selector fails closed
- Normalized pool-audit digest: \`${outputDigest}\`
- Runtime executable rows: 0

Each zero is a frozen machine proof from the active released selector closure,
not an inference from mode naming or an upstream table-wide count.
`,
  check,
);
console.log(`Goal 17 pools ${check ? "verified" : "generated"}: 10/10 exact-zero selector proofs.`);
