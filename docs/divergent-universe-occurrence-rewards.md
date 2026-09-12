# Divergent Universe occurrence rewards

## Current executable boundary

`DivergentUniverseCurioRuntime::acquire_accepted_states` and
`DivergentUniverseBlessingRuntime::acquire_accepted_identities` accept a nonempty,
catalog-bounded collection selected by a trusted owning content executor.
They update inventory through one shared Activity transaction. They are not
player commands and do not establish event eligibility or reward-pool membership.

Curio acquisition validates every state and its handbook identity, prevents
duplicate identities across mode copies and existing holdings, and initializes
active status, declared charges and zero activation counts. Blessing acquisition
adds base-level holdings and refreshes Equation progress using the complete
resulting inventory in the same transaction. An existing Blessing offer must
be resolved before this accepted-reward boundary can be used.

Unknown/unbound identities, duplicate rewards, invalid counts and stale hashes
reject without a partial reward. Inventory keys remain canonically ordered.
Blessing acquisition draws no RNG; Curio acquisition uses labeled draws for its
reviewed Path rewards. Neither boundary applies an event cost, opens a door or
declares a room complete. Curio acquisition also executes the explicitly authored
immediate fragment grants in the [acquisition contract](divergent-universe-curio-acquisition.md).
Other acquisition triggers remain pending. Single-item acquisition uses the
same path; these capabilities do not complete source mechanics.

The production-bundle fixtures in
`crates/starclock-mode-universe/src/divergent_universe/tests/reward_acquisition.rs`
check multi-item holdings, charges, rejection of an invalid second reward,
same-Curio mode-copy collisions, pending offers, fresh reconstruction and
same-transaction Equation expansion. Test-selected identities are not an event
pool or a production occurrence binding.

## A Dash of Color: current evidence boundary

The current reference identity is `divergent-universe.occurrence.108` and its
explicit handbook variant is `divergent-universe.occurrence-variant.722601`.
The variant publishes
`Config/Level/Rogue/RogueNPC/RogueNPC_410/RogueNPC722601.json`, absent at the
pinned source revision. Neither a nearby ID nor a matching localized option
title proves which option program belongs to that variant.

