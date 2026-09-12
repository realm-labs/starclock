# Divergent Universe base battle fragments

The production `BattleFragments` sheet declares one fixed base reward for each
publicly selectable proxy domain: Combat 40, Elite 100, Aberration 40. These
amounts are **project policy, not observed game values**. The binary Sora bundle,
schema and existing mode configuration/replay identity bind the exact table.
No Boss amount or original enemy/room eligibility is inferred.

## Evidence and replaceable policy

Decision source 41 records `ExcelOutput/RogueTournConstCommon.json` at released
4.4 revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
inspected 2026-09-12. Its SHA-256 is
`3ad1c238e4adbf89bb614cecba90327cdd1e4fa25386821b13854fab0c3b7901`.
The 34 inspected constants do not declare a base battle-fragment amount. This
does not prove that no other source defines one. Source 24's elite stage flags
and account-reward display IDs are not run-currency amount evidence. Source 39
supplies only the independently authenticated domain labels. Source 14 retains
the global fragment-gain text, not these base amounts.

`VersionedProjectPolicyFixedVerifiedDomainCredit` intentionally supplies a
nonzero repeatable battle income and a higher Elite credit while the original
drop program is unresolved. Amount, fixed versus random distribution,
per-battle versus per-enemy timing, difficulty scaling, occurrence reward
stacking and ordering are independently replaceable low-confidence fields.
Alternatives include enemy-specific credits and integer range sampling.
Replace these fields when released current-profile programs or reproducible
observations establish them. This policy does not terminalize any original
program or source obligation.

## Transaction boundary

Only a verified win credits the authenticated mode-owned domain. Source stage
flags, generic Encounter decisions and adapter-supplied labels do not select a
reward. Loss and fault grant nothing. The fixed reward consumes no RNG.

The ordered settlement is verified carry, base fragments (24060), Green Miracle
victory fragments (23700), Sage victory Blessings (24050), Curio battle lifetimes
(24110), then ordinary Blessing
candidates (23702). Each stage observes the preceding committed stage inside
one shared transaction. The base credit and each Curio credit separately use the
[global fragment-gain pipeline](divergent-universe-fragment-gains.md); their
original bases are not merged, and no recursive multiplier is introduced.
Active Blessing suppression blocks Blessings, not base fragments. Destroyed
gain modifiers do not contribute; repair and replacement affect the next credit.

An invalid result, overflow in any later grant, or failed destination advance
restores the pending result, carry, all credits, events and RNG. Duplicate results
cannot grant twice. Final run teardown can clear the credited wallet in the same
transaction when no final Blessing offer remains; its final balance is not an
intermediate reward measurement.

## Verification and remaining scope

Tests drive public Combat/Elite/Aberration selections through three actual
battles and fresh replay in both run families. Controlled modifier fixtures
cover all four gain components, additive stacking, destruction, repair,
replacement, Blessing suppression, exact reward draws, stage event order and
duplicate rejection. Counterfactual loss/fault projections exercise zero grants;
the Green overflow fixture lets the base stage succeed before a later Curio
grant fails, then verifies whole-result rollback and a successful retry.

Original amounts, full domain-specific base drops, multi-enemy reward programs,
room topology and complete content reachability remain pending under
[Goal 22](goals/22-divergent-universe-runtime.md).
