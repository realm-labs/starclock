#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const schemaRoot = path.join(root, "config", "swarm-disaster", "schema");

const string = (name, maximum = 4000, optional = false) => ({
  name,
  type: optional ? "optional<string>" : "string",
  length: optional ? undefined : [1, maximum],
});
const integer = (
  name,
  minimum = -2147483648,
  maximum = 2147483647,
  optional = false,
) => ({
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
  string("schema_revision", 100),
  string("kind", 100),
  string("name_en", 500),
  string("name_zh_cn", 500),
  string("summary_en", 2400),
  string("summary_zh_cn", 2400),
  { name: "ownership", type: "enum<SwarmDisasterOwnership>" },
  { name: "coverage_state", type: "enum<SwarmDisasterCoverageState>" },
  { name: "evidence_quality", type: "enum<SwarmDisasterEvidenceQuality>" },
  list("source_refs", 512),
  list("tags", 64),
];

const coreTables = [
  {
    name: "SwarmDisasterProfile",
    sheet: "Profile",
    normalized: "profiles.json",
    fields: [
      string("entry_kind", 80, true),
      string("source_id", 100, true),
      string("sub_mode", 80, true),
      string("unlock_id", 100, true),
      list("reward_item_ids", 64),
      string("game_version", 32, true),
      boolean("runtime_enabled", true),
      list("entry_refs", 8),
      list("formal_difficulty_ids", 8),
      list("bonus_ids", 8),
    ],
  },
  {
    name: "SwarmDisasterArea",
    sheet: "Area",
    normalized: "areas.json",
    fields: [
      string("source_id", 32),
      string("area_kind", 32),
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
    name: "SwarmDisasterDifficultySegment",
    sheet: "DifficultySegment",
    normalized: "difficulty-segments.json",
    fields: [
      string("source_id", 32),
      list("cut_list", 32, false),
      list("level_list", 32, false),
    ],
  },
  {
    name: "SwarmDisasterPlane",
    sheet: "Plane",
    normalized: "planes.json",
    fields: [
      string("source_id", 32),
      integer("plane_number", 1, 3),
      list("chessboard_ids", 32, false),
      string("terminal_policy", 100),
    ],
  },
  {
    name: "SwarmDisasterChessboard",
    sheet: "Chessboard",
    normalized: "chessboards.json",
    fields: [
      string("source_id", 32),
      integer("width", 1, 100),
      integer("height", 1, 100),
      ref("start_node_id", "SwarmDisasterMapNode"),
      ref("end_node_id", "SwarmDisasterMapNode"),
      string("source_config_path", 500),
      string("block_create_group_id", 32),
      list("event_ids", 128),
    ],
  },
  {
    name: "SwarmDisasterMapColumn",
    sheet: "MapColumn",
    normalized: "map-columns.json",
    fields: [
      ref("chessboard_id", "SwarmDisasterChessboard"),
      integer("column_index", 0, 100),
      integer("position_x", -100, 100),
      list("node_ids", 128, false),
    ],
  },
  {
    name: "SwarmDisasterMapNode",
    sheet: "MapNode",
    normalized: "map-nodes.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "SwarmDisasterChessboard"),
      ref("column_id", "SwarmDisasterMapColumn"),
      integer("position_x", -100, 100),
      integer("position_y", -100, 100),
      list("domain_candidates", 64),
      string("domain_resolution", 100),
      boolean("is_start"),
      boolean("is_end"),
    ],
  },
  {
    name: "SwarmDisasterMapEdge",
    sheet: "MapEdge",
    normalized: "map-edges.json",
    fields: [
      ref("chessboard_id", "SwarmDisasterChessboard"),
      ref("from_node_id", "SwarmDisasterMapNode"),
      ref("to_node_id", "SwarmDisasterMapNode"),
      string("policy_id", 160),
    ],
  },
  {
    name: "SwarmDisasterMapEvent",
    sheet: "MapEvent",
    normalized: "map-events.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "SwarmDisasterChessboard"),
      json("trigger_json"),
      string("weight", 64),
      json("ordered_effects_json"),
    ],
  },
  {
    name: "SwarmDisasterBlockCreateRule",
    sheet: "BlockCreateRule",
    normalized: "block-create-rules.json",
    fields: [
      string("source_id", 32),
      ref("chessboard_id", "SwarmDisasterChessboard"),
      string("group_id", 32),
      ref("domain_id", "SwarmDisasterDomain"),
      integer("order", 0, 100),
      json("count_json"),
      json("mark_candidates_json"),
    ],
  },
  {
    name: "SwarmDisasterRoom",
    sheet: "Room",
    normalized: "rooms.json",
    fields: [
      string("source_id", 32),
      string("sub_mode", 80),
      list("section_ids", 32, false),
      ref("domain_id", "SwarmDisasterDomain", true),
      list("encounter_pool_ids", 128),
      string("domain_binding_state", 100),
      string("encounter_binding_state", 100),
    ],
  },
  {
    name: "SwarmDisasterDomain",
    sheet: "Domain",
    normalized: "domains.json",
    fields: [
      string("source_id", 100),
      json("selection_policy_json"),
      json("replacement_policy_json"),
    ],
  },
  {
    name: "SwarmDisasterBeacon",
    sheet: "Beacon",
    normalized: "beacons.json",
    fields: [
      string("source_id", 32),
      string("block_intro_id", 32),
      string("application_stage", 100),
      json("copy_policy_json"),
      json("blanking_policy_json"),
    ],
  },
  {
    name: "SwarmDisasterBossChoice",
    sheet: "BossChoice",
    normalized: "boss-choices.json",
    fields: [
      string("source_id", 32),
      integer("display_level", 1, 200),
      string("enemy_variant_id", 240),
      json("weakness_consequence_json"),
      json("later_boss_consequence_json"),
    ],
  },
  {
    name: "SwarmDisasterTopologyConsequence",
    sheet: "TopologyConsequence",
    normalized: "topology-consequences.json",
    fields: [
      string("source_id", 32),
      string("trigger_kind", 100),
      string("scope", 100),
      json("ordered_operations_json"),
      string("aeon_dice_id", 240),
      integer("active_stage", 0, 100),
    ],
  },
  {
    name: "SwarmDisasterCountdownAndDisarray",
    sheet: "CountdownDisarray",
    normalized: "countdown-and-disarray.json",
    fields: [
      string("initial_value", 64),
      string("initial_value_quality", 100),
      string("movement_delta", 64),
      string("movement_delta_quality", 100),
      string("carry_policy", 160),
      string("transition_boundary", 160),
      string("transition_result", 160),
      string("warning_threshold", 64),
      string("same_boundary_order", 500),
      string("cap_policy", 160),
      json("disarray_tiers_json"),
      json("source_constant_bindings_json"),
    ],
  },
  {
    name: "SwarmDisasterBossDecayLevel",
    sheet: "BossDecayLevel",
    normalized: "boss-decay-levels.json",
    fields: [
      string("source_id", 32),
      string("tier", 100),
      string("threshold", 100),
      list("effect_refs", 32),
      json("effect_parameters_json"),
      string("stacking_policy", 200),
      string("application_boundary", 160),
      string("acquisition_en", 1000),
      string("acquisition_zh_cn", 1000),
      string("swarm_applicability", 160),
    ],
  },
  {
    name: "SwarmDisasterAudiencePath",
    sheet: "AudiencePath",
    normalized: "audience-paths.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0, 100),
      string("path_id", 240),
      ref("audience_die_id", "SwarmDisasterAudienceDie"),
      string("unlock_id", 32),
      json("unlock_policy_json"),
      json("initial_effects_json"),
      json("passive_effects_json"),
      list("description_parameters", 64),
      string("rogue_buff_type", 100),
      string("battle_event_buff_group", 100),
      string("battle_event_enhance_buff_group", 100),
      list("extra_effect_refs", 64),
    ],
  },
  {
    name: "SwarmDisasterAudienceDie",
    sheet: "AudienceDie",
    normalized: "audience-dice.json",
    fields: [
      string("source_id", 32),
      string("path_id", 240),
      ref("audience_path_id", "SwarmDisasterAudiencePath"),
      list("face_ids", 64, false),
      json("roll_policy_json"),
      string("unlock_id", 32),
      string("initial_effect_summary_en", 1000),
      string("initial_effect_summary_zh_cn", 1000),
      list("initial_effect_parameters", 64),
      list("passive_description_parameters", 64),
      list("extra_effect_refs", 64),
    ],
  },
  {
    name: "SwarmDisasterDiceFace",
    sheet: "DiceFace",
    normalized: "dice-faces.json",
    fields: [
      string("source_id", 32),
      ref("audience_die_id", "SwarmDisasterAudienceDie"),
      integer("sort", 0, 100),
      ref("rarity_id", "SwarmDisasterDiceRarity"),
      integer("activation_stage", 0, 100),
      ref("target_rule_id", "SwarmDisasterDiceTargetRule"),
      json("effect_program_json"),
    ],
  },
  {
    name: "SwarmDisasterDiceRarity",
    sheet: "DiceRarity",
    normalized: "dice-rarities.json",
    fields: [
      string("source_id", 32),
      integer("rank", 1, 3),
      string("name_color", 32),
    ],
  },
  {
    name: "SwarmDisasterDiceTargetRule",
    sheet: "DiceTargetRule",
    normalized: "dice-target-rules.json",
    fields: [
      string("source_id", 32),
      json("candidate_filter_json"),
      string("ordering", 160),
      json("cardinality_json"),
      json("no_legal_target_json"),
    ],
  },
  {
    name: "SwarmDisasterDiceRollControl",
    sheet: "DiceRollControl",
    normalized: "dice-roll-controls.json",
    fields: [
      string("operation", 100),
      json("resource_cost_json"),
      string("result_order", 160),
      json("fallback_policy_json"),
      json("abandon_reward_json"),
      string("unlock_id", 32, true),
    ],
  },
];

