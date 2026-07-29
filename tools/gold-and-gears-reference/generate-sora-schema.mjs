import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const schemaRoot = path.join(root, "config", "gold-and-gears", "schema");

const string = (name, maximum = 4000, optional = false) => ({
  name,
  type: optional ? "optional<string>" : "string",
  length: optional ? undefined : [1, maximum],
});
const integer = (name, minimum = -2147483648, maximum = 2147483647, optional = false) => ({
  name,
  type: optional ? "optional<i32>" : "i32",
  range: optional ? undefined : [minimum, maximum],
});
const boolean = (name, optional = false) => ({
  name,
  type: optional ? "optional<bool>" : "bool",
});
const list = (name, maximum = 256, optional = true) => ({
  name,
  type: optional ? "optional<list<string>>" : "list<string>",
  parser: { kind: "split", separator: "|" },
  length: optional ? undefined : [1, maximum],
});
const json = (name, optional = false) => string(name, 60000, optional);
const ref = (name, table, optional = false) => ({
  name,
  type: optional ? `optional<ref<${table}.id>>` : `ref<${table}.id>`,
});

const common = [
  integer("id", 1),
  string("stable_key", 240),
  string("schema_revision", 80),
  string("kind", 100),
  string("name_en", 500),
  string("name_zh_cn", 500),
  string("summary_en", 2000),
  string("summary_zh_cn", 2000),
  { name: "ownership", type: "enum<GoldGearsOwnership>" },
  { name: "coverage_state", type: "enum<GoldGearsCoverageState>" },
  { name: "evidence_quality", type: "enum<GoldGearsEvidenceQuality>" },
  list("source_refs", 256),
  list("tags", 64),
];

