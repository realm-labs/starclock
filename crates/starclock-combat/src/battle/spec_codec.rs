//! Canonical combat-owned encoding for immutable battle input.

use sha2::{Digest, Sha256};

use crate::{
    formula::toughness::EnemyRank,
    rule::model::{RuleSource, SourceClass},
    toughness::model::{BreakCreditPolicy, ToughnessWeaknessPolicy},
};

use super::spec::{
    CombatInputDigest, ConcedePolicy, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide,
};

const INPUT_MAGIC: &[u8; 4] = b"SCBI";
const INPUT_CODEC_VERSION: u16 = 1;

pub(super) fn combat_input_digest(
    rules_revision: &str,
    encounter: crate::EncounterId,
    participants: &[ParticipantSpec],
    player_resources: &TeamResourceSpec,
    enemy_resources: &TeamResourceSpec,
    concede: ConcedePolicy,
) -> CombatInputDigest {
    let mut encoder = Encoder(Sha256::new());
    encoder.raw(INPUT_MAGIC);
    encoder.u16(INPUT_CODEC_VERSION);
    encoder.text(crate::COMBAT_INPUT_CODEC_REVISION);
    encoder.text(rules_revision);
    encoder.u32(encounter.get());
    encoder.length(participants.len());
    for participant in participants {
        encode_participant(&mut encoder, participant);
    }
    encode_team_resources(&mut encoder, player_resources);
    encode_team_resources(&mut encoder, enemy_resources);
    encoder.u8(match concede {
        ConcedePolicy::Allowed => 0,
    });
    CombatInputDigest::from_computed(encoder.0.finalize().into())
}

struct Encoder(Sha256);

impl Encoder {
    fn raw(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    fn u8(&mut self, value: u8) {
        self.raw(&[value]);
    }
    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
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
        self.u64(u64::try_from(value).expect("battle input is bounded below u64::MAX"));
    }
    fn bytes(&mut self, value: &[u8]) {
        self.length(value.len());
        self.raw(value);
    }
    fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }
}

fn encode_participant(encoder: &mut Encoder, participant: &ParticipantSpec) {
    encoder.u8(side_tag(participant.side()));
    encoder.u8(participant.formation().get());
    encoder.u16(participant.wave());
    match participant.source() {
        ParticipantSource::Player => encoder.u8(0),
        ParticipantSource::EncounterEnemy(enemy) => {
            encoder.u8(1);
            encoder.u32(enemy.get());
        }
        ParticipantSource::Linked(source) => {
            encoder.u8(2);
            encoder.u32(source.get());
        }
    }
    encode_combatant(encoder, participant.combatant());
    encoder.raw(&participant.locked_combatant_digest().bytes());
    match participant.initial_state() {
        None => encoder.u8(0),
        Some(state) => {
            encoder.u8(1);
            encoder.i64(state.current_hp().get());
            encoder.i64(state.current_energy().scaled());
            encoder.u8(state.life() as u8);
            encoder.u8(state.presence() as u8);
        }
    }
}