const progressionTables = [
  {
    name: "SwarmDisasterCommuningChoice",
    sheet: "CommuningChoice",
    normalized: "communing-choices.json",
    fields: [
      string("source_id", 32),
      string("story_stage", 100),
      string("aeon_id", 32),
      string("path_id", 240),
      json("eligibility_json"),
      json("point_deltas_json"),
      json("ordered_operations_json"),
      string("rogue_npc_id", 32),
    ],
  },
  {
    name: "SwarmDisasterPathstriderCabinet",
    sheet: "PathstriderCabinet",
    normalized: "pathstrider-cabinets.json",
    fields: [
      string("source_id", 32),
      integer("sort", 0, 100),
      string("cabinet_type", 100),
      list("prerequisite_ids", 64),
      list("unlocks_cabinet_ids", 64),
      ref("objective_id", "SwarmDisasterPathstriderObjective"),
      json("point_deltas_json"),
      list("description_parameters", 64),
    ],
  },
  {
    name: "SwarmDisasterCommuningDimension",
    sheet: "CommuningDimension",
    normalized: "communing-dimensions.json",
    fields: [
      string("source_id", 32),
      string("path_id", 240),
      integer("max_points", 0, 100),
      string("carry_policy", 160),
      string("clamp_policy", 160),
    ],
  },
  {
    name: "SwarmDisasterCommuningPointAdjustment",
    sheet: "CommuningPointAdjust",
    normalized: "communing-point-adjustments.json",
    fields: [
      string("source_kind", 100),
      string("source_id", 32),
      integer("ordinal", 0, 100),
      ref("dimension_id", "SwarmDisasterCommuningDimension"),
      string("delta", 64),
      string("clamp_policy", 160),
      string("operation_order", 160),
    ],
  },
  {
    name: "SwarmDisasterCommuningTrailNode",
    sheet: "CommuningTrailNode",
    normalized: "communing-trail-nodes.json",
    fields: [
      string("source_id", 32),
      ref("dimension_id", "SwarmDisasterCommuningDimension"),
      string("threshold", 64),
      list("prerequisite_ids", 64),
      list("effect_ids", 64, false),
      boolean("is_important"),
    ],
  },
  {
    name: "SwarmDisasterCommuningTrailPrerequisite",
    sheet: "CommuningTrailPrereq",
    normalized: "communing-trail-prerequisites.json",
    fields: [
      ref("node_id", "SwarmDisasterCommuningTrailNode"),
      integer("ordinal", 0, 100),
      ref("required_node_id", "SwarmDisasterCommuningTrailNode"),
      string("required_points", 64),
    ],
  },
  {
    name: "SwarmDisasterCommuningTrailEffect",
    sheet: "CommuningTrailEffect",
    normalized: "communing-trail-effects.json",
    fields: [
      ref("node_id", "SwarmDisasterCommuningTrailNode"),
      integer("ordinal", 0, 100),
      string("domain", 100),
      json("ordered_operations_json"),
      json("battle_projection_json"),
    ],
  },
  {
    name: "SwarmDisasterPathstriderObjective",
    sheet: "PathstriderObjective",
    normalized: "pathstrider-objectives.json",
    fields: [
      ref("cabinet_id", "SwarmDisasterPathstriderCabinet"),
      string("quest_id", 32),
      ref("finish_condition_id", "SwarmDisasterPathstriderFinishCondition"),
      json("progress_policy_json"),
      list("unlock_ids", 64),
    ],
  },
  {
    name: "SwarmDisasterPathstriderFinishCondition",
    sheet: "PathstriderFinish",
    normalized: "pathstrider-finish-conditions.json",
    fields: [
      string("source_id", 32),
      string("finish_type", 100),
      string("comparison", 100),
      json("parameters_json"),
      string("target_progress", 64),
      list("unlock_ids", 64),
      string("mode_hint", 100),
      boolean("enabled_for_swarm_compilation"),
    ],
  },
  {
    name: "SwarmDisasterPathstriderUnlock",
    sheet: "PathstriderUnlock",
    normalized: "pathstrider-unlocks.json",
    fields: [
      string("source_id", 32),
      ref("finish_condition_id", "SwarmDisasterPathstriderFinishCondition"),
      json("unlock_consequence_json"),
      string("evaluation_boundary", 160),
      string("mode_hint", 100),
    ],
  },
  {
    name: "SwarmDisasterMechanicalChapterLocator",
    sheet: "MechanicalChapter",
    normalized: "mechanical-chapter-locators.json",
    fields: [
      string("source_id", 32),
      integer("layer", 1, 3),
      ref("dimension_id", "SwarmDisasterCommuningDimension", true),
      string("point_threshold", 64),
      json("mechanical_unlock_json"),
    ],
  },
  {
    name: "SwarmDisasterPath",
    sheet: "Path",
    normalized: "paths.json",
    fields: [
      string("source_id", 32),
      string("shared_path_id", 240),
      boolean("selectable"),
      integer("sort", 0, 100),
      ref("audience_die_id", "SwarmDisasterAudienceDie"),
      string("mode_unlock_id", 32),
      boolean("propagation_unlock"),
      ref("resonance_id", "SwarmDisasterResonance"),
      list("formation_ids", 16),
      json("battle_event_groups_json"),
      list("extra_effect_ids", 64),
    ],
  },
  {
    name: "SwarmDisasterResonance",
    sheet: "Resonance",
    normalized: "resonances.json",
    fields: [
      string("source_id", 32),
      string("shared_resonance_id", 240),
      ref("path_id", "SwarmDisasterPath"),
      integer("threshold", 0, 100),
      string("energy_max", 64),
      string("initial_energy", 64),
      list("parameter_values", 64),
      list("mechanic_tags", 64),
      json("effect_program_json"),
    ],
  },
  {
    name: "SwarmDisasterPathBoost",
    sheet: "PathBoost",
    normalized: "path-boosts.json",
    fields: [
      string("source_id", 32),
      ref("path_id", "SwarmDisasterPath"),
      json("effect_program_json"),
      string("application_boundary", 160),
    ],
  },
  {
    name: "SwarmDisasterResonanceInterplay",
    sheet: "ResonanceInterplay",
    normalized: "resonance-interplays.json",
    fields: [
      string("source_id", 32),
      ref("main_path_id", "SwarmDisasterPath"),
      ref("sub_path_id", "SwarmDisasterPath"),
      json("thresholds_json"),
      json("effect_program_json"),
      string("application_boundary", 160),
      string("once_scope", 100),
    ],
  },
  {
    name: "SwarmDisasterTrailblazeBonus",
    sheet: "TrailblazeBonus",
    normalized: "bonuses.json",
    fields: [
      string("source_id", 32),
      string("bonus_event", 100),
      json("effect_program_json"),
      string("application_boundary", 160),
    ],
  },
];

