#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const manifestHash =
  "7416da5808a771a6c0bc78eb11371f51b4f7abb9cb273dd47123f4842800a758";
const outputRoot = path.join(
  root,
  "content-manifests",
  "unknowable-domain-v1",
);

const files = [
  file("profiles.json", "UnknowableProfile", "P1-B1",
    ["profiles", "entry_points"], ["entry_refs", "initial_resources", "finish_condition_ids"]),
  file("finish-conditions.json", "FinishCondition", "P1-B1",
    ["finish_conditions"], ["finish_type", "parameters", "comparison", "progress"]),
  file("areas.json", "UnknowableArea", "P1-B1",
    ["areas"], ["area_group", "difficulty_ids", "layer_ids", "default_alignment"]),
  file("difficulty-compositions.json", "DifficultyComposition", "P1-B1",
    ["difficulty_compositions", "difficulty_drops"], ["level", "parameters", "drop_bindings"]),
  file("layers.json", "UnknowableLayer", "P1-B1",
    ["layers"], ["layer_number", "room_position_ids", "carry_policy"]),
  file("layer-rooms.json", "LayerRoomPosition", "P1-B1",
    ["layer_rooms"], ["layer_id", "ordinal", "room_pool_ids"]),
  file("rooms.json", "UnknowableRoom", "P1-B1",
    ["rooms", "room_types"], ["room_type", "npc_graph_ids", "encounter_pool_ids"]),
  file("stage-flow.json", "StageFlowRule", "P1-B1",
    ["areas", "layers", "layer_rooms"], ["from_state", "condition", "to_state", "ordered_operations"]),
  file("alignments.json", "ExtrapolationAlignment", "P1-B2",
    ["alignments"], ["unlock_id", "eligibility", "pool_ids", "rule_contribution_ids"]),
  file("scepters.json", "Scepter", "P1-B3",
    ["scepters"], ["style", "function", "level_ids", "slot_layout_ids"]),
  file("scepter-levels.json", "ScepterLevel", "P1-B3",
    ["scepter_levels", "scepter_locked_components"],
    ["scepter_id", "level", "power", "locked_component_ids", "effect_ranges"]),
  file("scepter-activation-rules.json", "ScepterActivationRule", "P1-B3",
    ["scepter_levels", "mechanic_source_files"],
    ["scepter_id", "trigger", "charge_or_speed", "target_rule", "ordered_operations"]),
  file("scepter-state-transitions.json", "ScepterStateTransition", "P1-B3",
    ["scepter_levels"], ["scepter_id", "from_state", "input", "to_state", "teardown"]),
  file("components.json", "Component", "P1-B4",
    ["components", "component_categories", "component_types"],
    ["category", "component_type", "level_ids", "style_ids"]),
  file("component-levels.json", "ComponentLevel", "P1-B4",
    ["component_levels", "component_effects"],
    ["component_id", "level", "shape", "range_ids", "effect_program"]),
  file("component-slot-compatibility.json", "ComponentSlotCompatibility", "P1-B4",
    ["component_levels", "slot_layouts"],
    ["component_id", "component_level", "slot_type", "range", "eligibility"]),
  file("decision-components.json", "DecisionComponent", "P1-B5",
    ["decision_components"], ["component_id", "eligibility", "scope", "choice_program_ids"]),
  file("component-choice-programs.json", "ComponentChoiceProgram", "P1-B5",
    ["decision_components", "mechanic_source_files"],
    ["decision_component_id", "candidate_set", "ordering", "outcomes", "fallback"]),
  file("slot-layouts.json", "ScepterSlotLayout", "P1-B5",
    ["slot_layouts"], ["active_count", "attach_count", "passive_count"]),
  file("loadouts.json", "ScepterLoadout", "P1-B5",
    ["scepters", "slot_layouts"], ["scepter_id", "slot_ids", "locked_component_ids"]),
  file("loadout-transition-rules.json", "LoadoutTransitionRule", "P1-B5",
    ["slot_layouts", "component_levels"],
    ["operation", "eligibility", "replacement_order", "no_legal_candidate"]),
  file("synthesis-rules.json", "ComponentSynthesisRule", "P1-B6",
    ["workbench_functions", "component_levels"],
    ["input_count", "input_eligibility", "output_pool", "cost", "fallback"]),
  file("upgrade-rules.json", "ScepterUpgradeRule", "P1-B6",
    ["workbench_functions", "scepter_levels"],
    ["input_level", "output_level", "cost", "cap", "ordered_operations"]),
  file("reforge-rules.json", "ComponentReforgeRule", "P1-B6",
    ["workbench_functions", "component_levels"],
    ["input_eligibility", "candidate_set", "ordering", "cost", "fallback"]),
  file("workbenches.json", "Workbench", "P1-B7",
    ["workbenches"], ["function_ids", "eligibility", "lifecycle"]),
  file("workbench-functions.json", "WorkbenchFunction", "P1-B7",
    ["workbench_functions"], ["function_type", "currency_id", "price", "offer_policy_id"]),
  file("gamble-groups.json", "GambleGroup", "P1-B7",
    ["gamble_groups"], ["gamble_type", "unit_ids", "offer_policy_id"]),
  file("gamble-units.json", "GambleUnit", "P1-B7",
    ["gamble_units"], ["unit_type", "parameters", "outcome_program"]),
  file("service-offer-rules.json", "ServiceOfferRule", "P1-B7",
    ["workbenches", "workbench_functions", "gamble_groups", "gamble_units"],
    ["service_id", "candidate_set", "ordering", "refresh", "no_legal_candidate"]),
  file("mode-constants.json", "ModeConstant", "P1-B8",
    ["mode_constants"], ["value_type", "value", "consumer_ids"]),
  file("talents.json", "UnknowableTalent", "P1-B8",
    ["talents"], ["level", "cost", "prerequisite_ids", "effect_ids"]),
  file("unlocks.json", "UnknowableUnlock", "P1-B8",
    ["unlocks"], ["finish_condition_id", "consequence", "evaluation_boundary"]),
  file("layer-effects.json", "LayerEffect", "P1-B8",
    ["layer_effects"], ["trigger", "parameters", "ordered_operations"]),
  file("maze-buffs.json", "UnknowableMazeBuff", "P1-B8",
    ["maze_buffs"], ["series", "rarity", "level", "binding", "parameters"]),
  file("score-inputs.json", "ScoreInput", "P1-B8",
    ["score_inputs"], ["world_level", "layer", "room", "score"]),
  file("progression-effects.json", "ProgressionEffect", "P1-B8",
    ["talents", "unlocks", "layer_effects", "maze_buffs", "score_inputs"],
    ["source_kind", "source_id", "scope", "ordered_operations", "battle_projection"]),
  file("blessings.json", "UnknowableBlessingBinding", "P2-B1",
    ["blessings"], ["shared_blessing_id", "pool_id", "reachability_proof"]),
  file("pool-membership.json", "UnknowablePoolMembership", "P2-B1",
    ["blessings", "curios", "occurrences", "components"],
    ["pool_id", "member_kind", "member_id", "eligibility", "weight"]),
  file("curios.json", "UnknowableCurio", "P2-B2",
    ["curios"], ["handbook_id", "state_ids", "pool_ids"]),
  file("curio-states.json", "UnknowableCurioState", "P2-B2",
    ["curio_states"], ["curio_id", "state", "charges", "effect_program"]),
  file("curio-groups.json", "UnknowableCurioGroup", "P2-B2",
    ["curio_groups"], ["weighted_members", "eligibility", "ordering"]),
  file("curio-rules.json", "UnknowableCurioRule", "P2-B2",
    ["curio_states", "curio_groups"],
    ["curio_id", "trigger", "lifecycle", "repair", "replacement"]),
  file("occurrences.json", "UnknowableOccurrence", "P2-B3",
    ["occurrences"], ["handbook_id", "variant_ids", "pool_ids"]),
  file("occurrence-variants.json", "UnknowableOccurrenceVariant", "P2-B3",
    ["occurrence_variants"], ["occurrence_id", "graph_path", "choice_ids"]),
  file("occurrence-choices.json", "UnknowableOccurrenceChoice", "P2-B3",
    ["occurrence_variants"], ["variant_id", "eligibility", "costs", "ordered_outcomes"]),
  file("mode-service-npcs.json", "ModeServiceNpc", "P2-B4",
    ["mode_service_npcs"], ["graph_path", "service_ids", "eligibility"]),
  file("adventure-outcomes.json", "UnknowableAdventureOutcome", "P2-B4",
    ["adventure_outcomes"], ["adventure_type", "tier", "offered_result"]),
  file("currencies.json", "UnknowableCurrency", "P2-B4",
    ["workbench_functions", "gamble_units"], ["initial_amount", "cap", "carry_policy"]),
  file("service-rules.json", "UnknowableServiceRule", "P2-B4",
    ["workbenches", "gamble_groups", "adventure_outcomes", "mode_service_npcs"],
    ["service_kind", "eligibility", "price", "outcome", "lifecycle"]),
  file("boss-choices.json", "UnknowableBossChoice", "P2-B5",
    ["boss_choices"], ["enemy_id", "display_level_bindings", "pool_id"]),
  file("encounter-source-obligations.json", "EncounterSourceObligation", "P2-B5",
    ["encounter_source_obligations"], ["parent_kind", "parent_id", "expansion_state"]),
  file("encounter-groups.json", "UnknowableEncounterGroup", "P2-B5",
    ["encounter_source_obligations"], ["room_ids", "difficulty_ids", "weighted_stage_ids"]),
  file("encounter-waves.json", "UnknowableEncounterWave", "P2-B5",
    ["encounter_source_obligations"], ["encounter_group_id", "ordinal", "enemy_slot_ids"]),
  file("enemy-slots.json", "UnknowableEnemySlot", "P2-B5",
    ["encounter_source_obligations"], ["wave_id", "ordinal", "enemy_variant_id", "level"]),
  file("boss-pools.json", "UnknowableBossPool", "P2-B5",
    ["boss_choices"], ["difficulty_id", "candidate_ids", "ordering", "fallback"]),
  file("mechanic-source-files.json", "MechanicSourceFile", "P2-B6",
    ["mechanic_source_files"], ["path", "source_sha256", "consumer_rule_ids"]),
  file("mechanic-rules.json", "UnknowableMechanicRule", "P2-B6",
    ["mechanic_source_files"], ["scope", "trigger", "ordered_operations", "battle_projection"]),
  file("sources.json", "SourceEvidence", "P2-B6",
    ["mechanic_source_files"], ["repository", "revision", "path", "locator", "sha256"]),
  file("coverage.json", "ReferenceCoverage", "P2-B6",
    ["semantic_fixture_families"], ["manifest_category", "manifest_record_id", "state", "data_ids"]),
  file("research-gaps.json", "ReferenceResearchGap", "P2-B6",
    ["semantic_fixture_families"], ["field", "known_fact", "policy", "replacement_condition"]),
  file("semantic-fixture-families.json", "SemanticFixtureFamily", "P2-B6",
    ["semantic_fixture_families"], ["minimum_cases", "must_cover"]),
  file("review-fixtures.json", "SemanticReviewFixture", "P2-B6",
    ["semantic_fixture_families"], ["family_id", "preconditions", "input", "expected_facts"]),
  file("reconciliation-receipts.json", "OwnershipReconciliationReceipt", "P2-B6",
    ["curios", "occurrences", "boss_choices"],
    ["source_path", "row_locator", "evidence_sha256", "outcome", "note"]),
  file("manifest.json", "ReferenceManifestSummary", "P2-B6",
    ["profiles"], ["content_manifest_sha256", "category_counts", "ownership_counts"]),
  file("pack-index.json", "ReferencePackIndex", "P2-B6",
    ["profiles"], ["file_digests", "pack_digest", "component_digest"]),
];

