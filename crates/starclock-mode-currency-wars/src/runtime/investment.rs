use starclock_activity::{ActivityRngLabel, GraphActivityCommandError};

use super::{
    AUGMENT_OFFERS, CurrencyWarsRun, CurrencyWarsRuntimeError, GOLD, INVESTMENT_FAMILY_MASK,
    INVESTMENT_OFFER_WIDTH, INVESTMENT_OFFERS, INVESTMENT_QUALITY, INVESTMENT_REROLLS, INVESTMENTS,
    SEASON_TALENTS, SELECTED_ENHANCEMENT_OFFERS, SELECTED_ENHANCEMENTS, add_integer, debug_error,
    error, program_id, set_integer, set_ordered_ids,
};
use crate::{
    CurrencyWarsAugmentQuality, CurrencyWarsBondMember, CurrencyWarsDifficulty,
    CurrencyWarsEnhancement, CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind,
    CurrencyWarsInvestmentOfferFamily, CurrencyWarsInvestmentOfferSpec, CurrencyWarsRoleState,
    CurrencyWarsSelectedEnhancement, CurrencyWarsSelectedEnhancementId, CurrencyWarsTalentKind,
    CurrencyWarsTypedInvestment,
};

const AUGMENT_OFFER_PURPOSE: u16 = 20;
const AUGMENT_OFFER_WIDTH: u16 = 3;
const INVESTMENT_OFFER_PURPOSE: u16 = 21;

impl CurrencyWarsRun {
    /// Opens one explicitly described cross-family offer. The caller owns the
    /// node timing; the run owns eligibility, RNG and atomic lifecycle.
    pub fn offer_investments(
        &mut self,
        spec: CurrencyWarsInvestmentOfferSpec,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if !self.current_investment_offers()?.is_empty() {
            return Err(error("Currency Wars investment offer is already active"));
        }
        self.generate_investment_offer(spec, false)
    }

    pub fn reroll_investments(
        &mut self,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if self.current_investment_offers()?.is_empty() || self.integer(INVESTMENT_REROLLS) <= 0 {
            return Err(error("Currency Wars investment offer cannot be rerolled"));
        }
        let spec = self.current_investment_offer_spec()?;
        self.generate_investment_offer(spec, true)
    }

    pub fn current_investment_offers(
        &self,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        self.ordered_ids(INVESTMENT_OFFERS)?
            .iter()
            .map(|raw| {
                CurrencyWarsInvestmentId::new(*raw)
                    .ok_or_else(|| error("Currency Wars investment offer ID is zero"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn choose_offered_investment(
        &mut self,
        investment: CurrencyWarsInvestmentId,
        replace: Option<CurrencyWarsInvestmentId>,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if self
            .current_investment_offers()?
            .binary_search(&investment)
            .is_err()
        {
            return Err(error("Currency Wars investment is not in the active offer"));
        }
        let definition = self
            .definition
            .catalog
            .investment(investment)
            .ok_or_else(|| error("Currency Wars offered investment is missing"))?;
        self.validate_investment_eligibility(investment, payment_confirmed)?;
        let mut owned = self.ordered_ids(INVESTMENTS)?.into_vec();
        if owned.binary_search(&investment.get()).is_ok() {
            return Err(error("Currency Wars offered investment is already owned"));
        }
        if let Some(old) = replace {
            let old_definition = self
                .definition
                .catalog
                .investment(old)
                .ok_or_else(|| error("Currency Wars replacement investment is missing"))?;
            if old_definition.kind != definition.kind {
                return Err(error(
                    "Currency Wars investment replacement crosses families",
                ));
            }
            let index = owned
                .binary_search(&old.get())
                .map_err(|_| error("Currency Wars replacement investment is not owned"))?;
            owned.remove(index);
        }
        owned.push(investment.get());
        owned.sort_unstable();
        let mut operations = Vec::with_capacity(6);
        if let Some(enhancement) = self
            .definition
            .catalog
            .augment_catalog()
            .enhancement(investment)
            && let Some(cost) = enhancement.gold_cost.filter(|cost| *cost > 0)
        {
            operations.push(add_integer(GOLD, -i64::from(cost)));
        }
        operations.extend([
            set_ordered_ids(INVESTMENTS, owned.into_boxed_slice()),
            set_ordered_ids(INVESTMENT_OFFERS, Box::new([])),
            set_integer(INVESTMENT_REROLLS, 0),
            set_integer(INVESTMENT_FAMILY_MASK, 0),
            set_integer(INVESTMENT_QUALITY, 0),
            set_integer(INVESTMENT_OFFER_WIDTH, 0),
        ]);
        self.apply_state(129, operations)
    }

    pub fn choose_enhancement(
        &mut self,
        investment: CurrencyWarsInvestmentId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let enhancement = self
            .definition
            .catalog
            .augment_catalog()
            .enhancement(investment)
            .cloned()
            .ok_or_else(|| error("Currency Wars Enhancement definition is missing"))?;
        self.validate_enhancement(&enhancement)?;
        let mut owned = self.ordered_ids(INVESTMENTS)?.into_vec();
        if owned.binary_search(&investment.get()).is_ok() {
            return Err(error("Currency Wars Enhancement is already owned"));
        }
        owned.push(investment.get());
        owned.sort_unstable();
        let mut operations = Vec::with_capacity(2);
        if let Some(cost) = enhancement.gold_cost.filter(|cost| *cost > 0) {
            operations.push(add_integer(GOLD, -i64::from(cost)));
        }
        operations.push(set_ordered_ids(INVESTMENTS, owned.into_boxed_slice()));
        self.apply_state(127, operations)
    }

    fn generate_investment_offer(
        &mut self,
        spec: CurrencyWarsInvestmentOfferSpec,
        reroll: bool,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        let candidates = self.eligible_investment_candidates(&spec)?;
        if candidates.len() < usize::from(spec.width) {
            return Err(error(
                "Currency Wars investment offer has too few eligible candidates",
            ));
        }
        let width = u16::from(spec.width);
        let family_mask = spec.family_mask();
        let quality = encode_quality(spec.augment_quality);
        let rerolls = if reroll {
            self.integer(INVESTMENT_REROLLS) - 1
        } else {
            i64::from(spec.rerolls)
        };
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(128), move |rng| {
                let selected = rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        INVESTMENT_OFFER_PURPOSE,
                        &vec![1; candidates.len()],
                        width,
                    )
                    .map_err(GraphActivityCommandError::Rng)?;
                let mut offered = selected
                    .iter()
                    .map(|index| candidates[*index as usize])
                    .collect::<Vec<_>>();
                offered.sort_unstable();
                Ok((
                    vec![
                        set_ordered_ids(
                            INVESTMENT_OFFERS,
                            offered.iter().map(|value| value.get()).collect(),
                        ),
                        set_integer(INVESTMENT_REROLLS, rerolls),
                        set_integer(INVESTMENT_FAMILY_MASK, i64::from(family_mask)),
                        set_integer(INVESTMENT_QUALITY, i64::from(quality)),
                        set_integer(INVESTMENT_OFFER_WIDTH, i64::from(width)),
                    ],
                    offered.into_boxed_slice(),
                ))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    fn eligible_investment_candidates(
        &self,
        spec: &CurrencyWarsInvestmentOfferSpec,
    ) -> Result<Vec<CurrencyWarsInvestmentId>, CurrencyWarsRuntimeError> {
        let owned = self.ordered_ids(INVESTMENTS)?;
        Ok(self
            .definition
            .catalog
            .investments()
            .iter()
            .filter(|definition| spec.contains(family(definition.kind)))
            .filter(|definition| owned.binary_search(&definition.id.get()).is_err())
            .filter(|definition| {
                spec.augment_quality.is_none_or(|quality| {
                    definition.kind != CurrencyWarsInvestmentKind::Augment
                        || self
                            .definition
                            .catalog
                            .augment_catalog()
                            .augment(definition.id)
                            .is_some_and(|augment| augment.quality == quality)
                })
            })
            .filter_map(|definition| {
                self.validate_investment_eligibility(definition.id, true)
                    .is_ok()
                    .then_some(definition.id)
            })
            .collect::<Vec<_>>())
    }

    fn current_investment_offer_spec(
        &self,
    ) -> Result<CurrencyWarsInvestmentOfferSpec, CurrencyWarsRuntimeError> {
        let mask = u8::try_from(self.integer(INVESTMENT_FAMILY_MASK))
            .map_err(|_| error("Currency Wars investment family mask is invalid"))?;
        let families = all_families()
            .into_iter()
            .filter(|family| mask & family.bit() != 0)
            .collect();
        CurrencyWarsInvestmentOfferSpec::new(
            families,
            decode_quality(self.integer(INVESTMENT_QUALITY))?,
            u8::try_from(self.integer(INVESTMENT_OFFER_WIDTH))
                .map_err(|_| error("Currency Wars investment offer width is invalid"))?,
            u8::try_from(self.integer(INVESTMENT_REROLLS))
                .map_err(|_| error("Currency Wars investment reroll count is invalid"))?,
        )
        .map_err(debug_error)
    }

    fn validate_investment_eligibility(
        &self,
        investment: CurrencyWarsInvestmentId,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let definition = self
            .definition
            .catalog
            .investment(investment)
            .ok_or_else(|| error("Currency Wars investment is missing"))?;
        match definition.kind {
            CurrencyWarsInvestmentKind::Augment => {
                let augment = self
                    .definition
                    .catalog
                    .augment_catalog()
                    .augment(investment)
                    .ok_or_else(|| error("Currency Wars Augment definition is missing"))?;
                let difficulty = self.current_difficulty()?;
                let plane = self
                    .current_plane()
                    .ok_or_else(|| error("Currency Wars investment has no active Plane"))?;
                if !augment.eligible(
                    difficulty.season_id,
                    plane,
                    self.definition.gambit,
                    self.definition
                        .catalog
                        .flow_catalog()
                        .profile_module_source_id(),
                ) {
                    return Err(error("Currency Wars Augment is not eligible"));
                }
            }
            CurrencyWarsInvestmentKind::Enhancement => {
                let enhancement = self
                    .definition
                    .catalog
                    .augment_catalog()
                    .enhancement(investment)
                    .ok_or_else(|| error("Currency Wars Enhancement definition is missing"))?;
                self.validate_enhancement(enhancement)?;
            }
            CurrencyWarsInvestmentKind::Orb
            | CurrencyWarsInvestmentKind::Portal
            | CurrencyWarsInvestmentKind::Projection
            | CurrencyWarsInvestmentKind::Talent => {
                self.validate_typed_investment(investment, payment_confirmed)?;
            }
        }
        Ok(())
    }

    fn validate_enhancement(
        &self,
        enhancement: &CurrencyWarsEnhancement,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let (active, maximum_star) =
            self.selected_enhancement_context(enhancement.trait_effect_id)?;
        if !active || !enhancement.condition.eligible(maximum_star) {
            return Err(error("Currency Wars Enhancement is not eligible"));
        }
        if enhancement.gold_cost.is_some_and(|cost| self.gold() < cost) {
            return Err(error("Currency Wars Enhancement requires more Gold"));
        }
        Ok(())
    }

    fn current_difficulty(&self) -> Result<&CurrencyWarsDifficulty, CurrencyWarsRuntimeError> {
        self.definition
            .catalog
            .difficulties()
            .iter()
            .find(|value| value.source_id == self.definition.difficulty)
            .ok_or_else(|| error("Currency Wars investment difficulty is missing"))
    }
    /// Generates an Augment offer at an explicit mode-program boundary.
    /// Scheduling is not inferred from Augment category IDs.
    pub fn offer_augments(
        &mut self,
        quality: CurrencyWarsAugmentQuality,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if !self.current_augment_offers()?.is_empty() {
            return Err(error("Currency Wars Augment offer is already active"));
        }
        let plane = self
            .current_plane()
            .ok_or_else(|| error("Currency Wars Augment offer has no active Plane"))?;
        let difficulty = self
            .definition
            .catalog
            .difficulties()
            .binary_search_by_key(&self.definition.difficulty, |value| value.source_id)
            .ok()
            .map(|index| &self.definition.catalog.difficulties()[index])
            .ok_or_else(|| error("Currency Wars Augment offer difficulty is missing"))?;
        let module_id = self
            .definition
            .catalog
            .flow_catalog()
            .profile_module_source_id();
        let owned = self.ordered_ids(INVESTMENTS)?;
        let candidates = self
            .definition
            .catalog
            .augment_catalog()
            .augments()
            .iter()
            .filter(|definition| {
                definition.quality == quality
                    && definition.eligible(
                        difficulty.season_id,
                        plane,
                        self.definition.gambit,
                        module_id,
                    )
                    && owned.binary_search(&definition.investment.get()).is_err()
            })
            .map(|definition| definition.investment)
            .collect::<Vec<_>>();
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(120), move |rng| {
                let selected = rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        AUGMENT_OFFER_PURPOSE,
                        &vec![1; candidates.len()],
                        AUGMENT_OFFER_WIDTH,
                    )
                    .map_err(GraphActivityCommandError::Rng)?;
                let mut offered = selected
                    .iter()
                    .map(|index| candidates[*index as usize])
                    .collect::<Vec<_>>();
                offered.sort_unstable();
                Ok((
                    vec![set_ordered_ids(
                        AUGMENT_OFFERS,
                        offered.iter().map(|value| value.get()).collect(),
                    )],
                    offered.into_boxed_slice(),
                ))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub fn choose_augment(
        &mut self,
        investment: CurrencyWarsInvestmentId,
        replace: Option<CurrencyWarsInvestmentId>,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if self
            .current_augment_offers()?
            .binary_search(&investment)
            .is_err()
        {
            return Err(error("Currency Wars Augment is not in the active offer"));
        }
        self.definition
            .catalog
            .augment_catalog()
            .augment(investment)
            .ok_or_else(|| error("Currency Wars Augment definition is missing"))?;
        let mut owned = self.ordered_ids(INVESTMENTS)?.into_vec();
        if owned.binary_search(&investment.get()).is_ok() {
            return Err(error("Currency Wars Augment is already owned"));
        }
        if let Some(old) = replace {
            self.definition
                .catalog
                .augment_catalog()
                .augment(old)
                .ok_or_else(|| error("Currency Wars replacement is not an Augment"))?;
            let index = owned
                .binary_search(&old.get())
                .map_err(|_| error("Currency Wars replacement Augment is not owned"))?;
            owned.remove(index);
        }
        owned.push(investment.get());
        owned.sort_unstable();
        self.apply_state(
            121,
            vec![
                set_ordered_ids(INVESTMENTS, owned.into_boxed_slice()),
                set_ordered_ids(AUGMENT_OFFERS, Box::new([])),
            ],
        )
    }

    pub fn choose_typed_investment(
        &mut self,
        investment: CurrencyWarsInvestmentId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        self.choose_typed_investment_with_payment(investment, false)
    }

    pub fn choose_talent(
        &mut self,
        investment: CurrencyWarsInvestmentId,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        self.choose_typed_investment_with_payment(investment, payment_confirmed)
    }

    fn choose_typed_investment_with_payment(
        &mut self,
        investment: CurrencyWarsInvestmentId,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        self.validate_typed_investment(investment, payment_confirmed)?;
        let mut owned = self.ordered_ids(INVESTMENTS)?.into_vec();
        if owned.binary_search(&investment.get()).is_ok() {
            return Err(error("Currency Wars typed investment is already owned"));
        }
        owned.push(investment.get());
        owned.sort_unstable();
        self.apply_state(
            124,
            vec![set_ordered_ids(INVESTMENTS, owned.into_boxed_slice())],
        )
    }

    fn validate_typed_investment(
        &self,
        investment: CurrencyWarsInvestmentId,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let definition = self
            .definition
            .catalog
            .cross_investment_catalog()
            .investment(investment)
            .ok_or_else(|| error("Currency Wars typed investment is missing"))?;
        let difficulty = self
            .definition
            .catalog
            .difficulties()
            .iter()
            .find(|value| value.source_id == self.definition.difficulty)
            .ok_or_else(|| error("Currency Wars investment difficulty is missing"))?;
        match &definition {
            CurrencyWarsTypedInvestment::Portal(portal) => {
                if !portal.eligible(
                    difficulty.season_id,
                    self.definition.gambit,
                    self.definition
                        .catalog
                        .flow_catalog()
                        .profile_module_source_id(),
                ) {
                    return Err(error("Currency Wars Portal is not eligible"));
                }
            }
            CurrencyWarsTypedInvestment::Projection(projection) => {
                if !self.roster()?.owns_role(projection.role) {
                    return Err(error("Currency Wars Projection role is not owned"));
                }
            }
            CurrencyWarsTypedInvestment::Talent(talent) => {
                if !payment_confirmed {
                    return Err(error("Currency Wars Talent cost is not settled"));
                }
                self.validate_talent_prerequisites(&talent.prerequisites, INVESTMENTS)?;
            }
            CurrencyWarsTypedInvestment::Orb(_) => {}
        }
        Ok(())
    }

    /// Executes a season Talent after an external caller confirms its authored
    /// cost was paid. Released data does not identify that cost's currency.
    pub fn choose_season_talent(
        &mut self,
        source_id: u32,
        payment_confirmed: bool,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if !payment_confirmed {
            return Err(error("Currency Wars season Talent cost is not settled"));
        }
        let definition = self
            .definition
            .catalog
            .cross_investment_catalog()
            .talent(CurrencyWarsTalentKind::Season, source_id)
            .ok_or_else(|| error("Currency Wars season Talent is missing"))?;
        let season = self
            .definition
            .catalog
            .difficulties()
            .iter()
            .find(|value| value.source_id == self.definition.difficulty)
            .map(|value| value.season_id)
            .ok_or_else(|| error("Currency Wars season Talent difficulty is missing"))?;
        if definition.season_id != Some(season) {
            return Err(error(
                "Currency Wars season Talent belongs to another season",
            ));
        }
        self.validate_talent_prerequisites(&definition.prerequisites, SEASON_TALENTS)?;
        let mut selected = self.ordered_ids(SEASON_TALENTS)?.into_vec();
        if selected.binary_search(&u64::from(source_id)).is_ok() {
            return Err(error("Currency Wars season Talent is already selected"));
        }
        selected.push(u64::from(source_id));
        selected.sort_unstable();
        self.apply_state(
            125,
            vec![set_ordered_ids(SEASON_TALENTS, selected.into_boxed_slice())],
        )
    }

    fn validate_talent_prerequisites(
        &self,
        prerequisites: &[u32],
        state_slot: u32,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let selected = self.ordered_ids(state_slot)?;
        if prerequisites.iter().any(|source_id| {
            let investment = if state_slot == INVESTMENTS {
                self.definition
                    .catalog
                    .cross_investment_catalog()
                    .talent(CurrencyWarsTalentKind::Permanent, *source_id)
                    .and_then(|value| value.investment)
                    .map_or(0, CurrencyWarsInvestmentId::get)
            } else {
                u64::from(*source_id)
            };
            investment == 0 || selected.binary_search(&investment).is_err()
        }) {
            return Err(error("Currency Wars Talent prerequisite is not selected"));
        }
        Ok(())
    }

    pub fn current_augment_offers(
        &self,
    ) -> Result<Box<[CurrencyWarsInvestmentId]>, CurrencyWarsRuntimeError> {
        self.ordered_ids(AUGMENT_OFFERS)?
            .iter()
            .map(|raw| {
                CurrencyWarsInvestmentId::new(*raw)
                    .ok_or_else(|| error("Currency Wars Augment offer ID is zero"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn eligible_selected_enhancements(
        &self,
        trait_effect_id: u32,
    ) -> Result<Box<[CurrencyWarsSelectedEnhancement]>, CurrencyWarsRuntimeError> {
        let (active, maximum_star) = self.selected_enhancement_context(trait_effect_id)?;
        Ok(self
            .definition
            .catalog
            .augment_catalog()
            .selected_enhancements()
            .iter()
            .filter(|value| {
                active && value.trait_effect_id == trait_effect_id && value.eligible(maximum_star)
            })
            .cloned()
            .collect())
    }

    pub fn offer_selected_enhancements(
        &mut self,
        trait_effect_id: u32,
    ) -> Result<Box<[CurrencyWarsSelectedEnhancementId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if !self.current_selected_enhancement_offers()?.is_empty() {
            return Err(error(
                "Currency Wars selected Enhancement offer is already active",
            ));
        }
        let offered = self
            .eligible_selected_enhancements(trait_effect_id)?
            .iter()
            .map(|value| value.id)
            .collect::<Box<[_]>>();
        if offered.is_empty() {
            return Err(error(
                "Currency Wars selected Enhancement has no eligible option",
            ));
        }
        self.apply_state(
            122,
            vec![set_ordered_ids(
                SELECTED_ENHANCEMENT_OFFERS,
                offered.iter().map(|value| u64::from(value.get())).collect(),
            )],
        )?;
        Ok(offered)
    }

    pub fn current_selected_enhancement_offers(
        &self,
    ) -> Result<Box<[CurrencyWarsSelectedEnhancementId]>, CurrencyWarsRuntimeError> {
        self.ordered_ids(SELECTED_ENHANCEMENT_OFFERS)?
            .iter()
            .map(|raw| {
                u32::try_from(*raw)
                    .ok()
                    .and_then(CurrencyWarsSelectedEnhancementId::new)
                    .ok_or_else(|| error("Currency Wars selected Enhancement offer ID is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn choose_selected_enhancement(
        &mut self,
        id: CurrencyWarsSelectedEnhancementId,
        trait_effect_id: u32,
        replace: Option<CurrencyWarsSelectedEnhancementId>,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if self
            .current_selected_enhancement_offers()?
            .binary_search(&id)
            .is_err()
        {
            return Err(error(
                "Currency Wars selected Enhancement is not in the active offer",
            ));
        }
        let (active, maximum_star) = self.selected_enhancement_context(trait_effect_id)?;
        let definition = self
            .definition
            .catalog
            .augment_catalog()
            .selected_enhancement(id)
            .filter(|value| {
                active && value.trait_effect_id == trait_effect_id && value.eligible(maximum_star)
            })
            .ok_or_else(|| error("Currency Wars selected Enhancement is not eligible"))?;
        let cost = definition.gold_cost.unwrap_or_default();
        if self.gold() < cost {
            return Err(error(
                "Currency Wars selected Enhancement requires more Gold",
            ));
        }
        let mut selected = self.ordered_ids(SELECTED_ENHANCEMENTS)?.into_vec();
        if selected.binary_search(&u64::from(id.get())).is_ok() {
            return Err(error(
                "Currency Wars selected Enhancement is already active",
            ));
        }
        if let Some(old) = replace {
            self.definition
                .catalog
                .augment_catalog()
                .selected_enhancement(old)
                .filter(|value| value.trait_effect_id == trait_effect_id)
                .ok_or_else(|| error("Currency Wars replacement Enhancement is invalid"))?;
            let index = selected
                .binary_search(&u64::from(old.get()))
                .map_err(|_| error("Currency Wars replacement Enhancement is not active"))?;
            selected.remove(index);
        }
        selected.push(u64::from(id.get()));
        selected.sort_unstable();
        let mut operations = Vec::with_capacity(3);
        if cost > 0 {
            operations.push(add_integer(GOLD, -i64::from(cost)));
        }
        operations.push(set_ordered_ids(
            SELECTED_ENHANCEMENTS,
            selected.into_boxed_slice(),
        ));
        operations.push(set_ordered_ids(SELECTED_ENHANCEMENT_OFFERS, Box::new([])));
        self.apply_state(123, operations)
    }

    fn selected_enhancement_context(
        &self,
        trait_effect_id: u32,
    ) -> Result<(bool, bool), CurrencyWarsRuntimeError> {
        let active = self
            .bond_snapshot()?
            .trait_effect_ids
            .binary_search(&trait_effect_id)
            .is_ok();
        let roster = self.roster()?;
        let maximum_star = self
            .definition
            .catalog
            .bonds()
            .iter()
            .filter(|bond| {
                bond.trait_effect_ids
                    .binary_search(&trait_effect_id)
                    .is_ok()
            })
            .flat_map(|bond| bond.members.iter())
            .filter_map(|member| match member {
                CurrencyWarsBondMember::RosterRole(role) => Some(*role),
                CurrencyWarsBondMember::ExternalAuthoredRole(_) => None,
            })
            .any(|role| {
                self.definition
                    .catalog
                    .role(role)
                    .is_some_and(|definition| {
                        CurrencyWarsRoleState::new(role, definition.maximum_star)
                            .is_ok_and(|state| roster.count(state) > 0)
                    })
            });
        Ok((active, maximum_star))
    }
}

fn all_families() -> [CurrencyWarsInvestmentOfferFamily; 6] {
    [
        CurrencyWarsInvestmentOfferFamily::Augment,
        CurrencyWarsInvestmentOfferFamily::Enhancement,
        CurrencyWarsInvestmentOfferFamily::Orb,
        CurrencyWarsInvestmentOfferFamily::Portal,
        CurrencyWarsInvestmentOfferFamily::Projection,
        CurrencyWarsInvestmentOfferFamily::Talent,
    ]
}

const fn family(kind: CurrencyWarsInvestmentKind) -> CurrencyWarsInvestmentOfferFamily {
    match kind {
        CurrencyWarsInvestmentKind::Augment => CurrencyWarsInvestmentOfferFamily::Augment,
        CurrencyWarsInvestmentKind::Enhancement => CurrencyWarsInvestmentOfferFamily::Enhancement,
        CurrencyWarsInvestmentKind::Orb => CurrencyWarsInvestmentOfferFamily::Orb,
        CurrencyWarsInvestmentKind::Portal => CurrencyWarsInvestmentOfferFamily::Portal,
        CurrencyWarsInvestmentKind::Projection => CurrencyWarsInvestmentOfferFamily::Projection,
        CurrencyWarsInvestmentKind::Talent => CurrencyWarsInvestmentOfferFamily::Talent,
    }
}

const fn encode_quality(quality: Option<CurrencyWarsAugmentQuality>) -> u8 {
    match quality {
        None => 0,
        Some(CurrencyWarsAugmentQuality::Silver) => 1,
        Some(CurrencyWarsAugmentQuality::Gold) => 2,
        Some(CurrencyWarsAugmentQuality::Prismatic) => 3,
    }
}

fn decode_quality(
    raw: i64,
) -> Result<Option<CurrencyWarsAugmentQuality>, CurrencyWarsRuntimeError> {
    match raw {
        0 => Ok(None),
        1 => Ok(Some(CurrencyWarsAugmentQuality::Silver)),
        2 => Ok(Some(CurrencyWarsAugmentQuality::Gold)),
        3 => Ok(Some(CurrencyWarsAugmentQuality::Prismatic)),
        _ => Err(error("Currency Wars investment offer quality is invalid")),
    }
}
