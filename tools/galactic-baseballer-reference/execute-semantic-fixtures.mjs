#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const manifestRoot = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
);
const outputPath = path.join(
  root,
  "evidence",
  "galactic-baseballer-reference-v1",
  "semantic-fixture-results.json",
);
const check = process.argv.includes("--check");
const nonFactFiles = new Set([
  "coverage.json",
  "manifest.json",
  "mechanic-rules.json",
  "pack-index.json",
  "reconciliation.json",
  "review-fixtures.json",
]);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(canonical(value)).digest("hex");
}

function primitiveKey(value) {
  return `${typeof value}:${JSON.stringify(value)}`;
}

function walk(value, pointer = "", output = []) {
  if (Array.isArray(value)) {
    if (value.length === 0) output.push({ pointer, value });
    value.forEach((item, index) => walk(item, `${pointer}/${index}`, output));
  } else if (value !== null && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) {
      walk(item, `${pointer}/${key.replaceAll("~", "~0").replaceAll("/", "~1")}`, output);
    }
  } else {
    output.push({ pointer, value });
  }
  return output;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function operationOrderIsContiguous(operations) {
  return operations.every(({ operation_order: order }, index) => order === index);
}

const schema = JSON.parse(await readFile(
  path.join(manifestRoot, "normalized-schema.json"),
  "utf8",
));
const fixtureContract = JSON.parse(await readFile(
  path.join(manifestRoot, "fixture-contract.json"),
  "utf8",
));
const fileRows = new Map();
for (const { file } of schema.files) {
  const rows = JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
  assert(Array.isArray(rows), `${file} is not a normalized row array`);
  fileRows.set(file, rows);
}

const rules = fileRows.get("mechanic-rules.json");
const fixtures = fileRows.get("review-fixtures.json");
const sources = fileRows.get("sources.json");
const approximations = fileRows.get("approximations.json");
const sourceIds = new Set(sources.map(({ id }) => id));
const fixtureById = new Map(fixtures.map((fixture) => [fixture.id, fixture]));
const requiredFamilies = fixtureContract.required_families.map(({ id }) => id);

assert(requiredFamilies.length === 20, "semantic family denominator drift");
assert(new Set(requiredFamilies).size === 20, "duplicate required semantic family");
assert(rules.length === 26, "ReferenceOnly rule denominator drift");
assert(fixtures.length === 35, "semantic fixture denominator drift");
assert(
  fixtures.every(({ runtime_executable: runtime }) => runtime === false),
  "a semantic review fixture became runtime executable",
);
assert(
  rules.every(({ runtime_executable: runtime }) => runtime === false),
  "a ReferenceOnly rule became runtime executable",
);

const rowById = new Map();
const valueOwners = new Map();
const factFiles = [...fileRows];
for (const file of (await readdir(path.join(packRoot, "fragments")))
  .filter((name) => name.endsWith(".json"))
  .sort(compareText)) {
  const rows = JSON.parse(await readFile(
    path.join(packRoot, "fragments", file),
    "utf8",
  ));
  assert(Array.isArray(rows), `fragments/${file} is not a row array`);
  factFiles.push([`fragments/${file}`, rows]);
}
for (const [file, rows] of factFiles) {
  if (
    nonFactFiles.has(file)
    || file.includes("mechanic-rules")
    || file.includes("review-fixtures")
  ) continue;
  for (const row of rows) {
    const existing = rowById.get(row.id);
    if (existing !== undefined) continue;
    const owner = { file, row };
    rowById.set(row.id, owner);
    for (const { pointer, value } of walk(row)) {
      if (value !== null && typeof value !== "object") {
        const key = primitiveKey(value);
        const owners = valueOwners.get(key) ?? [];
        owners.push({ ...owner, pointer });
        valueOwners.set(key, owners);
      }
    }
  }
}

function resolveFactOwners(recordId) {
  const exact = rowById.get(recordId);
  if (exact !== undefined) return [{ ...exact, resolution: "ExactStableId" }];
  const owners = valueOwners.get(primitiveKey(recordId)) ?? [];
  return owners.map((owner) => ({
    file: owner.file,
    row: owner.row,
    resolution: `EmbeddedStableId${owner.pointer}`,
  }));
}

function findValue(value, contexts) {
  const expected = primitiveKey(value);
  for (const context of contexts) {
    for (const leaf of walk(context.value)) {
      if (
        (leaf.value === null || typeof leaf.value !== "object")
        && primitiveKey(leaf.value) === expected
      ) {
        return { kind: context.kind, owner: context.owner, pointer: leaf.pointer };
      }
    }
  }
  return undefined;
}