const coreTables = [
  {
    name: "GoldGearsProfile",
    sheet: "Profile",
    normalized: "profiles.json",
    fields: [
      string("entry_kind", 80, true),
      string("source_id", 100, true),
      string("sub_mode", 80),
      string("unlock_id", 100, true),
      list("reward_item_ids", 64),
      string("game_version", 32, true),
      boolean("runtime_enabled", true),
    ],
  },
  {
    name: "GoldGearsArea",
    sheet: "Area",
    normalized: "areas.json",
    fields: [
      string("source_id", 32),
      string("area_group", 32),
      string("difficulty", 32),
      list("difficulty_segment_ids", 32, false),
      list("plane_ids", 8, false),
      string("unlock_id", 32),
      integer("recommended_level", 1, 200),
      list("recommended_elements", 8, false),
      json("displayed_monsters_json"),
      json("score_thresholds_json"),
    ],
  },
  {
    name: "GoldGearsDifficultySegment",
    sheet: "DifficultySegment",
    normalized: "difficulty-segments.json",
    fields: [
      string("source_id", 32),
      list("cut_positions", 32, false),
      list("levels", 32, false),
    ],
  },
  {
    name: "GoldGearsPlane",
    sheet: "Plane",
    normalized: "planes.json",
    fields: [string("source_id", 32)],
  },
  {
    name: "GoldGearsChessboard",
    sheet: "Chessboard",
    normalized: "chessboards.json",
    fields: [
      string("source_id", 32),
      integer("width", 1, 100),
      integer("height", 1, 100),
      ref("start_node_id", "GoldGearsMapNode"),
      ref("end_node_id", "GoldGearsMapNode"),
      string("config_path", 500),
      string("block_create_group_id", 32),
      list("event_ids", 64),
    ],
  },
  {
    name: "GoldGearsMapColumn",
    sheet: "MapColumn",
    normalized: "map-columns.json",
    fields: [
      ref("chessboard_id", "GoldGearsChessboard"),
      integer("column_index", 0, 100),
      integer("position_x", -100, 100),
      list("node_ids", 64, false),
    ],
  },
  {
    name: "GoldGearsMapNode",
    sheet: "MapNode",
    normalized: "map-nodes.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "GoldGearsChessboard"),
      ref("column_id", "GoldGearsMapColumn"),
      integer("position_x", -100, 100),
      integer("position_y", -100, 100),
      list("domain_ids", 32, false),
      string("domain_resolution", 80),
      boolean("is_start"),
      boolean("is_end"),
    ],
  },
  {
    name: "GoldGearsMapEdge",
    sheet: "MapEdge",
    normalized: "map-edges.json",
    fields: [
      ref("chessboard_id", "GoldGearsChessboard"),
      ref("source_node_id", "GoldGearsMapNode"),
      ref("target_node_id", "GoldGearsMapNode"),
      string("policy", 160),
    ],
  },
  {
    name: "GoldGearsMapEvent",
    sheet: "MapEvent",
    normalized: "map-events.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "GoldGearsChessboard"),
      string("trigger_type", 100),
      list("trigger_params", 64),
      string("effect_type", 100),
      list("effect_params", 64),
      list("secondary_effect_params", 64),
      string("weight", 64),
    ],
  },
  {
    name: "GoldGearsBlockCreateRule",
    sheet: "BlockCreateRule",
    normalized: "block-create-rules.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "GoldGearsChessboard"),
      string("group_id", 32),
      ref("domain_id", "GoldGearsDomain"),
      integer("order", 0, 100),
      json("create_count_weights_json"),
      json("beacon_weights_json"),
    ],
  },
  {
    name: "GoldGearsRoom",
    sheet: "Room",
    normalized: "rooms.json",
    fields: [
      string("source_id", 32),
      string("sub_mode", 80),
      list("section_ids", 32, false),
    ],
  },
  {
    name: "GoldGearsDomain",
    sheet: "Domain",
    normalized: "domains.json",
    fields: [string("source_id", 100)],
  },
  {
    name: "GoldGearsBeacon",
    sheet: "Beacon",
    normalized: "beacons.json",
    fields: [
      string("source_id", 32),
      string("modifier_name", 200),
      string("modifier_name_hash", 32),
    ],
  },
  {
    name: "GoldGearsBossChoice",
    sheet: "BossChoice",
    normalized: "boss-choices.json",
    fields: [
      string("source_id", 32),
      ref("area_id", "GoldGearsArea"),
      string("monster_id", 32),
      string("monster_template_id", 32),
      string("enemy_variant_stable_key", 240),
      integer("display_order", 0, 100),
    ],
  },
  {
    name: "GoldGearsCognitionRange",
    sheet: "CognitionRange",
    normalized: "cognition-ranges.json",
    fields: [
      string("source_id", 32),
      string("area_stable_key", 240),
      string("minimum_cognition", 64),
      string("maximum_cognition", 64),
      boolean("bounds_inclusive"),
      string("global_minimum_cognition", 64),
      string("global_maximum_cognition", 64),
      json("lifecycle_json"),
    ],
  },
  {
    name: "GoldGearsModeConstant",
    sheet: "ModeConstant",
    normalized: "mode-constants.json",
    fields: [
      string("source_id", 200),
      string("mechanical_role", 100),
      string("value_kind", 80),
      list("values", 256, false),
    ],
  },
  {
    name: "GoldGearsDiceCategory",
    sheet: "DiceCategory",
    normalized: "dice-categories.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0),
      string("name_text_hash", 32),
      string("icon_path", 500),
    ],
  },
  {
    name: "GoldGearsDiceDefinition",
    sheet: "DiceDefinition",
    normalized: "dice-definitions.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0),
      ref("category_id", "GoldGearsDiceCategory"),
      string("category_source_id", 32),
      string("name_text_hash", 32),
      string("introduction_text_hash", 32),
      string("effect_bundle_text_hash", 32),
      json("effect_parts_json"),
      list("initial_effect_extra_ids", 64),
      list("passive_effect_extra_ids", 64),
      string("starting_effect_toast_text_hash", 32),
      boolean("available_by_default"),
      string("unlock_id", 32, true),
      string("default_ultra_surface_id", 32),
      list("default_common_surface_ids", 16, false),
      list("default_surface_ids", 16, false),
      list("suggestive_surface_ids", 64),
      list("recommended_surface_ids", 64),
      string("dice_icon_path", 500),
    ],
  },
  {
    name: "GoldGearsDicePathValue",
    sheet: "DicePathValue",
    normalized: "dice-path-values.json",
    fields: [
      string("source_id", 64),
      ref("dice_id", "GoldGearsDiceDefinition"),
      string("dice_source_id", 32),
      string("path_stable_key", 240),
      string("path_source_id", 32),
      string("boost_stat", 100),
      string("trigger_interval", 64),
      string("boost_value", 64),
      string("boost_value_unit", 100),
      list("parameters", 64),
      string("effect_text_hash", 32),
    ],
  },
  {
    name: "GoldGearsDiceSlot",
    sheet: "DiceSlot",
    normalized: "dice-slots.json",
    fields: [
      string("source_id", 32),
      integer("slot_index", 1, 6),
      string("base_name_text_hash", 32),
      string("upgraded_name_en", 200),
      string("upgraded_name_zh_cn", 200),
      string("upgraded_name_text_hash", 32),
      integer("base_max_rarity", 1, 3),
      integer("extra_max_rarity", 1, 3, true),
      integer("upgraded_max_rarity", 1, 3),
    ],
  },
  {
    name: "GoldGearsDiceFace",
    sheet: "DiceFace",
    normalized: "dice-faces.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0),
      string("item_id", 32),
      integer("rarity", 1, 3),
      integer("activation_stage", 0, 100),
      list("parameters", 64),
      string("name_text_hash", 32),
      string("effect_text_hash", 32),
      list("extra_description_ids", 64),
      list("allowed_slot_ids", 8, false),
      list("allowed_slot_source_ids", 8, false),
      list("mechanical_tag_codes", 32),
      list("filter_tag_ids", 32),
      { name: "tag_mapping_evidence_quality", type: "enum<GoldGearsEvidenceQuality>" },
      string("unlock_display_id", 32),
      list("allowed_dice_ids", 16),
      list("allowed_dice_source_ids", 16),
      boolean("universal_dice_eligibility"),
      string("no_legal_target_behavior", 80),
      { name: "no_legal_target_evidence_quality", type: "enum<GoldGearsEvidenceQuality>" },
      json("target_resolution_policy_json"),
      string("icon_path", 500),
    ],
  },
  {
    name: "GoldGearsDiceFaceTag",
    sheet: "DiceFaceTag",
    normalized: "dice-face-tags.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0),
      string("name_text_hash", 32),
      string("mechanical_code", 100),
      { name: "mapping_evidence_quality", type: "enum<GoldGearsEvidenceQuality>" },
      string("mapping_replacement_condition", 1000),
    ],
  },
  {
    name: "GoldGearsKnowledgeRule",
    sheet: "KnowledgeRule",
    normalized: "knowledge-rules.json",
    fields: [
      string("source_id", 32),
      ref("dice_face_id", "GoldGearsDiceFace"),
      string("operation", 160),
      string("trigger_boundary", 100),
      string("target_scope", 160),
      string("selection_mode", 80),
      string("knowledge_access", 80),
      list("parameters", 64),
      integer("activation_stage", 0, 100),
      string("effect_text_hash", 32),
      json("target_policy_json"),
      json("simultaneous_resolution_policy_json"),
      json("custom_dice_interactions_json"),
    ],
  },
];

