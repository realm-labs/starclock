#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A09",
  output: "activity-decision-mechanic-a09-execution.json",
  partition_variant: "A09",
  programs: 64,
  operation_shapes: 191,
  operation_occurrences: 353,
  distinct_operation_types: 11,
  typed_boundaries: 2,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0625902-opt0625902-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413301-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_a09_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a09_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a09_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a09_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA09Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA09Execution(),
    process.argv.includes("--check"),
  );