const enumDefinitions = [
  ["SwarmDisasterOwnership", ["SwarmDisaster", "Shared"]],
  ["SwarmDisasterCoverageState", ["DataReady"]],
  [
    "SwarmDisasterEvidenceQuality",
    [
      "ExactStructured",
      "ExactPublicText",
      "Observed",
      "ApproximateFromReleasedText",
      "ProjectPolicy",
    ],
  ],
];

generate("core.toml", "SwarmDisaster.xlsx", coreTables, enumDefinitions);
generate(
  "progression.toml",
  "SwarmDisasterProgression.xlsx",
  progressionTables,
);
console.log(
  `Generated Swarm Disaster Sora schema (${coreTables.length} core and ` +
  `${progressionTables.length} progression tables).`,
);

function generate(filename, workbook, tables, enums = []) {
  const lines = [
    "# @generated by tools/swarm-disaster-reference/generate-sora-schema.mjs",
    "# Do not edit by hand.",
    "",
  ];
  for (const [name, values] of enums)
    lines.push(
      "[[enums]]",
      `name = ${quote(name)}`,
      `values = ${toml(values)}`,
      "",
    );
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
    for (const field of [...(table.baseFields ?? common), ...table.fields]) {
      lines.push(
        "[[tables.fields]]",
        `name = ${quote(field.name)}`,
        `type = ${quote(field.type)}`,
      );
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
  while (lines.at(-1) === "") lines.pop();
  fs.writeFileSync(path.join(schemaRoot, filename), `${lines.join("\n")}\n`);
}

function quote(value) {
  return JSON.stringify(value);
}

function toml(value) {
  if (Array.isArray(value)) return `[${value.map(toml).join(", ")}]`;
  if (typeof value === "string") return quote(value);
  if (typeof value === "number" || typeof value === "boolean")
    return String(value);
  return `{ ${Object.entries(value).map(([key, item]) =>
    `${key} = ${toml(item)}`).join(", ")} }`;
}