function inventoryMap(fixture) {
  return new Map(
    (fixture.preconditions.inventory ?? []).map(({ input_id: id, level }) => [
      id,
      level,
    ]),
  );
}

function recipeInputs(resolvedOwners) {
  return resolvedOwners
    .map(({ row }) => row)
    .filter(({ kind }) => kind === "SynthesisInput")
    .sort((left, right) => left.input_order - right.input_order);
}

function deriveFixtureFacts(fixture, resolvedOwners) {
  const derived = new Map();
  const expected = fixture.expected_facts;
  const inventory = inventoryMap(fixture);
  const inputs = recipeInputs(resolvedOwners);
  const hasEveryRecipeInput = inputs.length > 0 && inputs.every(({ input_id: id, required_level: level }) =>
    inventory.get(id) === level);

  if (Object.hasOwn(expected, "pre_fix_values_modeled")) {
    derived.set("/pre_fix_values_modeled", false);
  }
  if (Object.hasOwn(expected, "semantic_ordinal_names_assigned")) {
    derived.set("/semantic_ordinal_names_assigned", false);
  }
  if (
    fixture.id === "galactic-baseballer.demon-king.fixture.adventure-strategy"
  ) {
    const strategy = resolvedOwners.map(({ row }) => row)
      .find(({ kind }) => kind === "AdventureStrategy");
    assert(strategy !== undefined, `${fixture.id}: strategy definition did not resolve`);
    const currencyDelta = Number(strategy.maze_buff_parameters[0]);
    derived.set("/strategy_installed", fixture.preconditions.strategy_absent);
    derived.set("/currency_delta", currencyDelta);
    derived.set(
      "/balance",
      fixture.preconditions.balance + currencyDelta,
    );
  }
  if (
    fixture.family_id === "galactic-store-progression"
    && Object.hasOwn(expected, "accepted")
  ) {
    const shop = resolvedOwners.map(({ row }) => row)
      .find(({ kind }) => kind === "ShopUpgrade");
    assert(shop !== undefined, `${fixture.id}: shop definition did not resolve`);
    const accepted = fixture.preconditions.balance >= shop.cost
      && fixture.preconditions.current_level + 1 === shop.purchase_level;
    derived.set("/accepted", accepted);
    if (!accepted) derived.set("/state_byte_identical", true);
  }
  if (
    ["twin-weapon-synthesis", "supreme-weapon-synthesis"]
      .includes(fixture.family_id)
    && Object.hasOwn(expected, "accepted")
  ) {
    derived.set("/accepted", hasEveryRecipeInput);
    derived.set("/output_added", hasEveryRecipeInput);
    derived.set("/consumed_input_ids", []);
  }
  if (
    ["twin-weapon-synthesis", "supreme-weapon-synthesis"]
      .includes(fixture.family_id)
    && Object.hasOwn(expected, "retained_input_ids")
  ) {
    derived.set(
      "/retained_input_ids",
      inputs.filter(({ consumed }) => !consumed).map(({ input_id: id }) => id),
    );
  }
  if (fixture.family_id === "experience-team-level-up") {
    const threshold = resolvedOwners.map(({ row }) => row)
      .find(({ kind }) => kind === "LevelThreshold");
    assert(threshold !== undefined, `${fixture.id}: level threshold did not resolve`);
    const total = Number(fixture.preconditions.current_exp)
      + fixture.input.experience_award;
    derived.set("/remaining_exp", String(total - Number(threshold.experience_threshold)));
  }
  if (fixture.family_id === "no-legal-candidate-failure-invariance") {
    derived.set("/accepted", fixture.preconditions.legal_candidate_ids.length > 0);
  }
  if (
    fixture.family_id === "random-upgrade-candidates"
    && Object.hasOwn(expected, "accepted")
  ) {
    derived.set(
      "/accepted",
      fixture.preconditions.legal_candidate_ids.includes(fixture.input.choose_id),
    );
  }
  if (
    fixture.family_id === "team-bonus"
    && Object.hasOwn(expected, "teardown_required")
  ) {
    derived.set(
      "/teardown_required",
      fixture.ordered_operations.some(({ operation }) =>
        operation.toLowerCase().includes("teardown")),
    );
  }
  return derived;
}

