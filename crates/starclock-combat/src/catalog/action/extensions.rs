use super::*;

impl ScalingDamageDefinition {
    #[must_use]
    pub const fn class(self) -> crate::formula::model::DamageClass {
        self.class
    }
}

impl OrdinaryDamageDefinition {
    #[must_use]
    pub const fn class(self) -> crate::formula::model::DamageClass {
        self.class
    }

    /// Adds one already-resolved flat amount to the formula base before the
    /// ordered multiplicative stages.
    pub fn with_flat_base(mut self, value: Scalar) -> Result<Self, NumericError> {
        self.base_damage = self.base_damage.checked_add(value)?;
        if self.base_damage.scaled() < 0 {
            return Err(NumericError::OutOfDomain);
        }
        Ok(self)
    }
}

impl AbilityActionDefinition {
    /// Preserves authored labels while adding one orthogonal semantic tag.
    #[must_use]
    pub fn with_added_tag(mut self, tag: AbilityTag) -> Self {
        self.tags.0 |= 1_u32 << (tag as u8);
        self
    }
}
