#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root") ?? ".");
const manifestRelative =
  "content-manifests/divergent-universe-v1/content-manifest.json";
const manifestPath = path.join(root, manifestRelative);
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const manifestSha256 = digestBytes(fs.readFileSync(manifestPath));
const outputRoot = path.join(
  root,
  "content-manifests/divergent-universe-v1",
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function file(fileName, phase, manifestInputs, requiredDomainFields) {
  return {
    file: fileName,
    record_kind: `DivergentUniverse${pascal(fileName.replace(/\.json$/u, ""))}`,
    phase,
    manifest_category_inputs: manifestInputs,
    ordering_keys: ["id"],
    required_domain_fields: requiredDomainFields,
  };
}

const files = [
  file("profiles.json", "P1-B1",
    ["profiles", "entry_points", "enabled_modules"],
    ["entry_refs", "module_id", "initial_resources", "finish_condition_ids"]),
  file("modules.json", "P1-B1", ["enabled_modules"],
    ["sub_mode", "tourn_mode", "main_tourn_id", "sub_tourn_id"]),
  file("entries.json", "P1-B1", ["entry_points"],
    ["entry_kind", "module_id", "unlock_ids"]),
  file("finish-conditions.json", "P1-B1", ["finish_conditions"],
    ["condition_kind", "parameters", "terminal_disposition"]),
  file("areas.json", "P1-B1", ["area_groups", "areas"],
    ["area_type", "difficulty_ids", "layer_ids", "map_entry_id"]),
  file("difficulties.json", "P1-B1", ["difficulties"],
    ["level_list", "protocol_id", "enemy_scaling_refs"]),
  file("layers.json", "P1-B1", ["layers"],
    ["layer_number", "ordered_room_position_ids"]),
  file("layer-rooms.json", "P1-B1", ["layer_rooms"],
    ["layer_id", "room_index", "door_program"]),
  file("rooms.json", "P1-B1", ["room_reuse_candidates", "room_types"],
    ["room_type", "reachability_disposition", "stage_refs"]),
  file("stage-flow.json", "P1-B1", [],
    ["entry_id", "ordered_stage_refs", "carry_rules", "reset_rules"]),
  file("cyclical-challenges.json", "P1-B1", ["weekly_modifiers"],
    ["challenge_kind", "modifier_ids", "enemy_display_refs"]),
  file("protocols.json", "P1-B7", [],
    ["protocol_level", "entry_rules", "difficulty_changes", "enemy_changes"]),
  file("astronomical-divisions.json", "P1-B7",
    ["astronomical_divisions", "astronomical_division_effects"],
    ["division_level", "progress_boundary", "effect_ids"]),
  file("star-pioneer-practice.json", "P1-B7", [],
    ["mode_kind", "entry_rules", "available_content", "reset_rules"]),
  file("cognoculi.json", "P1-B7", [],
    ["source_locator", "effect_scope", "contribution_ids"]),

  file("arithmetic-mapping-eligibility.json", "P1-B2",
    ["arithmetic_mapping_avatars", "arithmetic_mapping_build_refs"],
    ["avatar_id", "eligibility", "account_comparison_policy"]),
  file("arithmetic-mapping-builds.json", "P1-B2",
    ["arithmetic_mapping_roles"],
    ["avatar_id", "level", "trace_state", "light_cone", "relics", "role_buff_id"]),
  file("arithmetic-mapping-rules.json", "P1-B2", [],
    ["selection_timing", "refresh_timing", "stronger_build_rule", "teardown"]),

  file("equations.json", "P1-B3", ["equations", "equation_displays"],
    ["category", "main_path", "sub_path", "recipe_id", "effect_ids"]),
  file("equation-recipes.json", "P1-B3", [],
    ["main_path_count", "sub_path_count", "required_blessing_states"]),
  file("equation-categories.json", "P1-B3", [],
    ["category", "offer_rules", "expansion_boundary"]),
  file("equation-offers.json", "P1-B3", ["equation_randomizers"],
    ["candidate_ids", "weight_program", "reroll_rule", "fallback"]),
  file("equation-progress.json", "P1-B3", [],
    ["equation_id", "contribution_rule", "progress_states"]),
  file("equation-expansion-states.json", "P1-B3", [],
    ["equation_id", "state", "entry_condition", "exit_condition"]),
  file("equation-effects.json", "P1-B3",
    ["equation_keywords", "equation_keyword_params"],
    ["equation_id", "keyword_ids", "parameters", "rule_contribution_ids"]),
  file("equation-replacement-rules.json", "P1-B3", [],
    ["operation", "candidate_policy", "preserved_state", "fallback"]),

  file("blessing-paths.json", "P1-B4", ["blessing_paths"],
    ["path_id", "equation_roles", "rewrite_rules"]),
  file("blessings.json", "P1-B4", ["blessings"],
    ["path_id", "category", "level_ids", "effect_ids"]),
  file("blessing-levels.json", "P1-B4", ["blessing_levels"],
    ["blessing_id", "level", "parameters", "effect_ids"]),
  file("blessing-groups.json", "P1-B4", ["blessing_groups"],
    ["candidate_ids", "selection_policy", "weight_program"]),
  file("blessing-rewrite-rules.json", "P1-B4", [],
    ["input_state", "output_state", "timing", "fallback"]),
  file("blessing-equation-contributions.json", "P1-B4", [],
    ["blessing_id", "equation_id", "contribution", "refresh_timing"]),

  file("curios.json", "P1-B5", ["curios"],
    ["category", "state_ids", "eligibility_rule_ids"]),
  file("curio-states.json", "P1-B5", ["curio_states"],
    ["curio_id", "state", "charges", "effect_ids"]),
  file("curio-groups.json", "P1-B5", ["curio_groups"],
    ["candidate_state_ids", "weights", "eligibility"]),
  file("curio-lifecycle-rules.json", "P1-B5", [],
    ["curio_id", "activation", "destruction", "repair", "replacement", "fallback"]),
  file("grand-miracles.json", "P1-B5", ["grand_miracles"],
    ["display_id", "maze_buff_id", "effect_ids", "state_ids"]),
  file("grand-miracle-eligibility.json", "P1-B5",
    ["grand_miracle_eligibility"],
    ["grand_miracle_id", "character_path", "element", "eligibility"]),
  file("grand-miracle-states.json", "P1-B5", [],
    ["grand_miracle_id", "state", "activation", "duration", "teardown"]),

  file("titan-types.json", "P1-B6", ["titan_types"],
    ["category", "boon_ids", "talent_ids"]),
  file("titan-boons.json", "P1-B6", ["titan_bless_levels"],
    ["titan_type", "level", "maze_buff_id", "effect_ids"]),
  file("titan-talents.json", "P1-B6", ["titan_talent_levels"],
    ["titan_type", "level", "cost", "effect_program"]),
  file("titan-choices.json", "P1-B6", [],
    ["candidate_ids", "eligibility", "selection_count", "fallback"]),
  file("titan-contributions.json", "P1-B6", [],
    ["source_id", "scope", "activation", "ordered_effects"]),

  file("workbenches.json", "P1-B8", ["workbenches"],
    ["function_ids", "currency_ids", "availability"]),
  file("workbench-functions.json", "P1-B8", ["workbench_functions"],
    ["function_type", "input_policy", "output_policy", "price_rule"]),
  file("gamble-groups.json", "P1-B8", ["gamble_groups"],
    ["group_type", "unit_ids", "offer_policy"]),
  file("gamble-units.json", "P1-B8", ["gamble_units"],
    ["unit_type", "parameters", "outcome_program"]),
  file("curse-chests.json", "P1-B8", ["curse_chests"],
    ["chest_type", "parameters", "choice_program"]),
  file("currencies.json", "P1-B8", [],
    ["scope", "gain_rules", "spend_rules", "reset_rule"]),
  file("service-rules.json", "P1-B8", [],
    ["service_kind", "currency_id", "price", "ordered_operations", "fallback"]),
  file("service-offer-rules.json", "P1-B8", [],
    ["service_id", "candidate_ids", "weights", "refresh_rule", "fallback"]),

  file("permanent-talents.json", "P1-B9", ["permanent_talents"],
    ["cost", "prerequisite_ids", "effect_ids", "scope"]),
  file("unlocks.json", "P1-B9", ["unlocks"],
    ["finish_condition_id", "unlocked_content_ids", "scope"]),
  file("common-constants.json", "P1-B9", ["common_constants"],
    ["value_kind", "canonical_value", "consumer_ids"]),
  file("weekly-modifiers.json", "P1-B9", ["weekly_modifiers"],
    ["content_ids", "detail_ids", "enemy_group_refs", "effect_ids"]),
  file("room-marks.json", "P1-B9", ["room_marks"],
    ["room_type", "mark_kind", "transition_rules"]),
  file("progression-effects.json", "P1-B9", [],
    ["source_id", "scope", "rule_contribution_ids"]),

  file("pool-membership.json", "P2-B1",
    ["blessing_paths", "blessings", "blessing_groups"],
    ["pool_id", "member_id", "membership_basis", "module_scope"]),
  file("curio-pool-membership.json", "P2-B2",
    ["curios", "curio_states", "curio_groups"],
    ["pool_id", "curio_state_id", "weight", "eligibility"]),
  file("occurrences.json", "P2-B3", ["occurrences"],
    ["variant_ids", "unlock_rules", "choice_ids"]),
  file("occurrence-variants.json", "P2-B3", ["occurrence_variants"],
    ["occurrence_id", "graph_path", "entry_conditions", "choice_ids"]),
  file("occurrence-choices.json", "P2-B3", [],
    ["variant_id", "ordinal", "conditions", "costs", "ordered_outcomes"]),
  file("mode-service-npcs.json", "P2-B4", ["mode_service_npcs"],
    ["graph_path", "service_kind", "choice_ids"]),
  file("adventure-outcomes.json", "P2-B4", ["adventure_outcomes"],
    ["adventure_type", "parameter_group_id", "abstract_outcome"]),
  file("encounter-source-obligations.json", "P2-B5",
    ["encounter_source_obligations"],
    ["parent_kind", "parent_id", "resolution_state"]),
  file("encounter-groups.json", "P2-B5", [],
    ["module_id", "area_id", "difficulty_id", "candidate_stage_ids"]),
  file("encounter-waves.json", "P2-B5", [],
    ["stage_id", "wave_index", "enemy_slot_ids", "trigger"]),
  file("enemy-slots.json", "P2-B5", [],
    ["wave_id", "slot_index", "monster_id", "level", "ability_refs"]),
  file("boss-pools.json", "P2-B5", [],
    ["area_id", "difficulty_id", "candidate_monster_ids", "selection_policy"]),

  file("mechanic-source-files.json", "P2-B6", ["mechanic_source_files"],
    ["source_path", "source_sha256", "mechanic_family", "disposition"]),
  file("mechanic-rules.json", "P2-B6", [],
    ["scope", "trigger", "ordered_operations", "state_lifecycle", "runtime_lowered"]),
  file("sources.json", "P2-B6", [],
    ["repository", "revision", "path", "locator", "sha256", "mechanism_quality"]),
  file("coverage.json", "P2-B6", [],
    ["manifest_category", "manifest_record_id", "normalized_record_ids", "state"]),
  file("research-gaps.json", "P2-B6", [],
    ["field", "known_facts", "selected_policy", "alternatives", "replacement_condition"]),
  file("semantic-fixture-families.json", "P2-B6",
    ["semantic_fixture_families"],
    ["minimum_cases", "must_cover"]),
  file("review-fixtures.json", "P2-B6", [],
    ["family_id", "preconditions", "input", "ordered_operations", "expected_facts"]),
  file("reconciliation-receipts.json", "P4-B3", [],
    ["source_path", "row_locator", "evidence_sha256", "checkpoint", "outcome"]),
  file("manifest.json", "P2-B6", [],
    ["content_manifest_sha256", "normalized_files", "record_counts"]),
  file("pack-index.json", "P2-B6", [],
    ["pack_digest", "file_digests", "stable_id_index"]),
];

const normalizedSchema = {
  schema_revision: "starclock.divergent-universe-normalized-schema.v1",
  goal_id: "divergent-universe-reference-v1",
  bound_content_manifest_sha256: manifestSha256,
  common_envelope: {
    required_fields: [
      "id",
      "schema_revision",
      "kind",
      "name_en",
      "name_zh_cn",
      "summary_en",
      "summary_zh_cn",
      "ownership",
      "coverage_state",
      "evidence_quality",
      "source_refs",
      "tags",
    ],
    id: {
      type: "string",
      pattern: "^[a-z0-9][a-z0-9._:-]*$",
      global_uniqueness: true,
    },
    schema_revision: {
      type: "string",
      value: "starclock.divergent-universe-row.v1",
    },
    kind: { type: "string", closed_by_file_contract: true },
    name_en: { type: "string", nonempty: true },
    name_zh_cn: { type: "string", nonempty: true },
    summary_en: { type: "string", nonempty: true, mechanical_only: true },
    summary_zh_cn: { type: "string", nonempty: true, mechanical_only: true },
    ownership: {
      enum: ["DivergentUniverse", "Shared"],
      candidate_rule:
        "SharedCandidate manifest obligations cannot become normalized rows until promoted by an exact reconciliation receipt",
    },
    coverage_state: {
      enum: ["Cataloged", "Researched", "DataReady", "Blocked"],
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
        "binary floating point",
        "exponent notation",
        "leading plus",
        "negative zero",
        "trailing fractional zero",
      ],
    },
    source_hash: { storage: "string", pattern: "^[0-9a-f]{64}$" },
    source_numeric_id: {
      storage: "string",
      reason:
        "preserve identifiers and TextMap hashes beyond JavaScript safe integer range",
    },
    source_ref: {
      required_fields: [
        "source_id",
        "repository",
        "revision",
        "path",
        "locator",
        "sha256",
        "access_date",
        "game_version",
        "evidence_quality",
        "mechanism_quality",
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
    object_key_order:
      "schema declaration order, then lexicographic extension fields",
    array_order:
      "explicit file ordering_keys; never filesystem or object iteration order",
    unicode_normalization: "NFC",
    null_policy: "omit optional absent values; never use null",
    boolean_policy: "JSON true/false only",
    integer_policy:
      "JSON integer only inside signed 53-bit range; otherwise source_numeric_id string",
    decimal_policy: "canonical_decimal strings only",
    digest_policy:
      "SHA-256 over encoded bytes; pack digest excludes pack-index.json itself",
  },
  manifest_mapping: {
    source_obligations:
      "every manifest category maps to one or more normalized files and closes exactly once in coverage.json",
    candidate_obligations:
      "SharedCandidate rows remain Cataloged or Blocked until an exact selector/reference/stable-ID receipt promotes or excludes them",
    derived_files:
      "derived rows reuse a parent category and cannot create or remove a source obligation",
    child_rows:
      "typed child rows carry parent stable ID and deterministic ordinal",
    unknown_reference: "reject",
  },
  reconciliation_policy: {
    checkpoint_proof_path:
      "evidence/divergent-universe-reference-v1/reconciliation-checkpoints.json",
    checkpoints: [
      checkpoint("G08", false),
      checkpoint("G09", true),
      checkpoint("G10", true),
    ],
    join_key: ["source_path", "row_locator", "evidence_sha256"],
    outcomes: [
      "MatchedShared",
      "DivergentUniverseOnly",
      "OtherGoalOnly",
      "DivergentRepresentation",
      "Conflict",
    ],
    conflict_behavior:
      "Blocked; record the conflict and wait for merge coordination without mutating another Goal",
    non_matching_digest_behavior:
      "Not a join; retain both evidence representations and report the " +
      "same-locator diagnostic without overwriting either Goal",
    required_receipt_fields: [
      "id",
      "source_path",
      "row_locator",
      "evidence_sha256",
      "checkpoint_goal",
      "checkpoint_commit",
      "checkpoint_ownership",
      "goal11_ownership",
      "outcome",
      "note",
    ],
  },
  files,
};

const mainWorkbookFiles = files
  .filter(({ phase }) => phase.startsWith("P1-"))
  .map(({ file: fileName }) => fileName);
const bindingWorkbookFiles = files
  .filter(({ phase, file: fileName }) => phase.startsWith("P2-")
    && ![
      "mechanic-source-files.json",
      "mechanic-rules.json",
      "sources.json",
      "coverage.json",
      "research-gaps.json",
      "semantic-fixture-families.json",
      "review-fixtures.json",
      "manifest.json",
      "pack-index.json",
    ].includes(fileName))
  .map(({ file: fileName }) => fileName);
const reviewWorkbookFiles = files
  .filter(({ file: fileName }) => !mainWorkbookFiles.includes(fileName)
    && !bindingWorkbookFiles.includes(fileName))
  .map(({ file: fileName }) => fileName);

const authoringContract = {
  schema_revision: "starclock.divergent-universe-authoring-contract.v1",
  goal_id: "divergent-universe-reference-v1",
  bound_content_manifest_sha256: manifestSha256,
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
    project: "config/divergent-universe/project.toml",
    schema_root: "config/divergent-universe/schema/",
    workbook_root: "config/divergent-universe/workbooks/",
    generated_root: "config/divergent-universe-generated/",
    generated_reader_root: "config/divergent-universe-generated/reader/",
    forbidden_outputs: [
      "config/generated/",
      "config/universe-generated/",
      "config/gold-and-gears-generated/",
      "config/swarm-disaster-generated/",
      "config/unknowable-domain-generated/",
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
    formulas:
      "reject unless a reviewed deterministic formula contract names the cells",
    excel_error_cells: "reject",
    unknown_columns: "reject",
    unknown_sheets: "reject",
  },
  sheet_contract: {
    sora_metadata_rows: "preserve rows 1 through 7 exactly",
    data_start_row: 8,
    freeze_panes: "A8",
    auto_filter: true,
    header_style: "goal11-header-v1",
    alternating_rows: true,
    wrapped_text: true,
    deterministic_widths: true,
    data_validation: true,
    typed_references: true,
    canonical_decimal_cells: "text",
  },
  workbooks: [
    {
      file: "DivergentUniverse.xlsx",
      purpose:
        "entry, flow, Arithmetic Mapping, Equations, Blessings, Curios, Grand Miracles, Titans, protocols and progression",
      normalized_files: mainWorkbookFiles,
    },
    {
      file: "DivergentUniverseBindings.xlsx",
      purpose:
        "content pools, services, Occurrences, Adventure and encounters",
      normalized_files: bindingWorkbookFiles,
    },
    {
      file: "DivergentUniverseReview.xlsx",
      purpose:
        "rules, provenance, coverage, gaps, reconciliation, fixtures and pack identity",
      normalized_files: reviewWorkbookFiles,
    },
  ],
  table_family_contract: {
    one_primary_table_per_normalized_file: true,
    child_tables:
      "allowed only for repeated typed fields; every child row carries parent stable ID and deterministic ordinal",
    table_names:
      "DivergentUniverse plus PascalCase normalized file stem; child tables append semantic role",
    sheet_names: "explicit in Sora schema and at most 31 characters",
    numeric_keys: "private workbook identities only",
    stable_keys: "required for every cross-workbook or inherited reference",
  },
  reconciliation_sheet: {
    workbook: "DivergentUniverseReview.xlsx",
    normalized_file: "reconciliation-receipts.json",
    join_key: "source path plus row locator plus evidence SHA-256",
    conflict_behavior:
      "Block Goal 11 publication; do not edit Goal 08/09/10 workbooks or manifests",
  },
  acceptance: {
    schema_check: "sora check --project config/divergent-universe/project.toml",
    schema_build: "sora build --project config/divergent-universe/project.toml",
    export: "sora export --project config/divergent-universe/project.toml",
    reader_load: "every generated table and row through the isolated reader",
    visual_review: "render and inspect every authored sheet",
    semantic_digest:
      "canonical cell values, types, validations, sheet order and table identity",
  },
};

const fixtureDetails = {
  "profile-and-module-selection": [
    "TournRogue entry",
    "module 6002201",
    "rejected historical module",
  ],
  "ordinary-and-cyclical-entry": [
    "entry eligibility",
    "initial resources",
    "weekly modifier binding",
  ],
  "area-difficulty-layer-transition": [
    "ordered stage flow",
    "legal and rejected transition",
    "difficulty binding",
  ],
  "finish-and-cross-battle-reset": [
    "terminal condition",
    "carry state",
    "reset state",
  ],
  "arithmetic-mapping-eligibility": [
    "eligible character",
    "ineligible character",
    "stronger account build boundary",
  ],
  "arithmetic-mapping-refresh-and-teardown": [
    "refresh timing",
    "temporary substitution",
    "account state unchanged",
  ],
  "equation-offer-recipe-progress-expansion": [
    "offer candidate set",
    "recipe counts",
    "progress and expansion transition",
  ],
  "equation-replacement-and-contribution": [
    "replacement",
    "Blessing ownership change",
    "contribution refresh",
  ],
  "divergent-blessing-level-and-transform": [
    "level state",
    "enhancement or rewrite",
    "Equation contribution",
  ],
  "curio-weight-charge-destruction-repair": [
    "weighted eligibility",
    "charge change",
    "destroy and repair",
  ],
  "grand-miracle-eligibility-and-lifecycle": [
    "character Path and element eligibility",
    "activation",
    "teardown",
  ],
  "golden-blood-titan-choice-and-level": [
    "choice candidates",
    "level transition",
    "run and battle contribution",
  ],
  "threshold-protocol": [
    "entry boundary",
    "difficulty change",
    "enemy or numeric contribution",
  ],
  "astronomical-division": [
    "progress boundary",
    "effect activation",
    "scope",
  ],
  "star-pioneer-practice-and-cognoculi": [
    "mode eligibility",
    "available content",
    "mechanical Cognoculus boundary",
  ],
  "workbench-operation-and-price": [
    "legal operation",
    "currency and price",
    "rejected operation",
  ],
  "gamble-offer-outcome-and-fallback": [
    "candidate set",
    "selected outcome",
    "no legal candidate",
  ],
  "permanent-talent-and-unlock": [
    "prerequisite",
    "unlock transition",
    "mechanical effect",
  ],
  "weekly-modifier-and-room-service": [
    "modifier activation",
    "service eligibility",
    "cross-battle scope",
  ],
  "occurrence-choice-cost-and-outcome": [
    "choice condition",
    "cost",
    "ordered outcome",
  ],
  "adventure-abstract-outcome": [
    "offered abstract result",
    "eligibility",
    "no action gameplay simulation",
  ],
  "encounter-wave-and-boss-binding": [
    "group selection",
    "ordered waves",
    "boss alternative",
  ],
  "simultaneous-trigger-order": [
    "same-phase triggers",
    "stable priority",
    "ordered contributions",
  ],
  "no-legal-candidate-fallback": [
    "empty candidate set",
    "explicit fallback",
    "authoritative state preservation",
  ],
  "battle-visible-and-cross-battle-contribution": [
    "BattleSpec contribution",
    "BattleResult handoff",
    "cross-battle lifecycle",
  ],
};
const fixtureIds = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id);
const fixtureContract = {
  schema_revision: "starclock.divergent-universe-fixture-contract.v1",
  goal_id: "divergent-universe-reference-v1",
  bound_content_manifest_sha256: manifestSha256,
  fixture_role:
    "reference semantics only; fixtures do not claim runtime lowering or playability",
  required_fields: [
    "id",
    "family_id",
    "name_en",
    "name_zh_cn",
    "source_record_ids",
    "preconditions",
    "input",
    "ordered_operations",
    "expected_facts",
    "evidence_refs",
    "evidence_quality",
  ],
  field_contracts: {
    source_record_ids:
      "nonempty sorted stable IDs resolving through pack-index.json",
    preconditions: "typed ordered state facts; no untyped value maps",
    input: "typed decision, transition, trigger or abstract Adventure result",
    ordered_operations:
      "reference operations in asserted semantic order; never runtime handlers",
    expected_facts:
      "exact state/event/contribution observations with canonical decimals",
    evidence_refs:
      "ordered source IDs resolving to sources.json and carrying field quality",
  },
  minimum_cases_per_family: 1,
  required_families: fixtureIds.map((id) => ({
    id,
    minimum_cases: 1,
    must_cover: fixtureDetails[id],
  })),
  coverage_rule: {
    family_coverage: "every required family has at least minimum_cases",
    source_coverage:
      "every distinct mechanic rule references at least one fixture",
    manifest_coverage:
      "every source obligation reaches DataReady or an explicit nonblocking approximation with replacement condition",
    candidate_coverage:
      "every SharedCandidate obligation is promoted by exact proof or explicitly excluded before DataReady",
    reconciliation_coverage:
      "every overlapping Goal 08/09/10 locator has a non-conflicting receipt",
    blocking_gap: "no fixture can satisfy a Blocked manifest obligation",
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
};

write("normalized-schema.json", normalizedSchema);
write("authoring-contract.json", authoringContract);
write("fixture-contract.json", fixtureContract);
console.log(
  `Divergent Universe contracts ${check ? "verified" : "generated"}: ` +
  `${files.length} normalized files, ${fixtureIds.length} fixture families, ` +
  `${authoringContract.workbooks.length} workbooks.`,
);

function checkpoint(goal, requiredNow) {
  const source = JSON.parse(
    fs.readFileSync(
      path.join(root, "content-manifests/divergent-universe-v1/foundation.json"),
      "utf8",
    ),
  ).ownership_checkpoints.find((entry) => entry.goal === goal);
  if (!source) throw new Error(`missing ${goal} ownership checkpoint`);
  return {
    goal: `${source.mode.replace(/([a-z])([A-Z])/gu, "$1-$2").toLowerCase()}-reference-v1`,
    commit: source.commit,
    manifest_sha256: source.content_manifest_sha256,
    records: source.records,
    required_now: requiredNow,
    ...(source.remote_reachable
      ? { remote_ancestor: `origin/${source.branch}` }
      : {
        replacement:
          "replace with a remote-backed or merged checkpoint before G11-P4-B3",
      }),
  };
}

function write(fileName, value) {
  const target = path.join(outputRoot, fileName);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    if (fs.readFileSync(target, "utf8") !== encoded)
      throw new Error(`${fileName} has generated drift`);
  } else {
    fs.mkdirSync(outputRoot, { recursive: true });
    fs.writeFileSync(target, encoded, "utf8");
  }
}

function pascal(value) {
  return value.split(/[-_]/u).map((part) =>
    `${part.charAt(0).toUpperCase()}${part.slice(1)}`).join("");
}

function digestBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
