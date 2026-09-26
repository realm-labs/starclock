# Accepted Workbench Curio synthesis

## Current executable boundary

`settle_curio_synthesis_accepted` on the production runtime factory settles an
owning service's `curio_synthesis::AcceptedCurioSynthesis`. This is a trusted
selection, not an untrusted player command or a random-candidate producer.
The active Workbench must contain released function 4: 106, 107 or 111.
Workbench membership is not proof of room/NPC placement.

The request canonically orders two distinct input state IDs and one different
output. Both inputs must be current active holdings with identical nonnegative
quality. The output must have an ordinary current owner, not an evolution-only
alias, and satisfy the function's same-or-higher-quality constraint. Any current
owner remains excluded, including the two consumed owners and their other mode
copies. Current aliases may be consumed as active evolved inputs, without
restoring their predecessor or leaking an alias into the output pool.

One shared generated Activity transaction removes both inputs' ownership,
charges and activations; acquires the selected output with its current mandatory
acquisition/Equation-expansion rewards; and increments the independent Run-wide
function-4 receipt. The cost is input Curios: no invented Fragment fee, Heat
debit or linear-price escalation is added. Rejections preserve authoritative
bytes, event/command sequences and RNG, including failures after reward draws
and earlier inventory operations. This receipt is an execution count, not proof
of a per-Workbench attempt cap.

The existing single-sacrifice acquisition planner now delegates to the same
mode-private multi-consumption plan. It does not introduce another inventory,
state machine, command processor, RNG or native handler. All mandatory authored
immediate rewards still execute. Missing effect lowering stays pending; generic
ownership/contribution changes do not turn an unimplemented Curio effect inert.

## Exact constraints and explicit policy

Accuracy is
`VersionedProjectPolicyAcceptedActiveCurioPairSameOrHigherQualityOriginalViewGrants`.

Released function text establishes equal-quality inputs and same/higher-quality
output. A pinned shared tutorial specifies two inputs, a choice among three
random candidates, the three-star maximum, exclusion of negative Curios and
the existence of a per-Workbench limit. It does not recover the current NPC
graph, precise output pools/weights or numeric limit.

The accepted boundary validates the broader function constraint, not the
tutorial's narrower higher-quality offer or any specific original candidate
pool. No output-selection parity is claimed. Active-only input admission,
excluding all pre-consumption owners, preservation of original-view acquisition
reward snapshots and the Run-wide receipt scope are explicit current policy.
Original-view snapshots apply to Blessing selection and Equation/expansion
planning. Fragment rewards retain the existing acquisition pipeline: read the
current balance before each grant and evaluate gain modifiers after the owning
inventory operations. They are not forced to use a pre-consumption balance or
the removed Curios' modifiers. See the [acquisition timing contract](divergent-universe-curio-acquisition.md).
Destroyed-input eligibility and original acquisition/consumption timing remain
unproven. Consumed expansion-reward Curios are never resurrected by a later
inventory plan.

Rejecting every authenticated pair is not an executable synthesis settlement.
Inferring membership from names/adjacent IDs, treating three candidates as
three consumed inputs, adding an unsupported cash fee, importing older modes'
weights or omitting mandatory rewards are rejected alternatives. Confidence in
original selection, destroyed-input eligibility and timing parity is absent.
Replace the policy when released graphs or reproducible observations establish
those fields, output eligibility/weights and attempt caps. A future offered
profile must bind `curio_synthesis::configuration_digest()` alongside exact
source/catalog/Workbench and its own selection-policy identity.

No production workbook, schema or bundle changed: this is the accepted owning
service boundary over existing Sora definitions. Static selector/limit promotion
still requires provenance-bearing Excel/openpyxl/Sora authoring. Reference
`UnspecifiedInputCount` and absent candidates are not silently relabeled exact.

## Released evidence

Pinned released Version 4.4 source:
`Dimbreath/turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`,
inspected 2026-09-26, with current bytes verified:

| File | SHA-256 |
| --- | --- |
| `ExcelOutput/RogueTournWorkbenchFunc.json` | `b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078` |
| `ExcelOutput/RogueTournWorkbench.json` | `26053803e691fe1fa7be57bf94d1d766ff0cc3e6cebda579fa013b129775d736` |
| `TextMap/TextMapEN.json` | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

Function locator: `FuncID=4;FuncType=MiracleCompose;FuncDesc.Hash=3527513478799937242`.
Independent bilingual summary: 消耗同品质奇物，获得同品质或更高品质奇物 /
consume equal-quality Curios to obtain a same/higher-quality Curio.
EN tutorial locators `16605371574484482783`, `4031257572520883730` and
`2983593752145917644` describe two inputs, three selectable random candidates
up to three stars, negative exclusion and a per-Workbench limit. These shared
tutorials are not a recovered Arcadian service graph.
The current [domain summary](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Domains)
and [guide](https://game8.co/games/Honkai-Star-Rail/archives/457407), accessed
2026-09-26, cross-check synthesis's domain role, not immutable Version 4.4
weights/caps. Historical Protean Hero discussions are not admitted as current
membership or numeric evidence; no beta/leaked source is used.

## Verification and remaining scope

Production fixtures execute all three capable Workbenches, both run families
and each allowed quality-pair shape with independently reconstructed canonical
state traces. Every current nonnegative, source-bound ordinary mode-copy input
has an accepted-consumption fixture; an evolved active input is separately
consumed without predecessor resurrection, and evolution-only output rejects.
Inventories are trusted test setup, not rewards granted by the Workbench.

Tests cover duplicate, unknown, unowned, mixed/negative/lower-quality and
destroyed states, stale hashes, unsupported/foreign/inactive Workbenches,
repeated selection and counter cleanup. Actual wax output draws and grants a
Blessing; receipt overflow after inventory/reward operations rejects twice
without changing bytes/RNG. Clearing the trusted fault permits one settlement.
Existing single-sacrifice/evolution/acquisition regressions remain intact.

Original candidate sampling/confirmation, numeric attempt limits, room/NPC
admission, public synthesis menus, default full topology and encoded profile
replay remain incomplete. Weighted Curio recasting/equipment are distinct
pending functions, not covered by this boundary. No terminal source/mechanic
credit, complete Curio effects or Workbench-family completion is claimed.