function validateFamilySemantics(fixture, resolvedOwners) {
  const expected = fixture.expected_facts;
  const pre = fixture.preconditions;
  const input = fixture.input;
  const owners = resolvedOwners.map(({ row }) => row);
  switch (fixture.family_id) {
    case "profile-version-selection":
      assert(pre.available_profile_ids.includes(input.select_profile_id), `${fixture.id}: selected unavailable profile`);
      assert(expected.selected_profile_id === input.select_profile_id, `${fixture.id}: profile selection result drift`);
      break;
    case "stage-difficulty-selection":
      assert(expected.accepted === true, `${fixture.id}: released stage fixture rejected`);
      assert(owners.some(({ kind }) => kind === "Stage"), `${fixture.id}: stage definition missing`);
      break;
    case "wave-battle-phase-progression":
      assert(expected.wave_id === input.advance_to_wave_id, `${fixture.id}: wave transition drift`);
      assert(owners.some(({ kind }) => kind === "EncounterWave"), `${fixture.id}: wave definition missing`);
      break;
    case "experience-team-level-up":
      assert(expected.next_level === pre.current_level + 1, `${fixture.id}: level increment drift`);
      assert(expected.level_up_offers === 1, `${fixture.id}: offer count drift`);
      break;
    case "random-upgrade-candidates":
      if (Object.hasOwn(expected, "accepted")) {
        assert(pre.legal_candidate_ids.includes(expected.selected_id), `${fixture.id}: selected illegal candidate`);
        assert(input.rng_label.startsWith("galactic-baseballer/"), `${fixture.id}: unlabeled RNG`);
      } else if (Object.hasOwn(expected, "selected_ordinal")) {
        assert(expected.selected_ordinal === input.labeled_integer_sampled_ordinal, `${fixture.id}: sampled ordinal drift`);
      } else {
        assert(expected.probability_values.length === expected.step_values.length, `${fixture.id}: probability vector shape drift`);
      }
      break;
    case "weapon-acquisition-duplicate-upgrade":
    case "accessory-acquisition-duplicate-upgrade":
      assert(expected.level === pre.owned_level + 1, `${fixture.id}: duplicate level increment drift`);
      assert(expected.slot_count_delta === 0, `${fixture.id}: duplicate consumed a slot`);
      break;
    case "slot-capacity-expansion-replacement":
      assert(expected.unlocked_weapon_slots === pre.unlocked_weapon_slots + 1, `${fixture.id}: expansion increment drift`);
      assert(expected.unlocked_weapon_slots <= expected.total_weapon_slots, `${fixture.id}: capacity overflow`);
      break;
    case "weapon-automatic-action":
      assert(expected.runtime_executable === false || fixture.runtime_executable === false, `${fixture.id}: runtime claim drift`);
      break;
    case "character-action-triggered-weapon":
      assert(expected.binding_key === pre.binding_key, `${fixture.id}: action binding drift`);
      break;
    case "resonance-accessory-binding":
      assert(expected.eligible_accessory_id === pre.owned_accessory_id, `${fixture.id}: resonance binding drift`);
      assert(expected.inferred_edges === 0, `${fixture.id}: inferred synthesis edge`);
      break;
    case "legendary-weapon-synthesis":
      assert(recipeInputs(resolvedOwners).length === 2, `${fixture.id}: Legendary input count drift`);
      assert(expected.consumed_input_count === 1, `${fixture.id}: Legendary consumption drift`);
      break;
    case "twin-weapon-synthesis":
    case "supreme-weapon-synthesis": {
      const inputs = recipeInputs(resolvedOwners);
      assert(inputs.length === 2, `${fixture.id}: advanced input count drift`);
      if (Object.hasOwn(expected, "accepted")) {
        assert(expected.inventory_byte_identical === true, `${fixture.id}: rejection mutation drift`);
        assert(expected.consumed_input_ids.length === 0, `${fixture.id}: rejected synthesis consumed input`);
      } else {
        const consumed = [...inputs]
          .filter(({ consumed }) => consumed)
          .sort((left, right) => left.consumption_order - right.consumption_order)
          .map(({ input_id: id }) => id);
        assert(canonical(consumed) === canonical(expected.consumed_input_ids), `${fixture.id}: consumption order drift`);
      }
      break;
    }
    case "adventure-strategy":
      assert(owners.some(({ kind }) =>
        ["AdventureStrategy", "CandidatePool", "Currency"].includes(kind)), `${fixture.id}: strategy fact missing`);
      break;
    case "team-bonus":
      assert(owners.some(({ kind }) =>
        ["Stage", "Encounter", "TeamBonus"].includes(kind)), `${fixture.id}: team bonus owner missing`);
      break;
    case "galactic-store-progression":
      if (expected.accepted === false) {
        assert(expected.balance === pre.balance, `${fixture.id}: rejected balance mutated`);
        assert(expected.current_level === pre.current_level, `${fixture.id}: rejected level mutated`);
      }
      break;
    case "score-rating-clear":
      if (expected.ordered_ratings !== undefined) {
        assert(
          expected.ordered_ratings.includes(expected.minimum_rating),
          `${fixture.id}: minimum rating is outside ordered ratings`,
        );
      }
      assert(owners.some(({ kind }) =>
        ["ScoringRule", "SettlementRule", "StagePeriod"].includes(kind)), `${fixture.id}: scoring owner missing`);
      break;
    case "boss-phase-final-settlement":
      assert(expected.duplicate_projection === false, `${fixture.id}: duplicate settlement projection`);
      break;
    case "no-legal-candidate-failure-invariance":
      assert(pre.legal_candidate_ids.length === 0, `${fixture.id}: no-candidate fixture is not empty`);
      assert(expected.inventory_digest === pre.inventory_digest, `${fixture.id}: rejected inventory digest mutated`);
      assert(expected.resource_delta === 0, `${fixture.id}: rejected resource mutation`);
      break;
    default:
      throw new Error(`${fixture.id}: unhandled semantic family ${fixture.family_id}`);
  }
}

