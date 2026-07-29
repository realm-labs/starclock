#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const schemaRoot = path.join(root, "config", "unknowable-domain", "schema");

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
  { name: "ownership", type: "enum<UnknowableDomainOwnership>" },
  { name: "coverage_state", type: "enum<UnknowableDomainCoverageState>" },
  {
    name: "evidence_quality",
    type: "enum<UnknowableDomainEvidenceQuality>",
  },
  list("source_refs", 512),
  list("tags", 64),
];

const coreTables = [
  {
    name: "UnknowableDomainProfile",
    sheet: "Profile",
    normalized: "profiles.json",
    fields: [
      string("entry_kind", 80, true),
      string("source_id", 100, true),
      string("sub_mode", 80),
      string("unlock_id", 100, true),
      json("initial_resources_json"),
      string("initial_resources_resolution", 100),
    ],
  },
  {
    name: "UnknowableDomainAlignment",
    sheet: "Alignment",
    normalized: "alignments.json",
    fields: [
      string("source_id", 32),
      string("display_id", 32),
      string("display_text_en", 500),
      string("display_text_zh_cn", 500),
      string("unlock_id", 100, true),
      string("eligibility", 100),
      string("selection_cardinality", 100),
      list("default_area_ids", 32, false),
      list("scepter_candidate_ids", 32, false),
      list("component_candidate_ids", 256),
      string("component_pool_resolution", 100),
      list("pool_ids", 256),
      list("rule_contribution_ids", 256),
      string("contribution_resolution", 160),
    ],
  },
  {
    name: "UnknowableDomainArea",
    sheet: "Area",
    normalized: "areas.json",
    fields: [
      string("source_id", 32),
      string("area_group", 80),
      ref("default_alignment_id", "UnknowableDomainAlignment"),
      string("unlock_id", 100, true),
      list("difficulty_ids", 128),
      list("source_difficulty_ids", 128, false),
      string("difficulty_resolution", 100),
      list("layer_ids", 32, false),
      string("extra_layer_key", 240, true),
      list("displayed_boss_ids", 32, false),
      json("customization_inputs_json"),
    ],
  },
  {
    name: "UnknowableDomainDifficultyComposition",
    sheet: "DifficultyComposition",
    normalized: "difficulty-compositions.json",
    fields: [
      string("source_id", 100),
      integer("level", 0, 100, true),
      string("unlock_id", 100, true),
      json("parameters_json", true),
      json("drop_bindings_json", true),
    ],
  },
  {
    name: "UnknowableDomainLayer",
    sheet: "Layer",
    normalized: "layers.json",
    fields: [
      string("source_id", 32),
      integer("layer_number", 0, 100),
      list("room_position_ids", 128, false),
      string("carry_policy", 160),
    ],
  },
  {
    name: "UnknowableDomainLayerRoom",
    sheet: "LayerRoom",
    normalized: "layer-rooms.json",
    fields: [
      string("source_id", 32),
      ref("layer_id", "UnknowableDomainLayer"),
      integer("ordinal", 0, 100),
      list("room_pool_ids", 256),
      string("room_pool_resolution", 100),
    ],
  },
  {
    name: "UnknowableDomainRoom",
    sheet: "Room",
    normalized: "rooms.json",
    fields: [
      string("source_id", 32),
      string("room_type", 80),
      list("npc_graph_ids", 128),
      list("encounter_pool_ids", 128),
      string("membership_resolution", 100),
    ],
  },
  {
    name: "UnknowableDomainStageFlow",
    sheet: "StageFlow",
    normalized: "stage-flow.json",
    fields: [
      ref("area_id", "UnknowableDomainArea", true),
      string("from_state", 240),
      string("condition", 500),
      string("to_state", 240),
      json("ordered_operations_json"),
      string("policy_id", 160),
    ],
  },
  {
    name: "UnknowableDomainFinishCondition",
    sheet: "FinishCondition",
    normalized: "finish-conditions.json",
    fields: [
      string("source_id", 32),
      string("finish_type", 160),
      string("parameter_type", 100),
      string("string_parameter", 500, true),
      list("integer_parameters", 64),
      list("item_parameters", 64),
      string("comparison", 100),
      string("progress", 64),
    ],
  },
];

