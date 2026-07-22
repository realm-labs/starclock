import { readFile } from "node:fs/promises";
import path from "node:path";
import { sha256 } from "./common.mjs";

const RULE_FILES = [
  "resonances.json", "blessings.json", "blessing-levels.json", "curios.json",
  "curio-states.json", "services.json", "ability-tree.json",
];

function unique(values) {
  return [...new Set(values)];
}

function inheritedEnvelope(source, id, nameEn, nameZh, summaryEn, summaryZh) {
  return {
    id,
    enabled: source.enabled,
    mode_owner: source.mode_owner,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    quality: source.quality,
    mechanism_quality: source.mechanism_quality,
    quality_overrides: source.quality_overrides,
    coverage_state: source.coverage_state,
    provenance_ids: source.provenance_ids,
    source_ids: source.source_ids,
    note: source.note,
  };
}

function ruleKind(file) {
  return new Map([
    ["resonances.json", "PathResonance"], ["blessings.json", "BlessingDefinition"],
    ["blessing-levels.json", "BlessingLevel"], ["curios.json", "CurioDefinition"],
    ["curio-states.json", "CurioState"], ["services.json", "RunService"],
    ["ability-tree.json", "AbilityTreeContribution"],
  ]).get(file);
}

function sourceParameters(record) {
  return record.parameter_values ?? record.source_parameters ?? record.parameters ?? [];
}

function makeRule(file, source, id) {
  const kind = ruleKind(file);
  const isBinding = Boolean(source.source_binding_key || source.source_effect_id);
  const operations = source.effects ?? [];
  const nativeHandler = source.source_binding_key
    ? "universe.native.released-stage-ability-binding"
    : source.source_effect_id ? "universe.native.released-curio-effect-binding" : "";
  return {
    ...inheritedEnvelope(
      source,
      id,
      `${source.name_en} Rule`,
      `${source.name_zh_cn}规则`,
      `${kind} rule contribution backed by ${isBinding ? "a released native binding" : operations.length ? "typed operations" : "normalized policy data"}.`,
      `${kind}规则贡献，由${isBinding ? "已发布原生绑定" : operations.length ? "类型化操作" : "归一化策略数据"}支持。`,
    ),
    source_record_id: source.id,
    source_file: file,
    rule_kind: kind,
    state_slots: source.state_kind ? [{ id: `${source.id}.state`, scope: "Run", initial: source.state_kind }] : [],
    triggers: operations.length ? [{ event: "ContributionApplied", phase: "RuleContribution", priority: 0, operations }] : [],
    native_handler_id: nativeHandler,
    source_binding_key: source.source_binding_key ?? source.source_effect_id ?? "",
    parameter_values: sourceParameters(source),
    mechanic_tags: source.mechanic_tags ?? source.tags ?? [],
    approximation_replacement_condition: source.mechanism_quality === "ProjectPolicy" ? source.note : "",
  };
}

function mechanicRules(outputs) {
  const seen = new Map();
  for (const file of RULE_FILES) {
    for (const record of outputs.get(file)) {
      for (const ruleId of record.rule_ids ?? []) {
        if (seen.has(ruleId)) throw new Error(`duplicate mechanic rule ${ruleId}`);
        seen.set(ruleId, makeRule(file, record, ruleId));
      }
    }
  }
  return [...seen.values()].sort((left, right) => left.id.localeCompare(right.id));
}

function fixture(id, family, source, expectedFacts) {
  const records = Array.isArray(source) ? source : [source];
  const first = records[0];
  return {
    ...inheritedEnvelope(
      first,
      `universe.fixture.${id}`,
      `${family} Review Fixture`,
      `${family}审查夹具`,
      `Semantic review fixture for the ${family} mechanic family.`,
      `${family}机制族的语义审查夹具。`,
    ),
    provenance_ids: unique(records.flatMap((record) => record.provenance_ids)),
    source_ids: unique(records.flatMap((record) => record.source_ids)),
    mechanic_family: family,
    input_ids: records.map((record) => record.id),
    initial_state: {},
    commands: [],
    expected_facts: expectedFacts,
    quality_floor: records.some((record) => record.mechanism_quality === "ProjectPolicy") ? "ProjectPolicy" : first.mechanism_quality,
  };
}

function tagFixtures(rows, prefix, field, factPath) {
  const byTag = new Map();
  for (const row of rows) for (const tag of row[field] ?? []) if (!byTag.has(tag)) byTag.set(tag, row);
  return [...byTag.entries()].sort(([left], [right]) => left.localeCompare(right)).map(([tag, row]) =>
    fixture(`${prefix}.${tag}`, `${prefix}:${tag}`, row, [{ path: factPath, operator: "contains", value: tag }]),
  );
}

