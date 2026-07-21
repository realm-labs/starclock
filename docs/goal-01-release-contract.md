# Goal 01 release contract

`G01-P8-B6` freezes the headless release surface for `core-combat-v1`. The
machine-readable policy and evidence are
[`release-contract.json`](../policy/release-contract.json) and
[`release-evidence.json`](../evidence/core-combat-v1/release/release-evidence.json).

## Command-line surface

The executable exposes four command families under JSON schema
`starclock-cli-v1`:

- `config validate [--bundle PATH] [--json]`;
- `catalog coverage [--goal core-combat-v1] [--category NAME] [--json]`;
- `battle run --scenario ID --seed U64 [--controller baseline|replay] [--replay-out PATH] [--json]`;
- `replay verify FILE [--json]`.

Exit classes 0 and 2–7, production bundle identity, 283/283 content coverage,
synthetic replay bytes and production Standard replay bytes are pinned by CLI
integration tests. The six `scenario.standard-v1.*` rows are the production
scenario namespace; future commands require an explicit CLI schema revision.

## Library surface

Eight library facade files and their owning Cargo manifests are digest-bound.
The release does not promise semantic-version stability beyond Goal 01, but it
does prevent accidental facade or dependency drift from passing the release
gate unnoticed. The architecture audit additionally freezes explicit
re-exports and rejects implementation-token leakage.

## Reproducibility

Coverage, production configuration and all Phase 8 hardening reports are bound
by SHA-256. `node tools/release/run-clean-checkout.mjs` archives the staged Git
tree into an isolated directory, bootstraps the checksum-bound Sora tool, and
runs the default repository gate without inheriting `target`, generated work
directories or the source cache. A checksum-verified Sora crate download or
previously bootstrapped tool installation may be seeded as a tool cache; its
version is revalidated inside the isolated checkout, while all repository build
outputs are created there.
