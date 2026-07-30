#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  sha256,
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

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}
function localRef(relative, row, locator) {
  return {
    source_id: `source.goal09.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}

const servicePolicy = await context.policyRef(
  "service-transaction-boundary",
  "Execute each service purchase as an atomic accepted Activity transaction. Resolve candidates in stable identity order, debit exact inherited prices before ordered grant operations, and reject insufficient currency or unresolved offer pools without mutation.",
  "Replace transaction order, offer construction or hidden shop weights when released Swarm service-controller evidence supplies them.",
);
const adventurePolicy = await context.policyRef(
  "abstract-adventure-outcome",
  "Accept one externally resolved Tier1, Tier2 or Tier3 Adventure result and apply only its validated reward payload. Movement, aiming, physics, timing input, score thresholds and unavailable reward tables remain outside the simulation and fail closed.",
  "Replace abstract result tiers when released structured reward/threshold tables become available without requiring action-minigame input simulation.",
);
const standardRelative =
  "content-reference/standard-universe-v1/services.json";
const standardServices = await localRows(standardRelative);
const beacons = await localRows(
  "content-reference/swarm-disaster-v1/beacons.json",
);
const manifest = await localRows(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const adventureEntries = await context.table("RogueDLCAdventureRoom");
const requiredServices = new Set(manifest.categories.shared_services.records
  .map(({ id }) => id));
const requiredAdventures = new Set(
  manifest.categories.adventure_outcomes.records.map(({ id }) => id),
);

function parameterMap(parameters) {
  return Object.fromEntries(parameters.map(({ key, value }) => [key, value]));
}
const services = standardServices
  .map((row, index) => ({ row, index }))
  .filter(({ row }) => requiredServices.has(row.id))
  .map(({ row: inherited, index }) => ({
    ...context.envelope({
      id: `swarm-disaster.service.${slug(inherited.id)}`,
      kind: "SwarmService",
      nameEn: inherited.name_en,
      nameZh: inherited.name_zh_cn,
      summaryEn:
        `Swarm Disaster reuses the shared ${inherited.name_en} service with inherited parameters and mode-local offer pools.`,
      summaryZh:
        `寰宇蝗灾复用共享的${inherited.name_zh_cn}服务、继承参数与模式内候选池。`,
      ownership: "Shared",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        localRef(standardRelative, inherited, index),
        servicePolicy,
      ],
      tags: ["service", "shared", slug(inherited.kind), "project-policy"],
    }),
    source_id: inherited.id,
    shared_service_id: inherited.id,
    service_kind: inherited.kind,
    resource_id: inherited.currency_id,
    inherited_price_formula_id: inherited.price_formula_id,
    inherited_offer_pool_id: inherited.offer_pool_id,
    inherited_rule_ids: inherited.rule_ids,
    parameters: inherited.parameters,
    eligibility: {
      rule: "ServicePresentAndAcceptedActivityCommand",
      unresolved_offer_behavior: "FailClosed",
    },
    price_policy: {
      source: "InheritedSharedServiceParameters",
      parameter_values: inherited.parameters,
      insufficient_resource: "RejectWithoutMutation",
    },
  }));
outputs.set("services.json", ordered(services, ["service_kind", "id"]));

const currencyService = services.find(({ service_kind: kind }) =>
  kind === "Currency");
if (!currencyService) throw new Error("missing Cosmic Fragments service");
const currencyParameters = parameterMap(currencyService.parameters);
const currencies = [{
  ...context.envelope({
    id: "swarm-disaster.currency.cosmic-fragments",
    kind: "SwarmCurrency",
    nameEn: "Cosmic Fragments",
    nameZh: "宇宙碎片",
    summaryEn:
      "Run-scoped currency used by Swarm services, Occurrence costs and Trailblaze Bonuses.",
    summaryZh:
      "寰宇蝗灾局内货币，用于服务、事件成本与开拓祝福。",
    ownership: "Shared",
    sourceRefs: currencyService.source_refs,
    tags: ["currency", "run-scoped", "shared"],
  }),
  resource_id: "universe.currency.cosmic-fragments",
  initial_value: currencyParameters.initial_amount ?? "0",
  cap_policy: {
    maximum: "",
    overflow: "CheckedUnboundedDomainValue",
    scope: "ActivityRun",
    reset_boundary: "RunStart",
  },
}];
outputs.set("currencies.json", currencies);

const adventureDefinitions = new Map([
  ["RogueCaptureMonster", {
    nameEn: "Trotter Catch",
    nameZh: "扑满捕捉挑战",
  }],
  ["RogueDestroyProp", {
    nameEn: "Barrel Breaker Challenge",
    nameZh: "破坏物挑战",
  }],
]);
const adventures = adventureEntries
  .filter(({ row }) => requiredAdventures.has(String(row.RoomID)))
  .map((entry) => {
    const sourceId = String(entry.row.RoomID);
    const definition = adventureDefinitions.get(entry.row.AdventureType);
    if (!definition)
      throw new Error(`unsupported Adventure type ${entry.row.AdventureType}`);
    return {
      ...context.envelope({
        id: `swarm-disaster.adventure-outcome.${sourceId}`,
        kind: "SwarmAdventureOutcome",
        nameEn: `${definition.nameEn} — Room ${sourceId}`,
        nameZh: `${definition.nameZh}·房间 ${sourceId}`,
        summaryEn:
          `${definition.nameEn} is represented as a validated external three-tier outcome; action input is excluded.`,
        summaryZh:
          `${definition.nameZh}表示为经验证的外部三档结果，不包含动作输入。`,
        ownership: "Shared",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), adventurePolicy],
        tags: [
          "adventure",
          slug(entry.row.AdventureType),
          "external-outcome",
          "project-policy",
        ],
      }),
      source_id: sourceId,
      adventure_type: entry.row.AdventureType,
      parameter_group_id: String(entry.row.ParamGroupID),
      tier: "ExternalTieredResult",
      offered_result: {
        accepted_values: ["Tier1", "Tier2", "Tier3"],
        cumulative: true,
        input_simulation: "Excluded",
      },
      reward_program: {
        operation: "ApplyValidatedExternalAdventureReward",
        payload_schema:
          "swarm-disaster.external-adventure-reward.v1",
        unresolved_payload: "RejectWithoutMutation",
        blessing_pool_id: "swarm-disaster.pool.blessings",
        curio_pool_prefix: "swarm-disaster.curio-pool.",
      },
    };
  });
outputs.set(
  "adventure-outcomes.json",
  ordered(adventures, ["adventure_type", "tier", "id"]),
);

function costParameters(service) {
  return service.parameters.filter(({ key }) =>
    /cost|price/iu.test(key));
}
function serviceOperation(kind) {
  return new Map([
    ["Currency", "InitializeRunCurrency"],
    ["ResetBlessing", "RegenerateBlessingCandidates"],
    ["Reviver", "ReviveSelectedCharacter"],
    ["Downloader", "AddExternalCharacterParticipant"],
    ["RespiteOffers", "ResolveRespiteOffer"],
    ["EnhanceBlessing", "EnhanceSelectedBlessing"],
    ["BlessingShop", "PurchaseBlessing"],
    ["CurioShop", "PurchaseCurio"],
  ]).get(kind) ?? "ResolveSharedService";
}
const serviceRules = services.map((service) => ({
  ...context.envelope({
    id: `swarm-disaster.service-rule.${slug(service.shared_service_id)}`,
    kind: "SwarmServiceRule",
    nameEn: `${service.name_en} Rule`,
    nameZh: `${service.name_zh_cn}规则`,
    summaryEn:
      `Resolve ${service.name_en} as one ordered Activity transaction.`,
    summaryZh:
      `将${service.name_zh_cn}解析为一个有序活动事务。`,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [...service.source_refs, servicePolicy],
    tags: ["service-rule", "activity-operation", "project-policy"],
  }),
  service_id: service.id,
  conditions: [{
    kind: "ServiceAvailableAndCommandAccepted",
    unresolved_offer_behavior: "FailClosed",
  }],
  costs: costParameters(service).map((parameter, index) => ({
    order: index,
    resource_id: service.resource_id,
    key: parameter.key,
    value: parameter.value,
  })),
  ordered_operations: [{
    order: 0,
    operation: serviceOperation(service.service_kind),
    inherited_rule_ids: service.inherited_rule_ids,
    parameters: service.parameters,
  }],
}));
for (const beacon of beacons)
  serviceRules.push({
    ...context.envelope({
      id: `swarm-disaster.service-rule.beacon.${beacon.source_id}`,
      kind: "SwarmServiceRule",
      nameEn: `${beacon.name_en} Beacon Rule`,
      nameZh: `${beacon.name_zh_cn}信标规则`,
      summaryEn:
        `Apply the released ${beacon.name_en} beacon contribution at domain entry.`,
      summaryZh:
        `进入区域时应用已发布的${beacon.name_zh_cn}信标贡献。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [...beacon.source_refs, servicePolicy],
      tags: ["beacon-rule", "service-rule", "project-policy"],
    }),
    service_id: beacon.id,
    conditions: [{
      kind: "NodeHasBeaconAtAcceptedDomainEntry",
      beacon_id: beacon.id,
    }],
    costs: [],
    ordered_operations: [{
      order: 0,
      operation: "ApplyBeaconContribution",
      block_intro_id: beacon.block_intro_id,
      boundary: beacon.application_stage,
    }],
  });
outputs.set(
  "service-rules.json",
  ordered(serviceRules, ["service_id", "id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Services ${check ? "verified" : "generated"}: ` +
  `${services.length} services, ${currencies.length} currency, ` +
  `${adventures.length} Adventure outcomes and ${serviceRules.length} rules.`,
);
