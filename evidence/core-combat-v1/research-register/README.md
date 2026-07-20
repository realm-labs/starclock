# Goal 01 Phase 0 research register

This directory is the implementation-blocker register frozen by `G01-P0-B3`.
It contains no copied ability descriptions and no guessed runtime behavior.

- `research-cases.json` names each question, owner, fixed expectation and exact
  prepared-record evidence hash.
- `fixture-specifications.json` defines the deterministic observation or golden
  envelope required to resolve each case.
- `decision-records.json` records the architecture/project-policy decisions that
  apply while game-specific timing remains unresolved.
- `source-register.json` records URLs, access dates, version/confidence notes and
  evidence hashes.
- `evidence-index.json` binds the generated files.

Cases resolved by exact source-bound observation move to `Observed` with their
bundle, command and evidence paths. `G01-P4-B11` additionally permits
`ResolvedProjectPolicy` only for an architecture blocker whose deterministic
generic behavior is fixed by a decision record and regression fixture. That
state is not a claim about an unobserved game fact: each such case retains an
explicit V1B observation/stronger-source gate before affected production
content may become `DataReady`. The ten Himeko Nova approximations remain
`Researching` for their Phase 7 mechanic prerequisite.
