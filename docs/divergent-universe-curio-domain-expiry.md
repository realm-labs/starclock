# Divergent Universe Curio domain expiry

## Current behavior

`CurioDomainExpiries` authors four released domain-entry limits independently of
their effects: mode-copy 9070 uses effect 2070 and three entries; 9079 uses effect
2079 and five; 9071 and 9072 use effects 2071/2072 and three each. States 9070/9079 have
[fragment-gain rates](divergent-universe-fragment-gains.md), while 9071 has the
intrinsic entry grant described below; 9072 grants a separate
[victory Blessing](divergent-universe-tawot-victory.md). All four copies
belong to handbook identity 9068 and remain mutually exclusive. Expiry discards
the holding, including its charge and activation counters; it does not create a
repairable destroyed holding. The global gain pipeline stops seeing it immediately.

Production `choose_battle_domain` authenticates the flow, graph, offered decision
and option. Its shared generated-choice transaction emits eligible intrinsic
entry grants, then decrements allowances,
discards expiring holdings, sets the selected domain and constructs the encounter.
The typed option requires the generated domain marker, so generic option execution
cannot bypass this boundary. Rejected, stale, repeated and foreign choices cannot
consume entries. Encounter RNG and all prefix mutations share rollback.

No second activity engine, shared content-ID branch, new slot or native handler
is introduced. Existing run-owned Curio charges represent remaining entries for
these authored states. Runtime initialization uses the reviewed expiry allowance;
the frozen reference's `declared_charges` metadata is not rewritten. Other Curios
retain their existing charge interpretation. Acquisition, replacement and evolution
share the same initialization helper; accepted charge updates cannot exceed the
authored maximum. Finalization still clears the entire run inventory.

## Evidence and replaceable policy

Sources 12, 14 and 42 bind current `Tourn3` state/effect membership, released EN
descriptions and exact `ParamList[1]` limits. Pinned source repository revision is
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, Version 4.4, reviewed 2026-09-12:

- `ExcelOutput/RogueTournMiracle.json`: states 9070 and 9079; SHA-256
  `176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438`.
- `ExcelOutput/RogueMiracleEffect.json`: effects 2070 and 2079; SHA-256
  `fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35`.
