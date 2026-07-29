#!/usr/bin/env node

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const packRoot = path.join(root, "content-reference", "gold-and-gears-v1");
const contractPath = path.join(
  root,
  "content-manifests",
  "gold-and-gears-v1",
  "fixture-contract.json",
);
const schemaPath = path.join(
  root,
  "content-manifests",
  "gold-and-gears-v1",
  "normalized-schema.json",
);
const outputPath = path.join(
  root,
  "evidence",
  "gold-and-gears-reference-v1",
  "semantic-fixture-results.json",
);

const contract = json(contractPath);
const schema = json(schemaPath);
const valuesByFile = new Map(
  schema.files.map(({ file }) => [file, json(path.join(packRoot, file))]),
);
const fixtures = valuesByFile.get("review-fixtures.json");
const gaps = valuesByFile.get("research-gaps.json");
const rules = valuesByFile.get("mechanic-rules.json");
const sources = valuesByFile.get("sources.json");
const sourceById = uniqueMap(sources, ({ id }) => id, "source");
const contentById = new Map();
const rowByFileAndId = new Map();
for (const [file, value] of valuesByFile) {
  if (!Array.isArray(value) || file === "sources.json") continue;
  const rows = new Map();
  rowByFileAndId.set(file, rows);
  for (const row of value) {
    assert(!contentById.has(row.id), `${file}/${row.id}: duplicate stable ID`);
    contentById.set(row.id, row);
    rows.set(row.id, row);
  }
}
const fixtureById = uniqueMap(fixtures, ({ id }) => id, "fixture");
const ruleById = uniqueMap(rules, ({ id }) => id, "mechanic rule");
const requiredFamilyById = uniqueMap(
  contract.required_families,
  ({ id }) => id,
  "required family",
);

assert(fixtures.length === 18, "semantic fixture denominator differs");
assert(requiredFamilyById.size === 18, "required family denominator differs");
const fixturesByFamily = Object.groupBy(fixtures, ({ family_id: id }) => id);
for (const family of requiredFamilyById.values()) {
  const matches = fixturesByFamily[family.id] ?? [];
  assert(
    matches.length === family.minimum_cases,
    `${family.id}: expected exactly ${family.minimum_cases} fixture(s)`,
  );
}
assert(
  Object.keys(fixturesByFamily).every((id) => requiredFamilyById.has(id)),
  "fixture uses a family outside the frozen contract",
);

const randomizedFamilies = new Set([
  "encounter-selection",
  "occurrence-choice",
  "resonance-extrapolation",
]);
const fixtureResults = [];
let operationCount = 0;
let assertionCount = 0;
let fixtureInputBindings = 0;
let fixtureEvidenceBindings = 0;
for (const fixture of fixtures) {
  auditFixtureShape(fixture);
  const records = fixture.source_record_ids.map((id) => {
    const record = contentById.get(id);
    assert(record, `${fixture.id}: source record ${id} does not resolve`);
    fixtureInputBindings += 1;
    return record;
  });
  const actual = executeFixture(fixture, records);
  const assertions = fixture.expected_facts.map((expected) => {
    assert(expected.operator === "equals", `${fixture.id}: unsupported operator`);
    const resolved = valueAt(actual, expected.path);
    if (Object.hasOwn(expected, "value")) {
      assert(
        canonical(resolved) === canonical(expected.value),
        `${fixture.id}: ${expected.path} expected ${canonical(expected.value)}, ` +
          `got ${canonical(resolved)}`,
      );
    } else {
      assert(nonempty(resolved), `${fixture.id}: ${expected.path} is empty`);
    }
    return {
      path: expected.path,
      operator: expected.operator,
      expected: Object.hasOwn(expected, "value")
        ? expected.value
        : "NonEmpty",
      actual: resolved,
      result: "pass",
    };
  });
  operationCount += fixture.ordered_operations.length;
  assertionCount += assertions.length;
  fixtureResults.push({
    fixture_id: fixture.id,
    family_id: fixture.family_id,
    evidence_quality: fixture.fixture_evidence_quality,
    source_record_count: records.length,
    operation_count: fixture.ordered_operations.length,
    assertion_count: assertions.length,
    assertions,
    policy_source_ids: fixture.evidence_refs.filter((id) =>
      policyQuality(sourceById.get(id)?.evidence_quality)),
    trace_sha256: sha256(canonical({
      preconditions: fixture.preconditions,
      input: fixture.input,
      ordered_operations: fixture.ordered_operations,
      actual,
    })),
    result: "pass",
  });
}

