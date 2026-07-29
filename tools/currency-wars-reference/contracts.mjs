#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root") ?? ".");
const manifestRelative =
  "content-manifests/currency-wars-v1/content-manifest.json";
const manifestPath = path.join(root, manifestRelative);
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const manifestSha256 = digestBytes(fs.readFileSync(manifestPath));
const outputRoot = path.join(
  root,
  "content-manifests/currency-wars-v1",
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
    record_kind: `CurrencyWars${pascal(fileName.replace(/\.json$/u, ""))}`,
    phase,
    manifest_category_inputs: manifestInputs,
    ordering_keys: ["id"],
    required_domain_fields: requiredDomainFields,
  };
}

const files = [
  file("profiles.json", "P1-B1",
    ["profiles", "entry_points", "enabled_modules"],
    ["entry_refs", "module_id", "gambit_mode_ids", "initial_resources", "finish_condition_ids"]),
  file("gambit-modes.json", "P1-B1", ["gambit_modes"],
    ["mode_kind", "unlock_ids", "entry_rules", "initial_resources"]),
  file("modules.json", "P1-B1", ["enabled_modules"],
    ["sub_mode", "tourn_mode", "main_tourn_id", "sub_tourn_id"]),
  file("entries.json", "P1-B1", ["entry_points"],
    ["entry_kind", "module_id", "unlock_ids", "gambit_mode_ids"]),
  file("finish-conditions.json", "P1-B1", ["finish_conditions"],
    ["condition_kind", "parameters", "terminal_disposition"]),
  file("area-groups.json", "P1-B1", ["area_groups"],
    ["area_ids", "selection_policy", "transition_rules"]),
  file("areas.json", "P1-B1", ["areas"],
    ["plane_number", "difficulty_ids", "layer_ids", "map_entry_id"]),
  file("difficulties.json", "P1-B1", ["difficulties"],
    ["rank_bounds", "enemy_scaling_refs", "gambit_rules"]),
  file("layers.json", "P1-B1", ["layers"],
    ["plane_id", "layer_number", "ordered_node_ids"]),
  file("rooms.json", "P1-B1", ["room_reuse_candidates"],
    ["room_type", "reachability_disposition", "stage_refs"]),
  file("nodes.json", "P1-B1", [],
    ["plane_id", "layer_id", "ordinal", "domain_composition_id", "room_pool_id"]),
  file("domain-compositions.json", "P1-B1", [],
    ["domain_type", "room_candidate_ids", "selection_policy", "fallback"]),
  file("stage-flow.json", "P1-B1", [],
    ["entry_id", "ordered_node_refs", "carry_rules", "reset_rules"]),

  file("squad-hp-rules.json", "P1-B2", ["squad_hp_action_value_envelopes"],
    ["initial_hp", "minimum_hp", "maximum_hp", "loss_rules", "recovery_rules"]),
  file("action-value-limits.json", "P1-B2", [],
    ["limit_kind", "initial_value", "decrement_rules", "timeout_boundary"]),
  file("battle-result-projections.json", "P1-B2", [],
    ["battle_outcome", "squad_hp_projection", "action_value_projection", "run_disposition"]),
  file("run-failure-rules.json", "P1-B2", [],
    ["failure_condition", "same_boundary_order", "terminal_disposition"]),

  file("roster-avatars.json", "P1-B3", ["roster_avatars"],
    ["avatar_id", "cost", "role_id", "build_mapping_id"]),
  file("economy-rules.json", "P1-B3", ["economy_shop_envelopes"],
    ["currency_ids", "experience_rules", "refresh_rules", "team_size_rules"]),
  file("roster-offers.json", "P1-B3", [],
    ["candidate_avatar_ids", "weights", "cost_rule", "fallback"]),
  file("roster-transactions.json", "P1-B3", [],
    ["operation", "price_rule", "eligibility", "ordered_state_changes"]),
  file("team-size-states.json", "P1-B3", [],
    ["level", "field_cap", "bench_cap", "transition_rules"]),

  file("role-mappings.json", "P1-B4", ["role_mappings"],
    ["avatar_id", "position_ids", "empowerment_ids"]),
  file("positions.json", "P1-B4", ["position_empowerment_envelopes"],
    ["position_kind", "field_index", "validation_rules", "battle_contributions"]),
  file("character-empowerments.json", "P1-B4", [],
    ["avatar_id", "position_id", "activation", "effect_ids", "teardown"]),
  file("battle-overrides.json", "P1-B4", [],
    ["rule_kind", "trigger", "parameters", "ordered_operations", "teardown"]),

  file("bonds.json", "P1-B5", ["bond_envelopes"],
    ["member_ids", "level_ids", "recompute_timing", "contribution_ids"]),
  file("bond-levels.json", "P1-B5", [],
    ["bond_id", "level", "threshold", "effect_ids"]),
  file("bond-contributions.json", "P1-B5", [],
    ["bond_id", "level", "scope", "activation", "ordered_effects"]),

  file("star-states.json", "P1-B6", ["star_upgrade_envelopes"],
    ["avatar_id", "star_level", "copy_count", "scaling_refs"]),
  file("star-combination-rules.json", "P1-B6", [],
    ["input_state", "required_copies", "output_state", "overflow_rule"]),
  file("star-lifecycle-rules.json", "P1-B6", [],
    ["operation", "replacement_rule", "sale_rule", "teardown"]),

  file("build-reference-avatars.json", "P1-B7", ["build_reference_avatars"],
    ["avatar_id", "owned_build_id", "trial_build_id", "eligibility"]),
  file("build-source-files.json", "P1-B7", ["build_source_files"],
    ["source_path", "source_sha256", "mapping_role", "disposition"]),
  file("build-mappings.json", "P1-B7", [],
    ["avatar_id", "level", "trace_state", "light_cone", "relics"]),
  file("build-substitution-rules.json", "P1-B7", [],
    ["selection_timing", "owned_trial_policy", "refresh_timing", "teardown"]),
  file("off-field-conversions.json", "P1-B7", [],
    ["source_kind", "eligibility", "conversion", "destination_state"]),
  file("equipment.json", "P1-B7", [],
    ["slot", "eligibility", "effect_ids", "replacement_rule"]),

  file("persona-constants-client.json", "P1-B8", ["persona_const_client"],
    ["value_name", "canonical_value", "consumer_ids"]),
  file("persona-constants-common.json", "P1-B8", ["persona_const_common"],
    ["value_name", "canonical_value", "consumer_ids"]),
  file("persona-layer-rooms.json", "P1-B8", ["persona_layer_room"],
    ["layer_id", "room_id", "style_ids", "selection_policy"]),
  file("persona-room-attributes.json", "P1-B8", ["persona_room_attribute"],
    ["attribute_kind", "parameters", "effect_ids"]),
  file("persona-room-composition-types.json", "P1-B8", ["persona_room_comp_type"],
    ["composition_kind", "eligibility", "selection_policy"]),
  file("persona-room-compositions.json", "P1-B8", ["persona_room_composition"],
    ["composition_type_id", "ordered_room_ids", "activation"]),
  file("persona-room-presets.json", "P1-B8", ["persona_room_preset"],
    ["composition_id", "room_ids", "weight_program"]),
  file("persona-styles.json", "P1-B8", ["persona_style"],
    ["environment_kind", "strategy_ids", "gift_pool_ids"]),
  file("persona-style-gifts.json", "P1-B8", ["persona_style_gift"],
    ["style_id", "gift_kind", "parameters", "effect_ids"]),
  file("persona-talents.json", "P1-B8", ["persona_talent"],
    ["talent_group_id", "cost", "prerequisite_ids", "effect_ids"]),
  file("persona-talent-groups.json", "P1-B8", ["persona_talent_group"],
    ["talent_ids", "selection_policy", "activation"]),
  file("investment-environments.json", "P1-B8", [],
    ["persona_style_ids", "entry_rules", "state_lifecycle"]),
  file("investment-strategies.json", "P1-B8", [],
    ["environment_id", "gift_ids", "offer_rules", "activation"]),

  file("rank-gambit-progression.json", "P1-B9",
    ["rank_gambit_progression_envelopes"],
    ["rank", "gambit_mode", "entry_boundary", "enemy_affix_ids"]),
  file("enemy-affixes.json", "P1-B9", [],
    ["rank_bounds", "difficulty_ids", "battle_contributions"]),
  file("permanent-progression.json", "P1-B9", [],
    ["source_id", "scope", "entry_changes", "available_choice_changes"]),

  file("blessing-paths.json", "P2-B1", ["blessing_paths"],
    ["path_id", "offer_roles", "formula_roles"]),
  file("blessings.json", "P2-B1", ["blessings"],
    ["path_id", "category", "level_ids", "effect_ids"]),
  file("blessing-levels.json", "P2-B1", ["blessing_levels"],
    ["blessing_id", "level", "parameters", "effect_ids"]),
  file("blessing-groups.json", "P2-B1", ["blessing_groups"],
    ["candidate_ids", "selection_policy", "weight_program"]),
  file("formulas.json", "P2-B1", ["formulas"],
    ["formula_kind", "recipe_id", "progress_states", "effect_ids"]),
  file("formula-displays.json", "P2-B1", ["formula_displays"],
    ["formula_id", "display_state", "mechanical_summary_ids"]),
  file("formula-randomizers.json", "P2-B1", ["formula_randomizers"],
    ["candidate_ids", "weight_program", "reroll_rule", "fallback"]),
  file("formula-recipes.json", "P2-B1", [],
    ["required_path_counts", "required_blessing_states", "completion_rule"]),
  file("formula-contributions.json", "P2-B1", [],
    ["formula_id", "source_state", "scope", "ordered_effects"]),

  file("curios.json", "P2-B2", ["curios"],
    ["category", "state_ids", "eligibility_rule_ids"]),
  file("curio-states.json", "P2-B2", ["curio_states"],
    ["curio_id", "state", "charges", "effect_ids"]),
  file("curio-groups.json", "P2-B2", ["curio_groups"],
    ["candidate_state_ids", "weights", "eligibility"]),
  file("curio-lifecycle-rules.json", "P2-B2", [],
    ["curio_id", "activation", "destruction", "repair", "replacement", "fallback"]),
  file("hex-states.json", "P2-B2", ["hex_states"],
    ["hex_id", "state", "activation", "effect_ids", "teardown"]),
  file("hex-eligibility.json", "P2-B2", ["hex_eligibility"],
    ["hex_id", "subject_id", "eligibility", "replacement"]),

  file("occurrences.json", "P2-B3", ["occurrences"],
    ["variant_ids", "unlock_rules", "choice_ids"]),
  file("occurrence-variants.json", "P2-B3", ["occurrence_service_variants"],
    ["occurrence_id", "graph_path", "entry_conditions", "choice_ids"]),
  file("occurrence-choices.json", "P2-B3", [],
    ["variant_id", "ordinal", "conditions", "costs", "ordered_outcomes"]),

  file("workbenches.json", "P2-B4", ["workbenches"],
    ["function_ids", "currency_ids", "availability"]),
  file("workbench-functions.json", "P2-B4", ["workbench_functions"],
    ["function_type", "input_policy", "output_policy", "price_rule"]),
  file("gamble-groups.json", "P2-B4", ["gamble_groups"],
    ["group_type", "unit_ids", "offer_policy"]),
  file("gamble-units.json", "P2-B4", ["gamble_units"],
    ["unit_type", "parameters", "outcome_program"]),
  file("curse-chests.json", "P2-B4", ["curse_chests"],
    ["chest_type", "parameters", "choice_program"]),
  file("adventure-outcomes.json", "P2-B4", ["adventure_outcomes"],
    ["adventure_type", "parameter_group_id", "abstract_outcome"]),
  file("currencies.json", "P2-B4", [],
    ["scope", "gain_rules", "spend_rules", "reset_rule"]),
  file("shop-services.json", "P2-B4", [],
    ["service_kind", "price_rule", "inventory_rule", "refresh_rule"]),
  file("service-offer-rules.json", "P2-B4", [],
    ["service_id", "candidate_ids", "weights", "fallback"]),

  file("encounter-source-obligations.json", "P2-B5",
    ["encounter_source_obligations"],
    ["parent_kind", "parent_id", "resolution_state"]),
  file("encounter-groups.json", "P2-B5", [],
    ["plane_id", "difficulty_id", "rank", "candidate_stage_ids"]),
  file("encounter-waves.json", "P2-B5", [],
    ["stage_id", "wave_index", "enemy_slot_ids", "trigger"]),
  file("enemy-slots.json", "P2-B5", [],
    ["wave_id", "slot_index", "monster_id", "level", "ability_refs"]),
  file("boss-pools.json", "P2-B5", [],
    ["plane_id", "difficulty_id", "candidate_monster_ids", "selection_policy"]),

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
  file("reconciliation-receipts.json", "P4-B1", [],
    ["source_path", "row_locator", "evidence_sha256", "checkpoint", "outcome"]),
  file("manifest.json", "P2-B6", [],
    ["content_manifest_sha256", "normalized_files", "record_counts"]),
  file("pack-index.json", "P2-B6", [],
    ["pack_digest", "file_digests", "stable_id_index"]),
];

