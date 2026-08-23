# Currency Wars Runtime Contract

This document summarizes the target boundary frozen by `G21-P0-B4`. The
machine-checked normative form is
[`runtime-contract.json`](../content-manifests/currency-wars-runtime-v1/runtime-contract.json).
It describes the complete Goal 21 runtime boundary; it does not claim that the
current partial skeleton already implements it.

## Public boundary

Production assembly starts in `starclock-data`, which privately loads the Sora
bundle and lowers immutable definitions. The intended mode facade consists of a
runtime factory, entry request, runtime instance, bounded observation, offered
command and typed error. Generated Sora rows, raw programs, source paths,
private catalogs, slot allocation and handler payloads do not cross that
facade.

Every adapter mutation must submit one currently offered
`GraphActivityCommand`. Shop, roster, route, reward, preparation and other mode
choices are option families, not side-channel mutator methods. The five generic
commands remain `ChooseOption`, `StartBattle`, `SubmitBattleResult`,
`SubmitExternalOutcome` and `Abandon`; each binds the current state hash and
decision identity.

## State and lifetime

Currency Wars maps Run, Plane, Node visit and battle/external attempt to the
generic `Activity`, `Section`, `Node` and `Attempt` scopes. Thirty-four named
slot families freeze the intended ownership, typed value domain, visibility,
carry and reset contract. Their numeric IDs remain private and are allocated by
the compiled definition.

The Activity scope owns entry, route, economy, roster, deployment, equipment,
Bonds, investments, formulas and permanent progression. Plane-local state uses
Section; offers and choices use Node; battle projection scratch and external
outcomes use Attempt. Battle and shorter effect lifetimes remain combat-owned.

## Battle handoff

Battle assembly consumes one immutable contribution snapshot containing route,
difficulty, Gambit, location, participant/deployment, resolved builds,
equipment, stars, Empowerments, Bonds, investments, encounter, affix, scaling
and boss inputs. Assembly cannot query live Activity state after snapshot
creation, and combat cannot mutate Activity.

The handoff identity binds Activity definition/configuration, participant lock,
scope, sequence, combat input, assembly and purpose-derived seed. Settlement
accepts only the exact pending handoff and requires terminal outcome/hash/event
fields, all participant carry states, Squad HP loss and remaining action value.
Projection, rewards, carry and graph traversal commit in one Activity
transaction.

## Components, RNG and handlers

The canonical component set has nine consumed identities: combat catalog,
build catalog, Activity core, mode profile, mode content, Activity handler
registry, combat rule registry, encounter overlay and controller. Replay
verification reconstructs the exact set and reports the first mismatch.

Currency Wars reuses the eight generic Activity RNG streams. Every draw has a
named label, purpose and stable ordered candidates; empty pools consume no draw
and rejected commands restore every counter.

The initial native-handler admission count is zero. A later static handler is
legal only when the capability inventory proves typed shared IR cannot express
the rule and the owning batch supplies bounded typed I/O, explicit timing and
scope, determinism/rejection tests and production execution evidence. Content
ID branches, runtime registration and no-op handlers are forbidden.

## Failure semantics

Catalog/lowering failure occurs before a run exists. Invalid or stale commands,
assembly requests and battle results leave canonical bytes, state hash, events
and RNG unchanged. Accepted execution either commits ordered events atomically
or enters the deterministic terminal fault path. Infrastructure failure before
a nested battle starts restores the pre-start Activity identity; combat faults
cross only through a sealed `BattleResult`. Replay verification operates on a
fresh reconstruction and never mutates a live session.
