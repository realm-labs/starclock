use crate::{NumericError, Ratio, RawToughness, SourceDefinitionId, formula::model::CombatElement};

/// Stable authored Toughness-layer family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ToughnessLayerKind {
    Ordinary,
    ExoToughness,
    Sequential,
    Shared,
}

/// Element eligibility retained per layer instead of inferred from its family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToughnessWeaknessPolicy {
    MatchingOnly,
    AnyElement,
    OffWeakness(Ratio),
}

/// Attribution rule selected explicitly by authored layer data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakCreditPolicy {
    HitApplier,
    LayerProvider(SourceDefinitionId),
}

/// Immutable initial layer definition carried by generic combat input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToughnessLayerSpec {
    key: u32,
    kind: ToughnessLayerKind,
    maximum: RawToughness,
    active: bool,
    locked: bool,
    weakness_policy: ToughnessWeaknessPolicy,
    reducible_while_broken: bool,
    recovery_ratio: Ratio,
    applies_break_damage: bool,
    applies_break_effect: bool,
    changes_global_broken: bool,
    break_element: Option<CombatElement>,
    break_credit: BreakCreditPolicy,
}

impl ToughnessLayerSpec {
    /// Creates the common single-bar policy. Reduction never spills implicitly.
    pub fn ordinary(key: u32, maximum: RawToughness) -> Result<Self, NumericError> {
        if key == 0 || maximum.get() == 0 {
            return Err(NumericError::OutOfDomain);
        }
        Ok(Self {
            key,
            kind: ToughnessLayerKind::Ordinary,
            maximum,
            active: true,
            locked: false,
            weakness_policy: ToughnessWeaknessPolicy::MatchingOnly,
            reducible_while_broken: false,
            recovery_ratio: Ratio::ONE,
            applies_break_damage: true,
            applies_break_effect: true,
            changes_global_broken: true,
            break_element: None,
            break_credit: BreakCreditPolicy::HitApplier,
        })
    }

    #[must_use]
    pub const fn with_kind(mut self, kind: ToughnessLayerKind) -> Self {
        self.kind = kind;
        self
    }
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    #[must_use]
    pub const fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }
    pub fn with_weakness_policy(
        mut self,
        policy: ToughnessWeaknessPolicy,
    ) -> Result<Self, NumericError> {
        if matches!(policy, ToughnessWeaknessPolicy::OffWeakness(value) if !(0..=1_000_000).contains(&value.scaled()))
        {
            return Err(NumericError::OutOfDomain);
        }
        self.weakness_policy = policy;
        Ok(self)
    }
    #[must_use]
    pub const fn with_break_behavior(
        mut self,
        reducible_while_broken: bool,
        applies_break_damage: bool,
        applies_break_effect: bool,
        changes_global_broken: bool,
    ) -> Self {
        self.reducible_while_broken = reducible_while_broken;
        self.applies_break_damage = applies_break_damage;
        self.applies_break_effect = applies_break_effect;
        self.changes_global_broken = changes_global_broken;
        self
    }
    #[must_use]
    pub const fn with_break_element(mut self, element: CombatElement) -> Self {
        self.break_element = Some(element);
        self
    }
    #[must_use]
    pub const fn with_break_credit(mut self, policy: BreakCreditPolicy) -> Self {
        self.break_credit = policy;
        self
    }
    pub fn with_recovery_ratio(mut self, ratio: Ratio) -> Result<Self, NumericError> {
        if !(0..=1_000_000).contains(&ratio.scaled()) {
            return Err(NumericError::OutOfDomain);
        }
        self.recovery_ratio = ratio;
        Ok(self)
    }

    #[must_use]
    pub const fn key(&self) -> u32 {
        self.key
    }
    #[must_use]
    pub const fn kind(&self) -> ToughnessLayerKind {
        self.kind
    }
    #[must_use]
    pub const fn maximum(&self) -> RawToughness {
        self.maximum
    }
    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }
    #[must_use]
    pub const fn locked(&self) -> bool {
        self.locked
    }
    #[must_use]
    pub const fn weakness_policy(&self) -> ToughnessWeaknessPolicy {
        self.weakness_policy
    }
    #[must_use]
    pub const fn reducible_while_broken(&self) -> bool {
        self.reducible_while_broken
    }
    #[must_use]
    pub const fn recovery_ratio(&self) -> Ratio {
        self.recovery_ratio
    }
    #[must_use]
    pub const fn applies_break_damage(&self) -> bool {
        self.applies_break_damage
    }
    #[must_use]
    pub const fn applies_break_effect(&self) -> bool {
        self.applies_break_effect
    }
    #[must_use]
    pub const fn changes_global_broken(&self) -> bool {
        self.changes_global_broken
    }
    #[must_use]
    pub const fn break_element(&self) -> Option<CombatElement> {
        self.break_element
    }
    #[must_use]
    pub const fn break_credit(&self) -> BreakCreditPolicy {
        self.break_credit
    }
}

/// Fully resolved hit operation that can deplete a layer and create a Break.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToughnessReductionDefinition {
    pub element: CombatElement,
    pub reduction: crate::formula::toughness::ToughnessReductionContext,
    pub break_damage: crate::formula::toughness::BreakDamageDefinition,
    /// Final clamped result of 150% base chance, EHR, Effect RES and debuff RES.
    pub break_effect_chance: crate::Probability,
}
