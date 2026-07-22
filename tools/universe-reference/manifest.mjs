import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const sourceRoot = path.join(root, ".cache", "content-reference", "turnbasedgamedata");
const output = path.join(root, "content-manifests", "standard-universe-v1", "content-manifest.json");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function compareId(left, right) {
  return String(left).localeCompare(String(right), "en", { numeric: true });
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(canonical(value), "utf8").digest("hex");
}

async function table(name) {
  const relativePath = `ExcelOutput/${name}.json`;
  const raw = await readFile(path.join(sourceRoot, ...relativePath.split("/")), "utf8");
  const safe = raw.replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"');
  return Object.entries(JSON.parse(safe)).map(([sourceKey, row]) => ({
    sourceKey,
    relativePath,
    row,
  }));
}

function item(entry, id, extra = {}) {
  return {
    id: String(id),
    source: `${entry.relativePath}#${entry.sourceKey}`,
    evidence_sha256: digest(entry.row),
    ...extra,
  };
}

function category(id, membershipBasis, records) {
  return {
    id,
    membership_basis: membershipBasis,
    count: records.length,
    records: records.sort((left, right) => compareId(left.id, right.id)),
  };
}

const managers = await table("RogueManager");
const areas = await table("RogueAreaConfig");
const aeons = await table("RogueAeon");
const buffs = await table("RogueBuff");
const mazeBuffs = await table("RogueMazeBuff");
const handbookCurios = await table("RogueHandbookMiracle");
const curios = await table("RogueMiracle");
const handbookEvents = await table("RogueHandBookEvent");
const npcs = await table("RogueNPC");
const roomTypes = await table("RogueRoomType");
const talents = await table("RogueTalent");
const bonuses = await table("RogueBonus");
const shops = await table("RogueShop");
const maps = await table("RogueMap");
const rooms = await table("RogueRoom");
const monsterGroups = await table("RogueMonsterGroup");
const monsters = await table("RogueMonster");
const standardRooms = rooms.filter(({ row }) => row.MapEntrance < 8110000);

const currentManager = managers
  .filter(({ row }) => row.RogueVersion === 1 && row.RogueAreaIDList.length > 0)
  .sort((left, right) => right.row.RogueSeason - left.row.RogueSeason)[0];
const currentAreaIds = new Set(currentManager.row.RogueAreaIDList);
const currentAreas = areas.filter(({ row }) => currentAreaIds.has(row.RogueAreaID) && !row.isActivityArea);
const areaWorldId = (row) => row.AreaProgress ?? Math.floor((row.RogueAreaID - 90) / 10);
const worldIds = [...new Set(currentAreas.map(({ row }) => areaWorldId(row)))].sort((a, b) => a - b);

const isResonance = (row) => {
  const suffix = row.MazeBuffID % 100;
  return row.MazeBuffLevel === 1 && row.RogueBuffCategory === "Legendary" && suffix >= 20 && suffix <= 23;
};
const isBlessing = (row) => {
  const suffix = row.MazeBuffID % 100;
  return row.MazeBuffLevel === 1 && (
    (row.RogueBuffCategory === "Legendary" && suffix >= 30 && suffix <= 32) ||
    (row.RogueBuffCategory === "Rare" && suffix >= 40 && suffix <= 46) ||
    (row.RogueBuffCategory === "Common" && suffix >= 50 && suffix <= 57)
  );
};
const blessingIds = new Set(buffs.filter(({ row }) => isBlessing(row)).map(({ row }) => row.MazeBuffID));
const blessingDetails = mazeBuffs.filter(({ row }) => blessingIds.has(row.ID));

const standardCurioHandbookIds = new Set(
  handbookCurios.filter(({ row }) => row.MiracleTypeList.includes(100)).map(({ row }) => row.MiracleHandbookID),
);
const standardEvents = handbookEvents.filter(({ row }) => row.EventTypeList.includes(100));
const baseNpcIds = new Set(
  standardEvents.flatMap(({ row }) => row.UnlockNPCProgressIDList.map((value) => value.FDOELDMEBPE)).filter((id) => id < 100000),
);
const npcById = new Map(npcs.map((entry) => [entry.row.RogueNPCID, entry]));

