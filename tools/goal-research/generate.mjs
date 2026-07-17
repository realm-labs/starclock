import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const PACK_SHA = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const MANIFEST_SHA = "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19";
const SCHEMA = "starclock-goal-research-v1";
const GENERATED_ON = "2026-07-17";
const root = path.resolve(process.cwd());
const refRoot = path.join(root, "content-reference", "v4.4");
const outputRoot = path.join(root, "evidence", "core-combat-v1", "research-register");
const checkOnly = process.argv.includes("--check");

const pack = readJson(path.join(refRoot, "pack-index.json"));
const manifest = readJson(path.join(root, "content-manifests", "core-combat-v1", "manifest-index.json"));
assert(pack.pack_sha256 === PACK_SHA, "reference pack digest mismatch");
assert(manifest.manifest_sha256 === MANIFEST_SHA, "goal manifest digest mismatch");
const abilities = readJson(path.join(refRoot, "character-abilities.json"));
const abilityById = new Map(abilities.map((row) => [row.id, row]));

const sources = [
  {
    id: "source.dimbreath.4.4",
    url: "https://gitlab.com/Dimbreath/turnbasedgamedata/-/commit/fd978d6ef09f941fba644c731ab54abd6f7c3568",
    accessed_on: GENERATED_ON,
    version: "Released data visible at Version 4.4 snapshot",
    confidence: "PreparedExactStructured",
    evidence_sha256: PACK_SHA,
    note: "Pinned structured baseline. Individual case bindings use normalized record text hashes and the full-pack regeneration proof; no descriptions are copied here.",
  },
  {
    id: "source.hoyolab.version-4.4",
    url: "https://www.hoyolab.com/article/45851903?reply=1",
    accessed_on: GENERATED_ON,
    version: "Version 4.4 update details",
    confidence: "OfficialVersionAndReleaseBoundary",
    evidence_sha256: "e9a64eabc9c657487fcc0b240a76d8dbfaaae095fa04b523649532e70f320557",
    note: "Raw HTTP response hash recorded for release/path/element corroboration only; it is not treated as a complete executable kit specification.",
  },
  {
    id: "source.honey-hunter.himeko-nova.4.4",
    url: "https://starrail.honeyhunterworld.com/himeko-nova-character/?lang=EN",
    accessed_on: GENERATED_ON,
    version: "Public Version 4.4 page as accessed",
    confidence: "SecondaryVersionSensitiveCrossCheck",
    evidence_sha256: "65af44357bf47ea4104a749dd3ebb239005fbdba13ee1a9332b609415193beb0",
    note: "Used only to frame reproducible Assist Skill and protocol observations. Prepared pinned records remain the factual baseline and unresolved ordering stays Researching.",
  },
];

const definitions = [];
function add(definition) {
  definitions.push({
    state: "Researching",
    confidence: "PreparedContractPendingExecutableGolden",
    source_ids: ["source.dimbreath.4.4"],
    ...definition,
  });
}

add({
  id: "G01-R-ASTA-UNIQUE-TARGET-CREDIT", family: "V1aAsta", owner_batch: "G01-P4-B2",
  record_ids: ["character.asta.ability.astrometry.skillp01", "character.asta.ability.meteor-storm.bpskill"],
  question: "At which hit boundary are distinct-target and Fire-weakness Charging credits sampled, and how are repeat bounces coalesced?",
  fixed_expectations: ["Credit is per distinct target within one action.", "Weakness is sampled at the qualifying hit.", "Repeated hits on one target do not create another distinct-target credit."],
  observations_required: ["Ordered event boundary between hit damage, weakness query and Charging mutation.", "Credit behavior when a target gains or loses Fire weakness between bounce hits."],
  fixture: actionFixture("asta-unique-target-credit", ["Use a seeded multi-bounce Skill against three stable slots, with one repeated target and one Fire-weak target."], ["Assert the ordered target set and one credit per distinct target.", "Repeat after changing one weakness immediately before its hit."]),
});
add({
  id: "G01-R-ASTA-AURA-STACK-QUERY", family: "V1aAsta", owner_batch: "G01-P4-B2",
  record_ids: ["character.asta.ability.astrometry.skillp01"],
  question: "Does the party ATK modifier query current Charging stacks dynamically at each stat query without replacing its effect instance?",
  fixed_expectations: ["Charging owns one bounded stack resource.", "The team aura value follows current stacks rather than a stale application snapshot."],
  observations_required: ["Stat-query result immediately before and after stack mutation in the same command."],
  fixture: actionFixture("asta-aura-stack-query", ["Create one aura instance, then mutate Charging through gain and decay boundaries."], ["Assert one modifier instance remains and its queried value follows the current stack count."]),
});
add({
  id: "G01-R-ASTA-INDEPENDENT-CLOCKS", family: "V1aAsta", owner_batch: "G01-P4-B5",
  record_ids: ["character.asta.ability.astrometry.skillp01", "character.asta.ability.astral-blessing.ultra"],
  question: "Which owner-turn boundary decays Charging, and how is it ordered relative to the independent Ultimate SPD duration clock?",
  fixed_expectations: ["Charging decay is owned by Asta's turn boundary.", "The SPD effect has its own authored duration and is not coupled to Charging."],
  observations_required: ["Before/after-turn ordering when decay and SPD expiry become eligible together."],
  fixture: turnFixture("asta-independent-clocks", ["Arrange Charging decay and SPD expiry on the same Asta turn boundary."], ["Assert separate effect clocks and stable boundary ordering."]),
});

add({
  id: "G01-R-KAFKA-DOT-SELECTION-ATTRIBUTION", family: "V1aKafka", owner_batch: "G01-P4-B5",
  record_ids: ["character.kafka.ability.caressing-moonlight.bpskill", "character.kafka.ability.twilight-trill.ultra"],
  question: "Which target-local DoT instances qualify for each authored detonation, and which actor/source receives the resulting damage credit?",
  fixed_expectations: ["Only qualifying DoTs on the selected target are detonated.", "Each detonation retains the original DoT applier, source, element and instance identity."],
  observations_required: ["Eligibility differences across ordinary, Break and special DoTs.", "Cause-chain actor/source fields for mixed-applier DoTs."],
  fixture: actionFixture("kafka-dot-selection-attribution", ["Apply four differently classified DoTs from two appliers to the primary target and one DoT to an adjacent target."], ["Assert exact selected instance IDs and unchanged attribution; assert the adjacent-only DoT is not selected by primary-target detonation."]),
});
add({
  id: "G01-R-KAFKA-DETONATION-SNAPSHOT-DURATION", family: "V1aKafka", owner_batch: "G01-P4-B5",
  record_ids: ["character.kafka.ability.caressing-moonlight.bpskill", "character.kafka.ability.twilight-trill.ultra"],
  question: "At what snapshot boundary is detonated DoT damage sampled, and does detonation alter remaining duration or stacks?",
  fixed_expectations: ["Detonation preserves the DoT's declared snapshot policy.", "Detonation does not consume, refresh or resnapshot duration unless an authored patch says so."],
  observations_required: ["Source/target stat changes between application and detonation.", "Remaining duration and stacks before and after detonation."],
  fixture: actionFixture("kafka-dot-snapshot-duration", ["Apply a DoT, change source and target modifiers, then detonate before its normal tick."], ["Assert captured versus dynamic fields exactly as declared and byte-identical duration/stack state absent a patch."]),
});
add({
  id: "G01-R-KAFKA-FOLLOWUP-ONCE-ORDER", family: "V1aKafka", owner_batch: "G01-P4-B5",
  record_ids: ["character.kafka.ability.gentle-but-cruel.skillp01"],
  question: "Which Kafka-turn scope gates the ally-Basic follow-up, and in what order do its damage and Shock application resolve?",
  fixed_expectations: ["The follow-up is once per Kafka turn, not once per ally or action.", "The follow-up preserves the triggering ally Basic in its cause chain."],
  observations_required: ["Damage-versus-Shock application order.", "Gate reset across Kafka extra turns and ordinary turns."],
  fixture: turnFixture("kafka-followup-once-order", ["Have two allies use Basics before Kafka's next ordinary turn, then repeat around an extra-turn boundary."], ["Assert only one follow-up in each authored Kafka-turn scope and capture damage/effect event order."]),
});

add({
  id: "G01-R-CLARA-CAUSE-ATTACKER-ALLY", family: "V1aClara", owner_batch: "G01-P4-B6",
  record_ids: ["character.clara.ability.because-were-family.skillp01"],
  question: "How are enemy attacker, attacked ally, Clara owner and Svarog damage actor retained across counter scheduling?",
  fixed_expectations: ["The incoming attack cause contains attacker and attacked ally.", "Counter ownership remains Clara/Svarog while targeting the original attacker."],
  observations_required: ["Actor/owner/source fields on each queued and resolved counter event."],
  fixture: reactionFixture("clara-cause-attacker-ally", ["Attack Clara, then a protected ally, from distinct enemy slots."], ["Assert counter target and complete cause ancestry for both cases."]),
});
add({
  id: "G01-R-CLARA-COUNTER-TARGET-PRIORITY", family: "V1aClara", owner_batch: "G01-P4-B6",
  record_ids: ["character.clara.ability.because-were-family.skillp01"],
  question: "What deterministic invalidation and priority policy applies if the original attacker becomes illegal before Svarog's counter resolves?",
  fixed_expectations: ["The counter is a queued reaction with explicit priority.", "There is no implicit random or nearest-target fallback."],
  observations_required: ["Cancel/retarget behavior after attacker defeat, departure and phase replacement."],
  fixture: reactionFixture("clara-counter-target-priority", ["Queue a counter, then invalidate the attacker before reaction resolution under three presence transitions."], ["Capture cancel/retarget result and prove no unspecified RNG draw."]),
});
add({
  id: "G01-R-CLARA-ENHANCED-CHARGE-SCOPE", family: "V1aClara", owner_batch: "G01-P4-B6",
  record_ids: ["character.clara.ability.promise-not-command.ultra", "character.clara.ability.because-were-family.skillp01"],
  question: "How are enhanced counter charges shared and consumed when multiple allies are attacked in one enemy action?",
  fixed_expectations: ["Enhanced counter charges are one Clara-owned shared resource.", "An attack on any ally may qualify while charges remain."],
  observations_required: ["Consumption point and coalescing behavior for AoE/multi-target attacks."],
  fixture: reactionFixture("clara-enhanced-charge-scope", ["With one remaining enhanced charge, resolve a stable-slot enemy AoE that hits all allies."], ["Assert the exact qualifying event, one charge mutation and one reaction insertion."]),
});
add({
  id: "G01-R-CLARA-MARK-CONSUME-ORDER", family: "V1aClara", owner_batch: "G01-P4-B6",
  record_ids: ["character.clara.ability.svarog-watches-over-you.bpskill"],
  question: "At what per-target phase does Clara's Skill sample and consume each Svarog mark?",
  fixed_expectations: ["Each target's mark is sampled for that target's damage.", "Consumption occurs only after that target's Skill damage."],
  observations_required: ["Order under multi-target defeat and mark-triggered reactions."],
  fixture: actionFixture("clara-mark-consume-order", ["Use Skill against three marked enemies where the first is defeated and the second has an on-hit reaction."], ["Assert each damage sample sees its own mark and each mark removal follows its target damage."]),
});

add({
  id: "G01-R-FIREFLY-HP-ENERGY-ATOMIC", family: "V1aFirefly", owner_batch: "G01-P4-B3",
  record_ids: ["character.firefly.ability.order-aerial-bombardment.bpskill", "character.firefly.ability.chrysalid-pyronexus.skillp01"],
  question: "How are Skill HP consumption, legal HP floor and Energy restoration committed when replacement or defeat interception is eligible?",
  fixed_expectations: ["HP cost and Energy restoration resolve as one checked authored program.", "Invalid cost cannot partially mutate authoritative state."],
  observations_required: ["Effective HP cost and Energy gain at minimum HP and with cost modifiers.", "Replacement-trigger visibility between the two operations."],
  fixture: actionFixture("firefly-hp-energy-atomic", ["Use normal Skill at ordinary HP, at the legal floor and with a pre-defeat replacement candidate."], ["Assert accepted/rejected transaction state, ordered HP/Energy events and no partial mutation."]),
});
add({
  id: "G01-R-FIREFLY-TRANSFORM-ADVANCE", family: "V1aFirefly", owner_batch: "G01-P4-B3",
  record_ids: ["character.firefly.ability.fyrefly-type-iv-complete-combustion.ultra", "character.firefly.ability.chrysalid-pyronexus.skillp01"],
  question: "At which phases do Complete Combustion's stat modifiers, ability replacements, action advance and countdown actor become visible?",
  fixed_expectations: ["Transformation owns explicit replacement/modifier instances.", "Action advance and countdown creation are resolver operations, not scheduler exceptions."],
  observations_required: ["Visibility order to immediate reactions and legal-decision queries."],
  fixture: actionFixture("firefly-transform-advance", ["Use Ultimate with Firefly behind two actors and with an interrupt eligible at each phase."], ["Assert ordered transform, replacement, modifier, countdown and action-gauge events."]),
});
add({
  id: "G01-R-FIREFLY-WEAKNESS-TOUGHNESS-ORDER", family: "V1aFirefly", owner_batch: "G01-P4-B4",
  record_ids: ["character.firefly.ability.fyrefly-type-iv-deathstar-overload.bpskill"],
  question: "Does enhanced Skill implant Fire weakness before every hit's Toughness eligibility query, including the first hit?",
  fixed_expectations: ["Fire weakness application precedes the enhanced Skill Toughness operation.", "Weakness and RES remain separately authored concepts."],
  observations_required: ["First-hit weakness/Toughness event ordering and expiry scope."],
  fixture: actionFixture("firefly-weakness-toughness-order", ["Target one enemy without Fire weakness and one with Fire resistance but no weakness."], ["Assert weakness mutation precedes first Toughness query and does not silently alter RES."]),
});
add({
  id: "G01-R-FIREFLY-SUPERBREAK-SAMPLE", family: "V1aFirefly", owner_batch: "G01-P4-B4",
  record_ids: ["character.firefly.ability.fyrefly-type-iv-deathstar-overload.bpskill"],
  question: "Which attempted/effective Toughness value and hit boundary feed Firefly's authored Super Break instances?",
  fixed_expectations: ["Super Break uses its dedicated formula family and an explicitly selected Toughness sample.", "It does not inherit ordinary DMG Boost or CRIT."],
  observations_required: ["Overkill Toughness reduction, zero remaining Toughness and layered/Exo-Toughness samples."],
  fixture: actionFixture("firefly-superbreak-sample", ["Run the same enhanced Skill hit against positive, nearly depleted, broken and layered Toughness states."], ["Record attempted/effective per-layer reduction and assert the selected Super Break input."]),
});
add({
  id: "G01-R-FIREFLY-COUNTDOWN-TEARDOWN", family: "V1aFirefly", owner_batch: "G01-P4-B7",
  record_ids: ["character.firefly.ability.fyrefly-type-iv-complete-combustion.ultra"],
  question: "Which teardown operations run when the countdown acts, a wave changes, or Firefly is downed/revived?",
  fixed_expectations: ["The ordinary ability set is restored and transform-owned state is removed exactly once.", "No orphan countdown actor survives teardown."],
  observations_required: ["Carry/reset behavior across AfterAction wave transition, defeat interception and revival."],
  fixture: boundaryFixture("firefly-countdown-teardown", ["End transformation independently through countdown, wave boundary and downed/revival paths."], ["Assert one teardown cause, ordinary abilities restored and no transform-owned actor/modifier remains."]),
});

add({
  id: "G01-R-AGLAEA-PRESENCE-TIMELINE-OWNERSHIP", family: "V1aAglaea", owner_batch: "G01-P4-B7",
  record_ids: ["character.aglaea.ability.rise-exalted-renown.bpskill", "character.aglaea.ability.rosy-fingered.skillp01"],
  question: "How do Garmentmaker's independent life, presence, timeline and owner link change on summon, heal and resummon?",
  fixed_expectations: ["Garmentmaker is an independently identified linked actor.", "Summoning versus healing is chosen from explicit presence state."],
  observations_required: ["Resummon behavior from downed, defeated, departed and already-present states."],
  fixture: boundaryFixture("aglaea-presence-timeline-ownership", ["Use Skill once in each supported Garmentmaker presence/life state."], ["Assert actor identity preservation/replacement, timeline entry and owner-link state."]),
});
add({
  id: "G01-R-AGLAEA-SPD-STACK-SCOPE", family: "V1aAglaea", owner_batch: "G01-P4-B7",
  record_ids: ["character.aglaea.ability.rosy-fingered.skillp01"],
  question: "At what attack boundary do summon SPD stacks accumulate, cap and reset, and when does the new SPD rescale action gauge?",
  fixed_expectations: ["SPD stacks belong to Garmentmaker, not Aglaea.", "Cap and reset are authored summon-state policies."],
  observations_required: ["Gauge rescaling boundary after stack gain and teardown/reset behavior."],
  fixture: turnFixture("aglaea-spd-stack-scope", ["Let Garmentmaker attack through cap, then transform and teardown while mid-gauge."], ["Assert stack owner/cap, gauge rescale policy and exact reset boundary."]),
});
add({
  id: "G01-R-AGLAEA-JOINT-ACTION-CONTRIBUTIONS", family: "V1aAglaea", owner_batch: "G01-P4-B7",
  record_ids: ["character.aglaea.ability.slash-by-a-thousandfold-kiss.normal"],
  question: "Which action envelope and cause chain contain Aglaea and Garmentmaker's separate joint-attack contributions?",
  fixed_expectations: ["A joint attack is one action envelope.", "Each contribution retains its own damage actor/source and formula context."],
  observations_required: ["Trigger eligibility and once-scope behavior at action, contribution and hit boundaries."],
  fixture: actionFixture("aglaea-joint-action-contributions", ["Execute the enhanced Basic with triggers filtered by action, actor, summon and hit."], ["Assert one action envelope, ordered contributions and distinct attribution without duplicate once-per-action credit."]),
});
add({
  id: "G01-R-AGLAEA-COUNTDOWN-TEARDOWN", family: "V1aAglaea", owner_batch: "G01-P4-B7",
  record_ids: ["character.aglaea.ability.dance-destined-weaveress.ultra", "character.aglaea.ability.rosy-fingered.skillp01"],
  question: "What is the exact once-only ordering of empowered-state countdown, summon teardown damage/resource changes and ability restoration?",
  fixed_expectations: ["The state ends through an explicit countdown actor.", "Every teardown operation executes exactly once under one cause."],
  observations_required: ["Damage, Energy/resource, departure and ability-restoration event order."],
  fixture: boundaryFixture("aglaea-countdown-teardown", ["Let the countdown act normally, then repeat with simultaneous summon defeat and wave boundary."], ["Assert one teardown program and a stable complete event order."]),
});

const elationRecords = {
  silver: ["character.silver-wolf-lv-999.ability.honkai-dmg-demo.elationdamage", "character.silver-wolf-lv-999.ability.pro-gamer-move.elationdamage", "character.silver-wolf-lv-999.ability.i-carry-we-win.skillp01"],
  trailblazer: ["character.trailblazer.elation.ability.i-said-elation-did-i-stutter.elationdamage", "character.trailblazer.elation.ability.may-the-trailblaze-fly-you-starward.ultra", "character.trailblazer.elation.ability.that-smile-hits-different.skillp01"],
  sparxie: ["character.sparxie.ability.signal-overflow-the-great-encore.elationdamage", "character.sparxie.ability.sleight-of-sparx-hand.skillp01"],
  yao: ["character.yao-guang.ability.let-thy-fortune-burst-in-flames.elationdamage", "character.yao-guang.ability.hexagram-of-feathered-fortune.ultra", "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01"],
};
add({
  id: "G01-R-ELATION-DAMAGE-ABILITY-TAGS", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.silver, ...elationRecords.trailblazer, ...elationRecords.sparxie, ...elationRecords.yao],
  question: "How are Elation ability identity and Elation damage category represented independently across released kits?",
  fixed_expectations: ["Elation damage is not ordinary additional or follow-up damage.", "Ability tags and emitted damage-class tags are separate queryable fields."],
  observations_required: ["Cross-kit trigger matrix for ordinary abilities that emit Elation damage and explicit Elation Skills."],
  fixture: actionFixture("elation-damage-ability-tags", ["Execute one ordinary ability with Elation emission and one explicit Elation Skill from at least Silver Wolf LV.999, Trailblazer (Elation), Sparxie and Yao Guang."], ["Assert independent ability/damage tags and a negative control using visually similar ordinary damage."]),
});
add({
  id: "G01-R-ELATION-PUNCHLINE-SCOPE-CREDIT", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.trailblazer, ...elationRecords.yao],
  question: "Is Punchline a team subsystem resource, which actor receives each credit, and when do spend/threshold triggers observe it?",
  fixed_expectations: ["Punchline has explicit team/actor ownership and one canonical mutation event.", "Threshold checks observe ordered effective resource changes."],
  observations_required: ["Simultaneous cross-kit credit ordering, caps/overflow and wave persistence."],
  fixture: actionFixture("elation-punchline-scope-credit", ["Trigger Punchline gains from two released forms in the same reaction chain, including an over-cap gain."], ["Assert owner, effective delta, threshold order and no duplicate mirrored mutation."]),
});
add({
  id: "G01-R-ELATION-CERTIFIED-BANGER-OWNERSHIP", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.silver, ...elationRecords.trailblazer, ...elationRecords.yao],
  question: "Is Certified Banger held by a unit, team subsystem or shared actor, and how do grant/replace/consume operations interact across providers?",
  fixed_expectations: ["The holder and provider remain distinct identities.", "Grant/replace/consume policy is authored rather than inferred from path."],
  observations_required: ["Cross-provider replacement and teardown at transformation/departure/wave boundaries."],
  fixture: boundaryFixture("elation-certified-banger-ownership", ["Grant the state from two providers, transform one holder and cross a wave boundary."], ["Assert explicit holder/provider identities, one active-policy result and authored teardown."]),
});
add({
  id: "G01-R-ELATION-FORCED-SKILL-ENVELOPE", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: elationRecords.trailblazer,
  question: "Does Trailblazer's forced ally Elation Skill create an extra action, forced ability envelope or nested program, and which triggers/once-scopes see it?",
  fixed_expectations: ["The forced use retains Trailblazer as cause/provider and the ally as ability actor.", "It is not an ordinary action advance."],
  observations_required: ["Turn consumption, gauge movement, action-start/end events and Ultimate/follow-up trigger eligibility."],
  fixture: reactionFixture("elation-forced-skill-envelope", ["Target an Elation ally and a non-Elation ally with the same Ultimate under identical gauge state."], ["Assert forced-skill versus action-advance branches, cause ownership and turn/gauge effects."]),
});
add({
  id: "G01-R-ELATION-FORCED-SKILL-TARGET-COST", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.trailblazer, ...elationRecords.sparxie],
  question: "Which target, cost, decision and invalidation policies apply to a forced Elation Skill?",
  fixed_expectations: ["Cost suppression/substitution and target choice are explicit forced-use arguments.", "Invalid target handling consumes no undeclared RNG or resource."],
  observations_required: ["Manual-target versus authored automatic-target behavior, insufficient substitute resource and target death."],
  fixture: reactionFixture("elation-forced-skill-target-cost", ["Force an Elation Skill with multiple targets, insufficient ordinary SP, substitute resource present/absent and target invalidation."], ["Assert target program, cost source, decision exposure and deterministic cancellation/retargeting."]),
});
add({
  id: "G01-R-ELATION-SHARED-ACTOR-AHA", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: elationRecords.yao,
  question: "What actor, owner, resource contribution and timeline semantics apply when Yao Guang grants Aha an extra turn?",
  fixed_expectations: ["Aha is a shared subsystem actor, not a hidden Yao Guang action.", "Its resource contribution and provider cause are explicit."],
  observations_required: ["Timeline identity, targeting, extra-turn priority, ownership after Yao Guang departure and concurrent providers."],
  fixture: turnFixture("elation-shared-actor-aha", ["Grant Aha an extra turn with Yao Guang present, departed and alongside another Elation provider."], ["Assert stable shared-actor identity, provider cause, timeline priority and resource sample."]),
});
add({
  id: "G01-R-ELATION-SP-SPEND-OBSERVATION", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.silver, ...elationRecords.sparxie, ...elationRecords.yao],
  question: "Which attempted/effective Skill Point spend event triggers shared Elation effects when substitute or suppressed costs are used?",
  fixed_expectations: ["Attempted cost, actual payer and effective spend are distinct event fields.", "Each rule declares which field it inspects."],
  observations_required: ["Ordinary SP, zero-cost, suppressed-cost and substitute-resource matrix across released forms."],
  fixture: actionFixture("elation-sp-spend-observation", ["Execute qualifying abilities under ordinary, zero, suppressed and substitute cost paths."], ["Assert one cost event with payer/source/effective delta and the exact cross-kit trigger matrix."]),
});
add({
  id: "G01-R-ELATION-GENERIC-API-BOUNDARY", family: "SharedElation", owner_batch: "G01-P4-B8", record_ids: [...elationRecords.silver, ...elationRecords.trailblazer, ...elationRecords.sparxie, ...elationRecords.yao],
  question: "Can the shared subsystem express all probe cases using generic tags, slots, actor links and operations without content-form IDs?",
  fixed_expectations: ["No core API or resolver branch names a released form.", "Kit-specific programs remain data or use a reviewed static handler only after an IR-insufficiency decision."],
  observations_required: ["Compilation audit of four released-form probes and negative one-kit API scan."],
  fixture: catalogFixture("elation-generic-api-boundary", ["Compile minimum probe rows for four released Elation forms from the dedicated Sora probe scope."], ["Assert shared domain shapes, no character-ID branch and equivalent event envelopes for common semantics."]),
});

const himekoApproximations = abilities.filter((row) => row.character_id === "character.himeko-nova" && row.mechanism_quality === "ApproximateFromReleasedText").sort((a, b) => a.id.localeCompare(b.id));
for (const row of himekoApproximations) {
  add({
    id: `G01-R-HIMEKO-NOVA-${slug(row.name_en)}`,
    family: "HimekoNovaApproximation",
    owner_batch: "G01-P7-M01",
    dependent_batch: "G01-P7-C04",
    confidence: "ReleasedTextBoundApproximation",
    source_ids: ["source.dimbreath.4.4", "source.hoyolab.version-4.4", "source.honey-hunter.himeko-nova.4.4"],
    record_ids: [row.id],
    question: `What exact target program, ordered operations and timing envelope implement ${row.name_en} (source skill ${row.source_skill_ids.join("/")})?`,
    fixed_expectations: [`Prepared numeric/level fields remain exact structured inputs.`, `The released-text SHA-256 remains ${row.source_text.sha256}.`, "No missing configuration operation is borrowed from a similarly named ability."],
    observations_required: ["Target selection with one, three and five living enemies and all supported ally/companion users.", "Actor/owner/source/cause identities for Assist Skill and protocol-triggered use.", "Cost/use-counter, threshold, target-invalidation and action/turn-consumption behavior."],
    fixture: observationFixture(`himeko-nova-${slug(row.name_en)}`, row.id),
  });
}

definitions.sort((a, b) => a.id.localeCompare(b.id));
const resolutions = new Map([
  ["G01-R-ASTA-AURA-STACK-QUERY", {
    state: "Observed",
    confidence: "Observed",
    observation: {
      accessed_on: GENERATED_ON,
      source_payload_sha256: "eca2d92a18987e4bd41ccdc5b307a858e03e819d2317b1825da22a7e65cc2ace",
      executable_bundle_sha256: "63b138645278e74e9836eafec09a9637aa9d91b05603a44160b0506e2874faed",
      result: "One ReplaceByCaster aura instance reads the current Charging slot; level-10 ATK ratio changes at 0/1/5/3 stacks without instance replacement.",
      evidence_paths: ["config/probes/v1a/asta-modifier/golden.json", "crates/starclock-data/src/probe_tests.rs"],
      validation_commands: ["node tools/config-probes/verify-asta-modifier.mjs", "cargo test -p starclock-data probe_tests"],
    },
  }],
  ["G01-R-ASTA-UNIQUE-TARGET-CREDIT", {
    state: "Observed",
    confidence: "Observed",
    observation: {
      accessed_on: GENERATED_ON,
      source_payload_sha256: "eca2d92a18987e4bd41ccdc5b307a858e03e819d2317b1825da22a7e65cc2ace",
      executable_bundle_sha256: "63b138645278e74e9836eafec09a9637aa9d91b05603a44160b0506e2874faed",
      result: "A seeded four-hit sequence against three stable targets yields credits 1/2/0/1; repeats are coalesced and Fire weakness is sampled for the current hit.",
      evidence_paths: ["config/probes/v1a/asta-modifier/golden.json", "crates/starclock-data/src/probe_tests.rs"],
      validation_commands: ["node tools/config-probes/verify-asta-modifier.mjs", "cargo test -p starclock-data probe_tests"],
    },
  }],
  ["G01-R-FIREFLY-HP-ENERGY-ATOMIC", {
    state: "Observed",
    confidence: "Observed",
    observation: {
      accessed_on: GENERATED_ON,
      source_payload_sha256: "edd2cf12b2944f2be234c77a6e77da9e162bda384b45123083c3f1df2b0fc19c",
      executable_bundle_sha256: "260363d9edbcb8046f403ba676d24d37e9e4b6c22d86c12ac7e8ac6258372b2b",
      result: "The normal Skill prepares ordered ConsumeHp, ModifyEnergy and Damage emissions: 40% Max HP with a one-HP floor, 60% Max Energy gain and level-10 200% ATK Fire damage. Invalid preparation leaves the caller's state unchanged.",
      evidence_paths: ["config/probes/v1a/firefly-damage/golden.json", "crates/starclock-data/src/probe_tests.rs"],
      validation_commands: ["node tools/config-probes/verify-firefly-damage.mjs", "cargo test -p starclock-data probe_tests"],
    },
  }],
  ["G01-R-FIREFLY-TRANSFORM-ADVANCE", {
    state: "Observed",
    confidence: "Observed",
    observation: {
      accessed_on: GENERATED_ON,
      source_payload_sha256: "edd2cf12b2944f2be234c77a6e77da9e162bda384b45123083c3f1df2b0fc19c",
      executable_bundle_sha256: "260363d9edbcb8046f403ba676d24d37e9e4b6c22d86c12ac7e8ac6258372b2b",
      result: "The Ultimate program makes the countdown creation and RedMode effect visible before full action advance, then resets Energy; the four operations retain one ordered typed Rule IR program.",
      evidence_paths: ["config/probes/v1a/firefly-damage/golden.json", "crates/starclock-data/src/probe_tests.rs"],
      validation_commands: ["node tools/config-probes/verify-firefly-damage.mjs", "cargo test -p starclock-data probe_tests"],
    },
  }],
]);
const cases = definitions.map((definition) => ({
  ...definition,
  ...(resolutions.get(definition.id) ?? {}),
  evidence: definition.record_ids.map(bindEvidence),
}));
const fixtures = cases.map((entry) => ({
  case_id: entry.id,
  fixture_id: entry.fixture.id,
  kind: entry.fixture.kind,
  initial_conditions: entry.fixture.initial_conditions,
  stimulus: entry.fixture.stimulus,
  assertions: entry.fixture.assertions,
  observations_required: entry.observations_required,
  replay_requirements: ["Fixed seed and stable formation slots.", "Canonical accepted-command stream and ordered event/cause capture.", "Pre/post authoritative state hash and RNG draw count.", "Exact reference-pack, rules and fixture revision in the replay header."],
  completion_rule: "Replace Researching only after the named observation is source-bound and its executable golden passes through the production data-to-domain boundary.",
  state: entry.state === "Observed" ? "GoldenVerified" : "PendingObservation",
  executable_evidence: entry.observation?.evidence_paths ?? [],
}));

const decisions = [
  decision("G01-D-P0-B3-01", "No convenient default for hidden timing", "Any target, ordering, snapshot, cost or invalidation fact not fixed by prepared structured evidence remains Researching until the named observation and golden fixture agree."),
  decision("G01-D-P0-B3-02", "Probe isolation", "V1a fixtures compile from a dedicated Excel/Sora probe scope, remain outside production catalogs and never count toward DataReady coverage."),
  decision("G01-D-P0-B3-03", "Ownership is first-class", "Every probe preserves provider, owner, actor, applier, payer, target and cause ancestry as separate fields; shared/summoned actors are never collapsed into their character owner."),
  decision("G01-D-P0-B3-04", "Shared Elation boundary", "Elation damage class, Elation ability tag, Punchline, Certified Banger, forced ability use and shared actors compile through generic tags/slots/links; form IDs are forbidden in core APIs and resolver branches."),
  decision("G01-D-P0-B3-05", "Himeko Nova Assist prerequisite", "Register G01-P7-M01 before G01-P7-C04 to implement the generic Assist Skill/shared-use/companion-protocol mechanism only after all ten source-bound approximations have resolved target and timing fixtures."),
  decision("G01-D-P0-B3-06", "Reproducible observation envelope", "Every live observation records source URL/access date/version/confidence/hash, exact build and encounter inputs, accepted commands, seed, RNG draws, ordered events and pre/post hashes; video-only impressions do not close a case."),
];

const familyCounts = countBy(cases, (entry) => entry.family);
const ownerCounts = countBy(cases, (entry) => entry.owner_batch);
const register = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  reference_pack_sha256: PACK_SHA,
  goal_manifest_sha256: MANIFEST_SHA,
  case_count: cases.length,
  state_counts: countBy(cases, (entry) => entry.state),
  family_counts: familyCounts,
  owner_counts: ownerCounts,
  cases: cases.map(({ fixture: _fixture, ...entry }) => entry),
};
const fixtureReport = { schema_revision: SCHEMA, generated_on: GENERATED_ON, fixture_count: fixtures.length, fixtures };
const sourceRegister = { schema_revision: SCHEMA, generated_on: GENERATED_ON, sources };
const decisionRecords = { schema_revision: SCHEMA, generated_on: GENERATED_ON, decisions };
const outputs = {
  "decision-records.json": decisionRecords,
  "fixture-specifications.json": fixtureReport,
  "research-cases.json": register,
  "source-register.json": sourceRegister,
};
const indexFiles = Object.entries(outputs).sort(([a], [b]) => a.localeCompare(b)).map(([name, value]) => ({ name, sha256: sha256Text(formatJson(value)) }));
const index = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  files: indexFiles,
  evidence_sha256: sha256Text(indexFiles.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("")),
};
outputs["evidence-index.json"] = index;

if (checkOnly) {
  for (const [name, value] of Object.entries(outputs)) {
    const file = path.join(outputRoot, name);
    assert(fs.existsSync(file), `missing generated ${name}`);
    assert(fs.readFileSync(file, "utf8") === formatJson(value), `${name} has generated drift`);
  }
  console.log(`Goal research register is current (${index.evidence_sha256}; ${cases.length} cases).`);
} else {
  fs.mkdirSync(outputRoot, { recursive: true });
  for (const [name, value] of Object.entries(outputs)) fs.writeFileSync(path.join(outputRoot, name), formatJson(value));
  console.log(`Wrote ${Object.keys(outputs).length} research evidence files (${index.evidence_sha256}; ${cases.length} cases).`);
}

function bindEvidence(recordId) {
  const row = abilityById.get(recordId);
  assert(row, `missing ability record ${recordId}`);
  assert(/^[0-9a-f]{64}$/.test(row.source_text.sha256), `case record ${recordId} lacks released-text evidence`);
  return {
    record_file: "character-abilities.json",
    record_id: row.id,
    quality: row.quality,
    mechanism_quality: row.mechanism_quality,
    source_skill_ids: row.source_skill_ids,
    source_text_sha256: row.source_text.sha256,
    source_file_ids: row.source_ability_files,
  };
}
function actionFixture(id, initial, assertions) { return fixture(id, "ActionGolden", initial, ["Submit the named legal ability command and drain its complete action/reaction envelope."], assertions); }
function turnFixture(id, initial, assertions) { return fixture(id, "TurnBoundaryGolden", initial, ["Advance only through explicit legal commands until the named owner-turn boundaries complete."], assertions); }
function reactionFixture(id, initial, assertions) { return fixture(id, "ReactionGolden", initial, ["Submit the trigger command, then drain the deterministic reaction queue without presentation timing inputs."], assertions); }
function boundaryFixture(id, initial, assertions) { return fixture(id, "LifecycleBoundaryGolden", initial, ["Reach each named boundary through accepted commands and drain allowed boundary reactions."], assertions); }
function catalogFixture(id, initial, assertions) { return fixture(id, "CatalogArchitectureGolden", initial, ["Validate, export, load and compile the dedicated probe bundle twice from clean inputs."], assertions); }
function observationFixture(id, recordId) {
  return fixture(id, "LiveObservationThenGolden", ["Use public Version 4.4 Himeko Nova at a recorded level/Trace/Eidolon state.", "Record one-, three- and five-enemy formations plus supported Trailblaze Companion and ordinary ally Assist users."], [`Invoke only ${recordId} through documented legal paths; repeat after target invalidation and at each relevant counter threshold.`], ["Bind the ordered observation transcript and hashes before upgrading mechanism quality.", "Reproduce the transcript in a dedicated Sora probe fixture before production import."]);
}
function fixture(id, kind, initial_conditions, stimulus, assertions) { return { id: `fixture.${id}`, kind, initial_conditions, stimulus, assertions }; }
function decision(id, title, text) { return { id, state: "AcceptedProjectPolicy", title, decision: text, effective_batch: "G01-P0-B3" }; }
function countBy(rows, key) { const out = {}; for (const row of rows) { const value = key(row); out[value] = (out[value] ?? 0) + 1; } return Object.fromEntries(Object.entries(out).sort(([a], [b]) => a.localeCompare(b))); }
function slug(value) { return value.toLowerCase().normalize("NFKD").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").toUpperCase(); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function formatJson(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
