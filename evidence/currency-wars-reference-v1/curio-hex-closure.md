# Currency Wars Curio, Miracle and Hex closure

Batch `G12-P2-B2` proves that released Version 4.4 Currency Wars has no
reachable Curio, Miracle or Hex-state content in the fixed GridFight closure.

## Closure proof

The generated manifest freezes the `curios_miracles_hex_states` denominator at
zero. The batch independently rechecks:

- all 153 direct `GridFight` Excel tables;
- all 984 mode-owned GridFight configuration files; and
- all six normalized Curio/Hex file contracts.

No direct table or configuration contains a Curio, Miracle, shared
`RogueMiracle`/`RogueCurio` or Hex-state reference. The only structured `Hex`
tokens are the legacy `HexName` and `HexDesc` field names on
`GridFightAugment`; those fields hold the Augment names and descriptions
already imported by P1-B8 and do not establish a separate Hex content type.

## Result

The six normalized files are canonical empty arrays. Their combined digest in
lexicographic order is
`b18664a06ed0bd101b25a7b58229152a46f6c4b013207f0721ed49960f9ec7a2`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-curio-hex-closure.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-curio-hex-closure.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