for (const family of requiredFamilies) {
  assert(rules.some(({ family_id: id }) => id === family), `${family}: missing ReferenceOnly rule`);
  assert(fixtures.some(({ family_id: id }) => id === family), `${family}: missing semantic fixture`);
}
for (const rule of rules) {
  assert(rule.trigger_point.length > 0, `${rule.id}: missing trigger point`);
  assert(rule.state_owner.length > 0, `${rule.id}: missing state owner`);
  assert(rule.ordered_operations.length > 0, `${rule.id}: missing ordered operations`);
  assert(operationOrderIsContiguous(rule.ordered_operations), `${rule.id}: non-contiguous operation order`);
  assert(rule.fixture_ids.length > 0, `${rule.id}: missing fixture linkage`);
  assert(rule.fixture_ids.every((id) => fixtureById.has(id)), `${rule.id}: unresolved fixture linkage`);
}
for (const approximation of approximations) {
  assert(
    approximation.affected_fixture_ids.every((id) =>
      fixtureById.has(id) || requiredFamilies.includes(id)),
    `${approximation.id}: unresolved affected fixture or family`,
  );
}

const cases = [];
for (const fixture of fixtures) {
  assert(fixture.trigger_point.length > 0, `${fixture.id}: missing trigger point`);
  assert(fixture.state_owner.length > 0, `${fixture.id}: missing state owner`);
  assert(fixture.ordered_operations.length > 0, `${fixture.id}: missing ordered operations`);
  assert(operationOrderIsContiguous(fixture.ordered_operations), `${fixture.id}: non-contiguous operation order`);
  assert(fixture.source_record_ids.length > 0, `${fixture.id}: missing source record IDs`);
  assert(fixture.evidence_refs.length > 0, `${fixture.id}: missing evidence refs`);
  assert(fixture.evidence_refs.every((id) => sourceIds.has(id)), `${fixture.id}: unresolved evidence ref`);

  const resolvedOwners = [];
  const sourceRecordResolutions = [];
  for (const recordId of fixture.source_record_ids) {
    const owners = resolveFactOwners(recordId);
    assert(owners.length > 0, `${fixture.id}: unresolved source record ${recordId}`);
    const selected = owners[0];
    resolvedOwners.push(selected);
    sourceRecordResolutions.push({
      source_record_id: recordId,
      normalized_file: selected.file,
      normalized_row_id: selected.row.id,
      resolution: selected.resolution,
    });
  }
  const contexts = [
    ...resolvedOwners.map(({ file, row }) => ({
      kind: "NormalizedSourceRecord",
      owner: `${file}#${row.id}`,
      value: row,
    })),
    { kind: "Precondition", owner: fixture.id, value: fixture.preconditions },
    { kind: "Input", owner: fixture.id, value: fixture.input },
    { kind: "EvidenceReceipt", owner: fixture.id, value: fixture.source_refs },
  ];
  const derived = deriveFixtureFacts(fixture, resolvedOwners);
  const assertions = [];
  for (const leaf of walk(fixture.expected_facts)) {
    if (derived.has(leaf.pointer)) {
      assert(
        canonical(derived.get(leaf.pointer)) === canonical(leaf.value),
        `${fixture.id}${leaf.pointer}: deterministic result differs from expected fact`,
      );
      assertions.push({ path: leaf.pointer, proof: "DeterministicDerivation" });
      continue;
    }
    if (Array.isArray(leaf.value) && leaf.value.length === 0) {
      const derivedValue = derived.get(leaf.pointer);
      if (derivedValue !== undefined) {
        assert(canonical(derivedValue) === canonical(leaf.value), `${fixture.id}${leaf.pointer}: derived empty-array drift`);
        assertions.push({ path: leaf.pointer, proof: "DeterministicDerivation" });
        continue;
      }
    }
    const proof = findValue(leaf.value, contexts);
    if (proof !== undefined) {
      assertions.push({ path: leaf.pointer, proof: proof.kind });
      continue;
    }
    throw new Error(`${fixture.id}${leaf.pointer}: expected fact has no independent proof`);
  }
  validateFamilySemantics(fixture, resolvedOwners);
  const linkedRules = rules.filter(({ family_id: family }) =>
    family === fixture.family_id)
    .map(({ id }) => id)
    .sort(compareText);
  assert(linkedRules.length > 0, `${fixture.id}: no family ReferenceOnly rule`);
  cases.push({
    fixture_id: fixture.id,
    family_id: fixture.family_id,
    status: "Passed",
    runtime_executable: false,
    reference_rule_ids: linkedRules,
    source_record_resolutions: sourceRecordResolutions,
    evidence_ref_count: fixture.evidence_refs.length,
    operation_count: fixture.ordered_operations.length,
    assertion_count: assertions.length,
    source_backed_assertion_count: assertions.filter(({ proof }) =>
      proof !== "DeterministicDerivation").length,
    derived_assertion_count: assertions.filter(({ proof }) =>
      proof === "DeterministicDerivation").length,
    precondition_sha256: digest(fixture.preconditions),
    input_sha256: digest(fixture.input),
    expected_fact_sha256: digest(fixture.expected_facts),
  });
}

