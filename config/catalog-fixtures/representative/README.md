# Representative catalog fixture

This isolated fixture proves the complete schema-template → workbook → pinned
Sora export → generated reader → private domain-definition path. Its TSV files
are deterministic test inputs for the workbook materializer, not production
content or a runtime data path. The verifier creates all 80 `.xlsx` workbooks
in ignored cache roots and compares two independent binary/debug exports before
checking the committed `config.sora` golden.

All three identities are disabled `ProjectFixture` rows. The production loader
must reject this bundle; only crate-internal tests may select fixture mode.
