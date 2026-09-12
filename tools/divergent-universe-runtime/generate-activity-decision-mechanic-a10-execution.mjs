#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A10",
  output: "activity-decision-mechanic-a10-execution.json",
  partition_variant: "A10",
  programs: 64,
  operation_shapes: 128,
  operation_occurrences: 128,
  distinct_operation_types: 2,
  typed_boundaries: 1,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413401-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614301-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_a10_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a10_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a10_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a10_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA10Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA10Execution(),
    process.argv.includes("--check"),
  );