const combatRoomTypes = new Set([1, 2, 6, 7]);
const monsterGroupById = new Map(monsterGroups.map((entry) => [entry.row.RogueMonsterGroupID, entry]));
const reachableMonsterGroupIds = new Set(
  standardRooms
    .filter(({ row }) => combatRoomTypes.has(row.RogueRoomType))
    .flatMap(({ row }) => Object.values(row.GroupWithContent ?? {}))
    .filter((id) => id && monsterGroupById.has(id)),
);
const reachableMonsterIds = new Set(
  [...reachableMonsterGroupIds]
    .flatMap((id) => Object.keys(monsterGroupById.get(id).row.RogueMonsterListAndWeight ?? {}))
    .map(Number),
);
const monsterById = new Map(monsters.map((entry) => [entry.row.RogueMonsterID, entry]));

const categories = [
  category(
    "worlds",
    "Distinct world ordinals 1-9 reachable from the latest non-empty RogueManager schedule; legacy Worlds 1-3 derive their ordinal from RogueAreaID because AreaProgress is absent.",
    worldIds.map((id) => ({ id: String(id), source: `${currentManager.relativePath}#${currentManager.sourceKey}`, evidence_sha256: digest(currentManager.row) })),
  ),
  category(
    "world_difficulties",
    "Non-activity RogueAreaConfig rows referenced by the latest non-empty RogueManager schedule.",
    currentAreas.map((entry) => item(entry, entry.row.RogueAreaID, { world_id: String(areaWorldId(entry.row)), difficulty: entry.row.Difficulty })),
  ),
  category(
    "paths",
    "All nine RogueAeon rows; RogueAeonListConfig DLC-only display rows are not path membership.",
    aeons.map((entry) => item(entry, entry.row.AeonID, { buff_type: entry.row.RogueBuffType })),
  ),
  category(
    "resonances_and_formations",
    "Per-path level-1 Legendary RogueBuff rows with canonical suffix 20-23; suffix 24-27 interplays are excluded.",
    buffs.filter(({ row }) => isResonance(row)).map((entry) => item(entry, entry.row.MazeBuffID, { path_buff_type: entry.row.RogueBuffType })),
  ),
  category(
    "blessings",
    "The canonical 18 selectable Blessings per path: suffixes 30-32, 40-46 and 50-57 at level 1.",
    buffs.filter(({ row }) => isBlessing(row)).map((entry) => item(entry, entry.row.MazeBuffID, { path_buff_type: entry.row.RogueBuffType, rarity: entry.row.RogueBuffCategory })),
  ),
  category(
    "blessing_levels",
    "RogueMazeBuff level rows whose IDs belong to the 162 main-world Blessings.",
    blessingDetails.map((entry) => item(entry, `${entry.row.ID}:${entry.row.Lv}`, { blessing_id: String(entry.row.ID), level: entry.row.Lv })),
  ),
  category(
    "curios",
    "RogueHandbookMiracle rows whose MiracleTypeList includes the CosmosRogue type 100.",
    handbookCurios.filter(({ row }) => standardCurioHandbookIds.has(row.MiracleHandbookID)).map((entry) => item(entry, entry.row.MiracleHandbookID)),
  ),
  category(
    "curio_states",
    "RogueMiracle base-mode effect rows below 1000 linked to a CosmosRogue type-100 handbook Curio; 1000/3000 mode copies are excluded.",
    curios.filter(({ row }) => standardCurioHandbookIds.has(row.UnlockHandbookMiracleID) && row.MiracleID < 1000).map((entry) => item(entry, entry.row.MiracleID, { curio_id: String(entry.row.UnlockHandbookMiracleID) })),
  ),
  category(
    "occurrences",
    "RogueHandBookEvent rows whose EventTypeList includes the CosmosRogue type 100.",
    standardEvents.map((entry) => item(entry, entry.row.EventHandbookID)),
  ),
  category(
    "occurrence_variants",
    "Base-mode NPC progress IDs below 100000 referenced by CosmosRogue handbook events; DLC-prefixed variants are excluded.",
    [...baseNpcIds].filter((id) => npcById.has(id)).map((id) => item(npcById.get(id), id)),
  ),
  category(
    "domains",
    "All nine base RogueRoomType rows used by CosmosRogue topology.",
    roomTypes.map((entry) => item(entry, entry.row.RogueRoomType)),
  ),
  category(
    "ability_tree",
    "All base RogueTalent nodes; reward-only interpretation is deferred to normalized classification.",
    talents.map((entry) => item(entry, entry.row.TalentID)),
  ),
  category(
    "run_bonuses",
    "All base RogueBonus definitions; availability and replacement generations remain explicit normalized fields.",
    bonuses.map((entry) => item(entry, entry.row.BonusID)),
  ),
  category(
    "shops",
    "Base RogueShop definitions below the 200000 DLC-owned ID boundary.",
    shops.filter(({ row }) => row.RogueShopID < 200000).map((entry) => item(entry, entry.row.RogueShopID)),
  ),
  category(
    "map_nodes",
    "All base RogueMap nodes below the explicitly activity-owned 10000+ map range.",
    maps.filter(({ row }) => row.RogueMapID < 10000).map((entry) => item(entry, `${entry.row.RogueMapID}:${entry.row.SiteID}`, { map_id: String(entry.row.RogueMapID), site_id: entry.row.SiteID })),
  ),
  category(
    "rooms",
    "Base RogueRoom rows whose map entrance is in the 800/803/810 Standard family; 811/812/813 DLC room families are excluded, while Standard Adventure rooms remain external-command nodes.",
    standardRooms.map((entry) => item(entry, entry.row.RogueRoomID, { domain_type: entry.row.RogueRoomType })),
  ),
  category(
    "encounter_groups",
    "RogueMonsterGroup rows directly referenced by the content map of Standard combat, elite or boss rooms; Occurrence/Encounter rooms remain external decisions.",
    [...reachableMonsterGroupIds].map((id) => item(monsterGroupById.get(id), id)),
  ),
  category(
    "encounter_members",
    "RogueMonster rows transitively referenced by the frozen main-world encounter groups.",
    [...reachableMonsterIds].filter((id) => monsterById.has(id)).map((id) => item(monsterById.get(id), id)),
  ),
];

