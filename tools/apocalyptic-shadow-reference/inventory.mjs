#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { revision, root, sourceBytes, sourcePaths } from "./source.mjs";

const check = process.argv.includes("--check");
const output = path.join(
  root,
  "content-manifests/apocalyptic-shadow-v1/source-inventory.json",
);
const tree = sourcePaths();
const dedicated = tree.filter((entry) =>
  entry.startsWith("ExcelOutput/ChallengeBoss")
  || entry === "ExcelOutput/ScheduleDataChallengeBoss.json"
  || entry === "ExcelOutput/StrongChallengeBossDetail.json"
);
const strongChallenge = tree.filter((entry) => entry.includes("StrongChallenge"));
const shared = [
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/ChallengeGeneralConfig.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/StageConfig.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
];
const adjacent = tree.filter((entry) =>
  /^ExcelOutput\/Challenge(?:Maze|Story|Peak|Activity|Badge|Skip)/.test(entry)
);
const rows = [];
for (const [classification, paths] of [
  ["dedicated", dedicated],
  ["mechanic-program", strongChallenge],
  ["shared-closure", shared],
  ["adjacent-exclusion", adjacent],
]) {
  for (const sourcePath of [...new Set(paths)].sort()) {
    const bytes = await sourceBytes(sourcePath);
    rows.push({
      classification,
      source_path: sourcePath,
      byte_length: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
}
rows.sort((a, b) => a.classification.localeCompare(b.classification)
  || a.source_path.localeCompare(b.source_path));
const document = {
  schema_revision: "starclock.apocalyptic-shadow-source-inventory.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  source_revision: revision,
  generated_at_policy: "deterministic-no-wall-clock",
  counts: Object.fromEntries([...new Set(rows.map((row) => row.classification))]
    .sort().map((classification) => [classification,
      rows.filter((row) => row.classification === classification).length])),
  files: rows,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  const existing = await readFile(output, "utf8").catch(() => "");
  if (existing !== bytes) throw new Error("source inventory drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, bytes);
}
console.log(`Apocalyptic Shadow inventory: ${rows.length} classified files.`);
