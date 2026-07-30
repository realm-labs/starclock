# Goal 16 Reference Boundary Audit

`G16-P4-B2` audits profile ownership, shared identities, synthesis and repository
isolation against the complete Version 4.4 reference pack. The machine result
is `reference-boundary-results.json` with SHA-256
`c50880c742c156aead77c599c8bc5adcba6991d94964f5a38eb760b4d587e157`.

## Profile correction and result

The first audit exposed that Demon King carried the explicit
`galactic-baseballer.shared-base.v1` relation while Departure expressed the
same relation only through prose and the common Activity module. P4-B2 added
the missing `shared_system_id` through the owning Departure normalizer.

The complete downstream chain was regenerated:

- the normalized profile and pack index;
- the Profile and Review workbooks;
- the Sora profile schema, lock and template;
- the profile reader, binary bundle and affected debug tables; and
- all 147 workbook review bands.

Both profiles now independently retain release identity (`2.2` and `3.3`),
bind the same shared system and Activity module, retain Version 4.4, remain
runtime-disabled, and preserve the explicit Demon King → Departure
non-replacement edge. The difference index still closes exactly as 38 repeated
values, 25 Demon King changes, 13 additions and seven Departure-only facts.

## Shared identity result

| Measure | Result |
|---|---:|
| Enemy-resolution rows | 104 |
| Distinct source Monster IDs | 88 |
| Enemy identity collisions | 0 |
| Enemy-skill resolution rows | 339 |
| Distinct source Skill IDs | 287 |
| Skill identity collisions | 0 |
| Exact source-status locators | 10 |
| Copied enemy or skill definitions | 0 |

Every enemy and skill row is `ownership=Shared`, resolves to the frozen
Version 4.4 stable identity, and carries an explicit receipt that the existing
definition is referenced without copying. Repeated source IDs across profiles
map to the same inherited identity.

## Synthesis result

All 27 recipes and 54 ordered inputs resolve to typed profile-owned weapons or
accessories. The graph contains 25 Legendary, one Twin and one Supreme recipe,
one inter-recipe dependency, no cross-profile input, no unknown endpoint and no
cycle. Its complete 27-node topological order has SHA-256
`3d9a3c9c00c2b64f149b0557f0e548b424afa0ba634776f0276f0140f9a16e4c`.

## Isolation and coverage result

All 2,232 obligations remain fixed: 2,207 are DataReady, 25 are explicitly
EvidenceOnly, none is Blocked, and all 12 research gaps remain
ReplaceableNonBlocking. The audit compares 29 protected Standard/other-mode
generated, manifest, reference and evidence roots to the Goal 16 branch base;
every Git tree identity is unchanged.

The complete committed and working-tree change set is restricted to the Goal
16 allowlist. A tracked-text scan finds zero Galactic Baseballer identifiers
in Rust crates, production configuration, Standard data, or any other mode
partition. The current and main checkouts are registered as distinct
worktrees, and the current branch is
`codex/goal16-galactic-baseballer-reference`.

## Current regenerated fingerprints

| Artifact | SHA-256 |
|---|---|
| Workbook semantic content | `2c0021b589e057bc398d7202ed73193eb48a2cdf97229b68cc9fd1b464091aac` |
| `GalacticBaseballerProfiles.xlsx` | `b0ab3f86c108da36734f3dc0e11d67efce290b0160d0936bd326dc2c97c89107` |
| `GalacticBaseballerReview.xlsx` | `b73198be04c28507adf284fef52afcea2b26e6d9224e9b0a7833a43374094f9b` |
| Sora schema lock | `cd0e4a3645da7d1a6e0526d506881879bd63f347549de7619aa85714e31da56b` |
| Sora bundle | `82e600ee2b8aaaaeada82810f223d4f45e193833eb4151c939a5e51950f78848` |
| Complete Sora generated tree | `fad82dc324258451c71039130f5f5043b12360efd80716d81e02ed4ffaf18e03` |
| Visual review record | `9a0b4626af8f3338ef53b55890dc2d555657f1dd664cd5f28eac758964396470` |

Sora 0.3.0 independently regenerated the release and the locked reader loaded
all 40 tables and 10,615 rows. All four contact sheets were visually inspected
after the final required-field metadata update with zero severe defects.

## Reproduction

```text
fnm exec --using 24.15.0 node tools/galactic-baseballer-reference/audit-reference-boundaries.mjs
fnm exec --using 24.15.0 node tools/galactic-baseballer-reference/audit-reference-boundaries.mjs --check
```
