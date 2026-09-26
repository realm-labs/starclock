# Accepted Workbench Equation overwrite

## Current executable boundary

`reforge_equation_policy_accepted` settles a trusted owning service's
`DivergentUniverseAcceptedEquationRewrite`. The active current Workbench must
contain released function 3. Workbenches 102–105 and 110 contain it; 101 does
not. This membership does not establish NPC admission to a source room.

The input must be a current owned Equation and the output a different current
unowned Equation of identical quality. Quality uses the production Equation
category, not inferred ID ranges or display text. Both expanded and unexpanded
inputs are supported. The accepted boundary checks all 80 catalog identities;
it does not establish their eligibility for any particular original offer.

One shared generated Activity transaction replaces ownership, removes the old
derived progress/expansion, recomputes final progress, debits Cosmic Fragments
and increments the function-3 receipt. Existing Equation acquisition hooks
(including the once-per-logical-domain missing-recipe Curio) and expansion
rewards execute in this same transaction with their existing Reward RNG.
An unrelated Equation or Blessing offer is not consumed or overwritten.
Dirty or inconsistent derived state rejects rather than being silently repaired.

Stale hashes, wrong Workbench/function, unknown/owned outputs, unowned inputs,
different quality, insufficient funds, arithmetic overflow and late reward
failure preserve authoritative bytes, command/event sequences and RNG. Payment
is never a separate command after replacement.

This is a trusted service-settlement API, not an untrusted player action.
The owning service authenticates the pair. It does not generate a public offer,
sample original selectors, admit a service NPC or supply a player action for a
pending decision.
The state-only replacement plan is available only within the mode and now backs
the [optional offered Respite service](divergent-universe-respite-equation-reforge.md);
shared command processing is not forked.

## Explicit replaceable price policy

`DivergentUniverseWorkbenchEquationReforgePolicy::new(base, increment)` requires
positive checked `i64`-bounded integer prices. Price is
`base + increment * successful_function_3_overwrites`, using checked arithmetic.
Currency is Cosmic Fragments. The current Run-wide receipt is independent of
function 2's Blessing overwrite count and is not reset by Workbench entry.
Failure increments neither count nor price. The accuracy label is
`VersionedProjectPolicyAcceptedEquationReforgeLinearRunPrice`.

Exact base/increment, rarity-specific prices, original reset scope, cancellation/
payment timing, per-Workbench attempt limits and discounts/free-attempt Curios
remain unresolved. These are not silently assigned observed-parity status.
The offered service supplies an explicit caller-selected logical-room cap;
the tutorial is not proof of that numeric cap or complete original parity.

Rejecting all calls or granting flat/free swaps are rejected alternatives
because they omit the known same-quality increasing-cost transformation.
Historical modes' price tables and weights are not imported. Confidence in
numeric/scope parity is absent. Replace this policy with current released
price, reset, attempt-limit and modifier-order evidence; replace candidate
admission separately. Profiles must bind `configuration_digest()` in their
payload. It includes every numeric input and the named function, currency,
quality and count-scope policy; it does not change an already running identity.

No static production price or selector is authored by this change. Host inputs
use the existing trusted service boundary. Promotion to static configuration
still requires Excel/openpyxl/Sora and provenance-bearing rows.

## Released evidence

Pinned released Version 4.4 source:
`Dimbreath/turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`,
inspected 2026-09-26. The current files were byte-verified against these hashes.

| File | SHA-256 |
| --- | --- |
| `ExcelOutput/RogueTournWorkbenchFunc.json` | `b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078` |
| `ExcelOutput/RogueTournWorkbench.json` | `26053803e691fe1fa7be57bf94d1d766ff0cc3e6cebda579fa013b129775d736` |
| `TextMap/TextMapEN.json` | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

Function locator: `FuncID=3;FuncType=FormulaReforge;FuncDesc.Hash=1127997012628269409`.
Independent bilingual summary: 方程覆写为同品质的其他方程，费用随覆写次数增加 /
replace with a different equal-quality Equation at increasing cost.
`RogueTournWorkbench.FuncList` supplies function membership, not room placement.
Released TextMap tutorial locators `17448214355832679458` and
`1928576352697941510` corroborate Fragment currency and a per-Workbench attempt
limit; they supply no numeric limit or current NPC selector. Public search
yielded historical discussion and beta-labelled tutorial results, which are
not used as Version 4.4 price or candidate evidence. No beta/leaked source is
admitted.

## Verification and remaining scope

The `equation_reforge` tests execute all 80 identities as owned inputs with
same-category outputs, all five capable Workbenches and both run families,
comparing every boundary with an independently constructed production fixture.
Inventories and funds are trusted test inputs, not room grants. Tests cover
checked prices, configuration identity, independent function counts, unknown/
owned/unowned/quality errors, stale state, unsupported benches, pending offers,
dirty progress, overflow and insufficient funds. Actual expanded replacement
executes Curio rewards; malformed expansion charges reject twice without payment
or draws. Acquisition-wax rewards use the existing logical-domain once budget.
An exhausted wax-receipt map fails after reward sampling and earlier ownership/
payment operations; both repeated failures restore exact bytes/RNG, and clearing
that trusted fixture fault allows the same pair to settle once. Inactive/foreign
Workbench and completed-run calls also reject without mutation.

Optional public Equation overwrite menus now exist with explicit policy.
Original candidates/weights, NPC admission, price/modifier/limit parity,
default source-position topology and encoded
profile replay remain incomplete. This accepted boundary grants no terminal
source/mechanic coverage and does not complete the Workbench family.
