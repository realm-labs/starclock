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

Cases remain `Researching` until their owning implementation batch records a
source-bound observation and passes the executable fixture; resolved probes move
to `Observed` with exact bundle, command and evidence paths. This is deliberate:
Phase 0 eliminates unnamed ambiguity; Phase 4 and the registered Himeko Nova
mechanic prerequisite close it without inventing defaults.