const normalizedSchema = {
  schema_revision: "starclock.currency-wars-normalized-schema.v1",
  goal_id: "currency-wars-reference-v1",
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
      value: "starclock.currency-wars-row.v1",
    },
    kind: { type: "string", closed_by_file_contract: true },
    name_en: { type: "string", nonempty: true },
    name_zh_cn: { type: "string", nonempty: true },
    summary_en: { type: "string", nonempty: true, mechanical_only: true },
    summary_zh_cn: { type: "string", nonempty: true, mechanical_only: true },
    ownership: {
      enum: ["CurrencyWars", "Shared"],
      candidate_rule:
        "EvidenceOnly manifest obligations cannot become normalized rows until promoted by an exact reconciliation receipt",
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
      "EvidenceOnly rows remain Cataloged or Blocked until an exact selector/reference/stable-ID receipt promotes or excludes them",
    derived_files:
      "derived rows reuse a parent category and cannot create or remove a source obligation",
    child_rows:
      "typed child rows carry parent stable ID and deterministic ordinal",
    unknown_reference: "reject",
  },
  reconciliation_policy: {
    checkpoints: [
      checkpoint("G08", false),
      checkpoint("G09", true),
      checkpoint("G10", true),
      goal11ConflictCheckpoint(),
    ],
    join_key: ["source_path", "row_locator", "evidence_sha256"],
    outcomes: [
      "MatchedShared",
      "CurrencyWarsOnly",
      "OtherGoalOnly",
      "DivergentRepresentation",
      "Conflict",
    ],
    conflict_behavior:
      "Blocked; record the conflict and wait for merge coordination without mutating another Goal",
    required_receipt_fields: [
      "id",
      "source_path",
      "row_locator",
      "evidence_sha256",
      "checkpoint_goal",
      "checkpoint_commit",
      "checkpoint_ownership",
      "currency_wars_ownership",
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
  schema_revision: "starclock.currency-wars-authoring-contract.v1",
  goal_id: "currency-wars-reference-v1",
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
    project: "config/currency-wars/project.toml",
    schema_root: "config/currency-wars/schema/",
    workbook_root: "config/currency-wars/workbooks/",
    generated_root: "config/currency-wars-generated/",
    generated_reader_root: "config/currency-wars-generated/reader/",
    forbidden_outputs: [
      "config/generated/",
      "config/universe-generated/",
      "config/gold-and-gears-generated/",
      "config/swarm-disaster-generated/",
      "config/unknowable-domain-generated/",
      "config/divergent-universe-generated/",
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
    header_style: "goal12-header-v1",
    alternating_rows: true,
    wrapped_text: true,
    deterministic_widths: true,
    data_validation: true,
    typed_references: true,
    canonical_decimal_cells: "text",
  },
  workbooks: [
    {
      file: "CurrencyWars.xlsx",
      purpose:
        "entry, flow, Squad HP/action value, roster economy, positions, Empowerments, Bonds, stars, builds, Persona and rank progression",
      normalized_files: mainWorkbookFiles,
    },
    {
      file: "CurrencyWarsBindings.xlsx",
      purpose:
        "Blessings, formulas, Curios, Hexes, Occurrences, services, Adventure and encounters",
      normalized_files: bindingWorkbookFiles,
    },
    {
      file: "CurrencyWarsReview.xlsx",
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
      "CurrencyWars plus PascalCase normalized file stem; child tables append semantic role",
    sheet_names: "explicit in Sora schema and at most 31 characters",
    numeric_keys: "private workbook identities only",
    stable_keys: "required for every cross-workbook or inherited reference",
  },
  reconciliation_sheet: {
    workbook: "CurrencyWarsReview.xlsx",
    normalized_file: "reconciliation-receipts.json",
    join_key: "source path plus row locator plus evidence SHA-256",
    conflict_behavior:
      "Block terminal Goal 12 reconciliation; do not edit Goal 08/09/10/11 workbooks or manifests",
  },
  acceptance: {
    schema_check: "sora check --project config/currency-wars/project.toml",
    schema_build: "sora build --project config/currency-wars/project.toml",
    export: "sora export --project config/currency-wars/project.toml",
    reader_load: "every generated table and row through the isolated reader",
    visual_review: "render and inspect every authored sheet",
    semantic_digest:
      "canonical cell values, types, validations, sheet order and table identity",
  },
};

const fixtureDetails = {
  "approximation-replacement-trigger": [
    "reviewed approximation", "released replacement evidence", "deterministic reclassification",
  ],
  "automatic-technique-energy-and-lethal-rescue": [
    "automatic Technique", "defeat-energy rule", "lethal rescue and countdown",
  ],
  "battle-visible-rule-contribution": [
    "BattleSpec contribution", "battle observation", "teardown",
  ],
  "blessing-level-offer-and-enhancement": [
    "offer candidates", "level transition", "enhanced contribution",
  ],
  "bond-membership-threshold-and-recompute": [
    "membership change", "threshold boundary", "simultaneous recompute",
  ],
  "candidate-order-and-no-legal-result": [
    "ordered candidates", "eligibility rejection", "explicit fallback",
  ],
  "cross-battle-state-and-reset": [
    "state creation", "battle boundary carry", "terminal reset",
  ],
  "curio-state-charge-destruction-and-repair": [
    "charge change", "destroyed state", "repair and replacement",
  ],
  "encounter-wave-elite-and-boss-binding": [
    "group selection", "ordered waves", "elite or boss alternative",
  ],
  "field-bench-position-and-empowerment": [
    "deployment validation", "position contribution", "Empowerment teardown",
  ],
  "formula-recipe-progress-and-contribution": [
    "recipe requirements", "progress transition", "battle contribution",
  ],
  "gambit-rank-and-enemy-affix": [
    "Gambit selection", "rank boundary", "enemy affix contribution",
  ],
  "goal11-selector-conflict-reconciliation": [
    "exact source locator", "incompatible ownership", "blocked merge outcome",
  ],
  "gold-coin-refresh-experience-and-team-size": [
    "currency mutation", "refresh cost", "Experience and team-size transition",
  ],
  "hex-eligibility-activation-and-teardown": [
    "eligibility", "activation", "replacement or teardown",
  ],
  "investment-environment-strategy-and-persona": [
    "Environment entry", "Strategy offer", "Persona activation lifecycle",
  ],
  "occurrence-choice-cost-and-outcome": [
    "choice condition", "cost", "ordered mechanical outcome",
  ],
  "off-field-conversion-and-equipment-slots": [
    "conversion eligibility", "three-slot cap", "replacement",
  ],
  "other-mode-ownership-rejection": [
    "named other-mode row", "failed selector", "no normalized promotion",
  ],
  "owned-trial-build-substitution-and-removal": [
    "owned build boundary", "trial fallback", "account state unchanged",
  ],
  "profile-gambit-entry-and-terminal": [
    "Standard or Overclock entry", "initial state", "terminal disposition",
  ],
  "roster-offer-cost-purchase-sale-and-cap": [
    "offer candidates", "purchase or sale", "field and bench cap",
  ],
  "shop-service-price-inventory-and-fallback": [
    "service eligibility", "price and inventory", "no legal offer fallback",
  ],
  "simultaneous-bond-star-and-roster-order": [
    "same-boundary changes", "stable operation order", "final recomputation",
  ],
  "squad-hp-action-value-same-boundary-order": [
    "battle result", "Squad HP projection", "action-value projection order",
  ],
  "squad-hp-victory-timeout-and-run-failure": [
    "victory", "timeout", "zero-HP run failure",
  ],
  "star-copy-combine-overflow-and-teardown": [
    "copy acquisition", "three-copy combination", "overflow and teardown",
  ],
  "three-plane-node-room-flow": [
    "Plane transition", "Node choice", "room and Stage binding",
  ],
};
const fixtureIds = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id);
const fixtureContract = {
  schema_revision: "starclock.currency-wars-fixture-contract.v1",
  goal_id: "currency-wars-reference-v1",
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
      "every EvidenceOnly obligation is promoted by exact proof or explicitly excluded before DataReady",
    reconciliation_coverage:
      "every overlapping Goal 08/09/10/11 locator has a receipt; conflicts remain Blocked until coordinated",
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
  `Currency Wars contracts ${check ? "verified" : "generated"}: ` +
  `${files.length} normalized files, ${fixtureIds.length} fixture families, ` +
  `${authoringContract.workbooks.length} workbooks.`,
);

function checkpoint(goal, requiredNow) {
  const source = JSON.parse(
    fs.readFileSync(
      path.join(root, "content-manifests/currency-wars-v1/foundation.json"),
      "utf8",
    ),
  ).ownership_checkpoints.find((entry) =>
    entry.goal.replace("Goal ", "G") === goal);
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
          "replace with a remote-backed or merged checkpoint before G12-P4-B3",
      }),
  };
}

function goal11ConflictCheckpoint() {
  const conflict = manifest.reconciliation.find(({ goal }) => goal === "Goal 11");
  if (!conflict) throw new Error("missing Goal 11 reconciliation conflict");
  return {
    goal: "divergent-universe-reference-v1",
    commit: conflict.commit,
    manifest_sha256: conflict.manifest_sha256,
    records: 6215,
    required_now: true,
    remote_ancestor: `${conflict.remote}/${conflict.branch}`,
    state: conflict.state,
    replacement_condition: conflict.replacement_condition,
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
