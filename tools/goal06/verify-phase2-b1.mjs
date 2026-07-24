#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const has = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`${label}: missing ${JSON.stringify(needle)}`);
};

const key = read("crates/starclock-mode-universe/src/battle_assembly.rs");
const composition = read(
  "crates/starclock-mode-universe/src/battle_materialization/catalog_composition.rs",
);
const materialization = read(
  "crates/starclock-mode-universe/src/battle_materialization.rs",
);
const production = read("crates/starclock-mode-universe/src/production_runtime.rs");
const tests = read("crates/starclock-mode-universe/tests/battle_materialization.rs");
const status = read("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");

for (const [text, needle, label] of [
  [key, "pub struct BattleAssemblyKey", "canonical key"],
  [key, "catalog_composition: [u8; 32]", "catalog key field"],
  [key, "participant_lock: ParticipantLockDigest", "roster/build key field"],
  [key, "encounter: [u8; 32]", "encounter key field"],
  [key, "contributions: [u8; 32]", "contribution key field"],
  [key, "carry: [u8; 32]", "carry key field"],
  [key, "technique: Option<[u8; 32]>", "technique key field"],
  [key, "DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY: usize = 8", "bounded default"],
  [key, "VecDeque<BattleAssemblyKey>", "deterministic FIFO"],
  [key, "entry.assembly_key() == key", "cache entry revalidation"],
  [composition, "pub struct UniverseBattleCatalogComposition", "immutable composition"],
  [composition, "for member in members(universe)", "member definitions composed once"],
  [composition, "for (index, binding) in universe.difficulty_enemy_bindings()", "difficulty definitions composed once"],
  [materialization, "compile_from_composition", "selected assembly boundary"],
  [production, "UniverseBattleCatalogComposition::compile(&catalog)", "production composition"],
  [production, ".compile_from_composition(", "production selected assembly"],
  [tests, "immutable_catalog_composition_and_bounded_exact_key_cache_are_separate", "cache contract test"],
  [status, "| `G06-P2-B1` | `Complete` |", "completed ledger row"],
]) has(text, needle, label);

if (key.includes("serde"))
  throw new Error("assembly key/cache must not use normal serialization");
if (key.includes("starclock_activity::ActivityState"))
  throw new Error("cache must not own authoritative Activity state");

console.log("Goal 06 P2-B1 verified (catalog composition, exact key, bounded FIFO cache).");
