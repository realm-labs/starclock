#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { root, sourceJson } from "./source.mjs";

const check = process.argv.includes("--check");
const output = path.join(root,
  "content-manifests/apocalyptic-shadow-v1/pool-selector-proofs.json");
function rows(value) { return Array.isArray(value) ? value : Object.values(value); }
const selections = [
  ["ExcelOutput/ChallengeBossGroupConfig.json", (row) => row.GroupID === 3019],
  ["ExcelOutput/ChallengeBossGroupExtra.json", (row) => row.GroupID === 3019],
  ["ExcelOutput/ChallengeBossMazeConfig.json", (row) => row.GroupID === 3019],
  ["ExcelOutput/ChallengeBossMazeExtra.json", (row) => row.ID >= 30191 && row.ID <= 30194],
  ["ExcelOutput/ChallengeBossMazeTierce.json", (row) =>
    Object.values(row).some((value) => value === 30195)],
];
const selected = [];
for (const [sourcePath, predicate] of selections) {
  for (const row of rows(await sourceJson(sourcePath)).filter(predicate)) {
    selected.push({ source_path: sourcePath, row });
  }
}
const families = {
  blessings: /blessing/i,
  curios: /curio|miracle/i,
  occurrences: /occurrence|eventchoice/i,
  services: /service/i,
  currencies: /currency|coin/i,
  shops: /shop|store/i,
};
function scan(value, pattern, prefix = "$") {
  if (Array.isArray(value)) return value.flatMap((child, index) =>
    scan(child, pattern, `${prefix}[${index}]`));
  if (value && typeof value === "object") return Object.entries(value)
    .flatMap(([key, child]) => [
      ...(pattern.test(key) ? [{ path: `${prefix}.${key}`, value: child }] : []),
      ...scan(child, pattern, `${prefix}.${key}`),
    ]);
  return [];
}
const proofs = Object.entries(families).map(([family, pattern]) => {
  const matches = selected.flatMap(({ source_path: sourcePath, row }) =>
    scan(row, pattern).map((match) => ({ source_path: sourcePath, ...match })));
  return {
    family,
    audited_selected_row_count: selected.length,
    matched_selectors: matches,
    selector_count: matches.length,
    conclusion: matches.length === 0 ? "ExactZero" : "NonZero",
  };
});
if (proofs.some((proof) => proof.selector_count !== 0)) {
  throw new Error("unexpected Apocalyptic Shadow content-pool selector");
}
const document = {
  schema_revision: "starclock.apocalyptic-shadow-pool-proof.v1",
  goal_id: "apocalyptic-shadow-reference-v1",
  selector_scope: "active group 3019, ordinary 30191-30194, tierce 30195",
  source_row_count: selected.length,
  proofs,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  if (await readFile(output, "utf8").catch(() => "") !== bytes)
    throw new Error("pool proof drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, bytes);
}
console.log(`Apocalyptic Shadow exact-zero pools: ${proofs.length}.`);
