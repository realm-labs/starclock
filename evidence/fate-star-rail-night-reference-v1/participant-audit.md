# Goal 19 Participant Audit

`G19-P1-B2` normalizes 85 participant records at digest
`16b2b37b66a1e53fbcc7b9d56485241a4a621fc7e15e298c879d564f87455e8b`:
eight classes, 21 handbook Masters, 21 Master definitions, six FateRin avatars,
ten Case Board Servants, nine team rows, six owners and two Master configuration
program digests. Two avatar-description rows remain evidence-only; 83 rows are
enabled reference facts.

The pack keeps Master, Servant, avatar, team and owner identities separate. It
does not infer uniqueness, loadout substitution or trial policy from matching
names; exact scalar relationships remain source-shaped until their semantic
fixtures are authored.

```text
node --max-old-space-size=4096 tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B2 --check
```