assert(rules.length === 1224, "mechanic rule denominator differs");
const ruleFamilyCounts = {};
for (const rule of rules) {
  assert(requiredFamilyById.has(rule.family_id), `${rule.id}: unknown fixture family`);
  assert(
    rule.execution_disposition === "ReferenceOnly" &&
      rule.runtime_handler_id === "" &&
      rule.fixture_ids.length === 1,
    `${rule.id}: reference-only execution boundary differs`,
  );
  const fixture = fixtureById.get(rule.fixture_ids[0]);
  assert(fixture, `${rule.id}: fixture does not resolve`);
  assert(
    fixture.family_id === rule.family_id,
    `${rule.id}: fixture family does not match rule family`,
  );
  if (rule.policy_bound) {
    assert(
      rule.unresolved_behavior === "FailClosed",
      `${rule.id}: policy-bound rule does not fail closed`,
    );
  } else {
    assert(
      rule.unresolved_behavior === "NotApplicable",
      `${rule.id}: exact rule has an unresolved behavior`,
    );
  }
  ruleFamilyCounts[rule.family_id] =
    (ruleFamilyCounts[rule.family_id] ?? 0) + 1;
}
assert(
  Object.keys(ruleFamilyCounts).every((id) => requiredFamilyById.has(id)),
  "one or more mechanic-rule families are outside the fixture contract",
);

const policySources = sources.filter(({ evidence_quality: quality }) =>
  policyQuality(quality));
const gapByPolicySource = uniqueMap(
  gaps,
  ({ policy_source_id: id }) => id,
  "research-gap policy source",
);
assert(gaps.length === 16, "research gap denominator differs");
assert(policySources.length === 16, "policy source denominator differs");
assert(
  gapByPolicySource.size === policySources.length &&
    policySources.every(({ id }) => gapByPolicySource.has(id)),
  "policy source and research-gap sets differ",
);