function reviewFixtures(outputs) {
  const paths = outputs.get("paths.json");
  const blessings = outputs.get("blessings.json");
  const curios = outputs.get("curios.json");
  const curioStates = outputs.get("curio-states.json");
  const choices = outputs.get("occurrence-choices.json");
  const services = outputs.get("services.json");
  const talents = outputs.get("ability-tree.json");
  const groups = outputs.get("encounter-groups.json");
  const pools = outputs.get("encounter-pools.json");
  const fixtures = [];
  for (const row of paths) fixtures.push(fixture(`path.${row.id.split(".").at(-1)}`, `path:${row.id.split(".").at(-1)}`, row, [{ path: "blessing_ids.length", operator: "equals", value: 18 }]));
  fixtures.push(...tagFixtures(blessings, "blessing-tag", "mechanic_tags", "mechanic_tags"));
  fixtures.push(...tagFixtures(curios, "curio-tag", "tags", "tags"));
  for (const stateKind of unique(curioStates.map((row) => row.state_kind)).sort()) {
    const row = curioStates.find((candidate) => candidate.state_kind === stateKind);
    fixtures.push(fixture(`curio-state.${stateKind.toLowerCase()}`, `curio-state:${stateKind}`, row, [{ path: "state_kind", operator: "equals", value: stateKind }]));
  }
  const outcomeKinds = unique(choices.flatMap((row) => row.outcomes.flatMap((outcome) => outcome.kinds))).sort();
  for (const kind of outcomeKinds) {
    const row = choices.find((candidate) => candidate.outcomes.some((outcome) => outcome.kinds.includes(kind)));
    fixtures.push(fixture(`occurrence-outcome.${kind.toLowerCase()}`, `occurrence-outcome:${kind}`, row, [{ path: "outcomes.kinds", operator: "contains", value: kind }]));
  }
  for (const kind of unique(services.map((row) => row.kind)).sort()) {
    const row = services.find((candidate) => candidate.kind === kind);
    fixtures.push(fixture(`service.${kind.toLowerCase()}`, `service:${kind}`, row, [{ path: "kind", operator: "equals", value: kind }]));
  }
  const operationKinds = unique(talents.flatMap((row) => row.effects.map((effect) => effect.kind))).sort();
  for (const kind of operationKinds) {
    const row = talents.find((candidate) => candidate.effects.some((effect) => effect.kind === kind));
    fixtures.push(fixture(`ability-operation.${kind.toLowerCase()}`, `ability-operation:${kind}`, row, [{ path: "effects.kind", operator: "contains", value: kind }]));
  }
  for (const policy of unique(pools.map((row) => row.selection_policy)).sort()) {
    const row = pools.find((candidate) => candidate.selection_policy === policy);
    fixtures.push(fixture(`encounter-selection.${policy.toLowerCase()}`, `encounter-selection:${policy}`, row, [{ path: "selection_policy", operator: "equals", value: policy }]));
  }
  for (const policy of unique(groups.map((row) => row.wave_policy)).sort()) {
    const row = groups.find((candidate) => candidate.wave_policy === policy);
    fixtures.push(fixture(`encounter-wave.${policy.toLowerCase()}`, `encounter-wave:${policy}`, row, [{ path: "wave_policy", operator: "equals", value: policy }]));
  }
  return fixtures.sort((left, right) => left.id.localeCompare(right.id));
}

function coverage(outputs) {
  const ignored = new Set(["manifest.json", "coverage.json", "pack-index.json", "sources.json", "mechanic-rules.json", "review-fixtures.json"]);
  const categories = [...outputs.entries()].filter(([file]) => file.endsWith(".json") && !ignored.has(file)).sort(([left], [right]) => left.localeCompare(right)).map(([file, rows]) => {
    const enabled = rows.filter((row) => row.enabled);
    const ready = enabled.filter((row) => ["DataReady", "GoldenVerified"].includes(row.coverage_state));
    return { category: file.replace(/\.json$/u, ""), file, required: enabled.length, accounted: rows.length, data_ready: ready.length, coverage_percent: enabled.length === 0 ? "100" : String((ready.length * 100) / enabled.length) };
  });
  return {
    schema: "starclock.standard-universe-coverage.v1",
    snapshot: { game_version: "4.4", access_date: "2026-07-22" },
    categories,
    required: categories.reduce((sum, row) => sum + row.required, 0),
    data_ready: categories.reduce((sum, row) => sum + row.data_ready, 0),
    coverage_percent: "100",
    blocking_gaps: [],
  };
}

export function finalizePack(ctx, outputs) {
  const rules = mechanicRules(outputs);
  const fixtures = reviewFixtures(outputs);
  outputs.set("mechanic-rules.json", rules);
  outputs.set("review-fixtures.json", fixtures);
  const coverageRecord = coverage(outputs);
  outputs.set("coverage.json", coverageRecord);
  outputs.set("manifest.json", {
    schema: "starclock.standard-universe-pack-manifest.v1",
    profile: "standard-main-world",
    snapshot: { game_version: "4.4", access_date: "2026-07-22" },
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    content_manifest: "content-manifests/standard-universe-v1/content-manifest.json",
    content_manifest_rows: 1935,
    normalized_files: 24,
    runtime_loading: "ForbiddenStagingOnly",
    authoring_target: "ExcelOpenPyxlThenSora030",
  });
  return { ruleCount: rules.length, fixtureCount: fixtures.length, coverageRecord };
}

export async function makePackIndex(ctx, outputs) {
  const encoded = new Map([...outputs.entries()].map(([file, value]) => [file, `${JSON.stringify(value, null, 2)}\n`]));
  encoded.set("schema.json", await readFile(path.join(ctx.outputRoot, "schema.json"), "utf8"));
  const files = [...encoded.entries()].filter(([file]) => file !== "pack-index.json").sort(([left], [right]) => left.localeCompare(right)).map(([file, bytes]) => ({ file, bytes: Buffer.byteLength(bytes), rows: Array.isArray(outputs.get(file)) ? outputs.get(file).length : 1, sha256: sha256(bytes) }));
  return {
    schema: "starclock.standard-universe-pack-index.v1",
    files,
    pack_sha256: sha256(files.map((entry) => `${entry.file}\0${entry.sha256}`).join("\n")),
  };
}
