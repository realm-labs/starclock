#!/usr/bin/env node

import process from "node:process";
import { execFileSync } from "node:child_process";

const options = process.argv.slice(2);
if (options.length !== 1 || options[0] !== "--hardening")
  throw new Error("usage: run-native-ci.mjs --hardening");

for (const [command, args] of [
  ["cargo", ["test", "-p", "starclock-mode-universe", "swarm_disaster_entry::hardening_tests", "--all-features", "--", "--test-threads", "1"]],
  ["cargo", ["test", "-p", "starclock-mode-universe", "swarm_disaster_entry::replay_tests::component_replay_reexecutes_real_battles_and_reports_every_first_boundary", "--all-features", "--", "--test-threads", "1"]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "exhaustive_suite", "swarm_disaster_hardening", "--all-features", "--", "--test-threads", "1"]],
  ["cargo", ["test", "-p", "starclock-test-kit", "--test", "exhaustive_suite", "replay", "--all-features", "--", "--test-threads", "1"]],
  ["node", ["tools/goal20/verify-determinism-hardening.mjs", "."]],
]) execFileSync(command, args, { stdio: "inherit" });

console.log("Goal 20 native determinism hardening gate passed.");