const gapResults = [];
let affectedBindingCount = 0;
let replacementConditionCount = 0;
for (const source of policySources) {
  const gap = gapByPolicySource.get(source.id);
  assert(gap.gap_state === "PolicyBound" && gap.blocking === false,
    `${gap.id}: approximation is blocking`);
  assert(nonempty(source.note) && nonempty(source.replacement_condition),
    `${source.id}: approximation boundary is incomplete`);
  assert(source.replacement_condition.startsWith("Replace"),
    `${source.id}: replacement condition is not actionable`);
  assert(gap.note === source.note,
    `${gap.id}: note differs from the policy source`);
  assert(gap.replacement_condition === source.replacement_condition,
    `${gap.id}: replacement condition differs from the policy source`);
  assert(gap.affected_records.length > 0,
    `${gap.id}: approximation has no affected records`);
  assert(
    canonical(gap.affected_records) === canonical([...gap.affected_records].sort(
      (left, right) =>
        left.file.localeCompare(right.file) || left.id.localeCompare(right.id),
    )),
    `${gap.id}: affected records are not canonically ordered`,
  );
  const affectedKeys = new Set();
  const fixtureIds = new Set();
  for (const affected of gap.affected_records) {
    const key = `${affected.file}/${affected.id}`;
    assert(!affectedKeys.has(key), `${gap.id}: duplicate affected record ${key}`);
    affectedKeys.add(key);
    const rows = rowByFileAndId.get(affected.file);
    assert(rows, `${gap.id}: affected file ${affected.file} missing`);
    const row = rows.get(affected.id);
    assert(row, `${gap.id}: affected record ${key} missing`);
    assert(row.coverage_state === "DataReady", `${gap.id}: ${key} is not DataReady`);
    const policyRef = row.source_refs?.find(({ source_id: id }) => id === source.id);
    assert(policyRef, `${gap.id}: ${key} does not bind its policy source`);
    assert(policyRef.note === source.note,
      `${gap.id}: ${key} policy note differs`);
    assert(policyRef.replacement_condition === source.replacement_condition,
      `${gap.id}: ${key} replacement condition differs`);
    for (const override of row.quality_overrides ?? []) {
      assert(nonempty(override.replacement_condition),
        `${gap.id}: ${key} has an incomplete field-level replacement condition`);
    }
    addFixtureLinks(row, fixtureIds);
    affectedBindingCount += 1;
  }
  assert(fixtureIds.size > 0, `${gap.id}: no semantic fixture family is linked`);
  replacementConditionCount += 1;
  gapResults.push({
    research_gap_id: gap.id,
    policy_source_id: source.id,
    evidence_quality: source.evidence_quality,
    affected_record_count: gap.affected_records.length,
    fixture_ids: [...fixtureIds].sort(),
    note_sha256: sha256(source.note),
    replacement_condition: source.replacement_condition,
    replacement_condition_sha256: sha256(source.replacement_condition),
    result: "pass",
  });
}
assert(affectedBindingCount === 5025, "research-gap affected binding count differs");

const report = {
  schema_revision: "starclock.gold-and-gears-semantic-fixture-results.v1",
  goal_id: "gold-and-gears-reference-v1",
  executed_at: "2026-07-29",
  result: "pass",
  inputs: {
    fixture_contract_sha256: sha256(fs.readFileSync(contractPath)),
    normalized_schema_sha256: sha256(fs.readFileSync(schemaPath)),
    fixture_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "review-fixtures.json")),
    ),
    research_gap_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "research-gaps.json")),
    ),
    mechanic_rule_data_sha256: sha256(
      fs.readFileSync(path.join(packRoot, "mechanic-rules.json")),
    ),
  },
  summary: {
    required_families: requiredFamilyById.size,
    executed_fixtures: fixtureResults.length,
    ordered_operations: operationCount,
    assertions: assertionCount,
    source_record_bindings: fixtureInputBindings,
    evidence_bindings: fixtureEvidenceBindings,
    mechanic_rules: rules.length,
    approximation_boundaries: policySources.length,
    replacement_conditions_verified: replacementConditionCount,
    affected_record_bindings: affectedBindingCount,
    blocking_gaps: 0,
    failed_assertions: 0,
  },
  mechanic_rule_families: sortedObject(ruleFamilyCounts),
  fixtures: fixtureResults.sort((left, right) =>
    left.family_id.localeCompare(right.family_id)),
  approximations: gapResults.sort((left, right) =>
    left.policy_source_id.localeCompare(right.policy_source_id)),
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath),
    "semantic fixture evidence is missing; run with --write");
  assert(fs.readFileSync(outputPath, "utf8") === encoded,
    "semantic fixture evidence drifted");
}
console.log(
  `Gold and Gears semantic fixtures passed (${fixtureResults.length} families; ` +
    `${operationCount} ordered operations; ${assertionCount} assertions; ` +
    `${replacementConditionCount} replacement conditions; ` +
    `${affectedBindingCount} affected bindings).`,
);