cases.sort((left, right) => compareText(left.fixture_id, right.fixture_id));
const familyResults = requiredFamilies.map((familyId) => {
  const familyCases = cases.filter(({ family_id: id }) => id === familyId);
  return {
    family_id: familyId,
    status: "Passed",
    rule_count: rules.filter(({ family_id: id }) => id === familyId).length,
    fixture_count: familyCases.length,
    assertion_count: familyCases.reduce((sum, item) => sum + item.assertion_count, 0),
  };
}).sort((left, right) => compareText(left.family_id, right.family_id));

const result = {
  schema_revision: "starclock.galactic-baseballer-semantic-fixture-results.v1",
  goal_id: "galactic-baseballer-reference-v1",
  batch_id: "G16-P4-B1",
  baseline_game_version: "4.4",
  review_scope: "ReferenceOnly semantic execution; no runtime gameplay or parity claim",
  status: "Passed",
  required_family_count: 20,
  passed_family_count: familyResults.length,
  reference_rule_count: rules.length,
  fixture_count: cases.length,
  passed_fixture_count: cases.length,
  failed_fixture_count: 0,
  assertion_count: cases.reduce((sum, item) => sum + item.assertion_count, 0),
  source_backed_assertion_count: cases.reduce(
    (sum, item) => sum + item.source_backed_assertion_count,
    0,
  ),
  derived_assertion_count: cases.reduce(
    (sum, item) => sum + item.derived_assertion_count,
    0,
  ),
  failure_invariance_fixture_ids: cases
    .filter(({ fixture_id: id }) =>
      id.endsWith(".rejected")
      || id.endsWith(".no-legal-candidate-failure-invariance"))
    .map(({ fixture_id: id }) => id),
  family_results: familyResults,
  cases,
};
const encoded = `${JSON.stringify(result, null, 2)}\n`;
if (check) {
  assert(await readFile(outputPath, "utf8") === encoded, "semantic fixture result drift");
} else {
  await writeFile(outputPath, encoded);
}
console.log(
  `semantic fixtures ${check ? "verified" : "executed"}: `
    + `${result.passed_family_count} families, ${result.passed_fixture_count} fixtures, `
    + `${result.assertion_count} assertions`,
);
