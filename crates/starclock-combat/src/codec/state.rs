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
    e.u32(state.encounter.definition.get());
    e.u64(state.encounter.wave.get());
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
    }
    e.u8(unit.side as u8);
    e.u8(unit.formation.get());
    e.u8(unit.level.get());
    e.u8(unit.life as u8);
    e.u8(unit.presence as u8);
    e.i64(unit.current_hp.get());
    e.i64(unit.maximum_hp.get());
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
