#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildActivityDecisionMechanicExecution,
  writeActivityDecisionMechanicExecution,
} from "./activity-decision-mechanic-execution-support.mjs";

const config = {
  batch: "G22-P6-A03",
  output: "activity-decision-mechanic-a03-execution.json",
  partition_variant: "A03",
  programs: 64,
  operation_shapes: 286,
  operation_occurrences: 463,
  distinct_operation_types: 15,
  typed_boundaries: 2,
  first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-opt0412801-json",
  last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0419504-act0419504-json",
  test_file: "crates/starclock-mode-universe/src/divergent_universe/tests/activity_decision_mechanic_a03_runtime.rs",
  probes: [
    { id: "exact-source-shape-closure", test: "activity_decision_a03_closes_all_exact_source_shapes" },
    { id: "all-programs-typed-decision", test: "all_activity_decision_a03_mechanics_commit_once_without_rng" },
    { id: "atomic-rejection-fresh-reconstruction", test: "activity_decision_a03_rejections_are_atomic_and_reconstruct_fresh" },
  ],
};

export function buildActivityDecisionMechanicA03Execution() {
  return buildActivityDecisionMechanicExecution(config);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  writeActivityDecisionMechanicExecution(
    config,
    buildActivityDecisionMechanicA03Execution(),
    process.argv.includes("--check"),
  );
