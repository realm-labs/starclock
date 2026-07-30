#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const outputRoot = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
);
const goalId = "galactic-baseballer-reference-v1";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const manifestPath = path.join(outputRoot, "content-manifest.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const expectedManifestSha256 =
  "92bf516ebb2c0baec8df4bbc5ccd435d090181fb4553db0261a4ce49a5b032a4";
const actualManifestSha256 = sha256(await readFile(manifestPath));
if (actualManifestSha256 !== expectedManifestSha256)
  throw new Error("P0-B3 content manifest digest drift");

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function sha256(value) {
  return createHash("sha256").update(
    Buffer.isBuffer(value) ? value : JSON.stringify(value),
  ).digest("hex");
}

const canonicalEncoding = {
  encoding: "UTF-8",
  line_endings: "LF",
  unicode_normalization: "NFC",
  indent_spaces: 2,
  terminal_newline: true,
  object_key_order: "schema declaration order, then lexicographic extension fields",
  array_order: "explicit ordering_keys; never filesystem, hash-map or object iteration order",
  set_array_order: "stable Starclock ID lexicographic unless a numeric source ordinal is exact",
  semantic_sequence_order: "preserve declared stage, candidate, operation, recipe, wave, slot and settlement order",
  null_policy: "omit absent optional values; null is forbidden as a second absence representation",
  boolean_policy: "JSON true/false only",
  integer_policy: "JSON integer only inside signed 53-bit range; otherwise source_numeric_id string",
  decimal_policy: "canonical_decimal strings only; never binary floating cells",
  digest_policy: "SHA-256 over encoded bytes; pack-index.json is excluded from its own pack digest",
};

const commonEnvelope = {
  required_fields: [
    "id",
    "schema_revision",
    "kind",
    "name_en",
    "name_zh_cn",
    "summary_en",
    "summary_zh_cn",
    "profile_ids",
    "ownership",
    "coverage_state",
    "evidence_quality",
    "mechanism_quality",
    "manifest_record_ids",
    "source_refs",
    "tags",
  ],
  id: {
    type: "string",
    pattern: "^[a-z0-9][a-z0-9._:-]*$",
    global_uniqueness: true,
  },
  schema_revision: { type: "string", value: rowRevision },
  profile_ids: {
    type: "array",
    minimum: 1,
    unique: true,
    ordering: "lexicographic",
    values: manifest.profiles,
  },
  ownership: {
    enum: ["Departure", "DemonKing", "SharedBase", "Shared"],
  },
  coverage_state: {
    enum: ["Cataloged", "Researched", "DataReady", "Blocked", "EvidenceOnly"],
  },
  evidence_quality: {
    enum: [
      "ExactStructured",
      "ExactPublicText",
      "Observed",
      "ApproximateFromReleasedText",
      "ProjectPolicy",
    ],
  },
  mechanism_quality: {
    enum: [
      "ExactProgram",
      "ExactRelationship",
      "ObservedBehavior",
      "IdentityCrossCheck",
      "PolicyBoundary",
      "ContextOnly",
    ],
  },
  manifest_record_ids: {
    type: "array",
    minimum: 1,
    unique: true,
    ordering: "lexicographic",
    resolution: "P0-B3 content-manifest category record",
  },
  source_refs: {
    type: "array",
    minimum: 1,
    ordered: true,
    item_type: "source_ref",
  },
  tags: { type: "array", unique: true, ordering: "lexicographic" },
};

const fileSpecs = [
  ["profiles.json", "Profile", "P1-B1", ["profiles"], ["id"]],
  ["release-boundaries.json", "ReleaseBoundary", "P1-B1", ["profiles"], ["release_order", "id"]],
  ["stages.json", "Stage", "P1-B1", ["profile_stages"], ["profile_id", "difficulty", "id"]],
  ["stage-periods.json", "StagePeriod", "P1-B1", ["stage_periods"], ["profile_id", "period_order", "id"]],
  ["weapons.json", "Weapon", "P1-B2", ["weapon_collections"], ["profile_id", "id"]],
  ["weapon-levels.json", "WeaponLevel", "P1-B2", ["weapon_levels"], ["weapon_id", "level", "id"]],
  ["weapon-triggers.json", "WeaponTrigger", "P1-B2", ["weapon_levels", "config_programs"], ["weapon_id", "trigger_order", "id"]],
  ["accessories.json", "Accessory", "P1-B2", ["accessory_levels"], ["profile_id", "id"]],
  ["accessory-levels.json", "AccessoryLevel", "P1-B2", ["accessory_levels"], ["accessory_id", "level", "id"]],
  ["accessory-bindings.json", "AccessoryBinding", "P1-B2", ["accessory_levels"], ["accessory_id", "binding_order", "id"]],
  ["synthesis-recipes.json", "SynthesisRecipe", "P1-B2/P2-B2", ["synthesis_materials"], ["profile_id", "tier", "id"]],
  ["synthesis-inputs.json", "SynthesisInput", "P1-B2/P2-B2", ["synthesis_materials"], ["recipe_id", "input_order", "id"]],
  ["level-thresholds.json", "LevelThreshold", "P1-B3", ["mode_constants"], ["profile_id", "level", "id"]],
  ["candidate-pools.json", "CandidatePool", "P1-B3", ["upgrade_cards", "upgrade_card_types", "offer_box_groups", "offer_box_items"], ["profile_id", "id"]],
  ["candidate-policies.json", "CandidatePolicy", "P1-B3", ["mode_constants"], ["profile_id", "decision_order", "id"]],
  ["inventory-slots.json", "InventorySlotPolicy", "P1-B3", ["mode_constants"], ["profile_id", "slot_kind", "id"]],
  ["inventory-operations.json", "InventoryOperation", "P1-B3", ["mode_constants"], ["profile_id", "operation_order", "id"]],
  ["encounters.json", "Encounter", "P1-B4/P2-B4", ["shared_stage_configs", "infinite_stage_groups"], ["profile_id", "stage_id", "id"]],
  ["waves.json", "EncounterWave", "P1-B4/P2-B4", ["infinite_waves"], ["encounter_id", "wave_order", "id"]],
  ["enemy-slots.json", "EnemySlot", "P1-B4/P2-B4", ["infinite_monster_groups"], ["wave_id", "slot_order", "id"]],
  ["enemies.json", "EnemyIdentity", "P1-B4/P2-B4", ["enemy_variants", "enemy_templates"], ["source_numeric_id", "id"]],
  ["enemy-skills.json", "EnemySkill", "P1-B4/P2-B4", ["enemy_skills"], ["enemy_id", "skill_order", "id"]],
  ["enemy-statuses.json", "EnemyStatus", "P1-B4/P2-B4", ["enemy_statuses"], ["source_numeric_id", "id"]],
  ["scoring-rules.json", "ScoringRule", "P1-B4/P2-B4", ["stage_periods", "mode_constants"], ["profile_id", "evaluation_order", "id"]],
  ["settlement-rules.json", "SettlementRule", "P1-B4/P2-B4", ["profile_stages", "stage_periods"], ["profile_id", "settlement_order", "id"]],
  ["profile-differences.json", "ProfileDifference", "P2-B1", ["profiles"], ["difference_order", "id"]],
  ["adventure-strategies.json", "AdventureStrategy", "P2-B3", ["upgrade_cards", "upgrade_card_types"], ["profile_id", "id"]],
  ["progression.json", "ProgressionRule", "P2-B3", ["mode_constants"], ["profile_id", "progression_order", "id"]],
  ["currencies.json", "Currency", "P2-B3", ["shop_progression"], ["profile_id", "id"]],
  ["shop-upgrades.json", "ShopUpgrade", "P2-B3", ["shop_progression"], ["profile_id", "upgrade_order", "id"]],
  ["unlocks.json", "UnlockRule", "P2-B3", ["profile_stages", "shop_progression"], ["profile_id", "unlock_order", "id"]],
  ["mechanic-rules.json", "MechanicRule", "P3-B1", ["semantic_fixture_families"], ["family_id", "id"]],
  ["sources.json", "Source", "P3-B1", Object.keys(manifest.categories), ["id"]],
  ["approximations.json", "Approximation", "P3-B1", ["semantic_fixture_families"], ["field_path", "id"]],
  ["reconciliation.json", "ReconciliationReceipt", "P3-B1", ["shared_stage_configs", "enemy_variants"], ["id"]],
  ["coverage.json", "CoverageRow", "P3-B1", Object.keys(manifest.categories), ["category_id", "record_id"]],
  ["research-gaps.json", "ResearchGap", "P3-B1", ["semantic_fixture_families"], ["state", "id"]],
  ["review-fixtures.json", "SemanticReviewFixture", "P3-B1", ["semantic_fixture_families"], ["family_id", "id"]],
  ["manifest.json", "PackManifest", "P3-B1", Object.keys(manifest.categories), ["id"]],
  ["pack-index.json", "PackIndex", "P3-B1", Object.keys(manifest.categories), ["id"]],
];

const normalizedSchema = {
  schema_revision: "starclock.galactic-baseballer-normalized-schema.v1",
  row_schema_revision: rowRevision,
  goal_id: goalId,
  profiles: manifest.profiles,
  authoritative_surface: "xlsx",
  normalized_root: "content-reference/galactic-baseballer-v1",
  common_envelope: commonEnvelope,
  types: {
    canonical_decimal: {
      storage: "string",
      pattern: "^(0|-?(?:[1-9][0-9]*(?:\\.[0-9]*[1-9])?|0\\.[0-9]*[1-9]))$",
      forbid: [
        "binary floating point",
        "exponent notation",
        "leading plus",
        "negative zero",
        "trailing fractional zero",
      ],
    },
    source_numeric_id: {
      storage: "string",
      reason: "preserve upstream IDs and TextMap hashes without JavaScript-safe-integer loss",
    },
    stable_ref: {
      storage: "string",
      resolution: "Goal 16 pack index or explicitly inherited stable identity",
      unknown_behavior: "reject",
    },
    source_ref: {
      required_fields: [
        "source_id",
        "repository_or_url",
        "revision_or_access_date",
        "game_version",
        "path_or_page",
        "locator",
        "sha256",
        "evidence_quality",
        "mechanism_quality",
        "note",
      ],
      optional_fields: ["replacement_condition"],
      approximation_rule:
        "ApproximateFromReleasedText and ProjectPolicy require a nonempty replacement_condition",
    },
    approximation: {
      required_fields: [
        "field_path",
        "unavailable_fact",
        "known_released_facts",
        "selected_policy",
        "rejected_alternatives",
        "rationale",
        "affected_fixture_ids",
        "confidence",
        "replacement_condition",
      ],
      minimum_rejected_alternatives: 2,
      confidence_enum: ["Low", "Medium", "High"],
      exact_claim_forbidden: true,
    },
    labeled_rng_stream: {
      label_format:
        "galactic-baseballer/{profile-id}/{activity-instance-id}/{decision-kind}/{decision-ordinal}",
      sampling: "project integer sampling only",
      candidates: "stable ordered Starclock IDs",
      forbidden: ["system RNG", "thread RNG", "floating probability draw", "generic shuffle"],
    },
  },
  canonical_encoding: canonicalEncoding,
  lifecycle_contract: {
    definitions_separate_from_state: true,
    battle_state_owner: "starclock-combat inherited identity and formulas; this pack defines reference-only contributions",
    activity_state_owner: "future starclock-activity mode profile; no runtime implementation in Goal 16",
    ordered_boundaries: [
      "profile and stage selection",
      "loadout and initial weapon projection",
      "battle contribution installation",
      "wave/phase start",
      "enemy defeat and experience award",
      "team level threshold crossing",
      "candidate generation and player decision",
      "inventory acquisition/upgrade/synthesis",
      "automatic and character-triggered weapon actions",
      "score/rating/clear evaluation",
      "battle settlement and profile progression projection",
      "teardown",
    ],
    rejected_command: "fixture must assert byte-identical authoritative preconditions",
    unknown_ordering:
      "ApproximateFromReleasedText or ProjectPolicy with two rejected alternatives and replacement condition",
    runtime_claim: false,
  },
  files: fileSpecs.map(([file, recordKind, phase, categoryInputs, orderingKeys]) => ({
    file,
    record_kind: recordKind,
    phase,
    manifest_category_inputs: categoryInputs,
    ordering_keys: orderingKeys,
  })),
};

const workbookSpecs = [
  {
    file: "GalacticBaseballerProfiles.xlsx",
    purpose: "profiles, releases, stages, growth, inventory, strategies, progression, currencies and store",
    normalized_files: fileSpecs.slice(0, 4).map(([file]) => file).concat(
      fileSpecs.slice(12, 17).map(([file]) => file),
      fileSpecs.slice(25, 31).map(([file]) => file),
    ),
  },
  {
    file: "GalacticBaseballerArsenal.xlsx",
    purpose: "weapons, accessories, trigger bindings and all synthesis tiers",
    normalized_files: fileSpecs.slice(4, 12).map(([file]) => file),
  },
  {
    file: "GalacticBaseballerEncounters.xlsx",
    purpose: "encounters, waves, enemies, skills, statuses, scores and settlement",
    normalized_files: fileSpecs.slice(17, 25).map(([file]) => file),
  },
  {
    file: "GalacticBaseballerReview.xlsx",
    purpose: "rules, sources, approximations, reconciliation, coverage, fixtures and pack identity",
    normalized_files: fileSpecs.slice(31).map(([file]) => file),
  },
];
const authoringContract = {
  schema_revision: "starclock.galactic-baseballer-authoring-contract.v1",
  goal_id: goalId,
  authority: {
    authoritative_format: "xlsx",
    editor: "python-openpyxl",
    editor_version: "3.1.5",
    schema_exporter: "sora-cli",
    schema_exporter_version: "0.3.0",
    production_artifact: "sora",
    json_role: "research-staging-and-debug-only",
    runtime_loading: false,
  },
  isolation: {
    project: "config/galactic-baseballer/project.toml",
    workbook_root: "config/galactic-baseballer/data",
    generated_root: "config/galactic-baseballer-generated",
    normalized_root: "content-reference/galactic-baseballer-v1",
    tool_root: "tools/galactic-baseballer-reference",
    evidence_root: "evidence/galactic-baseballer-reference-v1",
    forbidden_outputs: [
      "config/generated",
      "config/universe-generated",
      "config/gold-and-gears-generated",
      "config/swarm-disaster-generated",
      "config/unknowable-domain-generated",
      "config/divergent-universe-generated",
      "config/currency-wars-generated",
      "config/anomaly-arbitration-generated",
    ],
  },
  generation: {
    complete_workbook_set: true,
    overwrite_existing_target: false,
    patch_designer_workbook: false,
    fixed_document_timestamps: true,
    double_generation_byte_identical: true,
    formula_cells: "reject",
    excel_error_cells: "reject",
    unknown_columns: "reject",
    unknown_sheets: "reject",
    unknown_references: "reject",
  },
  sheet_contract: {
    sora_metadata_rows: "preserve rows 1 through 7 exactly",
    data_start_row: 8,
    freeze_panes: "A8",
    auto_filter: true,
    header_style: "galactic-baseballer-header-v1",
    alternating_rows: true,
    wrapped_text: true,
    deterministic_widths: true,
    data_validation: true,
    typed_references: true,
    canonical_decimal_cells: "text",
    source_numeric_id_cells: "text",
  },
  workbooks: workbookSpecs,
  table_family_contract: {
    one_primary_table_per_normalized_file: true,
    child_tables:
      "repeated typed fields require parent stable ID plus deterministic ordinal",
    table_names: "Gb plus PascalCase normalized file stem; child tables append a semantic role",
    sheet_names: "explicit in the Sora schema and at most 31 characters",
    numeric_keys: "private workbook identities only",
    stable_keys: "required for every cross-workbook, manifest or inherited reference",
    shared_rows: "reference inherited stable IDs and retain source path, locator and evidence digest",
  },
  visual_review: {
    render_every_sheet: true,
    render_every_schema_column: true,
    inspect: [
      "Sora metadata rows",
      "header clipping",
      "wrapped bilingual text",
      "column widths",
      "freeze panes",
      "filters",
      "validation prompts",
      "empty-table readability",
      "cross-workbook stable IDs",
    ],
    artifact_root: "evidence/galactic-baseballer-reference-v1/workbook-review",
  },
  acceptance: {
    schema_check: "sora check --project config/galactic-baseballer/project.toml",
    schema_build: "sora build --project config/galactic-baseballer/project.toml",
    export: "sora export --project config/galactic-baseballer/project.toml",
    reader_load: "every generated table and every row through the isolated generated reader",
    workbook_semantics:
      "canonical values, cell types, formulas, validations, sheet order, table identity and typed references",
    visual_review: "render and inspect every authored sheet and schema field column",
    clean_target: "generate complete outputs into a new path; never overwrite a designer-edited workbook",
  },
};

const familySpecs = [
  ["profile-version-selection", ["two independent profiles", "retained release", "legal selection", "invalid profile rejection"]],
  ["stage-difficulty-selection", ["profile-owned stages", "unlock", "difficulty", "initial weapon", "invalid stage rejection"]],
  ["wave-battle-phase-progression", ["stage period", "ordered wave", "phase transition", "boss boundary", "carry/reset"]],
  ["experience-team-level-up", ["enemy defeat", "experience award", "threshold", "multi-level crossing", "maximum level"]],
  ["random-upgrade-candidates", ["eligible pool", "stable candidate order", "labeled RNG", "refresh", "skip", "no legal candidate"]],
  ["weapon-acquisition-duplicate-upgrade", ["empty slot", "new weapon", "duplicate", "maximum level", "failure invariance"]],
  ["accessory-acquisition-duplicate-upgrade", ["empty slot", "new accessory", "duplicate", "maximum level", "failure invariance"]],
  ["slot-capacity-expansion-replacement", ["weapon slots", "accessory slots", "expansion", "full slots", "replacement order"]],
  ["weapon-automatic-action", ["trigger point", "counter/cooldown", "target order", "ordered operations", "teardown"]],
  ["character-action-triggered-weapon", ["basic", "skill", "ultimate", "follow-up", "DoT", "break", "summon", "once scope"]],
  ["resonance-accessory-binding", ["weapon/accessory relationship", "eligibility", "install", "remove", "unrelated accessory"]],
  ["legendary-weapon-synthesis", ["maximum weapon", "required accessory", "candidate", "consume order", "failure invariance"]],
  ["twin-weapon-synthesis", ["Demon King profile", "two weapon inputs", "candidate", "consume order", "acyclic graph"]],
  ["supreme-weapon-synthesis", ["Demon King profile", "advanced prerequisites", "candidate", "consume order", "acyclic graph"]],
  ["adventure-strategy", ["offer", "acquisition", "level", "effect", "profile boundary"]],
  ["team-bonus", ["stage binding", "install timing", "battle-visible contribution", "stacking", "teardown"]],
  ["galactic-store-progression", ["reputation", "Raccoon Coin", "mechanical upgrade", "cost", "unlock", "account reward exclusion"]],
  ["score-rating-clear", ["damage", "kill", "boss damage", "adventure score", "rating threshold", "clear"]],
  ["boss-phase-final-settlement", ["boss entry", "phase transition", "final score", "settlement", "profile progression projection"]],
  ["no-legal-candidate-failure-invariance", ["empty pool", "full/max inventory", "invalid synthesis", "ordered rejection", "byte-identical state"]],
];

const fixtureContract = {
  schema_revision: "starclock.galactic-baseballer-fixture-contract.v1",
  goal_id: goalId,
  fixture_role: "normalized ReferenceOnly semantic review; no runtime executability or parity claim",
  minimum_cases_per_family: 1,
  required_fields: [
    "id",
    "family_id",
    "name_en",
    "name_zh_cn",
    "source_record_ids",
    "trigger_point",
    "state_owner",
    "preconditions",
    "input",
    "ordered_operations",
    "expected_facts",
    "evidence_refs",
    "evidence_quality",
    "mechanism_quality",
  ],
  field_contracts: {
    source_record_ids: "nonempty sorted stable-ID list closing manifest obligations",
    trigger_point: "explicit lifecycle boundary; never implied by prose",
    state_owner: "battle-local, activity/profile-local or immutable definition owner",
    preconditions: "typed canonical facts only",
    input: "concrete record IDs, seed/label where random, and typed decision or observation",
    ordered_operations: "nonempty deterministic sequence using canonical decimals and stable IDs",
    expected_facts: "nonempty typed assertions including rejection/invariance where applicable",
    evidence_refs: "nonempty ordered source references",
  },
  determinism: {
    random_selection: normalizedSchema.types.labeled_rng_stream,
    ties: "released authored order; otherwise stable Starclock ID ascending with ProjectPolicy evidence",
    rejected_input: "authoritative preconditions and expected facts remain byte-identical",
    no_legal_candidate: "explicit outcome and continuation; never silent or inferred",
    simultaneous_synthesis_and_upgrade: "explicit ordered fixture and approximation record until exact evidence replaces it",
    decimal_values: "canonical decimal strings",
    operation_order: "asserted in ordered_operations and expected_facts",
  },
  approximation: {
    allowed_labels: ["ApproximateFromReleasedText", "ProjectPolicy"],
    required_fields: normalizedSchema.types.approximation.required_fields,
    minimum_rejected_alternatives: 2,
    exact_claim_forbidden: true,
  },
  required_families: familySpecs.map(([id, mustCover]) => ({
    id,
    minimum_cases: 1,
    must_cover: mustCover,
  })),
  coverage_rule: {
    family_coverage: "every required family has at least one semantic review fixture",
    rule_coverage: "every family has at least one ReferenceOnly MechanicRule",
    manifest_coverage: "all 2,207 target records become DataReady without reducing the denominator",
    evidence_only_coverage: "all 25 excluded locator records remain accounted for",
    approximation_coverage: "every approximate/policy field resolves to a complete approximation record and affected fixture",
    blocking_gap: "no fixture satisfies a Blocked manifest obligation",
  },
};

function approximation(
  id,
  fieldPath,
  unavailableFact,
  knownFacts,
  selectedPolicy,
  alternatives,
  rationale,
  fixtures,
  confidence,
  replacementCondition,
) {
  return {
    id,
    schema_revision: "starclock.galactic-baseballer-approximation.v1",
    field_path: fieldPath,
    unavailable_fact: unavailableFact,
    known_released_facts: knownFacts,
    selected_policy: selectedPolicy,
    rejected_alternatives: alternatives,
    rationale,
    affected_fixture_ids: fixtures,
    confidence,
    evidence_quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
    replacement_condition: replacementCondition,
  };
}

const approximations = [
  approximation(
    "gb.policy.upgrade-candidate-weight",
    "candidate_policies.random_weight",
    "released sources do not expose a complete authoritative draw weight for every eligible offer",
    ["only legal profile-owned candidates may be offered", "duplicate and maximum-level eligibility are state dependent"],
    "sample uniformly by project integer sampling from eligible stable IDs sorted lexicographically",
    ["infer weights from rarity or display position", "reuse upstream table iteration order as a probability distribution"],
    "uniform stable-ID sampling is deterministic, neutral and does not manufacture hidden rarity weights",
    ["random-upgrade-candidates", "no-legal-candidate-failure-invariance"],
    "Low",
    "replace when released structured weights or a reproducible released-version observation establishes the complete distribution",
  ),
  approximation(
    "gb.policy.upgrade-candidate-order",
    "candidate_policies.presentation_order",
    "the stable order of simultaneously drawn choices is not fully published",
    ["candidate membership is profile- and state-scoped", "each random decision must be replayable"],
    "preserve labeled draw ordinal; break duplicate/tie positions by stable ID ascending",
    ["sort by localized display name", "preserve filesystem or hash-map iteration order"],
    "draw ordinal preserves causality and stable ID removes locale/platform dependence",
    ["random-upgrade-candidates"],
    "Medium",
    "replace when released data exposes an explicit offer ordinal or a reproducible observation proves a different order",
  ),
  approximation(
    "gb.policy.no-legal-candidate",
    "candidate_policies.no_legal_candidate",
    "released text does not define every empty-pool/full-inventory combination",
    ["an illegal candidate cannot be selected", "rejected choices must not mutate authoritative state"],
    "emit a ReferenceOnly no-candidate outcome, consume no inventory resource and continue at the next declared lifecycle boundary",
    ["retry an unbounded random draw", "silently replace or downgrade an owned maximum-level item"],
    "bounded explicit failure preserves state and avoids nontermination",
    ["random-upgrade-candidates", "no-legal-candidate-failure-invariance"],
    "Low",
    "replace when released structured logic or reproducible observation defines each empty-candidate branch",
  ),
  approximation(
    "gb.policy.simultaneous-synthesis-order",
    "synthesis_recipes.simultaneous_upgrade_order",
    "the order between a newly eligible synthesis and an ordinary duplicate upgrade in one offer is not public for every recipe tier",
    ["recipes require explicit acyclic inputs", "inputs may be consumed only after all prerequisites validate"],
    "evaluate explicit synthesis candidates first by tier then recipe stable ID; otherwise apply the ordinary duplicate upgrade",
    ["always apply duplicate upgrade before synthesis", "choose an eligible recipe using an unlabeled random draw"],
    "tier and stable-ID order is bounded, auditable and keeps recipe consumption atomic",
    ["legendary-weapon-synthesis", "twin-weapon-synthesis", "supreme-weapon-synthesis"],
    "Low",
    "replace when released program evidence or a reproducible observation proves the exact simultaneous resolution order",
  ),
  approximation(
    "gb.policy.weapon-trigger-tie",
    "weapon_triggers.same_boundary_order",
    "same-boundary ordering between multiple ready weapons is not fully described in released text",
    ["each weapon retains its own counter/cooldown state", "all accepted actions require an explicit ordered operation trace"],
    "order ready weapon triggers by trigger phase, authored priority when present, then stable weapon ID",
    ["order by acquisition time", "order by localized weapon name"],
    "phase/priority preserves known timing while stable ID is replay-safe",
    ["weapon-automatic-action", "character-action-triggered-weapon"],
    "Medium",
    "replace when released structured priority data or reproducible observations establish a different tie-break",
  ),
  approximation(
    "gb.policy.target-tie",
    "weapon_triggers.target_tie_break",
    "several weapon target selectors do not publish tie-breaking among equally eligible combatants",
    ["target shape and eligibility remain weapon-specific", "candidate iteration cannot depend on hash-map order"],
    "sort eligible targets by the project stable combatant key before labeled integer selection or first-target choice",
    ["use battlefield insertion order without an authored contract", "use a floating random shuffle"],
    "stable combatant identity preserves deterministic replay without changing target eligibility",
    ["weapon-automatic-action", "character-action-triggered-weapon"],
    "Medium",
    "replace when released target-order data or reproducible observations prove a different selector",
  ),
  approximation(
    "gb.policy.refresh-exclusion",
    "candidate_policies.refresh_exclusion",
    "released sources do not fully specify whether every just-rejected candidate remains eligible after refresh",
    ["refresh generates a new legal offer", "maximum-level and profile-ineligible records remain illegal"],
    "exclude the immediately displayed candidate IDs for one refresh attempt, then fall back to the full legal pool if exclusion would empty it",
    ["allow the identical offer without an explicit fallback", "permanently exclude every previously seen candidate for the battle"],
    "one-attempt exclusion makes refresh meaningful while the bounded fallback prevents an artificial dead end",
    ["random-upgrade-candidates", "no-legal-candidate-failure-invariance"],
    "Low",
    "replace when released logic or reproducible observations define refresh exclusion memory and fallback",
  ),
  approximation(
    "gb.policy.score-rounding",
    "scoring_rules.intermediate_rounding",
    "released score displays do not expose every intermediate rounding boundary",
    ["final displayed scores and exact structured thresholds are integers", "authoritative arithmetic cannot use binary floats"],
    "retain canonical fixed-point intermediates and round toward zero only at each explicitly authored integer score contribution boundary",
    ["round after every multiplication", "banker's-round only the final aggregate"],
    "named contribution boundaries minimize invented rounding while producing deterministic integer totals",
    ["score-rating-clear", "boss-phase-final-settlement"],
    "Low",
    "replace when released formula programs or exact observations identify the authoritative rounding boundary and mode",
  ),
];
approximations.sort((left, right) => compareText(left.id, right.id));

const outputs = new Map([
  ["normalized-schema.json", normalizedSchema],
  ["authoring-contract.json", authoringContract],
  ["fixture-contract.json", fixtureContract],
  ["approximation-register.json", {
    schema_revision: "starclock.galactic-baseballer-approximation-register.v1",
    goal_id: goalId,
    exact_claim_forbidden: true,
    required_fields: normalizedSchema.types.approximation.required_fields,
    records: approximations,
  }],
]);

await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`generated contract drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Galactic Baseballer contracts ${check ? "verified" : "wrote"}: `
  + `${fileSpecs.length} normalized files, ${workbookSpecs.length} workbooks, `
  + `${familySpecs.length} fixture families, ${approximations.length} policies`,
);