- `TextMap/TextMapEN.json`: hashes 7205940712072507551 and 3429135723207096768;
  SHA-256 `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

The exact limits and discard semantics are source-backed. The independent timing
choices below are `VersionedProjectPolicyActiveFutureSelectedDomainsDiscard`,
not observed execution parity:

- Acquisition does not count the domain already entered. Initial event rewards
  receive a full allowance; later accepted domain selections consume it.
- Pre-selection layer services are outside the next selected domain. Internal
  initializer, battle, reward and service graph nodes never consume entries.
- The limiting entry discards before encounter generation or new-domain rewards,
  so even that domain's first fragment credit lacks the expired bonus.
- Destruction pauses the count. Repair resumes the remainder without refilling.
  Replacement or reacquisition starts a fresh allowance. An explicitly accepted
  zero-charge state is discarded at the next counted entry.

These timing/lifecycle fields have low parity confidence. Counting the acquisition
domain, counting while destroyed and expiring after entry rewards are alternatives.
Replace fields independently when released programs or reproducible observations
establish them. The current one-domain-per-layer route has only two future entries
after initial acquisition: it does not by itself demonstrate a fresh five-entry
state's expiry. Future multi-room/domain routes must bind their real entry boundary
to the same allowance operations; this implementation does not prove original
room placement, five-domain public progression or complete Curio/source disposition.
The current event policy selects only the lexicographically first state for each
handbook identity. For identity 9068 that is state 9068, not 9070 or 9079. Their
separately admitted [Tawot service](divergent-universe-tawot-service.md) now
executes paid acquisition of both states. Automatic original Forge placement
and a full five-domain public progression remain pending; controlled lifetime
vectors are still labeled separately. Other effects of this identity
are not silently assigned the reviewed states' behavior.

## Verification

Data fixtures reject mismatched state/effect/parameter/limit joins, duplicate
states, invalid ranges, missing summaries/policy and broken provenance. Operation
vectors cover the exact three/five allowance, all counter removal, paused
destruction, repair, replacement and reacquisition. Production route tests use a
controlled one-entry remainder to exercise real discard before Elite income,
generic bypass rejection, invalid/repeated selections and late prefix rollback.
Controlled accepted acquisition and the two future entries are exercised through
both run families with fresh-factory reconstruction comparing every state hash.
Separate public event/route transcript replay verifies the existing canonical
state selection and state 9068's separate
[battle-stat and battle-count behavior](divergent-universe-curio-battle-stats.md).

## Intrinsic domain-entry fragments

`CurioDomainGrants` binds state 9071 to an exact 60-fragment grant and references
its separate three-entry expiry row. Sources 55–57 use the same pinned revision
and file digests above, accessed 2026-09-12: `RogueTournMiracle.json` state 9071,
`Tourn3`, handbook 9068, effect 2071; `RogueMiracleEffect.json` effect 2071,
`ParamList[0]=60`, `[1]=3`; `TextMapEN.json` hash `3761031791427163166`.
The public [Tawot Cards page](https://honkai-star-rail.fandom.com/wiki/Tawot_Cards)
cross-checks those operands but requests Arcadian Chronicles verification;
it is not exact execution evidence.

`VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard` explicitly
differs from the passive-bonus expiry policy: the third qualifying entry grants
before discarding its owner. `VersionedProjectPolicyActivePositiveAllowanceFragmentsBeforeDiscard`
selects active positive-allowance holdings from the authenticated pre-entry view,
in stable state-key order. Each grant passes through the existing nonrecursive
fragment-credit pipeline before any expiry updates. Active global gain bonuses
apply to the original 60 independently: 9055 contributes 30 and 9159 contributes
18, for 108 total when both are held. This interaction is policy, not observed
stacking parity.

Acquisition does not retroactively reward/count the current domain. Internal
service, initializer, battle and reward nodes do not grant or count. Destruction
pauses both; repair resumes; replacement/reacquisition reset three. A trusted
zero allowance grants nothing and is discarded at the next entry. All credit,
discard, domain-marker, encounter and RNG changes commit or roll back together.

The data loader checks exact source operands, expiry-policy compatibility and
exact-once grant ownership; a grant-before-discard expiry without its grant is
invalid. Tests distinguish three-entry operation vectors from the public route's
two future entries. They cover the final grant, pause/repair, zero, replacement,
reacquisition, gain bonuses, base/bonus overflow, generic bypass, stale/repeated
choices and late failure rollback. Both families' paid acquisitions execute
two future domain grants, three actual battles and fresh encoded replay.
Original multi-domain topology, a full three-entry public lifetime and full
Tawot source-program disposition remain pending.

## Explicit source-position room entries

`compile_curio_domain_route` lowers the same authored expiry and grant policies
into finite entry programs over the current position layout. It counts the
accepted entry of each fixed or selected logical room, not deck preparation,
the unselected alternatives, service menu/card loops, encounter, battle or
reward nodes. The underlying `compile_domain_route` remains a pure fragment
composition API; it does not supply this lifecycle. Runtime battle and Tawot
bindings require the exact lowered entry program and reject omitted or changed
prefixes. Room entry must be a single-visit Choice node without internal incoming
edges. Missing entries and programs exceeding the shared depth/operation limits
reject during compilation.

This mapping of a fixed source position to a counted domain is an explicit
low-confidence interpretation of the existing future-domain policy, not observed
original-game timing. Both fixed and chosen source rooms count once; limiting
passive states are removed before room content, while 9071 receives its final
eligible grant through the existing bonus pipeline before removal. Alternatives
include counting only chosen combat domains, excluding service rooms or applying
expiry after the room's rewards. Replace this mapping independently when released
execution or reproducible observations establish it. It does not infer a Forge,
Boss, Occurrence or encounter producer from a source preset.

The program uses the three existing lifecycle maps and `RemoveCounter`, keeps
stable state-key order and embeds the continuation in every Conditional branch.
Destroyed states pause; zero allowance grants nothing and is removed; active
allowances outside their authored range reject. Credits, counter removal, room
initialization and any downstream rejection share the ordinary Activity command
transaction and RNG rollback boundary. Acquisition in an already entered room
does not count or grant retroactively.

Source-position fixtures cross a real proxy battle before controlled accepted
acquisition and then execute complete three/five-entry lifetimes, internal service
loops, pause/repair, reacquisition, zero, final intrinsic income with gain bonuses,
fresh reconstruction in both run families, omitted-prefix binding rejection and
base/bonus overflow or later fixed-room failure rollback. Their other payloads
remain explicit probes. They are not proof of original room admission, paid
acquisition of every variant, encoded source-position replay or complete runs;
the default short baseline and its two future entries remain unchanged.
