# Character/build schema golden fixture

This is synthetic, disabled schema evidence for Goal 01 batch `G01-P1-B7`.
It proves typed Sora 0.3.0 references, ordered child rows, Trace self-references,
the closed build-patch union, complete E1-E6 ranks and complete S1-S5 parameter
rows. It is not released content, production input, or coverage evidence.

Production authoring remains `.xlsx`-only. Batch `G01-P1-B10` migrates these
contracts to production workbooks and generated readers; the TOML source here
exists only because this committed golden project tests schema behavior.

Run `node tools/config-schema/verify-character-build.mjs`. Pass `--bless` only
when intentionally reviewing a schema/generator revision.