const systemTables = [
  {
    name: "UnknowableDomainScepter",
    sheet: "Scepter",
    normalized: "scepters.json",
    fields: [
      string("source_id", 32),
      string("style", 80),
      ref("alignment_id", "UnknowableDomainAlignment"),
      string("function", 80),
      string("source_function", 100),
      string("unlock_id", 100, true),
      list("level_ids", 8, false),
      list("slot_layout_ids", 8, false),
      string("trigger_text_en", 1000),
      string("trigger_text_zh_cn", 1000),
    ],
  },
  {
    name: "UnknowableDomainScepterLevel",
    sheet: "ScepterLevel",
    normalized: "scepter-levels.json",
    fields: [
      string("source_id", 32),
      ref("scepter_id", "UnknowableDomainScepter"),
      string("level", 32),
      string("power", 64),
      string("staff_maze_buff_id", 32),
      list("locked_component_ids", 16, false),
      ref("slot_layout_id", "UnknowableDomainSlotLayout"),
      json("slot_counts_json"),
      list("effect_ranges", 16, false),
      list("effect_types", 16, false),
    ],
  },
  {
    name: "UnknowableDomainScepterActivationRule",
    sheet: "ScepterActivationRule",
    normalized: "scepter-activation-rules.json",
    fields: [
      string("source_id", 100),
      ref("scepter_id", "UnknowableDomainScepter"),
      ref("scepter_level_id", "UnknowableDomainScepterLevel"),
      string("trigger", 160),
      string("trigger_text_en", 1000),
      string("trigger_text_zh_cn", 1000),
      json("charge_or_speed_json"),
      string("target_rule", 160),
      string("target_selection_order", 160),
      string("simultaneous_trigger_order", 160),
      json("ordered_operations_json"),
      string("binding_type", 160),
      string("binding_key", 240),
      string("ability_locator", 500),
    ],
  },
  {
    name: "UnknowableDomainScepterStateTransition",
    sheet: "ScepterStateTransition",
    normalized: "scepter-state-transitions.json",
    fields: [
      string("source_id", 100),
      ref("scepter_id", "UnknowableDomainScepter"),
      ref("scepter_level_id", "UnknowableDomainScepterLevel"),
      ref("activation_rule_id", "UnknowableDomainScepterActivationRule"),
      string("teardown", 160),
      integer("ordinal", 0, 100),
      string("from_state", 160),
      string("input", 240),
      string("to_state", 160),
    ],
  },
  {
    name: "UnknowableDomainComponent",
    sheet: "Component",
    normalized: "components.json",
    fields: [string("source_id", 32)],
  },
  {
    name: "UnknowableDomainComponentLevel",
    sheet: "ComponentLevel",
    normalized: "component-levels.json",
    fields: [
      string("source_id", 32),
      string("effect_source_id", 32),
      ref("component_id", "UnknowableDomainComponent"),
      string("level", 32),
      string("category", 80),
      string("component_type", 80),
      string("shape", 80),
      string("shape_basis", 160),
      list("range_ids", 16, false),
      list("effect_types", 32, false),
      json("effect_program_json"),
      string("description_en", 2400),
      string("description_zh_cn", 2400),
      string("simple_description_en", 2400),
      string("simple_description_zh_cn", 2400),
      list("style_ids", 16),
      string("style_resolution", 160),
    ],
  },
  {
    name: "UnknowableDomainComponentSlotCompatibility",
    sheet: "ComponentSlotCompatibility",
    normalized: "component-slot-compatibility.json",
    fields: [
      string("source_id", 100),
      ref("component_id", "UnknowableDomainComponent"),
      string("component_level", 32),
      ref("component_level_id", "UnknowableDomainComponentLevel"),
      string("slot_type", 80),
      string("range", 80),
      integer("ordinal", 0, 100),
      string("eligibility", 160),
      string("slot_layout_resolution", 160),
    ],
  },
  {
    name: "UnknowableDomainSlotLayout",
    sheet: "SlotLayout",
    normalized: "slot-layouts.json",
    fields: [
      string("source_id", 32),
      string("active_count", 32),
      string("attach_count", 32),
      string("passive_count", 32),
      string("total_count", 32),
      list("slot_types", 16, false),
    ],
  },
  {
    name: "UnknowableDomainLoadout",
    sheet: "Loadout",
    normalized: "loadouts.json",
    fields: [
      string("source_id", 100),
      ref("scepter_id", "UnknowableDomainScepter"),
      ref("scepter_level_id", "UnknowableDomainScepterLevel"),
      ref("slot_layout_id", "UnknowableDomainSlotLayout"),
      list("slot_ids", 16, false),
      json("slots_json"),
      list("locked_component_ids", 16, false),
      string("locked_slot_resolution", 160),
      json("authored_occupancy_json"),
    ],
  },
  {
    name: "UnknowableDomainLoadoutTransitionRule",
    sheet: "LoadoutTransitionRule",
    normalized: "loadout-transition-rules.json",
    fields: [
      string("source_id", 160),
      string("operation", 80),
      json("eligibility_json"),
      json("replacement_order_json"),
      string("rejected_mutation", 160),
      string("no_legal_candidate", 160),
      string("policy_id", 160),
    ],
  },
  {
    name: "UnknowableDomainDecisionComponent",
    sheet: "DecisionComponent",
    normalized: "decision-components.json",
    fields: [
      string("source_id", 32),
      ref("component_id", "UnknowableDomainComponent"),
      string("eligibility", 160),
      string("scope", 160),
      string("repetition", 160),
      list("choice_program_ids", 16, false),
      ref("effect_program_id", "UnknowableDomainComponentLevel"),
    ],
  },
  {
    name: "UnknowableDomainComponentChoiceProgram",
    sheet: "ComponentChoiceProgram",
    normalized: "component-choice-programs.json",
    fields: [
      string("source_id", 100),
      ref("decision_component_id", "UnknowableDomainDecisionComponent"),
      list("candidate_set", 256),
      string("candidate_set_basis", 160),
      string("offer_reachability", 160),
      string("ordering", 160),
      string("repetition", 160),
      json("outcomes_json"),
      string("fallback", 160),
    ],
  },
];

const enums = [
  ["UnknowableDomainOwnership", ["UnknowableDomain", "Shared"]],
  ["UnknowableDomainCoverageState", ["DataReady"]],
  [
    "UnknowableDomainEvidenceQuality",
    [
      "ExactStructured",
      "ExactPublicText",
      "Observed",
      "ApproximateFromReleasedText",
      "ProjectPolicy",
    ],
  ],
];

generate("core.toml", "UnknowableDomain.xlsx", coreTables, enums);
generate(
  "systems.toml",
  "UnknowableDomain.xlsx",
  systemTables,
);
console.log(
  `Generated Unknowable Domain Sora schema (${coreTables.length} core and ` +
  `${systemTables.length} system tables).`,
);

function generate(filename, workbook, tables, enumDefinitions = []) {
  const lines = [
    "# @generated by tools/unknowable-domain-reference/generate-sora-schema.mjs",
    "# Do not edit by hand.",
    "",
  ];
  for (const [name, values] of enumDefinitions)
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
    for (const field of [...common, ...table.fields]) {
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
      'name = "by_stable_key"',
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