const progressionTables = [
  {
    name: "GoldGearsSecret",
    sheet: "Secret",
    normalized: "secrets.json",
    fields: [
      string("source_id", 32),
      string("required_area_stable_key", 240),
      string("required_area_source_id", 32),
      integer("plane_layer", 1, 3),
      string("minimum_cognition", 64),
      string("maximum_cognition", 64),
      string("minimum_origin", 80),
      string("maximum_origin", 80),
      boolean("bounds_inclusive"),
      list("predecessor_secret_ids", 32),
      list("next_secret_ids", 32),
      string("evaluation_boundary", 100),
      string("trigger_condition_hash", 32),
      string("trigger_condition_digest", 64),
      boolean("terminal"),
      string("lifecycle_policy_id", 160),
    ],
  },
  {
    name: "GoldGearsNeuralNetwork",
    sheet: "NeuralNetwork",
    normalized: "neural-network.json",
    fields: [
      { name: "mechanism_quality", type: "enum<GoldGearsEvidenceQuality>" },
      json("quality_overrides_json"),
      string("source_id", 32),
      integer("topological_index", 1, 100),
      list("prerequisite_ids", 32),
      list("next_ids", 32),
      list("external_unlock_ids", 32),
      json("costs_json"),
      boolean("important"),
      string("disposition", 100),
      string("effect_domain", 100),
      string("effect_tag_en", 200),
      string("effect_tag_zh_cn", 200),
      string("effect_tag_text_hash", 32),
      string("title_text_hash", 32),
      string("description_text_hash", 32),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      json("source_parameters_json"),
      json("effect_contributions_json"),
      string("rule_contribution_id", 240),
    ],
  },
  {
    name: "GoldGearsConundrumLevel",
    sheet: "ConundrumLevel",
    normalized: "conundrum-levels.json",
    fields: [
      { name: "mechanism_quality", type: "enum<GoldGearsEvidenceQuality>" },
      json("quality_overrides_json"),
      string("source_id", 32),
      string("source_type", 100),
      string("track", 80),
      integer("level", 1, 6),
      integer("track_cap", 6, 6),
      integer("total_conundrum_cap", 12, 12),
      string("total_level_formula", 160),
      json("unlock_requirement_json"),
      string("composition_mode", 160),
      list("active_contribution_ids", 32, false),
      list("replaces_level_ids", 32),
      integer("source_tag", 0),
      integer("source_sort", 0),
      string("description_text_hash", 32),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      json("source_parameters_json"),
      json("effect_contributions_json"),
      string("rule_contribution_id", 240),
    ],
  },
  {
    name: "GoldGearsPath",
    sheet: "Path",
    normalized: "paths.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0),
      integer("buff_type", 0),
      ref("shared_resonance_id", "GoldGearsResonance"),
      list("shared_formation_ids", 8, false),
      ref("path_boost_id", "GoldGearsPathBoost"),
      string("normal_battle_event_group", 32),
      string("enhanced_battle_event_group", 32),
    ],
  },
  {
    name: "GoldGearsResonance",
    sheet: "Resonance",
    normalized: "resonances.json",
    fields: [
      string("source_id", 32),
      ref("path_id", "GoldGearsPath"),
      string("resonance_kind", 80),
      integer("threshold", 0, 100),
      string("energy_max", 64),
      string("initial_energy", 64),
      json("parameter_values_json"),
      list("mechanic_tags", 64),
      string("source_modifier_name", 200),
      string("source_binding_type", 160),
      string("source_binding_key", 160),
      list("inherited_rule_ids", 64),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
    ],
  },
  {
    name: "GoldGearsPathBoost",
    sheet: "PathBoost",
    normalized: "path-boosts.json",
    fields: [
      string("source_id", 32),
      ref("path_id", "GoldGearsPath"),
      string("aeon_source_id", 32),
      string("effect_type", 100),
      string("ability_name", 200),
      string("target_team", 80),
      string("target_property", 160),
      string("boost_stat", 100),
      string("stacking", 100),
      string("source_value_conversion", 160),
      list("dice_path_value_ids", 32, false),
      list("allowed_increment_values", 32, false),
      string("description_text_hash", 32),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      string("rule_contribution_id", 240),
    ],
  },
  {
    name: "GoldGearsResonanceExtrapolation",
    sheet: "ResonanceExtrapolation",
    normalized: "resonance-extrapolations.json",
    fields: [
      { name: "mechanism_quality", type: "enum<GoldGearsEvidenceQuality>" },
      json("quality_overrides_json"),
      string("source_id", 32),
      ref("path_id", "GoldGearsPath"),
      string("aeon_source_id", 32),
      string("buff_group_id", 32),
      boolean("enhanced"),
      ref("shared_resonance_id", "GoldGearsResonance"),
      string("shared_resonance_kind", 80),
      string("source_battle_event_type", 160),
      string("source_modifier_name", 200),
      string("source_binding_type", 160),
      string("source_binding_key", 160),
      json("source_parameters_json"),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      string("battle_scope", 100),
      json("controller_policy_json"),
      string("rule_contribution_id", 240),
    ],
  },
  {
    name: "GoldGearsResonanceInterplay",
    sheet: "ResonanceInterplay",
    normalized: "resonance-interplays.json",
    fields: [
      string("source_id", 32),
      ref("main_path_id", "GoldGearsPath"),
      ref("sub_path_id", "GoldGearsPath"),
      integer("main_blessing_threshold", 0, 100),
      integer("sub_blessing_threshold", 0, 100),
      string("buff_group_id", 32),
      string("shared_maze_buff_id", 32),
      string("source_modifier_name", 200),
      string("source_binding_type", 160),
      string("source_binding_key", 160),
      json("source_parameters_json"),
      string("name_text_hash", 32),
      string("description_text_hash", 32),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      string("rule_contribution_id", 240),
    ],
  },
  {
    name: "GoldGearsTrailblazeBonus",
    sheet: "TrailblazeBonus",
    normalized: "bonuses.json",
    fields: [
      string("source_id", 32),
      string("bonus_event_id", 32),
      string("title_text_hash", 32),
      string("description_text_hash", 32),
      string("tag_text_hash", 32),
      string("source_description_sha256_en", 64),
      string("source_description_sha256_zh_cn", 64),
      json("effect_contributions_json"),
      string("rule_contribution_id", 240),
    ],
  },
];

