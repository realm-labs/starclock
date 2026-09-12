#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A07",
  output: "activity-decision-mechanic-a07-execution.json",
  partition_variant: "A07",
  programs: 64,
  operation_shapes: 255,
  operation_occurrences: 669,
  distinct_operation_types: 10,
  typed_boundaries: 2,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0620701-opt0620701-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0623001-act0623001-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_a07_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a07_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a07_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a07_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA07Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA07Execution(),
    process.argv.includes("--check"),
  );