function auditFixtureShape(fixture) {
  for (const field of contract.required_fields)
    assert(Object.hasOwn(fixture, field), `${fixture.id}: missing ${field}`);
  assert(fixture.coverage_state === "DataReady", `${fixture.id}: not DataReady`);
  assert(fixture.fixture_evidence_quality === fixture.evidence_quality,
    `${fixture.id}: fixture evidence quality differs`);
  assert(
    fixture.source_record_ids.length > 0 &&
      canonical(fixture.source_record_ids) ===
        canonical([...fixture.source_record_ids].sort()),
    `${fixture.id}: source records are empty or not ordered`,
  );
  assert(
    fixture.ordered_operations.length > 0 &&
      fixture.ordered_operations.every(
        ({ sequence }, index) => sequence === index + 1,
      ),
    `${fixture.id}: operation sequence is not contiguous`,
  );
  assert(fixture.expected_facts.length > 0,
    `${fixture.id}: expected facts are empty`);
  assert(
    fixture.evidence_refs.length > 0 &&
      canonical(fixture.evidence_refs) ===
        canonical(fixture.source_refs.map(({ source_id: id }) => id)),
    `${fixture.id}: evidence references differ from provenance`,
  );
  for (const id of fixture.evidence_refs) {
    assert(sourceById.has(id), `${fixture.id}: evidence ${id} does not resolve`);
    fixtureEvidenceBindings += 1;
  }
  if (policyQuality(fixture.fixture_evidence_quality)) {
    assert(nonempty(fixture.note) && nonempty(fixture.replacement_condition),
      `${fixture.id}: approximation boundary is incomplete`);
    assert(fixture.evidence_refs.some((id) =>
      policyQuality(sourceById.get(id)?.evidence_quality)),
    `${fixture.id}: approximate fixture has no approximate source`);
  } else {
    assert(
      !Object.hasOwn(fixture, "note") &&
        !Object.hasOwn(fixture, "replacement_condition"),
      `${fixture.id}: exact fixture carries an approximation boundary`,
    );
  }
  if (randomizedFamilies.has(fixture.family_id))
    assert(fixture.input.seed === "0",
      `${fixture.id}: randomized review input has no explicit canonical seed`);
}

