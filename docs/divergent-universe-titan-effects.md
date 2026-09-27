# Divergent Universe Titan effect boundary

The current Titan code records type/Boon selections and talent unlocks and executes
one permanent-talent RunEntry effect. All other contribution effects remain
source descriptors rather than executable Activity or battle effects.
This is an incomplete part of the [complete runtime goal](goals/22-divergent-universe-runtime.md),
not a smaller acceptance target.

## Current production data and behavior

The real Sora reference bundle lowers 12 types, 84 Golden Blood's Boons,
36 talent levels, 36 offer definitions and 120 contribution descriptors.
Ten contribution scopes are Activity and 110 are Battle. Exact source child
joins, talent costs, directed prerequisites and descriptor ordering are retained.
Trusted accepted commands can record selections and pay/unlock talents through
the shared Activity transaction engine. Their tests verify those mutations and
reconstruction; they do not execute the described buffs, chance, stacking or
lifecycle consumers. Descriptors in a hash do not make those effects executable.

## Permanent-talent RunEntry fragments

`DivergentUniverseEntry::with_titan_talents` accepts an immutable caller-owned
ownership projection. Duplicate/oversized inputs reject; production compilation
checks every ID and the full directed prerequisite closure. The projection
initializes the existing talent inventory, never queries a live account and does
not charge already-paid permanent unlock costs. Every selected ID and the explicit
entry ordering policy bind the configuration identity. Empty input retains the
existing default entry behavior.

The pinned released `RogueTournTitanTalent` row 12302, locator 10, revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, has parameter `30` and description
hash `10402452538131023217`. The EN/CHS released text both describe an increase
to initial Cosmic Fragments. Production execution reads the owned Sora talent
operation/value, not the upstream table or normalized JSON. A one-visit automatic
Run-scoped entry node credits +30 through the existing economy gain operations
and emits ordinary gain-pipeline events under one command cause before any source-deck, Equation or
occurrence offer. Source text establishes the amount/RunEntry meaning, not this
headless physical-node ordering. That explicit deterministic composition policy
is replaceable and is not an observed-parity claim. It does not establish the
original total starting currency or a starting-loadout selector.

An in-run unlock cannot grant the initial bonus retroactively. Currency is spent
through the existing accepted economy commands, carried between layers and reset
at run settlement; permanent talent ownership remains in the captured inventory.
A fresh run obtains its own one-time entry gain. The canonical current replay
entry payload requires at most 36 strictly sorted unique stable talent IDs;
factory reconstruction checks source prerequisites again. Tests reconstruct the
entry state and events from fresh production inputs in both run families. This
is entry reconstruction, not a successful complete-run Titan replay: its
Day-conditioned prerequisites still have no battle consumer and remain rejected
by the fail-closed battle boundary below.

Public run/controller offer admission remains absent. Trusted selection APIs
must not be presented as publicly offered player commands. The frozen first
vertical slice's stage-ability resource marker is an architecture probe, not an
implementation of any Boon's gameplay effect or full Titan release acceptance.

## Fail-closed current battle assembly

`materialize_current_battle` and `resolve_current_battle` return
`UnimplementedTitanEffects` when the validated contribution snapshot contains
any selected Titan descriptor. Both Activity and Battle scopes reject: ignoring
the former would still omit a mechanically meaningful contribution. This check
runs after definition/state validation and before cache lookup, enemy assembly
or battle construction. Stale input still reports its existing typed error.
No Activity state, events, RNG counter or assembly-cache metric changes.

The rejection is a temporary correctness boundary, not the final gameplay
policy or terminal disposition. Real effect lowering must replace it alongside
production execution, rejection/control and fresh replay fixtures. The empty
Titan contribution case retains its existing battle behavior. The entry +30
consumer does not remove the guard: prerequisite talents and other selected
effects cannot silently enter a battle.

Native regressions load the production factory, select every one of the 84
Boons, and unlock all 36 talent levels in source prerequisite order. Both
Ordinary and Cyclical are tested. Each selected state is rejected repeatedly
through both assembly APIs, with canonical-state and cache equality checks.

## Current audit contract

[The current inventory](../content-manifests/divergent-universe-runtime-v1/titan-runtime-execution.json)
is generated by `generate-titan-runtime-execution.mjs`. Its consumer path is
retained, but its content is a current descriptor/partial-consumer inventory:

- all 132 exact-once source/definition assignments remain pending;
- the semantic family, research gap and two policy sources remain pending;
- neither test-target existence nor source-fragment matching emits a test-pass
  receipt, behavioral completion status or terminal coverage credit;
- the obsolete whole-goal completion/Grand Miracle prerequisite is absent; and
- altered source assignments, fake terminal promotion and missing assembly
  rejection fail validation.

The producer does not run Rust tests. Native Cargo execution is the behavior
authority. Other old release producers still require independent behavioral
audits; their stored reports cannot derive completion from this inventory.

```text
node tools/divergent-universe-runtime/generate-titan-runtime-execution.mjs --check
node tools/divergent-universe-runtime/verify-titan-runtime-execution.mjs
node --test tools/divergent-universe-runtime/titan-descriptor-inventory.test.mjs
cargo test -p starclock-mode-universe titan_effect_guard
cargo test -p starclock-mode-universe titan_entry
cargo test -p starclock-mode-universe titan_
cargo test -p starclock-mode-universe every_golden_blood_boon
```