fn encode_combatant(encoder: &mut Encoder, combatant: &ResolvedCombatantSpec) {
    encoder.u32(combatant.form().get());
    encoder.u8(combatant.level().get());
    encoder.i64(combatant.maximum_hp().get());
    encoder.i64(combatant.base_attack().scaled());
    encoder.i64(combatant.base_defense().scaled());
    encoder.i64(combatant.speed().scaled());
    encoder.i64(combatant.current_energy().scaled());
    encoder.i64(combatant.maximum_energy().scaled());
    encoder.u8(match combatant.rank() {
        EnemyRank::Normal => 0,
        EnemyRank::EliteOrBoss => 1,
    });
    encode_ids(encoder, combatant.weaknesses(), |value| *value as u32);
    encoder.length(combatant.toughness_layers().len());
    for layer in combatant.toughness_layers() {
        encoder.u32(layer.key());
        encoder.u8(layer.kind() as u8);
        encoder.i64(layer.maximum().get());
        encoder.bool(layer.active());
        encoder.bool(layer.locked());
        match layer.weakness_policy() {
            ToughnessWeaknessPolicy::MatchingOnly => encoder.u8(0),
            ToughnessWeaknessPolicy::AnyElement => encoder.u8(1),
            ToughnessWeaknessPolicy::OffWeakness(ratio) => {
                encoder.u8(2);
                encoder.i64(ratio.scaled());
            }
        }
        encoder.bool(layer.reducible_while_broken());
        encoder.i64(layer.recovery_ratio().scaled());
        encoder.bool(layer.applies_break_damage());
        encoder.bool(layer.applies_break_effect());
        encoder.bool(layer.changes_global_broken());
        match layer.break_element() {
            None => encoder.u8(0),
            Some(element) => {
                encoder.u8(1);
                encoder.u8(element as u8);
            }
        }
        match layer.break_credit() {
            BreakCreditPolicy::HitApplier => encoder.u8(0),
            BreakCreditPolicy::LayerProvider(source) => {
                encoder.u8(1);
                encoder.u32(source.get());
            }
        }
    }
    encode_ids(encoder, combatant.abilities(), |value| value.get());
    encode_ids(encoder, combatant.rule_bundles(), |value| value.get());
    encode_ids(encoder, combatant.modifiers(), |value| value.get());
    encoder.length(combatant.modifier_bindings().len());
    for binding in combatant.modifier_bindings() {
        encoder.u32(binding.definition().get());
        encoder.u32(binding.source().get());
    }
    encoder.length(combatant.sources().len());
    for source in combatant.sources() {
        encode_source(encoder, source);
    }
    encoder.raw(&combatant.digest().bytes());
}

fn encode_source(encoder: &mut Encoder, source: &RuleSource) {
    encoder.u32(source.definition().get());
    encoder.u8(source_class_tag(source.class()));
    encode_ids(encoder, source.tags(), |value| value.get());
    encoder.raw(&source.digest());
}

fn encode_team_resources(encoder: &mut Encoder, resources: &TeamResourceSpec) {
    encoder.u16(resources.skill_points());
    encoder.u16(resources.maximum_skill_points());
    encoder.length(resources.keyed().len());
    for resource in resources.keyed() {
        encoder.u32(resource.id().get());
        match resource.stable_key() {
            None => encoder.u8(0),
            Some(key) => {
                encoder.u8(1);
                encoder.text(key);
            }
        }
        encoder.u16(resource.initial());
        encoder.u16(resource.maximum());
        encoder.u8(match resource.wave() {
            TeamResourceWavePolicy::Persist => 0,
            TeamResourceWavePolicy::ResetToInitial => 1,
            TeamResourceWavePolicy::Clear => 2,
        });
    }
}

fn encode_ids<T>(encoder: &mut Encoder, values: &[T], raw: impl Fn(&T) -> u32) {
    encoder.length(values.len());
    for value in values {
        encoder.u32(raw(value));
    }
}

const fn side_tag(side: TeamSide) -> u8 {
    match side {
        TeamSide::Player => 0,
        TeamSide::Enemy => 1,
    }
}