const schema = {
  schema_revision: "starclock.unknowable-domain-normalized-schema.v1",
  goal_id: "unknowable-domain-reference-v1",
  bound_content_manifest_sha256: manifestHash,
  common_envelope: {
    required_fields: [
      "id", "schema_revision", "kind", "name_en", "name_zh_cn", "summary_en",
      "summary_zh_cn", "ownership", "coverage_state", "evidence_quality",
      "source_refs", "tags",
    ],
    id: {
      type: "string",
      pattern: "^[a-z0-9][a-z0-9._:-]*$",
      global_uniqueness: true,
    },
    schema_revision: {
      type: "string",
      value: "starclock.unknowable-domain-row.v1",
    },
    kind: { type: "string", closed_by_file_contract: true },
    name_en: { type: "string", nonempty: true },
    name_zh_cn: { type: "string", nonempty: true },
    summary_en: { type: "string", nonempty: true, mechanical_only: true },
    summary_zh_cn: { type: "string", nonempty: true, mechanical_only: true },
    ownership: { enum: ["UnknowableDomain", "Shared"] },
    coverage_state: {
      enum: ["Cataloged", "Researched", "DataReady", "Blocked"],
    },
    evidence_quality: {
      enum: [
        "ExactStructured", "ExactPublicText", "Observed",
        "ApproximateFromReleasedText", "ProjectPolicy",
      ],
    },
    source_refs: {
      type: "array",
      minimum: 1,
      ordered: true,
      item_type: "source_ref",
    },
    tags: { type: "array", unique: true, ordering: "lexicographic" },
  },
  types: {
    canonical_decimal: {
      storage: "string",
      pattern: "^(0|-?(?:[1-9][0-9]*(?:\\.[0-9]*[1-9])?|0\\.[0-9]*[1-9]))$",
      forbid: [
        "binary floating point", "exponent notation", "leading plus",
        "negative zero", "trailing fractional zero",
      ],
    },
    source_hash: { storage: "string", pattern: "^[0-9a-f]{64}$" },
    source_numeric_id: {
      storage: "string",
      reason: "preserve identifiers and TextMap hashes beyond JavaScript safe integer range",
    },
    source_ref: {
      required_fields: [
        "source_id", "repository", "revision", "path", "locator", "sha256",
        "access_date", "game_version", "evidence_quality", "mechanism_quality",
      ],
      optional_fields: ["note", "replacement_condition"],
      approximation_rule:
        "ApproximateFromReleasedText and ProjectPolicy require note and replacement_condition",
    },
    stable_ref: {
      storage: "string",
      resolution: "closed pack index or explicit inherited stable-ID set",
      unknown_behavior: "reject",
    },
    ordered_child: {
      required_fields: ["parent_id", "ordinal"],
      ordinal_type: "unsigned fixed-width integer",
      duplicate_behavior: "reject",
    },
  },
  canonical_encoding: {
    encoding: "UTF-8",
    line_endings: "LF",
    indent_spaces: 2,
    terminal_newline: true,
    object_key_order: "schema declaration order, then lexicographic extension fields",
    array_order: "explicit file ordering_keys; never filesystem or object iteration order",
    unicode_normalization: "NFC",
    null_policy: "omit optional absent values; never use null",
    boolean_policy: "JSON true/false only",
    integer_policy: "JSON integer only inside signed 53-bit range; otherwise source_numeric_id string",
    decimal_policy: "canonical_decimal strings only",
    digest_policy: "SHA-256 over encoded bytes; pack digest excludes pack-index.json itself",
  },
  manifest_mapping: {
    source_obligations:
      "every manifest category maps to one or more normalized files and closes exactly once in coverage.json",
    derived_files:
      "derived rows reuse a parent category and cannot create or remove a source obligation",
    child_rows: "typed child rows carry parent stable ID and deterministic ordinal",
    unknown_reference: "reject",
  },
  reconciliation_policy: {
    checkpoint_proof_path:
      "evidence/unknowable-domain-reference-v1/reconciliation-checkpoints.json",
    checkpoints: [
      {
        goal: "gold-and-gears-reference-v1",
        commit: "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
        manifest_sha256:
          "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
        required_now: true,
        completion_state: "Complete",
        registration_commit:
          "2688624c34a564d87076cadb405c8da506efd373",
        checkpoint_transport: "LocalCommittedReleaseRegistration",
      },
      {
        goal: "swarm-disaster-reference-v1",
        commit: "b8da6744a63cd92554b45f8e780d79a1be131f50",
        manifest_sha256:
          "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850",
        required_now: true,
        checkpoint_transport: "RemoteBranch",
        remote_ancestor: "origin/codex/goal09-swarm-disaster-reference",
      },
    ],
    join_key: ["source_path", "row_locator", "evidence_sha256"],
    outcomes: [
      "MatchedShared", "UnknowableOnly", "OtherGoalOnly",
      "DivergentRepresentation", "Conflict",
    ],
    conflict_behavior:
      "Blocked; record the conflict and wait for merge coordination without mutating another Goal",
    required_receipt_fields: [
      "id", "source_path", "row_locator", "evidence_sha256",
      "checkpoint_goal", "checkpoint_commit", "checkpoint_ownership",
      "goal10_ownership", "outcome", "note",
    ],
  },
  files,
};

