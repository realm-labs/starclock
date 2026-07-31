# Fate/Star Rail Night V1 Manifest

This directory owns the generated Version 4.4 selector, source and content
manifests for Goal 19. `foundation.json` freezes the execution boundary and
the 89-file discovery seed; it is not a content denominator. Later manifests
must derive membership from explicit FateRin selectors and transitive
references and must retain named exclusions.

Regenerate and check the foundation with:

```text
node tools/fate-star-rail-night-reference/freeze-foundation.mjs \
  --source-cache .cache/fate-star-rail-night-sources
node tools/fate-star-rail-night-reference/freeze-foundation.mjs \
  --source-cache .cache/fate-star-rail-night-sources --check
```
