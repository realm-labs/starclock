#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/currency-wars-reference/import-economy.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "roster-avatars.json": 77,
  "economy-rules.json": 1,
  "roster-offers.json": 10,
  "roster-transactions.json": 5,
  "team-size-states.json": 10,
};
const rowsByFile = Object.fromEntries(Object.keys(expected)
  .map((file) => [file, json(path.join(outputRoot, file))]));
const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
for (const [file, count] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  const contract = schema.files.find((entry) => entry.file === file);
  assert(rows.length === count && unique(rows.map(({ id }) => id)),
    `${file} row/count uniqueness drift`);
  assert(contract && rows.every((row) =>
    contract.required_domain_fields.every((field) => Object.hasOwn(row, field))),
  `${file} contract-field drift`);
  assert(rows.every(validEnvelope), `${file} envelope drift`);
}

const roster = rowsByFile["roster-avatars.json"];
assert(roster.every((row) =>
  ["1", "2", "3", "4", "5"].includes(row.rarity)
    && row.cost === row.rarity
    && row.in_pool === true
    && row.avatar_id
    && row.role_id),
"GridFight roster/cost closure drift");
assert(sourceLocators(roster, "ExcelOutput/GridFightRoleBasicInfo.json").size
  === 77, "GridFight RoleBasicInfo exact-once drift");

const economy = rowsByFile["economy-rules.json"][0];
assert(economy.refresh_rules.cards_per_refresh === "5"
  && economy.refresh_rules.refresh_gold === "2"
  && economy.interest_rules.deposit_per_interest === "10"
  && economy.interest_rules.standard_max_interest === "5"
  && economy.interest_rules.overclock_max_interest === "0"
  && economy.experience_rules.standard_wave_gain === "2"
  && economy.experience_rules.standard_boss_wave_gain === "10"
  && economy.team_size_rules.front_max === "4"
  && economy.team_size_rules.back_initial === "6"
  && economy.team_size_rules.back_max === "9"
  && economy.team_size_rules.bench_authored === "9",
"GridFight economy constant drift");

const offers = rowsByFile["roster-offers.json"];
assert(offers.every((row) =>
  row.offer_count === "5"
    && Object.values(row.weights).reduce(
      (sum, value) => sum + Number(value), 0) === 100
    && row.candidate_avatar_ids.every((id) =>
      roster.some((role) => role.id === id))),
"GridFight offer pool/weight drift");
assert(sourceLocators(offers, "ExcelOutput/GridFightLevelV2.json").size === 10
  && sourceLocators(offers,
    "ExcelOutput/GridFightConstValueCommonV2.json").size === 10,
"GridFight level/card-weight source closure drift");

const transactions = rowsByFile["roster-transactions.json"];
assert(transactions.map(({ source_id: id }) => id).join(",") === "1,2,3,4,5"
  && transactions.every((row) =>
    row.price_rule.buy_by_star.length === 4
      && row.price_rule.sell_by_star.length === 4),
"GridFight transaction-price closure drift");
assert(transactions.find(({ source_id: id }) => id === "5")
  .price_rule.buy_by_star.join(",") === "5,15,45,135"
  && transactions.find(({ source_id: id }) => id === "5")
    .price_rule.sell_by_star.join(",") === "5,14,44,132",
"GridFight rarity-5 buy/sell prices drift");

const team = rowsByFile["team-size-states.json"];
const teamByLevel = [...team].sort((left, right) => left.level - right.level);
assert(teamByLevel.map(({ field_cap: value }) => value).join(",")
  === "1,2,3,4,5,6,7,8,9,10"
  && teamByLevel.at(-1).next_level_experience === ""
  && team.every((row) => row.bench_cap === "9"
    && row.positional_front_cap === "4"),
"GridFight roster-level/team-size drift");
for (const sourcePath of [
  "ExcelOutput/GridFightLevelV2.json",
  "ExcelOutput/GridFightPlayerLevel.json",
  "ExcelOutput/GridFightRarityWeight.json",
])
  assert(sourceLocators(team, sourcePath).size === 10,
    `${sourcePath} exact-once team-state drift`);

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) =>
  row.source_refs.every((ref) =>
    !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into economy");
const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars economy verified (${allRows.length} rows; 77 roles; ` +
  `10 offer/team levels; digest ${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function sourceLocators(rows, sourcePath) {
  return new Set(rows.flatMap(({ source_refs: refs }) =>
    refs.filter(({ path: refPath }) => refPath === sourcePath)
      .map(({ locator }) => locator)));
}
function validEnvelope(row) {
  return row
    && /^[a-z0-9][a-z0-9._:-]*$/u.test(row.id)
    && row.name_en && row.name_zh_cn && row.summary_en && row.summary_zh_cn
    && ["CurrencyWars", "Shared"].includes(row.ownership)
    && ["Cataloged", "Researched", "DataReady", "Blocked"]
      .includes(row.coverage_state)
    && row.source_refs.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256))
    && JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort(compare));
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
