#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/pool-audits.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const names = {
  blessings: ["Blessing pool audit", "祝福池审计"],
  curios: ["Curio pool audit", "奇物池审计"],
  occurrences: ["Occurrence pool audit", "事件池审计"],
  gameplay_services: ["Gameplay service pool audit", "玩法服务池审计"],
  currencies: ["Gameplay currency pool audit", "玩法货币池审计"],
  random_content_pools: ["Other random content pool audit", "其他随机内容池审计"],
};
const records = Object.entries(names).map(([family, [nameEn, nameZh]]) => {
  const category = manifest.categories[family];
  const proof = manifest.zero_pool_proofs[family];
  if (category.count !== 0 || category.records.length !== 0 || proof.count !== 0)
    throw new Error(`${family} is no longer an exact-zero family`);
  return {
    id: `pool-audit.${family.replaceAll("_", "-")}`,
    schema_revision: "starclock.anomaly-arbitration-row.v1",
    kind: "PoolAudit",
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en:
      `The active Version 4.4 selector closure reaches no ${family.replaceAll("_", " ")} record.`,
    summary_zh_cn:
      `4.4 版本当期选择器闭包未到达任何${nameZh.replace("池审计", "")}记录。`,
    ownership: "AnomalyArbitration",
    coverage_state: "DataReady",
    evidence_quality: "ExactStructured",
    mechanism_quality: "ExactRelationship",
    manifest_record_ids: [],
    source_refs: [{
      source_id: `goal13:zero-pool-proof:${family}`,
      repository_or_url: "starclock",
      revision_or_access_date: "G13-P0-B3 manifest",
      game_version: "4.4",
      path_or_page:
        "content-manifests/anomaly-arbitration-v1/content-manifest.json",
      locator: `zero_pool_proofs.${family}`,
      sha256: proof.selector_closure_sha256,
      evidence_quality: proof.evidence_quality,
      mechanism_quality: "ExactRelationship",
      note: category.membership_basis,
    }],
    tags: ["exact-zero", "pool-audit", family.replaceAll("_", "-")].sort(),
    pool_family: family,
    active_member_count: 0,
    selector_scope: [
      "active-group-8",
      "aliases-801-through-804",
      "boss-alias-804",
      "five-stage-configs",
      "seven-battle-targets",
      "active-maze-buffs",
      "active-battle-events",
      "mechanical-common-constants",
    ],
    closure_rule:
      "No explicit active selector, direct reference or transitive stable-ID reference enters this family.",
    selector_closure_sha256: proof.selector_closure_sha256,
    replacement_condition: proof.replacement_condition,
    account_reward_locators_are_members: false,
    runtime_executable: false,
  };
});
const document = {
  schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  profile: "anomaly-arbitration-v1",
  file: "pool-audits.json",
  record_kind: "PoolAudit",
  records,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
await mkdir(path.dirname(output), { recursive: true });
if (check) {
  const existing = await readFile(output, "utf8").catch(() => "");
  if (existing !== bytes) throw new Error("pool-audits.json generation drift");
} else {
  await writeFile(output, bytes);
}
console.log(`Anomaly Arbitration pool audits generated: ${records.length}.`);
