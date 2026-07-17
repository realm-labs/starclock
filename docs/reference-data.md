# Reference Data

## Core constants

| Name | Value | Status |
|---|---:|---|
| Base Action Gauge | 10,000 | Verified |
| Default starting Skill Points | 3 | Verified |
| Default maximum Skill Points | 5 | Verified |
| Unbroken enemy damage multiplier | 0.9 | Verified |
| Broken enemy damage multiplier | 1.0 | Verified |
| Universal Weakness Break delay | 25% / 2,500 AG | Verified |
| Break debuff base chance | 150% | Observed/current wiki |
| Weakness Break Efficiency cap | 300% | Observed/current wiki |
| Most Basic ATK Energy | 20 | Default, exceptions exist |
| Most Skill Energy | 30 | Default, exceptions exist |
| Enemy defeat Energy | 10 | Default; mode exceptions exist |

## Attacker level multiplier

Break and Super Break formulas use this lookup table. Player characters normally use levels 1–80. Values 81–95 are currently described as enemy-exclusive for special break sources and are omitted from the core table.

Source: [Toughness — Level Multiplier](https://honkai-star-rail.fandom.com/wiki/Toughness#Level_Multiplier).

| Lv | Multiplier | Lv | Multiplier | Lv | Multiplier | Lv | Multiplier |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 54.0000 | 21 | 149.3323 | 41 | 408.1240 | 61 | 1752.3215 |
| 2 | 58.0000 | 22 | 158.8011 | 42 | 451.7883 | 62 | 1861.9011 |
| 3 | 62.0000 | 23 | 168.1768 | 43 | 494.6798 | 63 | 1969.1242 |
| 4 | 67.5264 | 24 | 177.4594 | 44 | 536.8188 | 64 | 2074.0659 |
| 5 | 70.5094 | 25 | 186.6489 | 45 | 578.2249 | 65 | 2176.7983 |
| 6 | 73.5228 | 26 | 195.7452 | 46 | 618.9172 | 66 | 2277.3904 |
| 7 | 76.5660 | 27 | 204.7484 | 47 | 658.9138 | 67 | 2375.9085 |
| 8 | 79.6385 | 28 | 213.6585 | 48 | 698.2325 | 68 | 2472.4160 |
| 9 | 82.7395 | 29 | 222.4754 | 49 | 736.8905 | 69 | 2566.9739 |
| 10 | 85.8684 | 30 | 231.1992 | 50 | 774.9041 | 70 | 2659.6406 |
| 11 | 91.4944 | 31 | 246.4276 | 51 | 871.0599 | 71 | 2780.3044 |
| 12 | 97.0680 | 32 | 261.1810 | 52 | 964.8705 | 72 | 2898.6022 |
| 13 | 102.5892 | 33 | 275.4733 | 53 | 1056.4206 | 73 | 3014.6029 |
| 14 | 108.0579 | 34 | 289.3179 | 54 | 1145.7910 | 74 | 3128.3729 |
| 15 | 113.4743 | 35 | 302.7275 | 55 | 1233.0585 | 75 | 3239.9758 |
| 16 | 118.8383 | 36 | 315.7144 | 56 | 1318.2965 | 76 | 3349.4730 |
| 17 | 124.1499 | 37 | 328.2905 | 57 | 1401.5750 | 77 | 3456.9236 |
| 18 | 129.4091 | 38 | 340.4671 | 58 | 1482.9608 | 78 | 3562.3843 |
| 19 | 134.6159 | 39 | 352.2554 | 59 | 1562.5178 | 79 | 3665.9099 |
| 20 | 139.7703 | 40 | 363.6658 | 60 | 1640.3068 | 80 | 3767.5533 |

## Break coefficients and debuff timing

| Element | Initial coefficient | Debuff | Duration | Special |
|---|---:|---|---:|---|
| Physical | 2.0 | Bleed | 2 | HP-scaled base with cap |
| Fire | 2.0 | Burn | 2 | `1.0 * level multiplier` base tick |
| Ice | 1.0 | Freeze | 1 | Skip action, then advance next by 50% |
| Lightning | 1.0 | Shock | 2 | `2.0 * level multiplier` base tick |
| Wind | 1.5 | Wind Shear | 2 | 1 or 3 starting stacks; max 5 |
| Quantum | 0.5 | Entanglement | 1 | 20% Break-Effect-scaled delay; max 5 hit stacks |
| Imaginary | 0.5 | Imprisonment | 1 | 30% Break-Effect-scaled delay; -10% SPD |

## Data governance

Copy these values into code or a data file only with a `rules_revision`. When a source changes, update this document, the revision, and golden test expectations in the same commit. Do not silently change balance constants.
