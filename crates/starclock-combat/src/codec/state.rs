use sha2::{Digest, Sha256};

use crate::{
    NUMERIC_POLICY_REVISION, STATE_HASH_REVISION,
    actor::store::{TimelineActorState, UnitState},
    battle::{
        spec::{ConcedePolicy, ParticipantSource, TeamSide},
        state::BattleState,
    },
    command::model::{Command, DecisionKind, DecisionOwner, DecisionPoint},
    rng::RNG_ALGORITHM_REVISION,
};

use super::BattleStateHash;

const STATE_MAGIC: &[u8; 4] = b"SCBS";
const STATE_CODEC_VERSION: u16 = 1;

pub(crate) fn hash_state(state: &BattleState) -> BattleStateHash {
    let mut sink = Sha256Sink(Sha256::new());
    encode_state(state, &mut sink);
    BattleStateHash::new(sink.0.finalize().into())
}

#[cfg(feature = "benchmark-instrumentation")]
pub(crate) fn canonical_state_len(state: &BattleState) -> u64 {
    #[derive(Default)]
    struct CountingSink(u64);

    impl Sink for CountingSink {
        fn write(&mut self, bytes: &[u8]) {
            self.0 = self
                .0
                .checked_add(u64::try_from(bytes.len()).expect("slice length fits u64"))
                .expect("canonical battle state is bounded below u64::MAX");
        }
    }

    let mut sink = CountingSink::default();
    encode_state(state, &mut sink);
    sink.0
}

#[cfg(test)]
pub(crate) fn collect_state(state: &BattleState) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_state(state, &mut bytes);
    bytes
}

#[cfg(test)]
pub(crate) fn hash_collected_state(state: &BattleState) -> BattleStateHash {
    BattleStateHash::new(Sha256::digest(collect_state(state)).into())
}

trait Sink {
    fn write(&mut self, bytes: &[u8]);
}

impl Sink for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

struct Sha256Sink(Sha256);

impl Sink for Sha256Sink {
    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

struct Encoder<'a, S>(&'a mut S);

impl<S: Sink> Encoder<'_, S> {
    fn raw(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
    fn u8(&mut self, value: u8) {
        self.raw(&[value]);
    }
    fn u16(&mut self, value: u16) {
        self.raw(&value.to_le_bytes());
    }
    fn u32(&mut self, value: u32) {
        self.raw(&value.to_le_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.raw(&value.to_le_bytes());
    }
    fn i64(&mut self, value: i64) {
        self.raw(&value.to_le_bytes());
    }
    fn length(&mut self, value: usize) {
        self.u64(value as u64);
    }
    fn bytes(&mut self, value: &[u8]) {
        self.length(value.len());
        self.raw(value);
    }
    fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }
}

