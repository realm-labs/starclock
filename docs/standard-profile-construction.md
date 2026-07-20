# Standard profile construction

Goal 01 batch `G01-P6-B2` implements Standard as a composition profile over the
generic one-Battle Activity aggregate. It does not introduce a Standard command
processor, result protocol, RNG stream or state hash.

## Profile shape

`StandardProfile` stores only its stable identity, referenced Activity identity,
maximum party size and ordinary encounter wave-transition default. Exactly one
player team is structural, party size is restricted to one through four, and no
clock, score or seasonal-rule field exists in the domain type. The four frozen
wave boundaries remain combat catalog policy; Standard does not add another
transition mechanism.

Before scenario construction, the profile validates that:

- the `ActivitySpec` has the referenced definition identity;
- its participant policy declares exactly one team and fits the profile party
  bound; and
- its result projection is exactly Outcome, FinalStateHash, EventDigest and
  TerminalFault in frozen sequence, with no undeclared metric.

`StandardActivityBinding` adds the stable authored BattleBinding identity around
one already validated generic `ActivitySpec`. It does not inspect or duplicate
the opaque `BattleSpec`.

## Scenario construction

`StandardScenario` resolves a profile and binding once, parses the authored
16-digit hexadecimal master seed exactly, and retains the expected Won, Lost or
Faulted result. Instantiation creates `starclock_activity::Activity` directly
with the parsed seed and a caller-supplied activity-instance identity. Two
instances with the same inputs therefore begin with the same canonical state.

`StandardActivity` is only a thin identity/expectation wrapper. All commands,
handoff validation and terminal mutation remain in the generic Activity. Its
terminal verifier maps Won to Complete, Lost to Failed and Faulted to Faulted;
nonterminal and mismatched results are typed failures.

## Deferred boundaries

The Phase 3 synthetic and benchmark fixtures remain intact for replay and
performance continuity. B2 does not import production rows, construct combat
catalogs, run controllers, extend replay records or expose CLI commands; those
remain assigned to later Phase 6 batches. Production inputs remain Excel/Sora
only, and no challenge, universe, route, fork, reward or shop type is added.
