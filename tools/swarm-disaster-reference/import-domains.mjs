#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}

function common(values) {
  return context.envelope(values);
}

function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      const a = left[field];
      const b = right[field];
      if (a < b) return -1;
      if (a > b) return 1;
    }
    return 0;
  });
}

async function normalized(name) {
  return JSON.parse(await fs.readFile(
    path.join(context.outputRoot, name),
    "utf8",
  ));
}

const bindingPolicy = await context.policyRef(
  "rooms",
  "RogueDLCRoom publishes room-to-section membership only. Domain and encounter-pool bindings remain empty until exact released joins are imported in G09-P2-B5.",
  "Replace an empty binding only with an exact released source join; never infer it from numeric room-ID encoding.",
);
const rooms = (await context.table("RogueDLCRoom"))
  .filter(({ row }) => row.RogueSubMode === "ChessRogue")
  .map((room) => ({
    ...common({
      id: `swarm-disaster.room.${room.row.RogueRoomID}`,
      kind: "SwarmRoom",
      nameEn: `Swarm Disaster Room ${room.row.RogueRoomID}`,
      nameZh: `寰宇蝗灾房间 ${room.row.RogueRoomID}`,
      summaryEn:
        `Released ChessRogue room membership for section(s) ${room.row.RogueRoomSections.join(", ")}; domain and encounter bindings are not present in this source row.`,
      summaryZh:
        `已发布的 ChessRogue 房间区段关系：${room.row.RogueRoomSections.join("、")}；该来源行不包含领域或遭遇池绑定。`,
      sourceRefs: [context.sourceRef(room), bindingPolicy],
      tags: ["room", "deferred-encounter-binding"],
    }),
    source_id: String(room.row.RogueRoomID),
    sub_mode: room.row.RogueSubMode,
    section_ids: [...room.row.RogueRoomSections].sort((left, right) =>
      left - right),
    domain_id: "",
    encounter_pool_ids: [],
    domain_binding_state: "NotPublishedInRoomRow",
    encounter_binding_state: "DeferredToG09-P2-B5",
  }));
outputs.set("rooms.json", ordered(rooms));

const nodes = await normalized("map-nodes.json");
const blockRules = await normalized("block-create-rules.json");
const manifest = JSON.parse(await fs.readFile(
  path.join(root, "content-manifests/swarm-disaster-v1/content-manifest.json"),
  "utf8",
));
const domainNames = {
  Adventure: ["Adventure", "冒险"],
  Empty: ["Blank", "空白"],
  Event: ["Occurrence", "事件"],
  MonsterBoss: ["Boss", "首领"],
  MonsterElite: ["Elite", "精英"],
  MonsterNormal: ["Combat", "战斗"],
  MonsterSwarm: ["Combat: Swarm", "战斗•虫群"],
  MonsterSwarmBoss: ["Boss: Swarm", "首领•虫群"],
  Respite: ["Respite", "休整"],
  Reward: ["Reward", "奖励"],
  SwarmEvent: ["Occurrence: Swarm", "事件•虫群"],
  Trade: ["Transaction", "交易"],
};
const domainPolicy = await context.policyRef(
  "domains",
  "Domain selection consumes only authored node candidates or block-creation weights. Topology mutations operate on stable node-ID ordering, and an empty legal set is a deterministic no-op.",
  "Replace individual selection or replacement clauses when released engine behavior supplies a stronger exact rule.",
);
const domains = manifest.categories.domains.records.map((record) => {
  const id = `swarm-disaster.domain.${slug(record.id)}`;
  const node = nodes.find(({ domain_candidates: candidates }) =>
    candidates.includes(id));
  const rule = blockRules.find(({ domain_id: domainId }) => domainId === id);
  const source = node?.source_refs[0] ?? rule?.source_refs[0];
  if (!source) throw new Error(`missing normalized source for domain ${record.id}`);
  const [nameEn, nameZh] = domainNames[record.id] ?? [record.id, record.id];
  return {
    ...common({
      id,
      kind: "SwarmDomain",
      nameEn,
      nameZh,
      summaryEn:
        `${record.id} is reachable from an authored ChessRogue node or block-creation rule.`,
      summaryZh:
        `${record.id} 可由已发布的 ChessRogue 节点或区块创建规则到达。`,
      ownership: "Shared",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [source, domainPolicy],
      tags: ["domain", record.id, "project-policy"],
    }),
    source_id: record.id,
    selection_policy: {
      candidate_source: "AuthoredNodeCandidatesOrBlockCreationWeights",
      candidate_order: "StableNodeId",
      weighted_sampling: "IntegerWeight",
      no_legal_target: "NoOp",
    },
    replacement_policy: {
      trigger_source: "TypedTopologyConsequence",
      mutation_order: "StableNodeId",
      preserve_terminal_nodes: true,
      preserve_unmentioned_metadata: true,
      no_legal_target: "NoOp",
    },
  };
});
outputs.set("domains.json", ordered(domains));

