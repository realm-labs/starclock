#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A02",
  output: "activity-decision-mechanic-a02-execution.json",
  partition_variant: "A02",
  programs: 64,
  operation_shapes: 295,
  operation_occurrences: 526,
  distinct_operation_types: 17,
  typed_boundaries: 3,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-maze-mazerogue-roguetourn-roguetourn-goup-waitdialogue-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-act0412801-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a02_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a02_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a02_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA02Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA02Execution(),
    process.argv.includes("--check"),
  );