const fn source_class_tag(class: SourceClass) -> u8 {
    match class {
        SourceClass::Unit => 0,
        SourceClass::Ability => 1,
        SourceClass::Effect => 2,
        SourceClass::Equipment => 3,
        SourceClass::Progression => 4,
        SourceClass::Enemy => 5,
        SourceClass::Encounter => 6,
        SourceClass::Activity => 7,
        SourceClass::Mode => 8,
        SourceClass::Synthetic => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AbilityId, AssemblyDigest, BattleSpec, CombatantSpecDigest, EncounterId, FormationIndex,
        Hp, ParticipantInitialState, ResolvedDefinitionBindings, Speed, TeamResourceWavePolicy,
        UnitDefinitionId, UnitLevel,
    };

    fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
        ResolvedCombatantSpec::new(
            UnitDefinitionId::new(form).unwrap(),
            UnitLevel::new(80).unwrap(),
            Hp::new(1_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![AbilityId::new(form).unwrap()],
                Vec::new(),
                Vec::new(),
            )
            .unwrap(),
            CombatantSpecDigest::new([digest; 32]).unwrap(),
        )
        .unwrap()
    }

    fn spec(
        revision: &str,
        assembly: u8,
        player: ResolvedCombatantSpec,
        initial: Option<ParticipantInitialState>,
        resources: TeamResourceSpec,
    ) -> BattleSpec {
        let player = match initial {
            Some(state) => ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            )
            .with_initial_state(state)
            .unwrap(),
            None => ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            ),
        };
        BattleSpec::new_with_assembly(
            revision,
            AssemblyDigest::new([assembly; 32]).unwrap(),
            EncounterId::new(1).unwrap(),
            vec![
                player,
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::EncounterEnemy(crate::EnemyDefinitionId::new(1).unwrap()),
                    combatant(2, 2),
                ),
            ],
            resources,
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap()
    }

    #[test]
    fn assembly_provenance_does_not_override_combat_identity() {
        let left = spec(
            "rules-v1",
            1,
            combatant(1, 1),
            None,
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        let right = spec(
            "rules-v1",
            2,
            combatant(1, 1),
            None,
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        assert_eq!(left.combat_input_digest(), right.combat_input_digest());
        assert_ne!(left.assembly_digest(), right.assembly_digest());
    }

    #[test]
    fn canonicalization_makes_participant_order_irrelevant() {
        let assembly = AssemblyDigest::new([7; 32]).unwrap();
        let participant = |side, formation, source, combatant| {
            ParticipantSpec::new(
                side,
                FormationIndex::new(formation).unwrap(),
                source,
                combatant,
            )
        };
        let forward = BattleSpec::new_with_assembly(
            "rules-v1",
            assembly,
            EncounterId::new(1).unwrap(),
            vec![
                participant(
                    TeamSide::Player,
                    0,
                    ParticipantSource::Player,
                    combatant(1, 1),
                ),
                participant(
                    TeamSide::Enemy,
                    0,
                    ParticipantSource::EncounterEnemy(crate::EnemyDefinitionId::new(1).unwrap()),
                    combatant(2, 2),
                ),
            ],
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap();
        let reverse = BattleSpec::new_with_assembly(
            "rules-v1",
            assembly,
            EncounterId::new(1).unwrap(),
            forward.participants().iter().cloned().rev().collect(),
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap();
        assert_eq!(forward.combat_input_digest(), reverse.combat_input_digest());
    }

    #[test]
    fn every_top_level_input_family_changes_identity() {
        let baseline = spec(
            "rules-v1",
            1,
            combatant(1, 1),
            None,
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        let revision = spec(
            "rules-v2",
            1,
            combatant(1, 1),
            None,
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        let changed_combatant = spec(
            "rules-v1",
            1,
            combatant(3, 3),
            None,
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        let carry = ParticipantInitialState::new(
            Hp::new(500).unwrap(),
            Hp::new(1_000).unwrap(),
            crate::Energy::ZERO,
            crate::Energy::ZERO,
            crate::LifeState::Alive,
            crate::PresenceState::Present,
        )
        .unwrap();
        let carry = spec(
            "rules-v1",
            1,
            combatant(1, 1),
            Some(carry),
            TeamResourceSpec::new(3, 5).unwrap(),
        );
        let keyed = crate::KeyedTeamResourceSpec::new(
            crate::SourceDefinitionId::new(1).unwrap(),
            1,
            2,
            TeamResourceWavePolicy::Persist,
        )
        .unwrap()
        .with_stable_key("elation")
        .unwrap();
        let resources = spec(
            "rules-v1",
            1,
            combatant(1, 1),
            None,
            TeamResourceSpec::new(3, 5)
                .unwrap()
                .with_keyed(vec![keyed])
                .unwrap(),
        );
        for changed in [&revision, &changed_combatant, &carry, &resources] {
            assert_ne!(
                baseline.combat_input_digest(),
                changed.combat_input_digest()
            );
        }
    }
}
