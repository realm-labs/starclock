# Agent session creation

`AgentSessionFactory::load_production` validates the embedded production bundle
and builds the frozen Standard-v1 combat catalog once. Cloned production
factories share immutable `Arc`-owned data and combat catalogs; every creation
allocates a distinct mutable `Battle` and incremental replay trace.

Untrusted creation accepts only:

- a checked opaque session ID;
- a checked `scenario.standard-v1.*` identity that must resolve to one of the
  six frozen production scenarios;
- either the authored scenario seed or one exact `u64` encoded as an
  `AgentUInt` string;
- the default `PlayerVisible` policy.

There is no creation parameter for `BattleSpec`, catalogs, abilities, programs,
Rule IR, commands, damage, targets or RNG state. Unknown but syntactically valid
scenario identities fail as `configuration_rejected`. Debug policy fails as
`unauthorized_policy`; its separately authorized session path is added with the
remote/local authorization composition rather than inferred from request data.

The session privately owns the battle, encounter/spec replay identities,
resolved master seed and an initially empty `BattleTraceEntry` recorder.
Session IDs do not enter battle creation, so two sessions with the same frozen
scenario and seed have identical initial authoritative hashes. The recorder is
incremental and will receive each accepted command/hash boundary during
settlement; live session work never verifies by replaying a growing prefix.

The shared factory was moved from private CLI code into the stable
`starclock-data::standard_v1` application/data seam. The CLI now delegates to
that same factory, eliminating a second production construction path.