const reviewFiles = [
  "mechanic-source-files.json", "mechanic-rules.json", "sources.json",
  "coverage.json", "research-gaps.json", "semantic-fixture-families.json",
  "review-fixtures.json", "reconciliation-receipts.json", "manifest.json",
  "pack-index.json",
];
const bindingFiles = files.map(({ file: name }) => name).filter((name) =>
  [
    "blessings.json", "pool-membership.json", "curios.json", "curio-states.json",
    "curio-groups.json", "curio-rules.json", "occurrences.json",
    "occurrence-variants.json", "occurrence-choices.json",
    "mode-service-npcs.json", "adventure-outcomes.json", "currencies.json",
    "service-rules.json", "boss-choices.json",
    "encounter-source-obligations.json", "encounter-groups.json",
    "encounter-waves.json", "enemy-slots.json", "boss-pools.json",
  ].includes(name));
const primaryFiles = files.map(({ file: name }) => name)
  .filter((name) => !reviewFiles.includes(name) && !bindingFiles.includes(name));
const authoring = {
  schema_revision: "starclock.unknowable-domain-authoring-contract.v1",
  goal_id: "unknowable-domain-reference-v1",
  bound_content_manifest_sha256: manifestHash,
  authority: {
    authoritative_format: "xlsx",
    editor: "python-openpyxl",
    editor_version: "3.1.5",
    schema_exporter: "sora-cli",
    schema_exporter_version: "0.3.0",
    production_artifact: "sora",
    json_role: "research-staging-debug-only",
    runtime_loading: false,
  },
  isolation: {
    project: "config/unknowable-domain/project.toml",
    schema_root: "config/unknowable-domain/schema/",
    workbook_root: "config/unknowable-domain/workbooks/",
    generated_root: "config/unknowable-domain-generated/",
    generated_reader_root: "config/unknowable-domain-generated/reader/",
    forbidden_outputs: [
      "config/generated/", "config/universe-generated/",
      "config/gold-and-gears-generated/", "config/swarm-disaster-generated/",
    ],
  },
  generation: {
    clean_target_required: true,
    overwrite_existing_target: false,
    patch_designer_workbook: false,
    double_generation_byte_identical: true,
    calculation_mode: "manual",
    external_links: "reject",
    macros: "reject",
    formulas: "reject unless a reviewed deterministic formula contract names the cells",
    excel_error_cells: "reject",
    unknown_columns: "reject",
    unknown_sheets: "reject",
  },
  sheet_contract: {
    sora_metadata_rows: "preserve rows 1 through 7 exactly",
    data_start_row: 8,
    freeze_panes: "A8",
    auto_filter: true,
    header_style: "goal10-header-v1",
    alternating_rows: true,
    wrapped_text: true,
    deterministic_widths: true,
    data_validation: true,
    typed_references: true,
    canonical_decimal_cells: "text",
  },
  workbooks: [
    {
      file: "UnknowableDomain.xlsx",
      purpose:
        "entry, flow, Alignments, Scepters, Components, progression and services",
      normalized_files: primaryFiles,
    },
    {
      file: "UnknowableDomainBindings.xlsx",
      purpose: "content pools, service bindings, Adventure and encounters",
      normalized_files: bindingFiles,
    },
    {
      file: "UnknowableDomainReview.xlsx",
      purpose:
        "rules, provenance, coverage, gaps, reconciliation, fixtures and pack identity",
      normalized_files: reviewFiles,
    },
  ],
  table_family_contract: {
    one_primary_table_per_normalized_file: true,
    child_tables:
      "allowed only for repeated typed fields; every child row carries parent stable ID and deterministic ordinal",
    table_names:
      "UnknowableDomain plus PascalCase normalized file stem; child tables append semantic role",
    sheet_names: "explicit in Sora schema and at most 31 characters",
    numeric_keys: "private workbook identities only",
    stable_keys: "required for every cross-workbook or inherited reference",
  },
  reconciliation_sheet: {
    workbook: "UnknowableDomainReview.xlsx",
    normalized_file: "reconciliation-receipts.json",
    join_key: "source path plus row locator plus evidence SHA-256",
    conflict_behavior:
      "Block Goal 10 publication; do not edit Goal 08/09 workbooks or manifests",
  },
  acceptance: {
    schema_check: "sora check --project config/unknowable-domain/project.toml",
    schema_build: "sora build --project config/unknowable-domain/project.toml",
    export: "sora export --project config/unknowable-domain/project.toml",
    reader_load: "every generated table and row through the isolated reader",
    visual_review: "render and inspect every authored sheet",
    semantic_digest:
      "canonical cell values, types, validations, sheet order and table identity",
  },
};

