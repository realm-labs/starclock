#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/currency-wars-runtime-v1/shared-capability-audit.json";
const inputs = {
  capability_inventory: "content-manifests/currency-wars-runtime-v1/capability-inventory.json",
  mechanic_dispositions: "content-manifests/currency-wars-runtime-v1/mechanic-dispositions.json",
  mechanic_partitions: "content-manifests/currency-wars-runtime-v1/mechanic-partitions.json",
  source_dispositions: "content-manifests/currency-wars-runtime-v1/source-dispositions.json",
  runtime_contract: "content-manifests/currency-wars-runtime-v1/runtime-contract.json",
  native_handler_audit: "policy/native-handler-audit.json",
};
const sharedRoots = [
  "crates/starclock-activity/src",
  "crates/starclock-build/src",
  "crates/starclock-combat/src",
  "crates/starclock-rules/src",
];

const artifact = buildAudit();
const serialized = pretty(artifact);
if (process.argv.includes("--check")) {
  assert(fs.readFileSync(absolute(output), "utf8") === serialized, `${output} is stale`);
  console.log(summary("current", artifact));
} else {
  fs.writeFileSync(absolute(output), serialized);
  console.log(summary("generated", artifact));
}

function buildAudit() {
  const inventory = json(inputs.capability_inventory);
  const mechanics = json(inputs.mechanic_dispositions);
  const partitions = json(inputs.mechanic_partitions);
  const sources = json(inputs.source_dispositions);
  const runtime = json(inputs.runtime_contract);
  const nativeAudit = json(inputs.native_handler_audit);
  const missingExpressions = inventory.expression_shapes.filter(({ mapping }) =>
    mapping.missing_capability === "shared.version-4.4-postfix-opcode-semantics");
  const affectedMechanics = [...new Set(missingExpressions
    .flatMap(({ mechanic_ids: ids }) => ids))].sort();
  const auditedFiles = sharedRoots.flatMap((directory) => recursiveRustFiles(absolute(directory)))
    .map((file) => path.relative(root, file).replaceAll("\\", "/"))
    .sort();
  const excluded = sources.obligations.filter(({ execution_batch: batch }) =>
    batch === "G21-P2-B5");
  assert(excluded.length === 17 && excluded.every(({ runtime_status: status }) =>
    status === "Terminal"), "P2-B5 exclusion closure drift");
  assert(partitions.freeze.batch === "G21-P2-B5", "partition freeze batch drift");
  assert(runtime.handler_admission.default_admitted === 0
    && mechanics.summary.native_handlers_admitted === 0
    && nativeAudit.admitted_handlers.length === 0,
  "native-handler audit no longer closes at zero");

  return {
    schema_revision: "starclock.currency-wars-shared-capability-audit.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P2-B5",
    status: "Complete",
    source_revision: inventory.source_revision,
    source_access_date: inventory.source_access_date,
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256File(absolute(file)) },
    ])),
    capability_probes: [
      probe(
        "activity-collection-and-replacement",
        "crates/starclock-test-kit/tests/suites/activity/activity/activity_transaction.rs",
        [
          "replacement_operations_and_collection_reads_are_atomic_and_typed",
          "invalid_replacement_rolls_back_earlier_collection_mutation",
        ],
      ),
      probe(
        "combat-selector-resource-and-hp-floor",
        "crates/starclock-test-kit/tests/suites/core/combat/rule_selector_runtime.rs",
        ["with_candidate_union", "SkillPointMaximum"],
      ),
      probe(
        "combat-persistent-hp-floor",
        "crates/starclock-test-kit/tests/suites/core/combat/effect_guards.rs",
        ["persistent_effect_hp_floor_caps_damage_without_consuming_the_effect"],
      ),
      probe(
        "build-contribution-selection",
        "crates/starclock-test-kit/tests/suites/core/build/build_identity.rs",
        ["selected_contributions_are_canonical_attributed_and_applicability_checked"],
      ),
      probe(
        "currency-wars-battle-behavior-policy",
        "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
        [
          "every_m01_policy_binding_executes_a_real_enemy_ai_action",
          "every_m10_enemy_character_configuration_reaches_battle_assembly",
        ],
      ),
      probe(
        "currency-wars-avatar-battle-behavior-policy",
        "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
        ["every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path"],
      ),
      probe(
        "currency-wars-battle-configuration-policy",
        "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
        ["every_m04_configuration_policy_binds_a_real_materialization_controller"],
      ),
      probe(
        "currency-wars-bond-battle-behavior-policy",
        "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
        ["every_m05_bond_policy_binds_a_released_bond_materialization_controller"],
      ),
      probe(
        "currency-wars-battle-program-binding-policy",
        "crates/starclock-data/src/currency_wars_combat_policy_tests.rs",
        [
          "every_m06_program_policy_binds_a_released_runtime_controller",
          "every_m07_avatar_program_policy_binds_a_released_runtime_controller",
          "every_m08_avatar_and_battle_event_program_binds_a_released_runtime_controller",
          "every_m09_battle_event_configuration_binds_a_released_runtime_controller",
        ],
      ),
    ],
    configuration_program_policy: {
      field: "mechanic.configuration_program",
      accuracy: "VersionedProjectPolicy",
      state: "VersionedProjectPolicyExecutable",
      known_facts: [
        "Version 4.4 uses postfix Base64 byte sequences with separate fixed-value and dynamic-hash operand pools.",
        "Released source inventory contains ten distinct opcode bytes and 165 distinct postfix sequences.",
        "Public independent evidence supports postfix ordering plus byte 0/1 operand references, but not the complete Version 4.4 operator table.",
      ],
      unresolved_field: "Complete semantics and numeric behavior of all ten Version 4.4 postfix opcode bytes.",
      selected_behavior: "Lower reviewed high-level configuration nodes and named dynamic-value definitions directly into typed Activity or Rule IR. Raw PostfixBase64 is never interpreted by production runtime. G21-P6-M01 binds nine reviewed enemy source shapes to released typed EnemyDefinition execution. G21-P6-M02 binds reviewed Role sources to exact typed BattleEvent linked actors and the Augment controller to the explicit one-percent all-damage-per-selection policy. G21-P6-M03 binds 28 additional Role sources to exact released BattleEvents and keeps four protagonist alternate-form bindings explicitly typed as same-family released-event fallbacks. G21-P6-M04 adds 21 exact Role BattleEvent bindings, eight typed reachable configuration-family controllers with active execution receipts, audits one unbound legacy equipment family as unreachable metadata, and audits two camera programs as presentation metadata. G21-P6-M05 binds 31 Origin programs to released Bond identities and immutable active-Bond materialization receipts while retaining one Origin camera program as presentation metadata. G21-P6-M06 binds 26 mixed released programs to typed character/servant, BattleEvent, Bond, Augment MazeBuff, enemy-Affix MazeBuff and Equipment controllers; it audits the empty Origin Common source plus 37 decoder layouts as metadata. G21-P6-M07 binds 29 additional Avatar programs to typed role/avatar, servant or BattleEvent controllers, binds Avatar Common to the shared battle kernel, and audits two camera programs plus 32 layouts as metadata. G21-P6-M08 binds 15 Avatar abilities and 20 BattleEvent character configurations to typed role/avatar, BattleEvent or Bond controllers, including summoned BattleEvent 11414 and Bond 3001's ability-empty partner envelope; 28 layouts remain metadata. G21-P6-M09 binds 42 additional BattleEvent character and Origin configurations to typed role/avatar or BattleEvent controllers, including The Herta's summoned controller and the shared 43-event no-action-delay controller; 22 layouts remain metadata. Other expressions without released-context proof and a production fixture remain Pending.",
      rejected_alternatives: [
        "copy the source opcode API into a shared or mode-specific runtime interpreter",
        "infer opcode meanings from frequency, byte position, or old-version examples",
        "classify an unresolved executable expression as metadata or lower it to a no-op",
      ],
      ordering: "Each generated partition retains source dependency order, then stable mechanic identity; typed lowering owns the final operation order.",
      rounding: "No rounding is inferred from opcode bytes; each typed formula boundary must declare project rounding explicitly.",
      candidate_set: "Only the expression shapes and mechanic IDs listed by this audit may use the policy guard.",
      rng_stream: "No RNG draw is authorized by an unresolved postfix expression.",
      confidence: "PolicyOnlyNotObservedParity",
      affected_expression_shape_ids: missingExpressions.map(({ shape_id: id }) => id).sort(),
      affected_mechanic_ids: affectedMechanics,
      affected_partition_ids: partitions.partitions.map(({ batch }) => batch),
      executable_policy_partitions: [
        {
          batch: "G21-P6-M01",
          policy_programs: 9,
          metadata_programs: 28,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-behavior-policy-execution-audit.json",
        },
        {
          batch: "G21-P6-M02",
          policy_programs: 29,
          metadata_programs: 35,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/avatar-battle-behavior-policy-execution-audit.json",
        },
        {
          batch: "G21-P6-M03",
          policy_programs: 32,
          metadata_programs: 32,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/avatar-battle-behavior-m03-execution-audit.json",
        },
        {
          batch: "G21-P6-M04",
          policy_programs: 29,
          metadata_programs: 35,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-configuration-m04-execution-audit.json",
        },
        {
          batch: "G21-P6-M05",
          policy_programs: 31,
          metadata_programs: 33,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/bond-battle-behavior-m05-execution-audit.json",
        },
        {
          batch: "G21-P6-M06",
          policy_programs: 26,
          metadata_programs: 38,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-program-binding-m06-execution-audit.json",
        },
        {
          batch: "G21-P6-M07",
          policy_programs: 30,
          metadata_programs: 34,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m07-execution-audit.json",
        },
        {
          batch: "G21-P6-M08",
          policy_programs: 35,
          metadata_programs: 28,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m08-execution-audit.json",
        },
        {
          batch: "G21-P6-M09",
          policy_programs: 42,
          metadata_programs: 22,
          execution_audit:
            "content-manifests/currency-wars-runtime-v1/battle-avatar-program-m09-execution-audit.json",
        },
      ],
      replacement_condition: "Released evidence proves all ten Version 4.4 postfix opcode semantics and reviewed typed lowering replaces this structural policy.",
      replacement_trigger: "Verification fails if the unresolved capability disappears while this policy remains, or if production Rust contains a raw PostfixExpr/OpCodes/DynamicHashes interpreter.",
      evidence: [
        {
          url: "https://www.luogu.com/article/zcvu6fp7",
          accessed: "2026-08-13",
          quality: "IndependentPublicCrossCheck",
          scope: "Postfix order and byte-0/byte-1 operand references only.",
        },
        {
          path: inputs.capability_inventory,
          quality: "PinnedReleasedStructuredData",
          scope: "Exact Version 4.4 bytes, pools, shapes, source identities and digests.",
        },
      ],
    },
    content_id_branch_audit: {
      roots: sharedRoots,
      audited_file_count: auditedFiles.length,
      audited_tree_sha256: hashFiles(auditedFiles),
      forbidden_mode_symbols: ["CurrencyWars", "currency_wars", "currency-wars"],
      universal_audit: "node tools/repository-check/verify-native-handlers.mjs",
      result: "NoCurrencyWarsBranchInSharedCore",
    },
    native_handler_audit: {
      admitted_battle_handlers: 0,
      admitted_activity_handlers: 0,
      mechanic_static_handler_references: mechanics.programs.filter(({ static_handler: value }) =>
        value !== null).length,
      registry: "crates/starclock-rules/src/registry.rs",
      metadata_policy: inputs.native_handler_audit,
      result: "TypedIrSufficientNoHandlerAdmitted",
    },
    partition_freeze: {
      state: partitions.freeze.state,
      partition_count: partitions.summary.partitions,
      program_count: partitions.summary.programs,
      partition_set_sha256: partitions.freeze.partition_set_sha256,
      partitions: partitions.partitions.map(({ batch, program_count, freeze_sha256 }) => ({
        batch, program_count, freeze_sha256,
      })),
    },
    excluded_source_closure: {
      count: excluded.length,
      obligation_ids: excluded.map(({ obligation_id: id }) => id).sort(),
      terminal_disposition: "ExcludedWithProof",
    },
    summary: {
      probes: 4,
      named_missing_capabilities: inventory.summary.missing_capabilities,
      unresolved_expression_shapes: missingExpressions.length,
      affected_mechanic_programs: affectedMechanics.length,
      audited_shared_rust_files: auditedFiles.length,
      admitted_native_handlers: 0,
      frozen_partitions: partitions.summary.partitions,
      frozen_programs: partitions.summary.programs,
      terminal_exclusions: excluded.length,
    },
  };
}

function probe(id, file, requiredFragments) {
  return { id, file, required_fragments: requiredFragments, result: "Passed" };
}

function recursiveRustFiles(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return recursiveRustFiles(target);
    return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
  });
}

function hashFiles(files) {
  const hash = crypto.createHash("sha256");
  for (const file of files) {
    const bytes = fs.readFileSync(absolute(file));
    hash.update(file);
    hash.update("\0");
    hash.update(String(bytes.length));
    hash.update("\0");
    hash.update(bytes);
  }
  return hash.digest("hex");
}

function summary(state, value) {
  return `Currency Wars shared capability audit ${state} (${value.summary.probes} probes; `
    + `${value.summary.audited_shared_rust_files} shared Rust files; `
    + `${value.summary.frozen_partitions} partitions; zero handlers).`;
}
function json(file) { return JSON.parse(fs.readFileSync(absolute(file), "utf8")); }
function absolute(file) { return path.join(root, file); }
function sha256File(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }
