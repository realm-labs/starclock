# Cross-Platform Determinism and Numeric Policy

This document is normative for authoritative battle and activity simulation. Its rules override earlier prototype guidance that allowed floating-point state.

## Determinism target

The project requires the same validated configuration bundle, seed, and accepted command stream to produce identical events and canonical state hashes across supported operating systems, CPU architectures, headless execution, Bevy, and other adapters.

Three distinct goals must not be conflated:

| Level | Requirement |
|---|---|
| Same binary replay | Mandatory from the first executable milestone. |
| Cross-platform replay | Mandatory before the starclock-combat API and configuration schema are declared stable. |
| Exact parity with the original game's hidden arithmetic | Not guaranteed because its internal precision and rounding boundaries are unpublished; approximate through observed golden vectors. |

Presentation timing, animation frames, wall-clock time, thread scheduling, and engine entity order are not simulation inputs.

## Fixed-point decision

Authoritative combat code uses decimal fixed-point arithmetic through exactly pinned [`fixnum` 0.9.5](https://docs.rs/fixnum/0.9.5/fixnum/):

```toml
[dependencies]
fixnum = { version = "=0.9.5", default-features = false, features = ["i64", "std"] }
```

The initial representation is a signed 64-bit decimal fixed-point value with six fractional digits:

```rust
type Repr = fixnum::FixedPoint<i64, fixnum::typenum::U6>;
```

This gives a resolution of `0.000001` and an approximate range of ±9.22 trillion. Multiplication and division promote the `i64` representation to a wider intermediate in the selected library. A range/precision benchmark may justify separate or wider internal representations later, but such a change increments `numeric_policy_revision` and invalidates old state hashes unless migrated.

Do not expose `fixnum` types outside the numeric module. The dependency is an arithmetic implementation detail, not the domain API.

## Domain types

Wrap the representation in types that make units and legal operations explicit:

```rust
pub struct Scalar(Repr);
pub struct Ratio(Scalar);
pub struct StatValue(Scalar);
pub struct Speed(Scalar);
pub struct ActionGauge(Scalar);
pub struct Probability(u32);
pub struct Hp(i64);
pub struct ShieldAmount(i64);
pub struct DamageAmount(i64);
pub struct HealingAmount(i64);
```

`Ratio::ONE` is exactly `1_000_000` raw units. HP, final shield values, applied damage, and applied healing are integral state-changing amounts. Calculators may retain fixed-point intermediates until the formula's documented finalization boundary.

Do not implement generic `Mul` or `Div` for domain wrappers merely for concise syntax. Prefer named checked operations whose signatures require a rounding policy when precision can be discarded.

## Checked arithmetic

All authoritative arithmetic is checked:

```rust
pub enum NumericError {
    Overflow,
    DivisionByZero,
    InvalidConversion,
    OutOfDomain,
}

pub enum Rounding {
    Floor,
    Ceil,
    TowardZero,
    AwayFromZero,
    NearestTiesAway,
    NearestTiesEven,
}
```

The numeric module maps each domain rounding mode to library operations or a tested integer implementation. It is the only module allowed to know the backing scale.

- Addition, subtraction, negation, multiplication, division, rescaling, and integer conversion must detect overflow.
- Overflow or division by zero becomes a typed deterministic simulation fault; it must never wrap silently.
- Saturation is not a general error strategy. Use explicit clamping only where a combat rule defines a bound, such as current HP within `[0, MaxHp]`.
- Conversion from a negative value to an unsigned/domain-positive type is checked.
- Intermediate formula values must not be rounded merely for UI display.
- No authoritative path uses `f32`, `f64`, platform math libraries, `fast-math`, or an approximate comparison epsilon.

An independent `f64` implementation may exist under tests/tools as a formula research oracle. Its results never enter `BattleState`, events, configuration bundles, replays, or golden hashes.

## Formula finalization

Every formula declares its rounding boundary and mode. Until reproducible observations establish more exact behavior, use these project policies:

| Result | Internal arithmetic | State-changing finalization |
|---|---|---|
| Ordinary, DoT, Break, Super Break, and true damage | Fixed-point through all documented multiplier stages | Floor once after the complete formula, minimum/cap rules afterward. |
| Healing | Fixed-point through base amount and outgoing/incoming modifiers | Floor once before applying HP bounds. |
| Shield creation | Fixed-point through base amount and shield modifiers | Floor once before creating the shield instance. |
| Percentage Energy/resource gain | Fixed-point against the authored maximum | Apply the ability's declared rounding mode; default Floor. |
| Action Gauge and Speed | Remain fixed-point state | Never convert through displayed Action Value. |
| Effect probability | Convert clamped fixed-point probability to an integer threshold | No floating random sample. |

If game observations require rounding at an intermediate stage, represent that stage explicitly in the formula program and add a golden test. Do not scatter ad hoc `.floor()` calls through character handlers.

## Action ordering

Action Gauge, Speed, advances, and delays use fixed-point values. Displayed Action Value is derived presentation data.

The scheduler orders simultaneous work with a total key equivalent to:

```text
(priority, phase, formation_side, formation_index,
 spawn_sequence, source_unit_id, action_id, insertion_sequence)
```

The exact fields used by each queue must be documented, but every comparison must have a final stable integer tie-breaker. No result may depend on address order, `HashMap` iteration, Bevy entity iteration, or thread scheduling.

## Deterministic RNG

Pin the authoritative implementation to `rand = "=0.10.2"` with `default-features = false` and features `std` plus `chacha`. Use `rand::rngs::ChaCha8Rng`; do not enable `thread_rng`, `sys_rng`, `unbiased`, or SIMD-dependent sampling behavior in an authoritative crate. The initial replay identifier is:

```text
rng_algorithm_revision = "chacha8-rand-0.10.2-intmap-v1"
```

The identifier covers both the generator and project-owned integer sampling. Do not call generic range, weighted-distribution, shuffle, or floating-probability helpers whose draw count or mapping could change between dependency versions. Implement unsigned rejection sampling and cumulative integer-weight selection once, with golden vectors. All sampling uses integers.

Each RNG draw has:

- monotonically increasing draw index;
- purpose tag such as `Crit`, `Debuff`, `BounceTarget`, or `AggroTarget`;
- candidates in stable ID/order sequence;
- integer weights or integer probability threshold;
- raw sample and selected result in diagnostic traces.

Specify whether an unsuccessful/no-candidate operation consumes a draw. Changing the algorithm, integer mapping, candidate ordering, or draw-consumption policy increments `rng_algorithm_revision`.

One master activity seed derives independent streams through SHA-256 over a canonical tuple containing `rng_algorithm_revision`, seed, activity/profile ID, activity instance ID, section/node/attempt/battle sequence, and an ASCII stream label. At minimum, use separate `graph`, `reward`, `shop`, `spawn`, and per-battle `battle` streams when those systems are present. Mode aliases such as run ID are metadata, not a different derivation algorithm. The first 32 digest bytes seed ChaCha8 directly. Consequently, adding a reward roll cannot shift later combat CRIT rolls. Never clone a live stream to create a substream.

## Collections and execution model

- `HashMap`/`HashSet` may support lookup but their iteration order is never authoritative.
- Before emitting operations or events from an unordered collection, sort by a stable domain key.
- Do not use `usize` in serialized or hashed authoritative state; convert collection indexes to explicit fixed-width IDs.
- One battle is logically single-threaded. Many isolated battles may run concurrently.
- One activity command is logically single-threaded. Logically forked nodes/battles may execute concurrently only from isolated snapshots/substreams and must merge in stable branch order.
- Parallel calculation inside one battle may be introduced only if it produces an identical predeclared operation order and uses no shared RNG; the default is to avoid it.
- A headless verifier may share an immutable validated catalog between worker
  jobs, but battle state, RNG, scratch buffers, journals and queues are isolated
  per job. Thread scheduling never selects command or operation order.

## Command atomicity and faults

`Battle::apply` validates command legality before mutation. Once resolution starts, one of two policies must complete deterministically:

- the command commits its ordered operations/events; or
- an internal limit/numeric/invariant fault transitions the battle to `Faulted` with a stable fault event and diagnostic context.

Returning an error while leaving undocumented partial state is forbidden. Use a mutation journal, transactional delta, or an explicitly committed fault state. Presentation adapters cannot retry an ambiguous command.

`Activity::apply` obeys the same contract for graph, roster, resource, clock, metric, checkpoint, and submitted-result mutations. A rejected `BattleResult` leaves the activity byte-identical; an internal activity fault follows a versioned rollback/commit-fault policy.

Hard limits are authored rules-revision constants. Battle limits cover events, reactions, trigger depth, extra actions, hits/bounces, effects, and linked actors. Activity limits cover commands, graph visits/loops, fork branches, options, participants, inventory entries, spawn count, checkpoints, and projected metrics.

## Canonical state hash

After every accepted command, compute or make available a canonical SHA-256 state digest. `state_hash_revision = "sha256-v1"` identifies both the algorithm and the canonical byte layout. Its byte stream uses:

- explicit field order and version;
- fixed-width integers in a declared byte order;
- entities/effects/actions sorted by stable IDs and sequence keys;
- raw fixed-point integers, never formatted decimal strings;
- RNG state/draw index, timeline state, wave state, team resources, units, effects, shields, marks, summons, and countdowns;
- the config bundle SHA-256, rules revision, numeric policy revision, and RNG algorithm revision.

Exclude caches, allocation capacity, pointers, logs, presentation state, wall-clock timestamps, and engine entity IDs. Do not use Rust's default `Hasher`, raw struct memory, or derived serialization whose field/version stability is not controlled by the replay format.

The canonical encoder targets a byte sink. Production state hashing streams the
same canonical fields directly into SHA-256 without materializing a full state
byte vector; golden tests may direct the encoder to a collecting sink and must
prove byte-for-byte equivalence. Buffer reuse, chunk size and hasher update
boundaries are non-authoritative implementation details.

`sha256-v1` is intentionally a full-state digest after each accepted command.
Caching encoded immutable definition bodies is unnecessary because the catalog
is represented by its digest. Incremental field hashes, Merkle roots or dirty
page hashing require a new documented `state_hash_revision` unless they are
only an internal accelerator that demonstrably emits the exact existing
canonical byte stream and final SHA-256 value.

The same `state_hash_revision` family defines a canonical activity digest after every accepted activity command. It includes definition/config digests, graph position/visits, scoped slots, participants and loadout locks, inventories/modifiers, clocks, metrics/objectives, RNG streams, pending options, pending `BattleSpec`, checkpoints, and completed `BattleResult` digests. It excludes calendar schedules, account rewards, UI state, and battle caches. The exact activity layout is a separate codec section so battle-only layout changes do not silently reinterpret activity bytes.

## Excel and Sora boundary

Do not route authoritative Excel decimals through `f32`/`f64` before fixed-point conversion. Store fixed-point source values as one of:

1. canonical decimal strings such as `"0.25"`, parsed and validated by `starclock-data`; or
2. explicitly scaled integers such as `250000` for a six-decimal ratio.

Canonical decimal strings are preferred for designer-facing ratios. Parsing rules are:

- optional leading minus only where the field domain permits it;
- ASCII digits and at most one decimal point;
- no exponent notation, locale separators, NaN, or infinity;
- at most six fractional digits unless that field defines another representation;
- checked range and domain validation;
- canonical debug output without loss of raw value.

Sora performs structural validation; `starclock-data` loads `config.sora`, performs decimal parsing and domain validation, and builds immutable combat/activity definitions containing raw fixed-point values. Runtime battles and activities consume only those validated definitions.

## Version pinning

Record these independently:

```text
sora_cli_version
config_bundle_sha256
coverage_manifest_sha256
rules_revision
numeric_policy_revision
rng_algorithm_revision
state_hash_revision
replay_format_version
```

Pin `fixnum` exactly in the workspace manifest and commit `Cargo.lock`. A library upgrade requires explicit review of arithmetic/rounding behavior and cross-platform golden replay results. Do not treat a patch update as semantically invisible.

## Cross-platform verification

The release gate runs the same bundle, seed, and battle/activity command fixtures on at least:

- Windows x86-64;
- Linux x86-64;
- macOS ARM64 when supported by CI;
- any additional shipping target such as WebAssembly before that target is claimed compatible.

Compare the event bytes or hashes and the canonical state hash after every command, not only the final winner. Add boundary tests for positive/negative rounding, exact halves, minimum/maximum values, multiplication overflow, division by zero, probability endpoints, exact Action Gauge ties, and long simulations that could accumulate precision loss.

The first implementation milestone should also compare the fixed-point calculator to the test-only `f64` oracle over generated legal inputs. Differences outside the declared fixed resolution require investigation, not a widened arbitrary epsilon.
