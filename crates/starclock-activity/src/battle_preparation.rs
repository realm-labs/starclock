use std::sync::Arc;

use sha2::{Digest, Sha256};
use starclock_combat::{
    AssemblyDigest, BattleSpec, BattleSpecDigest, CombatInputDigest, ParticipantSource, TeamSide,
};

use crate::{
    ActivityGraphDefinition, ActivityInstanceId, ActivityOptionId, ActivityScope,
    ActivityScopePath, ActivityTransactionState, BattleBinding, BattleSequence,
    EncounterPreparationDigest, LoadoutLockScope, NodeId, ParticipantId, ParticipantLock,
    ParticipantLockDigest, TechniqueContributionDigest, codec::ActivityStateEncoder,
};

pub const MAX_PREPARATION_TECHNIQUES: usize = 8;
pub const MAX_PREPARED_BATTLE_VARIANTS: usize = 512;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EncounterInitiativePolicy {
    PlayerControlled = 0,
    EnemyPreemptive = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TechniqueEngagement {
    Accumulate = 0,
    Engage = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TechniqueOptionDefinition {
    option: ActivityOptionId,
    participant: ParticipantId,
    point_cost: u16,
    engagement: TechniqueEngagement,
}

impl TechniqueOptionDefinition {
    #[must_use]
    pub const fn new(
        option: ActivityOptionId,
        participant: ParticipantId,
        point_cost: u16,
        engagement: TechniqueEngagement,
    ) -> Option<Self> {
        if point_cost == 0 {
            None
        } else {
            Some(Self {
                option,
                participant,
                point_cost,
                engagement,
            })
        }
    }
    #[must_use]
    pub const fn option(self) -> ActivityOptionId {
        self.option
    }
    #[must_use]
    pub const fn participant(self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn point_cost(self) -> u16 {
        self.point_cost
    }
    #[must_use]
    pub const fn engagement(self) -> TechniqueEngagement {
        self.engagement
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedBattleVariant {
    techniques: Box<[ActivityOptionId]>,
    contribution: TechniqueContributionDigest,
    binding: Arc<BattleBinding>,
}

impl PreparedBattleVariant {
    #[must_use]
    pub fn new(
        techniques: Vec<ActivityOptionId>,
        contribution: TechniqueContributionDigest,
        binding: BattleBinding,
    ) -> Self {
        Self {
            techniques: techniques.into_boxed_slice(),
            contribution,
            binding: Arc::new(binding),
        }
    }
    #[must_use]
    pub fn techniques(&self) -> &[ActivityOptionId] {
        &self.techniques
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> TechniqueContributionDigest {
        self.contribution
    }
    #[must_use]
    pub fn battle_spec(&self) -> &BattleSpec {
        self.binding.battle_spec()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterPreparationDefinition {
    normal_engagement: ActivityOptionId,
    initiative: EncounterInitiativePolicy,
    participant_lock: ParticipantLockDigest,
    team_index: u8,
    techniques: Box<[TechniqueOptionDefinition]>,
    variants: Box<[PreparedBattleVariant]>,
    digest: EncounterPreparationDigest,
}

impl EncounterPreparationDefinition {
    pub fn new(
        normal_engagement: ActivityOptionId,
        initiative: EncounterInitiativePolicy,
        participant_lock: ParticipantLockDigest,
        team_index: u8,
        mut techniques: Vec<TechniqueOptionDefinition>,
        mut variants: Vec<PreparedBattleVariant>,
    ) -> Result<Self, EncounterPreparationDefinitionError> {
        if techniques.len() > MAX_PREPARATION_TECHNIQUES {
            return Err(EncounterPreparationDefinitionError::TooManyTechniques);
        }
        if variants.is_empty() || variants.len() > MAX_PREPARED_BATTLE_VARIANTS {
            return Err(EncounterPreparationDefinitionError::InvalidVariantCount);
        }
        techniques.sort_by_key(|item| item.option);
        if techniques
            .windows(2)
            .any(|pair| pair[0].option == pair[1].option)
            || techniques
                .iter()
                .any(|item| item.option == normal_engagement)
        {
            return Err(EncounterPreparationDefinitionError::DuplicateOption);
        }
        if techniques.iter().enumerate().any(|(index, item)| {
            techniques[..index]
                .iter()
                .any(|prior| prior.participant == item.participant)
        }) {
            return Err(EncounterPreparationDefinitionError::DuplicateParticipant);
        }
        if initiative == EncounterInitiativePolicy::EnemyPreemptive && !techniques.is_empty() {
            return Err(EncounterPreparationDefinitionError::PreemptiveOffersTechnique);
        }
        variants.sort_by(|left, right| left.techniques.cmp(&right.techniques));
        validate_variants(participant_lock, &techniques, &variants)?;
        let digest = preparation_digest(
            normal_engagement,
            initiative,
            participant_lock,
            team_index,
            &techniques,
            &variants,
        );
        Ok(Self {
            normal_engagement,
            initiative,
            participant_lock,
            team_index,
            techniques: techniques.into_boxed_slice(),
            variants: variants.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn normal_engagement(&self) -> ActivityOptionId {
        self.normal_engagement
    }
    #[must_use]
    pub const fn initiative(&self) -> EncounterInitiativePolicy {
        self.initiative
    }
    #[must_use]
    pub const fn participant_lock_digest(&self) -> ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub const fn team_index(&self) -> u8 {
        self.team_index
    }
    #[must_use]
    pub fn techniques(&self) -> &[TechniqueOptionDefinition] {
        &self.techniques
    }
    #[must_use]
    pub fn variants(&self) -> &[PreparedBattleVariant] {
        &self.variants
    }
    #[must_use]
    pub const fn digest(&self) -> EncounterPreparationDigest {
        self.digest
    }

    fn variant(&self, techniques: &[ActivityOptionId]) -> Option<&PreparedBattleVariant> {
        self.variants
            .binary_search_by(|item| item.techniques.as_ref().cmp(techniques))
            .ok()
            .map(|index| &self.variants[index])
    }

    fn technique(&self, option: ActivityOptionId) -> Option<TechniqueOptionDefinition> {
        self.techniques
            .binary_search_by_key(&option, |item| item.option)
            .ok()
            .map(|index| self.techniques[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterPreparationDefinitionError {
    TooManyTechniques,
    InvalidVariantCount,
    DuplicateOption,
    DuplicateParticipant,
    PreemptiveOffersTechnique,
    MissingEmptyVariant,
    DuplicateVariant,
    UnknownTechnique,
    UnreachableTechnique,
    DuplicateTechniqueInSequence,
    SequenceAfterEngagement,
    MissingPrefixVariant,
    ParticipantLockMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRosterLock {
    boundary: ActivityScopePath,
    participants: Arc<ParticipantLock>,
}

impl ActivityRosterLock {
    pub fn new(
        boundary: ActivityScopePath,
        participants: ParticipantLock,
    ) -> Result<Self, ActivityRosterLockError> {
        if boundary.active_scope() != lock_scope(participants.policy().loadout_lock_scope()) {
            return Err(ActivityRosterLockError::BoundaryMismatch);
        }
        Ok(Self {
            boundary,
            participants: Arc::new(participants),
        })
    }
    #[must_use]
    pub const fn boundary(&self) -> ActivityScopePath {
        self.boundary
    }
    #[must_use]
    pub fn participants(&self) -> &ParticipantLock {
        &self.participants
    }
    #[must_use]
    pub fn digest(&self) -> ParticipantLockDigest {
        self.participants.digest()
    }

    fn covers(&self, path: ActivityScopePath) -> bool {
        self.boundary.activity() == path.activity()
            && self
                .boundary
                .section()
                .is_none_or(|value| path.section() == Some(value))
            && self
                .boundary
                .node()
                .is_none_or(|value| path.node() == Some(value))
            && self
                .boundary
                .attempt()
                .is_none_or(|value| path.attempt() == Some(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityRosterLockError {
    BoundaryMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityPreparationOptionKind {
    NormalEngagement,
    Technique(TechniqueEngagement),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityPreparationOptionView {
    id: ActivityOptionId,
    kind: ActivityPreparationOptionKind,
    participant: Option<ParticipantId>,
    point_cost: u16,
}

impl ActivityPreparationOptionView {
    #[must_use]
    pub const fn id(self) -> ActivityOptionId {
        self.id
    }
    #[must_use]
    pub const fn kind(self) -> ActivityPreparationOptionKind {
        self.kind
    }
    #[must_use]
    pub const fn participant(self) -> Option<ParticipantId> {
        self.participant
    }
    #[must_use]
    pub const fn point_cost(self) -> u16 {
        self.point_cost
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityPreparationView {
    initial_points: u16,
    remaining_points: u16,
    selected: Box<[ActivityOptionId]>,
    options: Box<[ActivityPreparationOptionView]>,
}

impl ActivityPreparationView {
    #[must_use]
    pub const fn initial_points(&self) -> u16 {
        self.initial_points
    }
    #[must_use]
    pub const fn remaining_points(&self) -> u16 {
        self.remaining_points
    }
    #[must_use]
    pub fn selected(&self) -> &[ActivityOptionId] {
        &self.selected
    }
    #[must_use]
    pub fn options(&self) -> &[ActivityPreparationOptionView] {
        &self.options
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingBattleSpec {
    path: ActivityScopePath,
    battle_sequence: BattleSequence,
    participant_lock: ParticipantLockDigest,
    initiative: EncounterInitiativePolicy,
    techniques: Box<[ActivityOptionId]>,
    contribution: TechniqueContributionDigest,
    remaining_points: u16,
    binding: Arc<BattleBinding>,
}

impl PendingBattleSpec {
    #[must_use]
    pub const fn path(&self) -> ActivityScopePath {
        self.path
    }
    #[must_use]
    pub const fn battle_sequence(&self) -> BattleSequence {
        self.battle_sequence
    }
    #[must_use]
    pub const fn participant_lock_digest(&self) -> ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub const fn initiative(&self) -> EncounterInitiativePolicy {
        self.initiative
    }
    #[must_use]
    pub fn techniques(&self) -> &[ActivityOptionId] {
        &self.techniques
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> TechniqueContributionDigest {
        self.contribution
    }
    #[must_use]
    pub const fn remaining_technique_points(&self) -> u16 {
        self.remaining_points
    }
    #[must_use]
    pub fn battle_spec(&self) -> &BattleSpec {
        self.binding.battle_spec()
    }
    #[must_use]
    pub fn battle_spec_digest(&self) -> BattleSpecDigest {
        self.binding.battle_spec().digest()
    }
    #[must_use]
    pub fn combat_input_digest(&self) -> CombatInputDigest {
        self.binding.battle_spec().combat_input_digest()
    }
    #[must_use]
    pub fn assembly_digest(&self) -> AssemblyDigest {
        self.binding.battle_spec().assembly_digest()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityPendingBattleView {
    battle_sequence: BattleSequence,
    combat_input_digest: CombatInputDigest,
    assembly_digest: AssemblyDigest,
    participant_lock: ParticipantLockDigest,
    initiative: EncounterInitiativePolicy,
    techniques: Box<[ActivityOptionId]>,
    remaining_points: u16,
}

impl ActivityPendingBattleView {
    #[must_use]
    pub const fn battle_sequence(&self) -> BattleSequence {
        self.battle_sequence
    }
    #[must_use]
    pub fn battle_spec_digest(&self) -> BattleSpecDigest {
        BattleSpecDigest::new(self.assembly_digest.bytes())
            .expect("assembly identities are non-zero")
    }
    #[must_use]
    pub const fn combat_input_digest(&self) -> CombatInputDigest {
        self.combat_input_digest
    }
    #[must_use]
    pub const fn assembly_digest(&self) -> AssemblyDigest {
        self.assembly_digest
    }
    #[must_use]
    pub const fn participant_lock_digest(&self) -> ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub const fn initiative(&self) -> EncounterInitiativePolicy {
        self.initiative
    }
    #[must_use]
    pub fn techniques(&self) -> &[ActivityOptionId] {
        &self.techniques
    }
    #[must_use]
    pub const fn remaining_technique_points(&self) -> u16 {
        self.remaining_points
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityPreparationBoundary {
    Decision,
    BattleReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityPreparationError {
    AttemptAlreadyActive,
    MissingAttempt,
    ScopeMismatch,
    RosterBoundaryMismatch,
    ParticipantLockMismatch,
    TeamOutsideLock,
    TechniqueParticipantOutsideTeam,
    BattleParticipantMismatch,
    DecisionNotOffered,
    TechniquePointsInsufficient,
    MissingPreparedVariant,
    BattleAlreadyPending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattlePreparationRequest {
    path: ActivityScopePath,
    roster: ActivityRosterLock,
    battle_sequence: BattleSequence,
    technique_points: u16,
    definition: Arc<EncounterPreparationDefinition>,
}

impl ActivityBattlePreparationRequest {
    #[must_use]
    pub fn new(
        path: ActivityScopePath,
        roster: ActivityRosterLock,
        battle_sequence: BattleSequence,
        technique_points: u16,
        definition: Arc<EncounterPreparationDefinition>,
    ) -> Self {
        Self {
            path,
            roster,
            battle_sequence,
            technique_points,
            definition,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActivityAttemptState {
    path: ActivityScopePath,
    roster: ActivityRosterLock,
    battle_sequence: BattleSequence,
    initial_points: u16,
    remaining_points: u16,
    selected: Vec<ActivityOptionId>,
    definition: Arc<EncounterPreparationDefinition>,
    pending: Option<PendingBattleSpec>,
    settled: bool,
}

impl ActivityAttemptState {
    pub(crate) fn begin(
        instance: ActivityInstanceId,
        current_node: NodeId,
        graph: &ActivityGraphDefinition,
        request: ActivityBattlePreparationRequest,
    ) -> Result<(Self, ActivityPreparationBoundary), ActivityPreparationError> {
        let ActivityBattlePreparationRequest {
            path,
            roster,
            battle_sequence,
            technique_points,
            definition,
        } = request;
        let attempt = path
            .attempt()
            .ok_or(ActivityPreparationError::MissingAttempt)?;
        let node = graph
            .node(current_node)
            .ok_or(ActivityPreparationError::ScopeMismatch)?;
        if path.activity() != instance
            || path.node() != Some(current_node)
            || path.section() != Some(node.section())
        {
            return Err(ActivityPreparationError::ScopeMismatch);
        }
        if !roster.covers(path) {
            return Err(ActivityPreparationError::RosterBoundaryMismatch);
        }
        debug_assert_eq!(path.attempt(), Some(attempt));
        validate_roster(&roster, &definition)?;
        let mut state = Self {
            path,
            roster,
            battle_sequence,
            initial_points: technique_points,
            remaining_points: technique_points,
            selected: Vec::new(),
            definition,
            pending: None,
            settled: false,
        };
        if state.definition.initiative == EncounterInitiativePolicy::EnemyPreemptive {
            state.prepare_pending()?;
            Ok((state, ActivityPreparationBoundary::BattleReady))
        } else {
            Ok((state, ActivityPreparationBoundary::Decision))
        }
    }

    pub(crate) fn apply_option(
        &mut self,
        option: ActivityOptionId,
    ) -> Result<ActivityPreparationBoundary, ActivityPreparationError> {
        if self.pending.is_some() || self.settled {
            return Err(ActivityPreparationError::BattleAlreadyPending);
        }
        if option == self.definition.normal_engagement {
            self.prepare_pending()?;
            return Ok(ActivityPreparationBoundary::BattleReady);
        }
        let technique = self
            .definition
            .technique(option)
            .ok_or(ActivityPreparationError::DecisionNotOffered)?;
        if self.selected.contains(&option) {
            return Err(ActivityPreparationError::DecisionNotOffered);
        }
        if technique.point_cost > self.remaining_points {
            return Err(ActivityPreparationError::TechniquePointsInsufficient);
        }
        let mut next = self.selected.clone();
        next.push(option);
        if self.definition.variant(&next).is_none() {
            return Err(ActivityPreparationError::DecisionNotOffered);
        }
        self.remaining_points -= technique.point_cost;
        self.selected = next;
        if technique.engagement == TechniqueEngagement::Engage {
            self.prepare_pending()?;
            Ok(ActivityPreparationBoundary::BattleReady)
        } else {
            Ok(ActivityPreparationBoundary::Decision)
        }
    }

    pub(crate) fn view(&self) -> Option<ActivityPreparationView> {
        (!self.settled && self.pending.is_none()).then(|| ActivityPreparationView {
            initial_points: self.initial_points,
            remaining_points: self.remaining_points,
            selected: self.selected.clone().into_boxed_slice(),
            options: self.offered_options(),
        })
    }

    pub(crate) const fn pending(&self) -> Option<&PendingBattleSpec> {
        self.pending.as_ref()
    }

    pub(crate) fn pending_view(&self) -> Option<ActivityPendingBattleView> {
        self.pending
            .as_ref()
            .map(|pending| ActivityPendingBattleView {
                battle_sequence: pending.battle_sequence,
                combat_input_digest: pending.combat_input_digest(),
                assembly_digest: pending.assembly_digest(),
                participant_lock: pending.participant_lock,
                initiative: pending.initiative,
                techniques: pending.techniques.clone(),
                remaining_points: pending.remaining_points,
            })
    }

    pub(crate) fn encode(&self, writer: &mut ActivityStateEncoder) {
        writer.u32(self.path.section().expect("attempt has section").get());
        writer.u32(self.path.node().expect("attempt has node").get());
        writer.u32(self.path.attempt().expect("attempt exists").get());
        encode_roster_boundary(writer, self.roster.boundary());
        writer.digest(self.roster.digest().bytes());
        writer.u32(self.battle_sequence.get());
        writer.u32(u32::from(self.initial_points));
        writer.u32(u32::from(self.remaining_points));
        writer.digest(self.definition.digest.bytes());
        writer.u32(self.selected.len() as u32);
        for option in &self.selected {
            writer.u64(option.get());
        }
        writer.bool(self.pending.is_some());
        if let Some(pending) = &self.pending {
            writer.text(starclock_combat::COMBAT_INPUT_CODEC_REVISION);
            writer.digest(pending.combat_input_digest().bytes());
            writer.digest(pending.assembly_digest().bytes());
            writer.digest(pending.contribution.bytes());
            writer.byte(pending.initiative as u8);
        }
        writer.bool(self.settled);
    }

    pub(crate) fn participant_specs(
        &self,
    ) -> Vec<(ParticipantId, &starclock_combat::ParticipantSpec)> {
        let Some(pending) = self.pending.as_ref() else {
            return Vec::new();
        };
        self.roster
            .participants
            .entries()
            .iter()
            .filter(|entry| entry.team_index() == self.definition.team_index)
            .filter_map(|entry| {
                pending
                    .battle_spec()
                    .participants()
                    .iter()
                    .find(|spec| {
                        spec.side() == TeamSide::Player
                            && spec.formation().get() == entry.formation_index()
                    })
                    .map(|spec| (entry.participant(), spec))
            })
            .collect()
    }

    pub(crate) fn mark_settled(&mut self) {
        self.pending = None;
        self.settled = true;
    }

    pub(crate) const fn is_settled(&self) -> bool {
        self.settled
    }

    fn offered_options(&self) -> Box<[ActivityPreparationOptionView]> {
        let mut options = vec![ActivityPreparationOptionView {
            id: self.definition.normal_engagement,
            kind: ActivityPreparationOptionKind::NormalEngagement,
            participant: None,
            point_cost: 0,
        }];
        for technique in self.definition.techniques.iter().copied() {
            if technique.point_cost > self.remaining_points
                || self.selected.contains(&technique.option)
            {
                continue;
            }
            let mut sequence = self.selected.clone();
            sequence.push(technique.option);
            if self.definition.variant(&sequence).is_some() {
                options.push(ActivityPreparationOptionView {
                    id: technique.option,
                    kind: ActivityPreparationOptionKind::Technique(technique.engagement),
                    participant: Some(technique.participant),
                    point_cost: technique.point_cost,
                });
            }
        }
        options.sort_by_key(|item| item.id);
        options.into_boxed_slice()
    }

    fn prepare_pending(&mut self) -> Result<(), ActivityPreparationError> {
        let variant = self
            .definition
            .variant(&self.selected)
            .ok_or(ActivityPreparationError::MissingPreparedVariant)?;
        self.pending = Some(PendingBattleSpec {
            path: self.path,
            battle_sequence: self.battle_sequence,
            participant_lock: self.roster.digest(),
            initiative: self.definition.initiative,
            techniques: self.selected.clone().into_boxed_slice(),
            contribution: variant.contribution,
            remaining_points: self.remaining_points,
            binding: Arc::clone(&variant.binding),
        });
        Ok(())
    }
}

impl ActivityTransactionState {
    pub fn begin_battle_preparation(
        &mut self,
        instance: ActivityInstanceId,
        graph: &ActivityGraphDefinition,
        request: ActivityBattlePreparationRequest,
    ) -> Result<ActivityPreparationBoundary, ActivityPreparationError> {
        if self
            .attempt
            .as_ref()
            .is_some_and(|attempt| !attempt.is_settled())
            || self.awaiting_battle.is_some()
        {
            return Err(ActivityPreparationError::AttemptAlreadyActive);
        }
        if self.terminal().is_some() {
            return Err(ActivityPreparationError::BattleAlreadyPending);
        }
        let (attempt, boundary) =
            ActivityAttemptState::begin(instance, self.current_node(), graph, request)?;
        self.attempt = Some(attempt);
        Ok(boundary)
    }

    pub fn choose_preparation_option(
        &mut self,
        option: ActivityOptionId,
    ) -> Result<ActivityPreparationBoundary, ActivityPreparationError> {
        let mut working = self
            .attempt
            .clone()
            .ok_or(ActivityPreparationError::MissingAttempt)?;
        let boundary = working.apply_option(option)?;
        self.attempt = Some(working);
        Ok(boundary)
    }

    #[must_use]
    pub fn preparation_view(&self) -> Option<ActivityPreparationView> {
        self.attempt.as_ref().and_then(ActivityAttemptState::view)
    }

    #[must_use]
    pub fn pending_battle(&self) -> Option<&PendingBattleSpec> {
        self.attempt
            .as_ref()
            .and_then(ActivityAttemptState::pending)
    }

    pub(crate) fn pending_battle_view(&self) -> Option<ActivityPendingBattleView> {
        self.attempt
            .as_ref()
            .and_then(ActivityAttemptState::pending_view)
    }
}

fn validate_variants(
    participant_lock: ParticipantLockDigest,
    techniques: &[TechniqueOptionDefinition],
    variants: &[PreparedBattleVariant],
) -> Result<(), EncounterPreparationDefinitionError> {
    if !variants.iter().any(|variant| variant.techniques.is_empty()) {
        return Err(EncounterPreparationDefinitionError::MissingEmptyVariant);
    }
    if variants
        .windows(2)
        .any(|pair| pair[0].techniques == pair[1].techniques)
    {
        return Err(EncounterPreparationDefinitionError::DuplicateVariant);
    }
    for variant in variants {
        if variant.binding.participant_lock_digest() != participant_lock {
            return Err(EncounterPreparationDefinitionError::ParticipantLockMismatch);
        }
        for (index, option) in variant.techniques.iter().copied().enumerate() {
            let technique = techniques
                .binary_search_by_key(&option, |item| item.option)
                .ok()
                .map(|found| techniques[found])
                .ok_or(EncounterPreparationDefinitionError::UnknownTechnique)?;
            if variant.techniques[..index].contains(&option) {
                return Err(EncounterPreparationDefinitionError::DuplicateTechniqueInSequence);
            }
            if index + 1 < variant.techniques.len()
                && technique.engagement == TechniqueEngagement::Engage
            {
                return Err(EncounterPreparationDefinitionError::SequenceAfterEngagement);
            }
        }
        if !variant.techniques.is_empty()
            && variants
                .binary_search_by(|candidate| {
                    candidate
                        .techniques
                        .as_ref()
                        .cmp(&variant.techniques[..variant.techniques.len() - 1])
                })
                .is_err()
        {
            return Err(EncounterPreparationDefinitionError::MissingPrefixVariant);
        }
    }
    if techniques.iter().any(|technique| {
        !variants
            .iter()
            .any(|variant| variant.techniques.contains(&technique.option))
    }) {
        return Err(EncounterPreparationDefinitionError::UnreachableTechnique);
    }
    Ok(())
}

fn validate_roster(
    roster: &ActivityRosterLock,
    definition: &EncounterPreparationDefinition,
) -> Result<(), ActivityPreparationError> {
    if roster.digest() != definition.participant_lock {
        return Err(ActivityPreparationError::ParticipantLockMismatch);
    }
    let entries = roster
        .participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() == definition.team_index)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(ActivityPreparationError::TeamOutsideLock);
    }
    if definition.techniques.iter().any(|technique| {
        !entries
            .iter()
            .any(|entry| entry.participant() == technique.participant)
    }) {
        return Err(ActivityPreparationError::TechniqueParticipantOutsideTeam);
    }
    for variant in definition.variants.iter() {
        let players = variant
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .collect::<Vec<_>>();
        if players.len() != entries.len()
            || players
                .iter()
                .zip(entries.iter())
                .any(|(actual, expected)| {
                    actual.source() != ParticipantSource::Player
                        || actual.formation().get() != expected.formation_index()
                        || actual.combatant().form() != expected.character()
                        || actual.locked_combatant_digest()
                            != expected.build().resolved_spec_digest()
                })
        {
            return Err(ActivityPreparationError::BattleParticipantMismatch);
        }
    }
    Ok(())
}

fn preparation_digest(
    normal: ActivityOptionId,
    initiative: EncounterInitiativePolicy,
    lock: ParticipantLockDigest,
    team: u8,
    techniques: &[TechniqueOptionDefinition],
    variants: &[PreparedBattleVariant],
) -> EncounterPreparationDigest {
    let mut hash = Sha256::new();
    hash.update(b"SCAP");
    hash.update(1_u32.to_le_bytes());
    hash.update(normal.get().to_le_bytes());
    hash.update([initiative as u8, team]);
    hash.update(lock.bytes());
    hash.update((techniques.len() as u32).to_le_bytes());
    for technique in techniques {
        hash.update(technique.option.get().to_le_bytes());
        hash.update(technique.participant.get().to_le_bytes());
        hash.update(technique.point_cost.to_le_bytes());
        hash.update([technique.engagement as u8]);
    }
    hash.update((variants.len() as u32).to_le_bytes());
    for variant in variants {
        hash.update((variant.techniques.len() as u32).to_le_bytes());
        for option in variant.techniques.iter() {
            hash.update(option.get().to_le_bytes());
        }
        hash.update(variant.contribution.bytes());
        hash.update(starclock_combat::COMBAT_INPUT_CODEC_REVISION.as_bytes());
        hash.update(variant.binding.battle_spec().combat_input_digest().bytes());
        hash.update(variant.binding.battle_spec().assembly_digest().bytes());
        hash_text(&mut hash, variant.binding.seed_stream_label());
        hash_text(&mut hash, variant.binding.battle_spec_policy_revision());
    }
    EncounterPreparationDigest::new(hash.finalize().into()).expect("SHA-256 output is non-zero")
}

fn hash_text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u32).to_le_bytes());
    hash.update(value.as_bytes());
}

fn encode_roster_boundary(writer: &mut ActivityStateEncoder, path: ActivityScopePath) {
    writer.byte(path.active_scope() as u8);
    if let Some(section) = path.section() {
        writer.u32(section.get());
    }
    if let Some(node) = path.node() {
        writer.u32(node.get());
    }
    if let Some(attempt) = path.attempt() {
        writer.u32(attempt.get());
    }
}

const fn lock_scope(scope: LoadoutLockScope) -> ActivityScope {
    match scope {
        LoadoutLockScope::Activity => ActivityScope::Activity,
        LoadoutLockScope::Section => ActivityScope::Section,
        LoadoutLockScope::Node => ActivityScope::Node,
        LoadoutLockScope::Attempt => ActivityScope::Attempt,
    }
}
