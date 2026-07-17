# Common schema fixture

This is synthetic Sora evidence for `config/schema/common.toml`. Its TOML rows
exercise schema references and deterministic exporters without becoming a
production authoring or runtime path. They are always disabled, use
`ProjectFixture`/`SyntheticFixture` labels, and never contribute to Goal 01
coverage.

`G01-P1-B10` owns the first authoritative `.xlsx` layout, generated production
reader and Sora-to-domain conversion boundary. Run
`node tools/config-schema/verify-common.mjs` to regenerate this fixture in an
ignored work directory, verify the exact generated Excel-template list, and
compare every byte-stable artifact with `expected/`. Raw workbook ZIP bytes are
not golden identities.
