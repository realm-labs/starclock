# Workbench Curio synthesis settlement and candidate policy

## Current executable boundary

`settle_curio_synthesis_accepted` on the production runtime factory settles an
owning service's `curio_synthesis::AcceptedCurioSynthesis`. This is a trusted
selection, not an untrusted player command or a random-candidate producer.
The active Workbench must contain released function 4: 106, 107 or 111.
Workbench membership is not proof of room/NPC placement.

The request canonically orders two distinct input state IDs and one different
output. Both inputs must be current active holdings of distinct owners with identical nonnegative
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

## Immutable candidate compiler

The factory's `curio_synthesis_offers` compiles
`curio_synthesis::offers::CurioSynthesisOffers` from current production inputs.
Its read-only `first_inputs`, `second_inputs` and `plan` APIs validate active
equal-quality holdings without consuming RNG or inventory. Duplicate-owner
inventories reject, and accepted settlement cannot consume two copies of the
same identity as its pair. First inputs require another compatible input and a
nonempty output pool. A plan stores a canonical
input pair and a bounded, ordered pool; it is not a player command, a cached
offer or permission to settle a stale view.

Candidate accuracy is
`VersionedProjectPolicyCurrentOwnersUniformHigherQualityTopTierSameQuality`.
The explicit policy is:

- Common/Rare inputs admit all strictly higher ordinary qualities; highest
  quality admits the same quality. Empty pools reject without a lower-quality
  fallback. The top-tier fallback is not observed original selector parity.
- Each ordinary current owner contributes its lowest fixed-width state-key
  eligible base state. Unbound states, negative states and evolution-only aliases
  are excluded. Catalog membership is not proof of original offer membership.
- Every current holding excludes its entire owner, including consumed inputs,
  other mode copies and destroyed/evolved holdings. Multiple copies do not
  increase an owner's probability or appear as separate outputs.
- `CurioSynthesisCandidatePlan::sample` uses the shared integer Reward stream,
  purpose 22564, to sample uniformly without replacement. It returns three
  candidates, or all remaining owners for pools of one/two. Each sampled owner
  consumes a draw plus any integer rejection draws; outputs return in canonical
  state-key order. Observation and empty/invalid plan construction do not draw.

Sampling is a trusted host primitive, not an exposed reroll action. Its owning
service must authenticate the offered command first, cache the sample and require
confirmation inside the shared generated-choice transaction; failure restores
RNG. Settlement must revalidate current holdings. The candidate compiler itself
does not provide a host; the bounded room capability below now supplies its
cache and confirmation graph, without original NPC or numeric-limit parity.
Its digest binds exact production component/decision inputs, accepted settlement,
candidate policy and draw purpose. Profiles additionally bind placement and caps.

Original selectors/weights, destroyed-input eligibility, top-tier behavior and
small-pool behavior remain unavailable. Deterministic owner-uniform sampling
avoids overweighting duplicate mode copies; lower-quality fallback, refreshing
held outputs, importing historical weights and treating three candidates as
three consumed inputs are rejected alternatives. Confidence in parity is absent.
Replace these fields when released Version 4.4 graphs or reproducible public
observations recover their actual eligibility, quality mapping and weights. Tests
below bind the current policy; they are not original-mechanic completion evidence.

## Bound source-room choice graph

`curio_synthesis::room::CurioSynthesisRoomCompiler` explicitly places an exact
function-4 Workbench at a caller-selected, validated current source-position
context. It does not infer NPC membership from Reforge/Respite names or attach
synthesis to every such card. Production Workbenches 106, 107 and 111 pass;
unsupported Workbenches, changed contexts, invalid caps and overlapping host
addresses reject. `compile_curio_domain_route` supplies the exact existing entry
lifecycle; internal menu movement never counts as another selected domain.

The five physical nodes are entry, service menu, first input, second input and
confirmation. The menu checks two distinct active equal-quality owners and an
unowned eligible output using a compact four-way predicate, rather than repeating
all state pairs in every option. Opening caches the actual legal first inputs;
selecting one caches compatible second inputs, without RNG or inventory cost.
Selecting the second draws and caches up to three outputs. Confirmation consumes
both inputs, acquires the cached output with all current mandatory rewards,
increments the Run receipt and logical-room completion count, clears the cache
and returns to the menu in one shared generated-choice transaction.

Pre-draw cancellation clears selections without consuming Curios, charging
Fragments/Heat or drawing. After sampling, only cached outputs may be confirmed:
no cancel, leave or reroll action is available. A caller selects a confirmed-use
limit of 1..=64 per logical room. A separate 64-opening budget bounds cancellation
loops; cancelled openings consume that budget, not the function receipt. These
two numeric bounds and mandatory confirmation are explicit policy, not original
service-limit/payment/cancellation parity. Exhausted or unavailable menus offer
only leave. Room counters reset only on a new logical room; Run receipts persist.