function executeFixture(fixture, records) {
  const byId = new Map(records.map((row) => [row.id, row]));
  const one = (id) => {
    const value = byId.get(id);
    assert(value, `${fixture.id}: fixture record ${id} missing`);
    return value;
  };
  switch (fixture.family_id) {
    case "profile-entry": {
      const profile = one("gold-gears.profile.v1");
      const areas = records.filter(({ kind }) => kind === "FormalDifficulty");
      const bonuses = records.filter(({ kind }) => kind === "TrailblazeBonus");
      assert(profile.runtime_enabled === false,
        `${fixture.id}: reference profile became runtime-enabled`);
      assert(areas.length === 5 && areas.every(({ area_group: group }) =>
        group === "Formal"), `${fixture.id}: Formal difficulty closure differs`);
      const bonusSourceIds = bonuses.map(({ source_id: id }) => id).sort();
      assert(
        canonical(bonusSourceIds) === canonical(["201", "202", "203", "204", "205"]),
        `${fixture.id}: Trailblaze Bonus closure differs`,
      );
      return {
        formal_difficulty_count: String(areas.length),
        bonus_source_ids: bonusSourceIds,
      };
    }
    case "topology-generation": {
      const board = one("gold-gears.chessboard.1111011");
      const columns = records.filter(({ kind }) => kind === "MapColumn")
        .sort((left, right) => left.position_x - right.position_x);
      const edge = one("gold-gears.edge.1111011.102.203");
      const unspecifiedNode = one("gold-gears.node.2111011.101");
      assert(columns.length === 2 && columns[0].position_x < columns[1].position_x,
        `${fixture.id}: authored columns are not strictly ordered`);
      assert(
        contentById.has(board.start_node_id) && contentById.has(board.end_node_id),
        `${fixture.id}: start or end node does not resolve`,
      );
      assert(
        edge.chessboard_id === board.id &&
          edge.source_node_id !== edge.target_node_id,
        `${fixture.id}: derived edge does not bind the selected board`,
      );
      assert(edge.policy === "forward-nearest-column-within-one-row-v1",
        `${fixture.id}: edge policy differs`);
      assert(unspecifiedNode.domain_ids.length === 0,
        `${fixture.id}: Unspecified node unexpectedly binds a domain`);
      return {
        edge: { policy: { policy_id: edge.policy } },
        unspecified_domain: unspecifiedNode.domain_resolution,
      };
    }
    case "topology-event-order": {
      const event = one("gold-gears.map-event.2111011.3001");
      const rule = one("gold-gears.block-rule.2111011.211101101");
      assert(event.chessboard_id === rule.chessboard_id,
        `${fixture.id}: event and block rule boards differ`);
      assert(event.trigger_type === fixture.input.trigger_type,
        `${fixture.id}: event trigger does not match input`);
      return { event: { weight: event.weight }, block_rule: { order: String(rule.order) } };
    }
    case "cognition-lifecycle": {
      const range = one("gold-gears.cognition-range.301");
      assert(precondition(fixture, "area_id") === range.area_id,
        `${fixture.id}: area precondition differs`);
      assert(fixture.input.plane_boundary === "BossDefeated",
        `${fixture.id}: plane boundary input differs`);
      assert(
        canonical(range.lifecycle.adjustment_order) === canonical(
          fixture.ordered_operations.slice(0, 3).map(({ operation }) => operation),
        ),
        `${fixture.id}: Cognition adjustment order differs`,
      );
      assert(range.lifecycle.no_match_result === "no-secret-unlocked",
        `${fixture.id}: Cognition empty frontier does not fail closed`);
      return {
        bounds: { inclusive: range.bounds_inclusive },
        evaluation_boundary: range.lifecycle.plane_end_evaluation ===
          "after-current-plane-boss-defeat"
          ? "CurrentPlaneBossDefeated"
          : range.lifecycle.plane_end_evaluation,
      };
    }
    case "secret-threshold": {
      const secret = one("gold-gears.secret.2023");
      assert(precondition(fixture, "required_area") === secret.required_area,
        `${fixture.id}: required area differs`);
      assert(
        canonical(precondition(fixture, "predecessors")) ===
          canonical(secret.predecessor_secret_ids),
        `${fixture.id}: predecessor frontier differs`,
      );
      assert(fixture.input.cognition === secret.minimum_cognition,
        `${fixture.id}: input does not exercise the inclusive minimum`);
      return {
        minimum_cognition: secret.minimum_cognition,
        bounds_inclusive: secret.bounds_inclusive,
      };
    }
    case "custom-dice-passive": {
      const dice = one("gold-gears.custom-dice.403");
      const pathValue = one("gold-gears.dice-path-value.403.1");
      const operations = fixture.ordered_operations;
      assert(
        canonical(operations[0].ids) === canonical(dice.initial_effect_extra_ids) &&
          canonical(operations[1].ids) === canonical(dice.passive_effect_extra_ids),
        `${fixture.id}: dice initial or passive effects differ`,
      );
      assert(pathValue.dice_id === dice.id &&
        pathValue.path_id === fixture.input.selected_path_id,
      `${fixture.id}: selected Path value does not resolve`);
      return {
        dice: { source_id: dice.source_id },
        path_boost: { value: pathValue.boost_value },
      };
    }
    case "dice-face-targeting": {
      const face = one("gold-gears.dice-face.2058");
      const slot = one("gold-gears.dice-slot.1");
      assert(fixture.input.eligible_targets.length === 0,
        `${fixture.id}: fixture does not exercise an empty target set`);
      assert(
        face.target_resolution_policy.candidate_order ===
          "stable-node-or-content-id-ascending" &&
          face.target_resolution_policy.unpublished_empty_set_behavior ===
            "FailClosed",
        `${fixture.id}: generic target ordering or fallback differs`,
      );
      return {
        no_legal_target_behavior: face.no_legal_target_behavior,
        slot_legal: face.allowed_slot_ids.includes(slot.id),
      };
    }
    case "dice-reroll-and-cheat": {
      const dice = one("gold-gears.custom-dice.403");
      const node = one("gold-gears.neural-network-node.1701");
      const initial = dice.effect_parts.find(({ role }) => role === "InitialEffect");
      const passive = dice.effect_parts.find(({ role }) => role === "PassiveEffect");
      const policy = node.effect_contributions[0].selection_policy;
      assert(initial.parameters[0] === "1" && passive.parameters[0] === "1",
        `${fixture.id}: released cheat or reroll attempt count differs`);
      assert(fixture.input.eligible_results.length === 0,
        `${fixture.id}: fixture does not exercise an empty reroll set`);
      assert(
        policy.candidate_order === "stable-dice-face-id-ascending" &&
          policy.draw_mode === "seeded-from-eligible-candidates",
        `${fixture.id}: reroll ordering or RNG binding differs`,
      );
      return {
        empty_candidate_behavior: policy.empty_candidate_behavior,
        cheat_attempts_per_plane: initial.parameters[0],
      };
    }
    case "knowledge-lifecycle": {
      const accessModes = records.map(({ knowledge_access: access }) => access).sort();
      const policyIds = new Set(records.map(
        ({ simultaneous_resolution_policy: policy }) => policy.policy_id,
      ));
      assert(
        records.every(({ target_policy: policy }) =>
          policy.candidate_order === "stable-node-id-ascending" &&
          policy.random_selection === "seeded-without-replacement" &&
          policy.empty_candidate_behavior === "NoEffect"),
        `${fixture.id}: Knowledge target policy differs`,
      );
      assert(policyIds.size === 1,
        `${fixture.id}: simultaneous Knowledge policy is not unique`);
      const interactions = records[0].custom_dice_interactions;
      assert(
        canonical(Object.values(interactions).filter((value) =>
          typeof value === "string" && value.startsWith("gold-gears.custom-dice.")))
          === canonical(fixture.input.active_dice_ids),
        `${fixture.id}: active Custom Dice interactions differ`,
      );
      return {
        supported_access_modes: accessModes,
        simultaneous_resolution: [...policyIds][0],
      };
    }
    case "neural-network-effect": {
      const node = one("gold-gears.neural-network-node.201");
      assert(
        canonical(node.prerequisite_ids) ===
          canonical(precondition(fixture, "prerequisite_ids")),
        `${fixture.id}: prerequisite DAG differs`,
      );
      assert(canonical(node.costs) === canonical(fixture.input.paid_costs),
        `${fixture.id}: Neural Impulse cost differs`);
      assert(
        canonical(node.effect_contributions) ===
          canonical(fixture.ordered_operations[2].effects),
        `${fixture.id}: effect contribution differs`,
      );
      return {
        node: {
          disposition: node.disposition,
          effect_domain: node.effect_domain,
        },
      };
    }
    case "conundrum-stats": {
      const level = one("gold-gears.conundrum-level.stats.6");
      assert(String(level.level) === fixture.input.stats_level,
        `${fixture.id}: Stats level differs`);
      assert(
        level.effect_contributions[0].numeric_binding.resolution_state ===
          "UnresolvedFailClosed" &&
          level.effect_contributions[0].numeric_binding.authoritative_behavior ===
            "RejectBattleCompilation",
        `${fixture.id}: unpublished Stats numerics do not fail closed`,
      );
      return {
        composition_mode: level.composition_mode,
        track_cap: String(level.track_cap),
      };
    }
    case "conundrum-auxiliary": {
      const level = one("gold-gears.conundrum-level.auxiliary.6");
      assert(String(level.level) === fixture.input.auxiliary_level,
        `${fixture.id}: Auxiliary level differs`);
      assert(level.active_contribution_ids.length === level.level,
        `${fixture.id}: Auxiliary contributions are not cumulative`);
      return {
        composition_mode: level.composition_mode,
        total_conundrum_cap: String(level.total_conundrum_cap),
      };
    }
    case "path-boost": {
      const boost = one("gold-gears.path-boost.650103");
      assert(precondition(fixture, "selected_path_id") === boost.path_id,
        `${fixture.id}: selected Path differs`);
      assert(boost.allowed_increment_values.includes(fixture.input.source_increment),
        `${fixture.id}: source increment is not an exact allowed value`);
      assert(contentById.has(boost.path_id),
        `${fixture.id}: shared Path does not resolve`);
      return { target_team: boost.target_team, stacking: boost.stacking };
    }
    case "resonance-extrapolation": {
      const normal = one("gold-gears.resonance-extrapolation.1232001");
      const formation = one("gold-gears.resonance-extrapolation.1232101");
      assert(normal.path_id === fixture.input.offered_path_id &&
        formation.path_id === normal.path_id,
      `${fixture.id}: offered Path bindings differ`);
      assert(
        [normal, formation].every(({ battle_scope: scope }) =>
          scope === precondition(fixture, "battle_scope")),
        `${fixture.id}: battle scope differs`,
      );
      assert(
        normal.controller_policy.candidate_order ===
          "stable-source-tag-ascending" &&
          normal.controller_policy.formation_selection ===
            "seeded-activity-stream-without-replacement" &&
          normal.controller_policy.action_and_polarity_lowering ===
            "UnresolvedFailClosed",
        `${fixture.id}: controller ordering or fallback differs`,
      );
      return {
        normal: { enhanced: normal.enhanced },
        formation: { enhanced: formation.enhanced },
      };
    }
    case "curio-lifecycle": {
      const state = one("gold-gears.curio-state.3105");
      const curio = one("gold-gears.curio.105");
      assert(curio.mode_copy_id === precondition(fixture, "mode_copy_id"),
        `${fixture.id}: mode-copy identity differs`);
      assert(curio.state_ids.includes(state.id) && state.curio_id === curio.id,
        `${fixture.id}: Curio/state relationship differs`);
      assert(
        state.selection_policy.candidate_order ===
          "stable-handbook-order-then-source-id" &&
          state.selection_policy.unresolved_offer_behavior === "FailClosed",
        `${fixture.id}: Curio offer ordering or fallback differs`,
      );
      return {
        initial_state_id: curio.initial_state_id,
        repair_target: state.repair_target,
      };
    }
    case "occurrence-choice": {
      const policyChoice = one("gold-gears.occurrence-choice.310101.01");
      const costChoice = one("gold-gears.occurrence-choice.310501.01");
      assert(canonical(costChoice.condition_ids) ===
        canonical(precondition(fixture, "condition_ids")),
      `${fixture.id}: choice conditions differ`);
      assert(
        canonical(costChoice.costs) ===
          canonical(fixture.ordered_operations[1].costs) &&
          canonical(policyChoice.outcomes) ===
            canonical(fixture.ordered_operations[2].outcomes),
        `${fixture.id}: choice cost or outcome order differs`,
      );
      const policy = policyChoice.outcomes[0];
      assert(policy.probability_policy === "SeededUniformStableSourceOrder",
        `${fixture.id}: hidden-weight selection is not stable and seeded`);
      return {
        unresolved_pool_behavior: policy.unresolved_candidate_pool,
        choice_order_preserved:
          policyChoice.node_index === 1 &&
          policyChoice.choice_index === 1 &&
          costChoice.node_index === 1 &&
          costChoice.choice_index === 1,
      };
    }
    case "service-and-adventure": {
      const service = one("universe.service.shop.100011");
      const adventure = one("gold-gears.adventure-outcome.1210601");
      assert(service.currency_id === precondition(fixture, "currency_id"),
        `${fixture.id}: service currency differs`);
      assert(
        service.selection_policy.candidate_order === "stable-source-id" &&
          service.selection_policy.randomness === "seeded-activity-stream" &&
          service.selection_policy.unresolved_pool_behavior === "FailClosed" &&
          adventure.reward_selection_policy.candidate_order ===
            "stable-source-id" &&
          adventure.reward_selection_policy.unresolved_pool_behavior ===
            "FailClosed",
        `${fixture.id}: offer/reward ordering or fallback differs`,
      );
      assert(adventure.rewards_are_cumulative,
        `${fixture.id}: Adventure rewards are not cumulative`);
      return {
        service: { inventory: service.gold_gears_offer_rule.inventory },
        adventure: { reward_tier_count: String(adventure.reward_tiers.length) },
      };
    }
    case "encounter-selection": {
      const group = one("gold-gears.encounter-group.223003");
      const wave = one("gold-gears.encounter-wave.223003.2230031.1");
      const slots = records.filter(({ kind }) => kind === "EnemySlot")
        .sort((left, right) => left.slot_index - right.slot_index);
      assert(group.source_group_id === fixture.input.source_group_id,
        `${fixture.id}: source group differs`);
      assert(
        group.parent_room_scope.kind ===
          precondition(fixture, "room_parent_scope") &&
          group.parent_room_scope.unresolved_behavior === "FailClosed",
        `${fixture.id}: room-parent policy differs`);
      assert(
        group.selection_policy.candidate_order ===
          "source-group-member-order" &&
          group.selection_policy.randomness === "seeded-activity-stream" &&
          group.selection_policy.unresolved_behavior === "FailClosed",
        `${fixture.id}: encounter selection ordering or fallback differs`,
      );
      assert(
        group.weighted_members.some(({ wave_ids: ids }) => ids.includes(wave.id)),
        `${fixture.id}: selected wave is not a weighted group member`,
      );
      assert(canonical(wave.enemy_slot_ids) === canonical(slots.map(({ id }) => id)),
        `${fixture.id}: enemy slot order differs`);
      const bossChoiceIds = new Set(slots.flatMap(
        ({ boss_choice_ids: ids }) => ids,
      ));
      assert([...bossChoiceIds].every((id) => contentById.has(id)),
        `${fixture.id}: displayed boss alternative does not resolve`);
      return {
        enemy_slot_count: String(slots.length),
        boss_choice_count: String(bossChoiceIds.size),
      };
    }
    default:
      throw new Error(`${fixture.id}: no semantic executor for family`);
  }
}