All following structured locators refer to released Version 4.4 transcription
at [Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`, inspected 2026-09-08.
Digests are SHA-256 of the exact Git blob bytes, not decoded row JSON.

| File under `ExcelOutput/` | Locator | Blob SHA-256 |
|---|---|---|
| `RogueTournHandBookEvent.json` | `EventHandbookID=108`, array index 49 | `15e0e448186b799687a1ef4582e96b40da43888fee5a0e31278eb6cbc29106b9` |
| `RogueTournNPC.json` | `RogueNPCID=722601` | `e3ef2da06848f45704310e1a35ae06390febf7d4e5a6027591912929b4e4c478` |
| `RogueDialogueOptionDisplay.json` | `OptionDisplayID=22601..22603`, `222601..222603` | `a6328c0188b43f7b752e84615e9da6acd922f66a9579b6f68916bc0dab8ed02a` |
| `RogueDialogueOption.json` | `OptionID=2260101`, `2260201`, `2260301` | `26419c810193d2720791a19c14733831ca0c235903440f288b186e583fcf3202` |

The last three option rows explicitly reference display rows `222601..222603`
and carry single parameters `300`, `3`, `3`. Those parameter values are exact
facts about those rows, **not** proven current-event reward values. Display rows
`22601..22603` instead use parameter placeholder `#2`; display rows
`222601..222603` use `#1`. These sets must not be merged by title or suffix.

The independently maintained [BWIKI event list, revision 92645](https://wiki.biligame.com/sr/index.php?title=%E4%BA%8B%E4%BB%B6%E4%B8%80%E8%A7%88&oldid=92645)
and [Star Rail Wiki's event entry](https://honkai-star-rail.fandom.com/wiki/A_Dash_of_Color)
describe three alternatives: 200 Cosmic Fragments, two random 1–2-star Curios,
or two random 1–2-star Blessings. This is community-reported evidence, not a
verified 4.4 program binding. Both were reviewed 2026-09-08. The BWIKI HTML
response for the canonical event-list URL was 529,315 bytes with SHA-256
`c0a1e384cd77623e8f95ebcc0ac5491e73bf946890e309fde6dc40bee630adc1`
and declared revision 92645. The second source was accessible through indexed
search text only; its full-page request was blocked and no full-page hash is
claimed. No raw page, dialogue or media is committed.

Searching all retained Rogue graphs for `2260101` could not establish the
missing relationship: the partial clone attempted to fetch a missing blob from
its GitLab promisor remote, and the connection failed. That failure is not
evidence that no retained graph references the option.

## Authored current-version policy

The executable-decision authoring project is
`config/divergent-universe-decisions.toml`. Its openpyxl-authored
`DivergentUniverseDecisions.xlsx` contains twenty-four strongly typed tables and 134
rows: fifty-one sources, one event policy, one event, three choices, three ordered
outcomes, eighteen Curio acquisition definitions, four fragment-gain components
and one [normal-battle Blessing policy](divergent-universe-battle-blessings.md)
with eight separately authored Curio Path-weight definitions, and one
[Equation-acquisition grant](divergent-universe-equation-grants.md), one
[initial Equation policy](divergent-universe-initial-equations.md) and one
[layer battle route](divergent-universe-layer-battles.md), plus its separately
authored encounter pool, three [full-HP Curio victory grants](divergent-universe-curio-battle-grants.md),
four [evolution edges](divergent-universe-curio-evolutions.md), four
[evolution events with twelve options](divergent-universe-evolution-events.md),
three [Sage victory grants](divergent-universe-sage-victory.md) and three
[public domain choices](divergent-universe-layer-battles.md#authenticated-public-domains),
plus three [base fragment policies](divergent-universe-battle-fragments.md) and
two [Curio domain-expiry definitions](divergent-universe-curio-domain-expiry.md)
and one [Curio battle-stat/lifetime definition](divergent-universe-curio-battle-stats.md)
plus one [Curio battle-healing reaction](divergent-universe-curio-battle-reactions.md)
and four [Tawot service definitions](divergent-universe-tawot-service.md) executing
through an explicitly admitted purchase/offer graph, not automatic Forge placement.
The acquisition rows carry
explicit fragment ordering/rounding and Path Blessing sampling/exhaustion policies.
Curio pools exclude reviewed wax rewards whose mandatory Blessing acquisition
cannot complete; their initial offer conditions use the same eligibility.
This boundary keeps executable operations out
of the reference pack's `payload_json` cells. The reference pack remains a
required current identity/evidence input, not a fallback executable format.

The fragment option describes a base reward; its actual credit includes active
reviewed [fragment-gain components](divergent-universe-fragment-gains.md). The
same pipeline applies to Curio acquisition grants and curse-chest gains.

`DecisionCatalog` in `starclock-data` loads only the new binary Sora bundle,
validates both directions of the handbook/variant join against the current
reference catalog, and exposes immutable Starclock-owned values. Its digest
binds the decision schema, complete decision binary and reference component.
The mode factory validates this catalog during construction and includes its
digest in the Activity configuration identity. The baseline replay's ModeContent
component binds the same combined decision/reference digest. Configuration
changes therefore affect canonical Activity state and exact replay component
matching. Data lowering and identity binding alone grant no runtime disposition
or complete-run credit; the explicit graph binding below executes the choices.

`tests/decision_identity.rs` under the mode's `divergent_universe` module
checks the dependency separately from gameplay: an isolated decision-digest
change leaves the graph definition digest unchanged but changes canonical
Activity state, fresh construction is deterministic, and both run families'
replay manifests reject a reference-only ModeContent component.

The selected `du.policy.color.current-options` policy applies the community's
200/2/2 alternatives to variant 722601. It is an explicit replaceable Version
4.4 policy, not a claim that the absent current source graph has been recovered.

| Unresolved field | Selected policy | Alternatives and reason | Confidence / replacement |
|---|---|---|---|
| Current option binding and amounts | Three public alternatives, 200 fragments or two 1–2-star rewards | Do not substitute the unbound 300/3/3 source rows; the public event description is the available event-specific evidence | Medium; replace with a released current graph or reproducible current observation |
| Costs | Zero fragments for each choice | No cost is reported by the event description; inventing a debit would change that public mechanic | Medium; replace when a current cost/condition is observed |
| Eligibility and hidden weights | Current Common/Rare identities, exclude owned, one canonical state per Curio, uniform integer weight per identity, without replacement | Do not weight multiple state copies as separate Curios or infer an original event pool from shared tables | Low; replace pool members, rarity mapping, state choice and weights independently when proven |
| Insufficient candidates | Disable the affected choice before selection; keep the free fragment branch available | Silent no-reward completion or partial reward would violate the stated count; inventing fragment compensation has no source support | Low; replace when exhaustion behavior is observed |

Canonical Curio state selection means sorting current, identity-bound state
IDs by their stable project key and selecting the first state. This selector is
implemented in the reward executor; it does not establish an original-game
state preference or implement the selected Curio's acquisition effects.
All rows retain exact source locators and the policy replacement condition.

The new production-bundle and negative-input tests check typed values,
contiguous child order, foreign references, missing provenance, invalid numeric
ranges and exact policy labels. The workbook verifier independently rebuilds
the schema lock, readers, binary and debug export, then compares a fresh
openpyxl-authored workbook's Sora export. Neither check is an event-playthrough
fixture; these rows are not claimed `DataReady` gameplay.

## Production selection and remaining work

`decision_rewards::DecisionRewardRuntime` now lowers the current three
single-outcome choices into checked currency operations or inventory rewards.
It samples distinct unowned identities in stable project-key order through the
shared Reward RNG stream, with separate Curio/Blessing purposes. Common/Rare
rarity mapping and canonical Curio state selection are the authored replaceable
policy, not recovered hidden weights. It does not interpret multi-outcome
sequences; those reject explicitly instead of dropping unimplemented outcomes.

The inventory boundaries share pure read-view operation generation with this
executor. Blessing rewards validate clean Equation state and no active offer,
then refresh progress from the resulting holdings in the same operation list.
Preflight reports insufficient currency or candidates without drawing RNG.
The owning offer compiler removes unavailable choices through shared Activity
conditions over these same pools. Balanced expressions keep complete-catalog
candidate counts within the shared expression-depth limit. Destroyed and
noncanonical Curio copies still exclude their handbook identity; enhanced
Blessings remain owned. Selection rechecks eligibility before drawing RNG.

Production-data composition tests execute all three grants through the shared
generated-option transaction, verify fresh reconstruction and late rejection
rollback, and check owned/exhausted pools. Their choice graph is explicitly a
test fixture, not the production occurrence/room graph. Separate production
binding fixtures now exercise all three branches in both run families.

The shared Activity layer now provides `choose_option_with_generated_prefix`:
authored selection operations, RNG-generated state operations, the actual
offered choice and automatic graph advance have one rollback boundary. Its
integration fixtures cover ordering, one-time consumption, fresh reconstruction
and late rejection/fault rollback. `occurrence_binding` uses this primitive to
commit the grant, consume the actual choice, record headless dialogue completion
and open exits in one transaction. Its generated reward-acceptance slot is a
same-transaction guard, cleared by the authored choice body. Ordinary
`GraphActivity::choose_option` cannot skip the reward executor. The guard is not
source-mechanic completion evidence; actual inventory operations precede it.

`DivergentUniverseEntry::with_initial_occurrence` explicitly binds an authored
variant to the first logical checkpoint. The baseline fixture chooses the
lexicographically first authored variant. This is a replaceable project policy
for the initial checkpoint only, not an observed room selector, random event
pool or physical room binding. The factual room-candidate slot remains empty.
Placement is part of the entry and graph identities. Once selected, the event
exposes the existing Encounter/Route offer at the same node; traversal retains
the completion/door guard and resets Node-scoped state. Raw reference-only
variants are not admitted by this binding.

Placement policy boundary:

- Missing fact: released membership and selection rules for the initial event
  room, including the empty layer-room table and absent current event graph.
- Selected behavior: explicit entry variant; the baseline uses the first
  authored variant at logical checkpoint one.
- Alternatives/rationale: an invented random pool would assert unproven
  membership; caller-owned explicit selection keeps placement replaceable and
  deterministic while permitting the authored policy to execute.
- Confidence: project orchestration only, not observed parity. Replace the
  baseline selector independently when released membership and topology are
  established; preserve explicit input identity and atomic selection.

The default runner and Agent adapter use the mode executor. Replay verification
checks components and entry before executing recorded public selections, then
regenerates and compares every boundary's scores, nested battle commands/events
and hashes before advancing. It does not substitute default-controller choices.
Both families' three alternatives complete the current one-battle baseline and
verify from fresh production inputs. Counterfactual owned-inventory fixtures
apply the actual production node program to prove zero/one/two remaining
candidate filtering; they are not an account-inventory entry feature.

The following remain required for complete production gameplay:

- Finish each selected reward's acquisition effects and any resulting
  decisions before treating content completion as complete gameplay. The current
  binding completes inventory and the reviewed immediate fragment/Path Blessing grants only; omitted effects are
  an implementation gap, not an accepted no-effect policy.
- Replace logical-checkpoint placement with the required room/topology execution
  and lower remaining event programs without weakening missing-graph gates.
- Keep policy accuracy separate from exact handbook identity and prove all
  complete-run matrix axes through actual execution.

Three authored choices execute currency/inventory rewards in the production
Activity graph. They do not establish complete event parity. The source obligations,
program dispositions and complete-run release gates remain pending as described
in [current state](state.md) and the
[runtime goal](goals/22-divergent-universe-runtime.md).
