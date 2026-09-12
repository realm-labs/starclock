#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A11",
  output: "activity-decision-mechanic-a11-execution.json",
  partition_variant: "A11",
  programs: 62,
  operation_shapes: 124,
  operation_occurrences: 124,
  distinct_operation_types: 2,
  typed_boundaries: 1,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614401-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-330-roguenpc627001-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_a11_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a11_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a11_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a11_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA11Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA11Execution(),
    process.argv.includes("--check"),
  );