function addFixtureLinks(row, destination) {
  if (row.kind === "ReviewFixture") destination.add(row.id);
  for (const id of row.fixture_ids ?? []) {
    assert(fixtureById.has(id), `${row.id}: fixture ${id} does not resolve`);
    destination.add(id);
  }
  const ruleIds = [
    ...(row.inherited_rule_ids ?? []),
    ...(row.rule_contribution_id ? [row.rule_contribution_id] : []),
  ];
  for (const id of ruleIds) {
    const rule = ruleById.get(id);
    assert(rule, `${row.id}: mechanic rule ${id} does not resolve`);
    for (const fixtureId of rule.fixture_ids) destination.add(fixtureId);
  }
}

function precondition(fixture, name) {
  const item = fixture.preconditions.find(({ fact }) => fact === name);
  assert(item, `${fixture.id}: precondition ${name} missing`);
  return item.value;
}

function valueAt(value, dottedPath) {
  return dottedPath.split(".").reduce((current, segment) => {
    assert(
      current !== null &&
        typeof current === "object" &&
        Object.hasOwn(current, segment),
      `semantic projection path ${dottedPath} does not resolve`,
    );
    return current[segment];
  }, value);
}

function policyQuality(value) {
  return value === "ProjectPolicy" ||
    value === "ApproximateFromReleasedText";
}

function uniqueMap(values, keyOf, label) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    assert(!result.has(key), `duplicate ${label} ${key}`);
    result.set(key, value);
  }
  return result;
}

function sortedObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object")
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function nonempty(value) {
  if (typeof value === "string") return value.trim().length > 0;
  return value !== undefined && value !== null;
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
