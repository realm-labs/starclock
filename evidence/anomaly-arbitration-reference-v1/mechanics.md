# Active traits and mechanical contributions

The five active stages select eight shared enemy-trait MazeBuff rows. Their
canonical parameters and before-character-born ability bindings are preserved
without lowering the programs into runtime behavior:

- Knight 1 selects Taunting (`3033023`);
- Knight 2 selects Depowered (`3033038`) and Flow Break (`3033063`);
- Knight 3 selects Hemovore (`3033058`);
- normal King selects Equilibrium (`3033051`) and Enrage (`3033069`);
- Plight selects Equilibrium+ (`3033052`) and Enrage+ (`3033070`).

The three active Arbitral Quadrants each bind to both King stages. Plugin
`0014` is present in the fixed extracted ability list. Layout references prove
the bindings to plugins `0022` and `0023`, but their program bodies are absent
from that list; the rows therefore retain exact selector relationships and an
explicit non-runtime program-body boundary.

The contribution inventory accounts exactly once for all eight traits, three
Quadrants, three stage-selected battle events and 73 transitive configuration
programs. Configuration rows preserve source path, locator, reachability and
selector, but deliberately do not import program bodies or claim runtime
executability.