`CurioSynthesisSlots` declares six disjoint addresses at or above 70: first input,
second input, cached choices, completed uses, openings and accepted-command gate.
The first five survive physical menu movement. The gate resets on each physical
node, and every offered option requires a generated acceptance prefix, including
cancel/leave. The existing Workbench marker is physically scoped and is restored
by every menu. Input caches are bounded by the 235 current state rows; confirmed
output caches must contain one to three distinct eligible ordinary owners.

`CompiledCurioSynthesisRoom::bind` validates exact programs, nodes, internal/exit
edges, declarations, logical scope paths and absence of injected random policies.
`BoundCurioSynthesisRoom::offered/choose` authenticates the entire immutable
definition, accepts fresh structurally identical reconstruction, and rejects
foreign definitions, nodes, decisions, hidden IDs, stale hashes and raw choices
without consuming RNG. Rejection after sampling, inventory/reward changes,
receipt overflow or downstream graph initialization restores canonical bytes.
The same state-only settlement plan serves this capability and the trusted API;
there is no nested mutation call, second state machine or skipped reward path.

Room accuracy is
`VersionedProjectPolicyExplicitPlacementBoundedOpeningsCachedPairMandatoryConfirmation`.
The room digest binds exact candidate/settlement inputs, selected Workbench,
placement, namespace, host addresses and both budgets. The owning profile must
also bind its whole graph, deck, bootstrap inventory and other payloads. The
public mode API is executable. `bind_position_curio_synthesis_rooms` attaches
exact factory/context/fragment capabilities to an already immutable battle
profile; empty, overlapping, foreign and repeated attachments reject. The Flow
observes/chooses only whole-definition authenticated phases, and the existing
baseline runner dispatches every phase through that capability. Unbound runners
cannot bypass the generated command gate. No second controller or state machine
is introduced. Default topology placement and encoded profile reconstruction
are still pending.
No workbook/schema/bundle or terminal coverage changed. Placement and actual
numeric limits must be replaced/promoted with provenance-bearing Excel/Sora
authoring when released graphs or reproducible observations establish them.

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

Candidate tests use both families and all three input qualities; independently
reconstructed production definitions produce identical samples and canonical
state traces. Current fixed-seed vectors bind candidate IDs and draw counts.
Zero/one/two/three remaining owners exercise explicit depletion behavior.
Observations are byte/RNG-inert, copies do not overweight owners, destroyed owners
remain excluded, evolved inputs remain eligible, and sampled output selections
settle through the existing accepted boundary with its mandatory authored rewards.
A late receipt overflow after sampling restores bytes/RNG on repeated attempts;
stale-view plans cannot bypass current settlement eligibility. Trusted test
inventories and state-only draw fixtures are not a public offer/cache graph.
Duplicate-owner inventory regressions reject both observation and consuming
two copies, preserving canonical state on repeated attempts in both families.

Bound-room tests cover both families, all three capable Workbenches, all three
input qualities, fresh whole-definition state traces, cached mandatory outputs,
raw/stale/hidden/foreign command rejection, changed-program binding, finite
pre-draw cancellation, logical-room reset and persistent receipts. Receipt
failure preserves the cached offer, ownership and RNG. Leaving into a deliberately
failing next fixed Boss probe restores room/deck/cache/clock state. Other payloads
are explicitly probes, and initial inventories are trusted fixtures, not source
rewards or complete nested runs. Neither these tests nor terminal probe traversal
provide release or original NPC-selector credit.

Controller tests independently construct production fixtures for both families,
execute opening, input selection, sampling, atomic confirmation and leave, then
complete three actual nested Boss proxy battles. Canonical bytes agree at every
boundary with a freshly constructed profile. First selection is RNG-inert,
second selection draws the cached candidates without consuming inventory, and
confirmation exchanges two holdings for one and increments the function receipt.
Changed room caps/slot addresses, duplicate/repeated/foreign attachments and
raw/unbound/foreign/stale commands reject. Placement at the first source position,
initial holdings and proxy Boss selection are explicit test policies; all other
payloads remain probes. This is controller integration evidence, not a complete
original topology, complete Curio effects or encoded fresh-profile replay.

Original candidate selector/weights, cancellation/confirmation and numeric limits,
room/NPC admission, default full topology and encoded profile
replay remain incomplete. Weighted Curio recasting/equipment are distinct
pending functions, not covered by this boundary. No terminal source/mechanic
credit, complete Curio effects or Workbench-family completion is claimed.
