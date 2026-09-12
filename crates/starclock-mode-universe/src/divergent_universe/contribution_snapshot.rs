//! Unified immutable battle contribution snapshot for Divergent Universe.

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{ActivityStateHash, GraphActivity, TechniqueContributionDigest};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseDifficultyId,
    divergent_universe_progression_catalog::DivergentUniverseProtocolId,
};

use super::{
    DivergentUniverseBlessingInteractionError, DivergentUniverseBlessingInteractionRuntime,
    DivergentUniverseBlessingInteractionSnapshot, DivergentUniverseCurioRuntime,
    DivergentUniverseCurioRuntimeError, DivergentUniverseCurioSnapshot,
    DivergentUniverseFlowInstance, DivergentUniverseMappingSnapshotDigest,
    DivergentUniversePermanentProgressionRuntime,
    DivergentUniversePermanentProgressionRuntimeError,
    DivergentUniversePermanentProgressionSnapshot, DivergentUniverseProtocolContribution,
    DivergentUniverseProtocolRule, DivergentUniverseRuntimeFactory, DivergentUniverseTitanRuntime,
    DivergentUniverseTitanRuntimeError, DivergentUniverseTitanSnapshot,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseContributionSnapshotAccuracy {
    ExactCurrentImmutableComponentDigestsAndStableOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseDifficultyProtocolSnapshot {
    difficulty: DivergentUniverseDifficultyId,
    difficulty_levels: Box<[u16]>,
    protocol: Option<DivergentUniverseProtocolId>,
    attack_increase: i64,
    maximum_hp_increase: i64,
    speed_increase: i64,
    maximum_toughness_increase: Option<i64>,
    difficulty_changes: Box<[DivergentUniverseProtocolRule]>,
    entry_rules: Box<[DivergentUniverseProtocolRule]>,
    berserk_changes: Box<[DivergentUniverseProtocolRule]>,
    boss_identity: Box<str>,
    digest: [u8; 32],
}

impl DivergentUniverseDifficultyProtocolSnapshot {
    #[must_use]
    pub const fn difficulty(&self) -> &DivergentUniverseDifficultyId {
        &self.difficulty
    }
    #[must_use]
    pub fn difficulty_levels(&self) -> &[u16] {
        &self.difficulty_levels
    }
    #[must_use]
    pub const fn protocol(&self) -> Option<&DivergentUniverseProtocolId> {
        self.protocol.as_ref()
    }
    #[must_use]
    pub const fn attack_increase_millionths(&self) -> i64 {
        self.attack_increase
    }
    #[must_use]
    pub const fn maximum_hp_increase_millionths(&self) -> i64 {
        self.maximum_hp_increase
    }
    #[must_use]
    pub const fn speed_increase_millionths(&self) -> i64 {
        self.speed_increase
    }
    #[must_use]
    pub const fn maximum_toughness_increase_millionths(&self) -> Option<i64> {
        self.maximum_toughness_increase
    }
    #[must_use]
    pub fn difficulty_changes(&self) -> &[DivergentUniverseProtocolRule] {
        &self.difficulty_changes
    }
    #[must_use]
    pub fn entry_rules(&self) -> &[DivergentUniverseProtocolRule] {
        &self.entry_rules
    }
    #[must_use]
    pub fn berserk_changes(&self) -> &[DivergentUniverseProtocolRule] {
        &self.berserk_changes
    }
    #[must_use]
    pub fn boss_identity(&self) -> &str {
        &self.boss_identity
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseContributionSnapshotDigest([u8; 32]);

impl DivergentUniverseContributionSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBattleContributionSnapshot {
    source_state_hash: ActivityStateHash,
    mapping: DivergentUniverseMappingSnapshotDigest,
    difficulty_protocol: DivergentUniverseDifficultyProtocolSnapshot,
    equation_blessing: DivergentUniverseBlessingInteractionSnapshot,
    curios: DivergentUniverseCurioSnapshot,
    titan: DivergentUniverseTitanSnapshot,
    progression: DivergentUniversePermanentProgressionSnapshot,
    ordered_component_digests: Box<[[u8; 32]]>,
    digest: DivergentUniverseContributionSnapshotDigest,
}

impl DivergentUniverseBattleContributionSnapshot {
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub const fn mapping_digest(&self) -> DivergentUniverseMappingSnapshotDigest {
        self.mapping
    }
    #[must_use]
    pub const fn difficulty_protocol(&self) -> &DivergentUniverseDifficultyProtocolSnapshot {
        &self.difficulty_protocol
    }
    #[must_use]
    pub const fn equation_blessing(&self) -> &DivergentUniverseBlessingInteractionSnapshot {
        &self.equation_blessing
    }
    #[must_use]
    pub const fn curios(&self) -> &DivergentUniverseCurioSnapshot {
        &self.curios
    }
    #[must_use]
    pub const fn titan(&self) -> &DivergentUniverseTitanSnapshot {
        &self.titan
    }
    #[must_use]
    pub const fn progression(&self) -> &DivergentUniversePermanentProgressionSnapshot {
        &self.progression
    }
    #[must_use]
    pub fn ordered_component_digests(&self) -> &[[u8; 32]] {
        &self.ordered_component_digests
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseContributionSnapshotDigest {
        self.digest
    }
    #[must_use]
    pub fn technique_contribution_digest(&self) -> TechniqueContributionDigest {
        TechniqueContributionDigest::new(self.digest.0)
            .expect("SHA-256 contribution snapshot digest is non-zero")
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseContributionSnapshotRuntime {
    blessing_interaction: DivergentUniverseBlessingInteractionRuntime,
    curio: DivergentUniverseCurioRuntime,
    titan: DivergentUniverseTitanRuntime,
    progression: DivergentUniversePermanentProgressionRuntime,
    component_digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn contribution_snapshot_runtime(
        &self,
    ) -> Result<
        DivergentUniverseContributionSnapshotRuntime,
        DivergentUniverseContributionSnapshotError,
    > {
        Ok(DivergentUniverseContributionSnapshotRuntime {
            blessing_interaction: self.blessing_interaction_runtime()?,
            curio: self.curio_runtime()?,
            titan: self.titan_runtime()?,
            progression: self.permanent_progression_runtime()?,
            component_digest: self.bundle.identity().component_digest().bytes(),
        })
    }
}

impl DivergentUniverseContributionSnapshotRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseContributionSnapshotAccuracy {
        DivergentUniverseContributionSnapshotAccuracy::ExactCurrentImmutableComponentDigestsAndStableOrder
    }

    pub fn snapshot(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &GraphActivity,
    ) -> Result<
        DivergentUniverseBattleContributionSnapshot,
        DivergentUniverseContributionSnapshotError,
    > {
        if flow.component_digest != self.component_digest
            || activity.definition().identity() != flow.definition.identity()
        {
            return Err(DivergentUniverseContributionSnapshotError::DefinitionMismatch);
        }
        if activity.player_view().terminal().is_some() {
            return Err(DivergentUniverseContributionSnapshotError::ActivityCompleted);
        }
        let mapping = flow
            .mapping_snapshot()
            .ok_or(DivergentUniverseContributionSnapshotError::MappingRequired)?;
        mapping
            .validate(self.component_digest, flow.definition.participants())
            .map_err(|_| DivergentUniverseContributionSnapshotError::MappingRequired)?;
        let source_state_hash = activity.state_hash();
        let equation_blessing = self.blessing_interaction.snapshot(activity)?;
        let curios = self.curio.snapshot(activity)?;
        let titan = self.titan.snapshot(activity)?;
        let progression = self.progression.snapshot(activity)?;
        if equation_blessing.source_state_hash() != source_state_hash
            || curios.source_state_hash() != source_state_hash
            || titan.source_state_hash() != source_state_hash
            || progression.source_state_hash() != source_state_hash
        {
            return Err(DivergentUniverseContributionSnapshotError::StateHashMismatch);
        }
        let difficulty_protocol = difficulty_protocol_snapshot(flow)?;
        let ordered_component_digests = vec![
            mapping.digest().bytes(),
            difficulty_protocol.digest,
            equation_blessing.digest().bytes(),
            curios.digest().bytes(),
            titan.digest().bytes(),
            progression.digest().bytes(),
        ];
        let digest = contribution_digest(
            self.component_digest,
            source_state_hash,
            &ordered_component_digests,
        );
        Ok(DivergentUniverseBattleContributionSnapshot {
            source_state_hash,
            mapping: mapping.digest(),
            difficulty_protocol,
            equation_blessing,
            curios,
            titan,
            progression,
            ordered_component_digests: ordered_component_digests.into_boxed_slice(),
            digest: DivergentUniverseContributionSnapshotDigest(digest),
        })
    }
}

fn difficulty_protocol_snapshot(
    flow: &DivergentUniverseFlowInstance,
) -> Result<DivergentUniverseDifficultyProtocolSnapshot, DivergentUniverseContributionSnapshotError>
{
    let projection = flow.progression();
    let protocol = projection.protocol_contribution();
    let values = protocol.map_or_else(ProtocolValues::empty, ProtocolValues::from_contribution);
    let difficulty = flow.difficulty().clone();
    let levels = projection.difficulty_levels().to_vec().into_boxed_slice();
    let selected = projection.selected_protocol().cloned();
    if selected.is_some() != protocol.is_some() {
        return Err(DivergentUniverseContributionSnapshotError::InvalidProgression);
    }
    let digest = difficulty_protocol_digest(DifficultyProtocolDigestInput {
        difficulty: &difficulty,
        levels: &levels,
        protocol: selected.as_ref(),
        values: &values,
    });
    Ok(DivergentUniverseDifficultyProtocolSnapshot {
        difficulty,
        difficulty_levels: levels,
        protocol: selected,
        attack_increase: values.attack,
        maximum_hp_increase: values.hp,
        speed_increase: values.speed,
        maximum_toughness_increase: values.toughness,
        difficulty_changes: values.difficulty_changes.into_boxed_slice(),
        entry_rules: values.entry_rules.into_boxed_slice(),
        berserk_changes: values.berserk_changes.into_boxed_slice(),
        boss_identity: values.boss,
        digest,
    })
}

struct ProtocolValues {
    attack: i64,
    hp: i64,
    speed: i64,
    toughness: Option<i64>,
    difficulty_changes: Vec<DivergentUniverseProtocolRule>,
    entry_rules: Vec<DivergentUniverseProtocolRule>,
    berserk_changes: Vec<DivergentUniverseProtocolRule>,
    boss: Box<str>,
}

impl ProtocolValues {
    fn empty() -> Self {
        Self {
            attack: 0,
            hp: 0,
            speed: 0,
            toughness: None,
            difficulty_changes: Vec::new(),
            entry_rules: Vec::new(),
            berserk_changes: Vec::new(),
            boss: Box::from(""),
        }
    }

    fn from_contribution(value: &DivergentUniverseProtocolContribution) -> Self {
        Self {
            attack: value.attack_increase().scaled(),
            hp: value.maximum_hp_increase().scaled(),
            speed: value.speed_increase().scaled(),
            toughness: value
                .maximum_toughness_increase()
                .map(|ratio| ratio.scaled()),
            difficulty_changes: value.difficulty_changes().to_vec(),
            entry_rules: value.entry_rules().to_vec(),
            berserk_changes: value.berserk_changes().to_vec(),
            boss: value.boss_identity().into(),
        }
    }
}

struct DifficultyProtocolDigestInput<'a> {
    difficulty: &'a DivergentUniverseDifficultyId,
    levels: &'a [u16],
    protocol: Option<&'a DivergentUniverseProtocolId>,
    values: &'a ProtocolValues,
}

fn difficulty_protocol_digest(input: DifficultyProtocolDigestInput<'_>) -> [u8; 32] {
    let mut hash = framed_hasher(b"starclock.divergent-universe.difficulty-protocol-snapshot.v1");
    frame(&mut hash, input.difficulty.as_str().as_bytes());
    frame_u64(&mut hash, input.levels.len());
    for level in input.levels {
        frame(&mut hash, &level.to_le_bytes());
    }
    frame_optional_text(&mut hash, input.protocol.map(|value| value.as_str()));
    for ratio in [input.values.attack, input.values.hp, input.values.speed] {
        frame(&mut hash, &ratio.to_le_bytes());
    }
    match input.values.toughness {
        Some(value) => {
            frame(&mut hash, &[1]);
            frame(&mut hash, &value.to_le_bytes());
        }
        None => frame(&mut hash, &[0]),
    }
    frame_rules(&mut hash, &input.values.difficulty_changes);
    frame_rules(&mut hash, &input.values.entry_rules);
    frame_rules(&mut hash, &input.values.berserk_changes);
    frame(&mut hash, input.values.boss.as_bytes());
    hash.finalize()
}

fn contribution_digest(
    component: [u8; 32],
    state: ActivityStateHash,
    components: &[[u8; 32]],
) -> [u8; 32] {
    let mut hash = framed_hasher(b"starclock.divergent-universe.battle-contribution-snapshot.v1");
    frame(&mut hash, &component);
    frame(&mut hash, &state.bytes());
    frame_u64(&mut hash, components.len());
    for component in components {
        frame(&mut hash, component);
    }
    hash.finalize()
}

fn frame_rules(hash: &mut CanonicalDigestBuilder, rules: &[DivergentUniverseProtocolRule]) {
    frame_u64(hash, rules.len());
    for rule in rules {
        match rule {
            DivergentUniverseProtocolRule::IncreaseStorePrice { ratio } => {
                frame(hash, &[15]);
                frame(hash, &ratio.scaled().to_le_bytes());
            }
            DivergentUniverseProtocolRule::GrantRandomLevelOneDomains { count } => {
                frame(hash, &[16]);
                frame(hash, &[*count]);
            }
            _ => frame(hash, &[protocol_rule_tag(*rule)]),
        }
    }
}

fn protocol_rule_tag(rule: DivergentUniverseProtocolRule) -> u8 {
    match rule {
        DivergentUniverseProtocolRule::IncreaseDomainCount => 1,
        DivergentUniverseProtocolRule::EnableSecondPlaneConversionDomains => 2,
        DivergentUniverseProtocolRule::IncreaseEquationRandomness => 3,
        DivergentUniverseProtocolRule::IncreaseMaskWishpowerRequirement => 4,
        DivergentUniverseProtocolRule::IncreaseEquationBlessingRequirement => 5,
        DivergentUniverseProtocolRule::ReplaceFirstAndSecondPlaneBosses => 6,
        DivergentUniverseProtocolRule::AdvanceBerserkOnset => 7,
        DivergentUniverseProtocolRule::IncreaseBerserkStackRate => 8,
        DivergentUniverseProtocolRule::FurtherIncreaseDomainCount => 9,
        DivergentUniverseProtocolRule::GrantOneGrandMiracleAtFirstPlaneEntry => 10,
        DivergentUniverseProtocolRule::AdvanceBerserkEnemyAfterAttacked => 11,
        DivergentUniverseProtocolRule::IncreaseAllyDamageAfterBerserk => 12,
        DivergentUniverseProtocolRule::DecreaseAllyHealingAfterBerserk => 13,
        DivergentUniverseProtocolRule::DecreaseAllyShieldAfterBerserk => 14,
        DivergentUniverseProtocolRule::GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry => 17,
        DivergentUniverseProtocolRule::IncreaseStorePrice { .. }
        | DivergentUniverseProtocolRule::GrantRandomLevelOneDomains { .. } => {
            unreachable!("parameterized Protocol rules are framed separately")
        }
    }
}

fn framed_hasher(domain: &[u8]) -> CanonicalDigestBuilder {
    let mut hash = CanonicalDigestBuilder::new();
    frame(&mut hash, domain);
    hash
}

fn frame(hash: &mut CanonicalDigestBuilder, value: &[u8]) {
    hash.update(
        u64::try_from(value.len())
            .expect("slice length fits u64")
            .to_le_bytes(),
    );
    hash.update(value);
}

fn frame_u64(hash: &mut CanonicalDigestBuilder, value: usize) {
    frame(
        hash,
        &u64::try_from(value)
            .expect("collection length fits u64")
            .to_le_bytes(),
    );
}

fn frame_optional_text(hash: &mut CanonicalDigestBuilder, value: Option<&str>) {
    match value {
        Some(value) => {
            frame(hash, &[1]);
            frame(hash, value.as_bytes());
        }
        None => frame(hash, &[0]),
    }
}

#[derive(Debug)]
pub enum DivergentUniverseContributionSnapshotError {
    DefinitionMismatch,
    MappingRequired,
    ActivityCompleted,
    InvalidProgression,
    StateHashMismatch,
    Blessing(DivergentUniverseBlessingInteractionError),
    Curio(DivergentUniverseCurioRuntimeError),
    Titan(DivergentUniverseTitanRuntimeError),
    Progression(DivergentUniversePermanentProgressionRuntimeError),
}

impl From<DivergentUniverseBlessingInteractionError>
    for DivergentUniverseContributionSnapshotError
{
    fn from(value: DivergentUniverseBlessingInteractionError) -> Self {
        Self::Blessing(value)
    }
}
impl From<DivergentUniverseCurioRuntimeError> for DivergentUniverseContributionSnapshotError {
    fn from(value: DivergentUniverseCurioRuntimeError) -> Self {
        Self::Curio(value)
    }
}
impl From<DivergentUniverseTitanRuntimeError> for DivergentUniverseContributionSnapshotError {
    fn from(value: DivergentUniverseTitanRuntimeError) -> Self {
        Self::Titan(value)
    }
}
impl From<DivergentUniversePermanentProgressionRuntimeError>
    for DivergentUniverseContributionSnapshotError
{
    fn from(value: DivergentUniversePermanentProgressionRuntimeError) -> Self {
        Self::Progression(value)
    }
}
impl core::fmt::Display for DivergentUniverseContributionSnapshotError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe contribution snapshot error: {self:?}"
        )
    }
}
impl std::error::Error for DivergentUniverseContributionSnapshotError {}