const beaconIds = new Set(manifest.categories.beacons.records
  .map(({ id }) => String(id)));
const markTypes = await context.table("RogueDLCMarkType");
const markById = new Map(markTypes
  .filter(({ row }) => beaconIds.has(String(row.MarkTypeID)))
  .map((mark) => [String(mark.row.MarkTypeID), mark]));
const beaconPolicy = await context.policyRef(
  "beacons",
  "Beacon mutation is a typed topology operation. Domain copying and blanking do not implicitly copy or clear a beacon unless the originating effect explicitly requests that operation.",
  "Replace per-effect behavior when released structured or reproducible evidence explicitly couples beacon state to domain copying or blanking.",
);
const beacons = manifest.categories.beacons.records.map((record) => {
  const beaconId = String(record.id);
  const mark = markById.get(beaconId);
  if (!mark) throw new Error(`missing RogueDLCMarkType ${beaconId}`);
  const rule = blockRules.find(({ beacon_weights: weights }) =>
    weights.some(({ beacon_id: id }) =>
      id === `swarm-disaster.beacon.${beaconId}`));
  if (!rule) throw new Error(`missing creation source for beacon ${beaconId}`);
  const name = localized(
    mark.row.MarkTypeNameID,
    `Beacon ${beaconId}`,
    `信标 ${beaconId}`,
  );
  return {
    ...common({
      id: `swarm-disaster.beacon.${beaconId}`,
      kind: "SwarmBeacon",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Shared DLC beacon ${beaconId} is reachable from ChessRogue block-creation weights.`,
      summaryZh:
        `共享 DLC 信标 ${beaconId} 可由 ChessRogue 区块创建权重生成。`,
      ownership: "Shared",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(mark),
        rule.source_refs[0],
        beaconPolicy,
      ],
      tags: ["beacon", "project-policy"],
    }),
    source_id: beaconId,
    block_intro_id: String(mark.row.BlockIntroID),
    application_stage: "TopologyMutationResolution",
    copy_policy: "OnlyWhenExplicitlyRequestedByTypedEffect",
    blanking_policy: "PreserveUnlessExplicitlyRequestedByTypedEffect",
  };
});
outputs.set("beacons.json", ordered(beacons));

const areaEntries = (await context.table("RogueDLCArea"))
  .filter(({ row }) => row.SubType === "ChessRogue");
const displayedBosses = new Map();
for (const area of areaEntries)
  for (const [monsterId, displayLevel] of Object.entries(
    area.row.DisplayMonsterMap ?? {},
  ))
    if (!displayedBosses.has(monsterId))
      displayedBosses.set(monsterId, { area, displayLevel });
const monsters = await context.table("MonsterConfig");
const monsterById = new Map(monsters.map((monster) => [
  String(monster.row.MonsterID),
  monster,
]));
const bossPolicy = await context.policyRef(
  "boss_choices",
  "RogueDLCArea identifies displayed boss candidates but does not bind a selected candidate to a RogueDLCBossDecay row. Preserve intrinsic weaknesses now and require an exact typed decay reference from G09-P1-B3.",
  "Replace the unresolved consequence only when an exact released boss-choice-to-decay join is available.",
);
const bossChoices = [...displayedBosses.entries()].map(([
  monsterId,
  { area, displayLevel },
]) => {
  const monster = monsterById.get(monsterId);
  if (!monster) throw new Error(`missing MonsterConfig ${monsterId}`);
  const name = localized(
    monster.row.MonsterName,
    `Monster ${monsterId}`,
    `敌人 ${monsterId}`,
  );
  return {
    ...common({
      id: `swarm-disaster.boss-choice.${monsterId}`,
      kind: "BossChoice",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn:
        `Displayed ChessRogue boss candidate ${monsterId} at first released display level ${displayLevel}.`,
      summaryZh:
        `ChessRogue 展示首领候选 ${monsterId}，首次发布展示等级为 ${displayLevel}。`,
      ownership: "Shared",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(area),
        context.sourceRef(monster),
        bossPolicy,
      ],
      tags: ["boss-choice", "project-policy"],
    }),
    source_id: monsterId,
    display_level: displayLevel,
    enemy_variant_id: String(monster.row.MonsterTemplateID),
    weakness_consequence: {
      kind: "IntrinsicWeaknessSet",
      elements: [...monster.row.StanceWeakList].sort(),
    },
    later_boss_consequence: {
      kind: "TypedBossDecayReference",
      resolution_state: "DeferredToG09-P1-B3",
      unresolved_reference_policy: "RejectAtPackCompilation",
    },
  };
});
outputs.set("boss-choices.json", ordered(bossChoices));

const topologyEffectTypes = new Set([
  "ReplicateLastCell",
  "ReplicateCellToAround",
  "ReplicateAllAroundCell",
  "SelectMiracleToEmpty",
  "SelectBuffToEmpty",
  "TurnFightCellToEmpty",
  "TurnEventCellToEmpty",
  "TrunEmptyToReward",
  "SetMarkToRandomCell",
  "SetMarkType",
  "SetAroundBlockType",
  "TriggerMark",
  "ToRandomBlockType",
]);
const consequencePolicy = await context.policyRef(
  "topology-consequences",
  "Each released topology-changing dice face becomes one typed ordered operation. Random target sets are sorted by stable node ID before labeled integer sampling; empty legal target sets are no-ops.",
  "Replace an operation policy only when released engine behavior or a reproducible observation provides a stronger exact ordering rule.",
);
const topologyConsequences = (await context.table("RogueDLCAeonDiceSurface"))
  .filter(({ row }) => topologyEffectTypes.has(row.DiceEffectType))
  .map((surface) => {
    const name = localized(
      surface.row.DiceSurfaceName,
      `Dice Face ${surface.row.AeonSurfaceDiceID}`,
      `骰面 ${surface.row.AeonSurfaceDiceID}`,
    );
    const description = localized(
      surface.row.DiceSurfaceDesc,
      `Execute ${surface.row.DiceEffectType}.`,
      `执行 ${surface.row.DiceEffectType}。`,
    );
    return {
      ...common({
        id: `swarm-disaster.topology-consequence.dice-face.${surface.row.AeonSurfaceDiceID}`,
        kind: "TopologyConsequence",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn: description.en,
        summaryZh: description.zh,
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(surface), consequencePolicy],
        tags: [
          "topology-consequence",
          surface.row.DiceEffectType,
          "project-policy",
        ],
      }),
      source_id: String(surface.row.AeonSurfaceDiceID),
      trigger_kind: "AudienceDiceFace",
      scope: "CurrentPlane",
      ordered_operations: [{
        order: 0,
        operation: "ExecuteAuthoredTopologyEffect",
        effect_type: surface.row.DiceEffectType,
        parameters: (surface.row.DiceEffectParam ?? []).map(decimal),
        target_order: "StableNodeId",
        no_legal_target: "NoOp",
      }],
      aeon_dice_id: String(surface.row.AeonDiceID),
      active_stage: surface.row.DiceActiveStage,
    };
  });
outputs.set("topology-consequences.json", ordered(
  topologyConsequences,
  ["trigger_kind", "source_id", "id"],
));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster domains ${check ? "verified" : "generated"}: ` +
  `${rooms.length} rooms, ${domains.length} domains, ${beacons.length} beacons, ` +
  `${bossChoices.length} boss choices, ` +
  `${topologyConsequences.length} topology consequences.`,
);