const mustCover = {
  "profile-entry-and-finish":
    ["entry eligibility", "initial resources", "finish and terminal boundary"],
  "area-layer-room-transition":
    ["ordered transition", "carry", "reset", "illegal transition"],
  "difficulty-composition":
    ["difficulty parameters", "drop binding", "difficulty application boundary"],
  "alignment-selection":
    ["four Alignments", "eligibility", "offered pool", "no legal selection"],
  "scepter-activation":
    ["trigger", "targeting", "ordered operations", "single activation"],
  "scepter-charge-and-speed":
    ["gain", "spend", "action ordering", "cap and teardown"],
  "component-slot-legality":
    ["shape", "range", "slot type", "rejected insertion"],
  "component-insertion-removal-replacement":
    ["insert", "remove", "replace", "locked Component"],
  "decision-component-choice":
    ["eligibility", "candidate order", "scope", "fallback"],
  "component-synthesis":
    ["input count", "consumption", "output pool", "no legal output"],
  "component-upgrade":
    ["level transition", "cost", "cap", "ordering"],
  "component-reforge":
    ["candidate set", "cost", "replacement", "empty candidate"],
  "workbench-offer-and-cost":
    ["eligibility", "price", "offer ordering", "refresh lifecycle"],
  "gamble-offer-and-outcome":
    ["weighted candidate set", "seed", "outcome", "failure"],
  "talent-and-unlock":
    ["prerequisite", "cost", "unlock consequence", "evaluation boundary"],
  "layer-and-difficulty-effect":
    ["entry trigger", "cross-battle state", "battle projection", "teardown"],
  "curio-lifecycle":
    ["mode state", "charges", "repair", "replacement"],
  "occurrence-choice":
    ["variant", "condition", "cost", "ordered outcome"],
  "service-and-adventure":
    ["service eligibility", "price or tier", "offered abstract outcome"],
  "encounter-selection":
    ["room parent", "candidate order", "wave", "enemy slots"],
  "wave-and-boss-binding":
    ["StageConfig row", "wave order", "boss alternative", "difficulty"],
  "cross-battle-carry-reset":
    ["state slot", "carry boundary", "reset boundary", "finish cleanup"],
  "simultaneous-trigger-order":
    ["trigger phase", "priority", "stable tie key", "expected facts"],
  "no-legal-candidate-fallback":
    ["empty set", "explicit fallback", "no hidden RNG draw", "review fact"],
};
const fixtureContract = {
  schema_revision: "starclock.unknowable-domain-fixture-contract.v1",
  goal_id: "unknowable-domain-reference-v1",
  bound_content_manifest_sha256: manifestHash,
  fixture_role: "normalized semantic review; no runtime executability claim",
  minimum_cases_per_family: 1,
  required_fields: [
    "id", "family_id", "name_en", "name_zh_cn", "source_record_ids",
    "preconditions", "input", "ordered_operations", "expected_facts",
    "evidence_refs", "evidence_quality",
  ],
  field_contracts: {
    id: "globally unique stable string",
    family_id: "one of required_families",
    source_record_ids: "nonempty sorted stable-ID list closing manifest obligations",
    preconditions: "typed canonical facts only",
    input:
      "typed offered outcome or deterministic selection input; never simulated Adventure actions",
    ordered_operations:
      "nonempty ordered operation/fallback trace with canonical decimal strings",
    expected_facts: "nonempty typed assertions; no presentation prose",
    evidence_refs: "nonempty ordered references resolving in sources.json",
    evidence_quality: "one normalized evidence label",
  },
  determinism: {
    random_selection: "explicit seed plus ordered candidate stable IDs",
    ties: "stable ID ascending unless released authored order is exact",
    no_legal_target: "explicit expected fallback; never silent no-op",
    operation_order: "asserted as part of expected_facts",
    decimal_values: "canonical decimal strings",
    external_outcomes:
      "record the offered abstract Adventure result and eligibility only",
  },
  approximation: {
    allowed_labels: ["ApproximateFromReleasedText", "ProjectPolicy"],
    required_fields: ["note", "replacement_condition"],
    exact_claim_forbidden: true,
  },
  required_families: Object.entries(mustCover).map(([id, cover]) => ({
    id,
    minimum_cases: 1,
    must_cover: cover,
  })),
  coverage_rule: {
    family_coverage: "every required family has at least minimum_cases",
    source_coverage: "every distinct mechanic rule references at least one fixture",
    manifest_coverage:
      "every source obligation reaches DataReady or an explicit nonblocking approximation with replacement condition",
    reconciliation_coverage:
      "every overlapping Goal 08/09 locator has a non-conflicting receipt",
    blocking_gap: "no fixture can satisfy a Blocked manifest obligation",
  },
};

write("normalized-schema.json", schema);
write("authoring-contract.json", authoring);
write("fixture-contract.json", fixtureContract);
console.log(
  `Unknowable Domain contracts ${check ? "verified" : "generated"}: ` +
  `${files.length} normalized files, 3 workbooks, ` +
  `${fixtureContract.required_families.length} fixture families.`,
);

function file(name, recordKind, phase, inputs, domainFields) {
  return {
    file: name,
    record_kind: recordKind,
    phase,
    manifest_category_inputs: inputs,
    ordering_keys: ["id"],
    required_domain_fields: domainFields,
  };
}
function write(name, value) {
  const target = path.join(outputRoot, name);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    if (fs.readFileSync(target, "utf8") !== encoded)
      throw new Error(`Unknowable Domain contract has generated drift: ${name}`);
  } else {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, encoded, "utf8");
  }
}