const payload = {
  schema: "starclock.standard-universe-content-manifest.v1",
  snapshot: { game_version: "4.4", access_date: "2026-07-22" },
  source: { repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git", revision },
  profile: "standard-main-world",
  current_schedule: {
    rogue_season: currentManager.row.RogueSeason,
    schedule_data_id: currentManager.row.ScheduleDataID,
    begin_time: currentManager.row.BeginTime,
    end_time: currentManager.row.EndTime,
  },
  exclusions: [
    "RogueDLC, RogueNous, RogueMagic, RogueTourn and RogueEndless-owned rows",
    "Resonance Interplays with blessing suffixes 24-27",
    "presentation, account rewards and historical weekly scheduling",
  ],
  categories: Object.fromEntries(categories.map((entry) => [entry.id, entry])),
};

if (payload.categories.worlds.count !== 9) throw new Error("expected exactly nine Worlds");
if (payload.categories.paths.count !== 9) throw new Error("expected exactly nine Paths");
if (payload.categories.blessings.count !== 162) throw new Error("expected 18 Blessings for each of nine Paths");
if (payload.categories.blessing_levels.count !== 324) throw new Error("expected two levels for every Blessing");
if (payload.categories.resonances_and_formations.count !== 36) throw new Error("expected four resonance rows for every Path");
if (payload.categories.curios.count !== 61 || payload.categories.curio_states.count !== 61) throw new Error("expected 61 Standard Curios and base-mode effect states");
if (payload.categories.rooms.count !== 163) throw new Error("expected 163 Standard room rows");
if (payload.categories.shops.count !== 9) throw new Error("expected nine Standard shop rows");
if (payload.categories.encounter_groups.count !== 74 || payload.categories.encounter_members.count !== 171) throw new Error("Standard encounter reachability drifted");

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  if (await readFile(output, "utf8") !== encoded) throw new Error("standard universe content manifest has generated drift");
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, encoded, "utf8");
}
console.log(`Standard universe content manifest ${check ? "verified" : "generated"}: ${categories.reduce((sum, entry) => sum + entry.count, 0)} records.`);