const enumDefinitions = [
  ["GoldGearsOwnership", ["GoldAndGears", "Shared"]],
  ["GoldGearsCoverageState", ["DataReady"]],
  [
    "GoldGearsEvidenceQuality",
    ["ExactStructured", "ExactPublicText", "Observed", "ApproximateFromReleasedText", "ProjectPolicy"],
  ],
];

generate("core.toml", "GoldAndGears.xlsx", coreTables, enumDefinitions);
generate("progression.toml", "GoldAndGearsProgression.xlsx", progressionTables);
console.log(`Generated Gold and Gears Sora schema (${coreTables.length + progressionTables.length} tables).`);

function generate(filename, workbook, tables, enums = []) {
  const lines = [
    "# @generated by tools/gold-and-gears-reference/generate-sora-schema.mjs",
    "# Do not edit by hand.",
    "",
  ];
  for (const [name, values] of enums) {
    lines.push("[[enums]]", `name = ${quote(name)}`, `values = ${toml(values)}`, "");
  }
  for (const table of tables) {
    lines.push(
      "[[tables]]",
      `name = ${quote(table.name)}`,
      'mode = "map"',
      'key = "id"',
      "[tables.source]",
      'format = "xlsx"',
      `file = ${quote(workbook)}`,
      `sheet = ${quote(table.sheet)}`,
    );
    for (const field of [...common, ...table.fields]) {
      lines.push("[[tables.fields]]", `name = ${quote(field.name)}`, `type = ${quote(field.type)}`);
      if (field.parser) lines.push(`parser = ${toml(field.parser)}`);
      if (field.length) lines.push(`length = ${toml(field.length)}`);
      if (field.range) lines.push(`range = ${toml(field.range)}`);
    }
    lines.push(
      "[[tables.indexes]]",
      `name = ${quote("by_stable_key")}`,
      'fields = ["stable_key"]',
      "unique = true",
      "",
    );
  }
  fs.mkdirSync(schemaRoot, { recursive: true });
  fs.writeFileSync(path.join(schemaRoot, filename), `${lines.join("\n")}\n`);
}

function quote(value) {
  return JSON.stringify(value);
}

function toml(value) {
  if (Array.isArray(value)) return `[${value.map(toml).join(", ")}]`;
  if (typeof value === "string") return quote(value);
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return `{ ${Object.entries(value).map(([key, item]) => `${key} = ${toml(item)}`).join(", ")} }`;
}