fn encode_state<S: Sink>(state: &BattleState, sink: &mut S) {
    let mut e = Encoder(sink);
    e.raw(STATE_MAGIC);
    e.u16(STATE_CODEC_VERSION);
    e.text(state.identity.catalog_revision.as_str());
    e.raw(&state.identity.catalog_digest.bytes());
    e.text(&state.identity.rules_revision);
    e.raw(&state.identity.spec_digest.bytes());
    e.text(NUMERIC_POLICY_REVISION);
    e.text(RNG_ALGORITHM_REVISION);
    e.text(STATE_HASH_REVISION);
    e.raw(&state.identity.seed.bytes());
    e.u8(state.phase as u8);
    match state.fault {
        None => e.u8(0),
        Some(fault) => {
            e.u8(1);
            e.u8(fault.kind() as u8);
            e.u8(fault.boundary() as u8);
            e.u8(fault.policy() as u8);
            e.u32(fault.context_code());
            match fault.numeric_context() {
                None => e.u8(0),
                Some(value) => {
                    e.u8(1);
                    e.i64(value);
                }
            }
        }
    }
    encode_decision(&mut e, state.decision.as_ref());
    encode_units(&mut e, state);
    encode_actors(&mut e, state);
    e.length(state.links.canonical_entries().len());
    for link in state.links.canonical_entries() {
        e.u64(link.owner.get());
        match link.entity {
            crate::LinkedEntity::Unit(unit) => {
                e.u8(0);
                e.u64(unit.get());
            }
            crate::LinkedEntity::TimelineActor(actor) => {
                e.u8(1);
                e.u64(actor.get());
            }
        }
        e.u8(link.kind as u8);
        e.u8(link.owner_defeat as u8);
        e.u8(link.owner_departure as u8);
        e.u8(link.wave as u8);
        e.u8(u8::from(link.active));
    }
    e.length(state.formations.canonical_entries().len());
    for entry in state.formations.canonical_entries() {
        e.u8(entry.side as u8);
        e.u8(entry.index.get());
        e.u64(entry.unit.get());
    }
    for side in [TeamSide::Player, TeamSide::Enemy] {
        let team = state.teams.get(side);
        e.u8(team.side as u8);
        e.u16(team.skill_points);
        e.u16(team.maximum_skill_points);
    }
    e.length(state.shields.canonical_entries().len());
    for shield in state.shields.canonical_entries() {
        e.u64(shield.id.get());
        e.u64(shield.owner.get());
        e.u64(shield.source_operation.get());
        e.i64(shield.remaining.get());
        e.u8(match shield.policy {
            crate::formula::shield::ShieldAbsorptionPolicy::ConcurrentLargest => 0,
            crate::formula::shield::ShieldAbsorptionPolicy::AdditiveByInstance => 1,
        });
    }
    e.length(state.break_effects.canonical_entries().len());
    for effect in state.break_effects.canonical_entries() {
        e.u64(effect.id.get());
        e.u64(effect.owner.get());
        e.u64(effect.applier.get());
        e.u64(effect.source_operation.get());
        e.u32(effect.source_definition.get());
        e.u8(effect.plan.element as u8);
        match effect.plan.base_damage {
            None => e.u8(0),
            Some(value) => {
                e.u8(1);
                e.i64(value.scaled());
            }
        }
        e.u8(effect.plan.duration_turns);
        e.u8(effect.remaining_turns);
        e.u8(effect.stacks);
        e.u8(effect.plan.maximum_stacks);
        e.i64(effect.plan.additional_delay.scaled());
        e.i64(effect.plan.speed_reduction.scaled());
        e.u8(u8::from(effect.plan.skips_action));
        match effect.speed_before {
            None => e.u8(0),
            Some(speed) => {
                e.u8(1);
                e.i64(speed.scaled());
            }
        }
        e.i64(effect.damage.attacker_level_multiplier.scaled());
        for factor in [
            effect.damage.ability_multiplier,
            effect.damage.break_effect,
            effect.damage.break_damage_increase,
            effect.damage.defense_multiplier,
            effect.damage.resistance_multiplier,
            effect.damage.vulnerability_multiplier,
            effect.damage.mitigation_multiplier,
            effect.damage.unbroken_multiplier,
        ] {
            e.i64(factor.scaled());
        }
    }
    e.length(state.effects.canonical_entries().len());
    for effect in state.effects.canonical_entries() {
        e.u64(effect.id.get());
        e.u32(effect.definition.get());
        e.u32(effect.source_definition.get());
        e.u64(effect.source_operation.get());
        e.u64(effect.applier.get());
        e.u64(effect.target.get());
        e.u8(effect.category as u8);
        e.u8(effect.dispel as u8);
        e.u16(effect.stacks);
        e.u16(effect.stack_limit);
        match effect.remaining {
            None => e.u8(0),
            Some(value) => {
                e.u8(1);
                e.u16(value);
            }
        }
        e.u8(effect.duration_clock as u8);
        e.u8(effect.tick_phase as u8);
        e.u8(effect.stack_policy as u8);
        e.u8(effect.snapshot_policy as u8);
        e.u8(effect.teardown_policy as u8);
        e.i64(i64::from(effect.application_priority));
        e.i64(effect.magnitude.scaled());
        e.length(effect.tags.len());
        for tag in &effect.tags {
            e.u32(tag.get());
        }
        e.length(effect.controlled_actions.len());
        for action in &effect.controlled_actions {
            e.u8(*action as u8);
        }
        match effect.dot {
            None => e.u8(0),
            Some(dot) => {
                e.u8(1);
                e.u8(dot.element() as u8);
                match dot.detonation_tag() {
                    None => e.u8(0),
                    Some(tag) => {
                        e.u8(1);
                        e.u32(tag.get());
                    }
                }
                e.i64(dot.formula().base_damage().scaled());
                for factor in dot.formula().multipliers().ordered() {
                    e.i64(factor.scaled());
                }
            }
        }
        e.u64(effect.application_sequence);
    }
    e.length(state.rules.iter_by_id().len());
    for instance in state.rules.iter_by_id() {
        e.u64(instance.id.get());
        e.u32(instance.rule.get());
        match instance.owner {
            None => e.u8(0),
            Some(owner) => {
                e.u8(1);
                e.u64(owner.get());
            }
        }
        e.length(instance.slots.len());
        for (definition, value) in &instance.slots {
            e.u32(definition.id().get());
            encode_rule_value(&mut e, value);
        }
        e.length(instance.ledger.canonical_keys().len());
        for key in instance.ledger.canonical_keys() {
            e.u64(key.rule_instance.get());
            e.u32(key.trigger.get());
            e.u8(key.scope as u8);
            e.u64(key.first);
            e.u64(key.second);
        }
    }
    e.u32(state.encounter.definition.get());
    e.u64(state.encounter.wave.get());
    e.u16(state.encounter.number);
    e.u16(state.encounter.total_waves);
    encode_timeline(&mut e, state);
    e.u8(match state.concede {
        ConcedePolicy::Allowed => 0,
    });
    e.raw(&state.rng.seed().bytes());
    e.u64(state.rng.draw_count());
    for next in state.sequences.canonical_next_values() {
        e.u64(next);
    }
    e.u64(state.committed_revision);
}

fn encode_rule_value<S: Sink>(e: &mut Encoder<'_, S>, value: &crate::rule::model::RuleValue) {
    use crate::rule::model::RuleValue as V;
    match value {
        V::Integer(value) => {
            e.u8(0);
            e.i64(*value);
        }
        V::Scalar(value) => {
            e.u8(1);
            e.i64(value.scaled());
        }
        V::Boolean(value) => {
            e.u8(2);
            e.u8(u8::from(*value));
        }
        V::StableId(value) => {
            e.u8(3);
            e.u64(*value);
        }
        V::OptionalStableId(value) => match value {
            None => {
                e.u8(4);
                e.u8(0);
            }
            Some(value) => {
                e.u8(4);
                e.u8(1);
                e.u64(*value);
            }
        },
        V::OrderedStableIdSet(values) => {
            e.u8(5);
            e.length(values.len());
            for value in values {
                e.u64(*value);
            }
        }
    }
}

fn encode_timeline<S: Sink>(e: &mut Encoder<'_, S>, state: &BattleState) {
    match state.timeline.active_turn {
        None => e.u8(0),
        Some(turn) => {
            e.u8(1);
            encode_turn(e, turn);
        }
    }
    match &state.timeline.interrupt {
        None => e.u8(0),
        Some(window) => {
            e.u8(1);
            e.u8(window.kind as u8);
            encode_turn(e, window.turn);
            e.length(window.pending.entries().len());
            for pending in window.pending.entries() {
                e.u8(pending.priority as u8);
                e.u8(pending.side as u8);
                e.u8(pending.formation.get());
                e.u64(pending.spawn.get());
                e.u64(pending.actor.get());
                e.u32(pending.ability.get());
                e.u64(pending.insertion);
            }
        }
    }
}

fn encode_turn<S: Sink>(e: &mut Encoder<'_, S>, turn: crate::timeline::state::NormalTurnState) {
    e.u64(turn.actor.get());
    e.u64(turn.owner.get());
    e.u64(turn.unit.get());
    match turn.automatic {
        None => e.u8(0),
        Some((ability, origin)) => {
            e.u8(1);
            e.u32(ability.get());
            e.u8(origin as u8);
        }
    }
    e.u8(turn.side as u8);
    e.u8(turn.formation.get());
    e.u64(turn.spawn.get());
}

fn encode_units<S: Sink>(e: &mut Encoder<'_, S>, state: &BattleState) {
    let slots = state.units.canonical_slots();
    e.length(slots.len());
    for slot in slots {
        match slot {
            None => e.u8(0),
            Some(unit) => {
                e.u8(1);
                encode_unit(e, unit);
            }
        }
    }
}

fn encode_unit<S: Sink>(e: &mut Encoder<'_, S>, unit: &UnitState) {
    e.u64(unit.id.get());
    e.u64(unit.spawn.get());
    e.u32(unit.form.get());
    match unit.source {
        ParticipantSource::Player => e.u8(0),
        ParticipantSource::EncounterEnemy(enemy) => {
            e.u8(1);
            e.u32(enemy.get());
        }
        ParticipantSource::Linked(source) => {
            e.u8(2);
            e.u32(source.get());
        }
    }
    e.u8(unit.side as u8);
    e.u8(unit.formation.get());
    e.u16(unit.entry_wave);
    e.u8(unit.level.get());
    e.u8(unit.life as u8);
    e.u8(unit.presence as u8);
    e.i64(unit.current_hp.get());
    e.i64(unit.maximum_hp.get());
    e.i64(unit.current_energy.scaled());
    e.i64(unit.maximum_energy.scaled());
    e.u8(match unit.rank {
        crate::formula::toughness::EnemyRank::Normal => 0,
        crate::formula::toughness::EnemyRank::EliteOrBoss => 1,
    });
    e.length(unit.weaknesses.len());
    for weakness in &unit.weaknesses {
        e.u8(*weakness as u8);
    }
    e.length(unit.permanent_weaknesses.len());
    for weakness in &unit.permanent_weaknesses {
        e.u8(*weakness as u8);
    }
    e.length(unit.temporary_weaknesses.len());
    for weakness in &unit.temporary_weaknesses {
        e.u8(weakness.element as u8);
        e.u64(weakness.applier.get());
        e.u64(weakness.source_operation.get());
        e.u8(weakness.remaining_turns);
    }
    e.u8(u8::from(unit.weakness_broken));
    e.length(unit.toughness_layers.len());
    for layer in &unit.toughness_layers {
        let spec = &layer.spec;
        e.u32(spec.key());
        e.u8(spec.kind() as u8);
        e.i64(spec.maximum().get());
        e.i64(layer.current.get());
        e.u8(u8::from(spec.active()));
        e.u8(u8::from(spec.locked()));
        match spec.weakness_policy() {
            crate::ToughnessWeaknessPolicy::MatchingOnly => e.u8(0),
            crate::ToughnessWeaknessPolicy::AnyElement => e.u8(1),
            crate::ToughnessWeaknessPolicy::OffWeakness(value) => {
                e.u8(2);
                e.i64(value.scaled());
            }
        }
        e.u8(u8::from(spec.reducible_while_broken()));
        e.i64(spec.recovery_ratio().scaled());
        e.u8(u8::from(spec.applies_break_damage()));
        e.u8(u8::from(spec.applies_break_effect()));
        e.u8(u8::from(spec.changes_global_broken()));
        match spec.break_element() {
            None => e.u8(0),
            Some(element) => {
                e.u8(1);
                e.u8(element as u8);
            }
        }
        match spec.break_credit() {
            crate::BreakCreditPolicy::HitApplier => e.u8(0),
            crate::BreakCreditPolicy::LayerProvider(source) => {
                e.u8(1);
                e.u32(source.get());
            }
        }
    }
    e.length(unit.abilities.len());
    for id in &unit.abilities {
        e.u32(id.get());
    }
    e.length(unit.rule_bundles.len());
    for id in &unit.rule_bundles {
        e.u32(id.get());
    }
    e.length(unit.modifiers.len());
    for id in &unit.modifiers {
        e.u32(id.get());
    }
    e.raw(&unit.digest.bytes());
    match &unit.transformation {
        None => e.u8(0),
        Some(transform) => {
            e.u8(1);
            e.u64(transform.source_operation.get());
            e.u32(transform.original_form.get());
            e.length(transform.original_abilities.len());
            for ability in &transform.original_abilities {
                e.u32(ability.get());
            }
            e.u8(transform.original_presence as u8);
            match transform.countdown_actor {
                None => e.u8(0),
                Some(actor) => {
                    e.u8(1);
                    e.u64(actor.get());
                }
            }
            e.u8(transform.defeat as u8);
            e.u8(transform.wave as u8);
        }
    }
}

fn encode_actors<S: Sink>(e: &mut Encoder<'_, S>, state: &BattleState) {
    let slots = state.actors.canonical_slots();
    e.length(slots.len());
    for slot in slots {
        match slot {
            None => e.u8(0),
            Some(actor) => {
                e.u8(1);
                encode_actor(e, actor);
            }
        }
    }
}

fn encode_actor<S: Sink>(e: &mut Encoder<'_, S>, actor: &TimelineActorState) {
    e.u64(actor.id.get());
    e.u64(actor.owner.get());
    match actor.unit {
        None => e.u8(0),
        Some(unit) => {
            e.u8(1);
            e.u64(unit.get());
        }
    }
    match actor.kind {
        None => e.u8(0),
        Some(kind) => {
            e.u8(1);
            e.u8(kind as u8);
        }
    }
    match actor.automatic_ability {
        None => e.u8(0),
        Some(ability) => {
            e.u8(1);
            e.u32(ability.get());
        }
    }
    e.u8(u8::from(actor.active));
    e.i64(actor.gauge.scaled());
    e.i64(actor.speed.scaled());
}

fn encode_decision<S: Sink>(e: &mut Encoder<'_, S>, decision: Option<&DecisionPoint>) {
    let Some(decision) = decision else {
        e.u8(0);
        return;
    };
    e.u8(1);
    e.u64(decision.id().get());
    e.u8(match decision.kind() {
        DecisionKind::BattleStart => 0,
        DecisionKind::NormalAction => 1,
        DecisionKind::InterruptWindow => 2,
        DecisionKind::BattleChoice => 3,
    });
    match decision.owner() {
        DecisionOwner::System => e.u8(0),
        DecisionOwner::Team(side) => {
            e.u8(1);
            e.u8(side as u8);
        }
    }
    e.length(decision.legal_commands().len());
    for command in decision.legal_commands() {
        encode_command(e, command);
    }
}

fn encode_command<S: Sink>(e: &mut Encoder<'_, S>, command: &Command) {
    match command {
        Command::StartBattle { decision } => {
            e.u8(0);
            e.u64(decision.get());
        }
        Command::UseAbility {
            decision,
            actor,
            ability,
            primary_target,
        } => {
            e.u8(1);
            encode_action_command(e, *decision, *actor, *ability, *primary_target);
        }
        Command::UseInterrupt {
            decision,
            actor,
            ability,
            primary_target,
        } => {
            e.u8(2);
            encode_action_command(e, *decision, *actor, *ability, *primary_target);
        }
        Command::PassInterruptWindow { decision } => {
            e.u8(3);
            e.u64(decision.get());
        }
        Command::Concede { decision } => {
            e.u8(4);
            e.u64(decision.get());
        }
    }
}

fn encode_action_command<S: Sink>(
    e: &mut Encoder<'_, S>,
    decision: crate::DecisionId,
    actor: crate::UnitId,
    ability: crate::AbilityId,
    primary_target: Option<crate::UnitId>,
) {
    e.u64(decision.get());
    e.u64(actor.get());
    e.u32(ability.get());
    match primary_target {
        None => e.u8(0),
        Some(target) => {
            e.u8(1);
            e.u64(target.get());
        }
    }
}
