//! Compilation from validated data definitions into public combat/build catalogs.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use starclock_build::{
    ability::{AbilityLevel, AbilityLevelRow, AbilityLevelTable},
    catalog::{
        BuildCatalog, BuildCatalogBuilder, BuildCatalogRevision, CharacterBuildDefinition,
        CharacterStatRow,
    },
    eidolon::{EidolonDefinition as BuildEidolonDefinition, EidolonSetDefinition},
    id::{EidolonDefinitionId, TraceNodeId},
    light_cone::CombatPath,
    patch::BuildPatch,
    spec::{EidolonLevel, PromotionStage},
    trace::{TraceGraphDefinition, TraceNodeDefinition},
};
use starclock_combat::{
    AbilityId, Energy, Hp, ProgramId, Ratio, RawToughness, ResolvedDefinitionBindings, Rounding,
    RuleBundleId, SelectorId, SourceDefinitionId, Speed, StatValue, UnitDefinitionId,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, ScalingDamageDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition as CombatAbilityDefinition,
            AbilityParameterDefinition as CombatAbilityParameterDefinition,
            CharacterResourceDefinition, EffectDefinition, ProgramDefinition, RuleBundle,
            RuleDefinition, SelectorDefinition, UnitDefinition,
        },
    },
    rule::model::{RuleSource, RuleValue, SourceClass},
};

use crate::{
    build_lower::{BuildDefinitions, CharacterDataDefinition, DataBuildPatch},
    catalog::{
        CatalogLoadError, CombatDefinitions, IdentityDefinition, IdentityKind, LoadMode,
        domain_fail,
    },
    encounter_lower::EncounterDefinitions,
};

const VARIANT_BASE: u32 = 1_000_000_000;
const DEFAULT_PROGRAM_BASE: u32 = 1_600_000_000;
const ABILITY_SELECTOR_BASE: u32 = 1_700_000_000;

pub(super) fn compile(
    revision: &str,
    digest: [u8; 32],
    identities: &[IdentityDefinition],
    combat: &CombatDefinitions,
    builds: &BuildDefinitions,
    encounters: &EncounterDefinitions,
    mode: LoadMode,
) -> Result<(Arc<CombatCatalog>, BuildCatalog), CatalogLoadError> {
    let combat_catalog = compile_combat(
        revision, digest, identities, combat, builds, encounters, mode,
    )?;
    let build_catalog = compile_build(revision, builds, &combat_catalog)?;
    Ok((combat_catalog, build_catalog))
}

fn compile_combat(
    revision: &str,
    digest: [u8; 32],
    identities: &[IdentityDefinition],
    combat: &CombatDefinitions,
    builds: &BuildDefinitions,
    encounters: &EncounterDefinitions,
    mode: LoadMode,
) -> Result<Arc<CombatCatalog>, CatalogLoadError> {
    let mut builder = CombatCatalogBuilder::new(revision, digest);
    let variant_ids = variant_map(builds)?;
    let ultimate_costs = builds
        .characters
        .iter()
        .flat_map(|character| {
            character
                .abilities
                .iter()
                .map(|binding| (binding.ability, character.base_energy))
        })
        .collect::<BTreeMap<_, _>>();
    let ability_parameters = ability_parameter_map(builds)?;

    for selector in &combat.selectors {
        builder.add_selector(
            SelectorDefinition::new(selector.id).with_rule_units(selector.units.clone()),
        );
    }
    for ability in &combat.abilities {
        let selector = ability_selector(ability.id)?;
        builder.add_selector(
            SelectorDefinition::new(selector)
                .with_unit_targets(target_selector(ability.target_pattern)?),
        );
    }

    let mut existing_programs = BTreeSet::new();
    for program in &combat.programs {
        existing_programs.insert(program.id);
        builder.add_program(
            ProgramDefinition::new(
                program.id,
                Vec::new(),
                program.selectors.to_vec(),
                program.effects.to_vec(),
                Vec::new(),
            )
            .with_steps(program.steps.to_vec()),
        );
    }
    for ability in &combat.abilities {
        let program = ability_program(ability);
        if existing_programs.insert(program) {
            builder.add_program(ProgramDefinition::new(
                program,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));
        }
        add_combat_ability(
            &mut builder,
            ability.id,
            ability,
            program,
            ultimate_costs.get(&ability.id).copied(),
            &combat.hit_plans,
            &ability_parameters,
        )?;
        for variant in variant_ids.get(&ability.id).into_iter().flatten() {
            add_combat_ability(
                &mut builder,
                *variant,
                ability,
                program,
                ultimate_costs.get(&ability.id).copied(),
                &combat.hit_plans,
                &ability_parameters,
            )?;
        }
    }

    for group in combat.modifiers.groups() {
        builder.add_modifier_group(group.clone());
    }
    for modifier in combat.modifiers.definitions() {
        builder.add_modifier(modifier.clone());
    }
    for effect in &combat.effects {
        builder.add_effect(
            EffectDefinition::new(
                effect.id(),
                effect.rules().to_vec(),
                effect.modifiers().to_vec(),
            )
            .with_runtime_template(effect.runtime_template().clone()),
        );
    }
    for rule in &combat.rules {
        let mut programs = rule
            .runtime
            .triggers()
            .iter()
            .map(|trigger| trigger.program)
            .collect::<Vec<_>>();
        programs.sort_unstable();
        programs.dedup();
        builder.add_rule(
            RuleDefinition::new(rule.id, programs, Vec::new()).with_runtime(rule.runtime.clone()),
        );
        builder.add_rule_bundle(RuleBundle::new(
            RuleBundleId::new(rule.id.get()).expect("rule ID is nonzero"),
            vec![rule.id],
        ));
    }

    for linked in &combat.linked_units {
        builder.add_unit(UnitDefinition::new(
            linked.unit(),
            linked.abilities().to_vec(),
            linked.rule_bundles().to_vec(),
        ));
        builder.add_linked_unit(linked.clone());
    }
    for countdown in &combat.countdowns {
        builder.add_countdown(*countdown);
    }

    let mut unit_ids = BTreeSet::new();
    unit_ids.extend(
        combat
            .linked_units
            .iter()
            .map(|definition| definition.unit()),
    );
    for character in builds
        .characters
        .iter()
        .filter(|character| character.complete_progression_required || mode != LoadMode::Production)
    {
        let mut abilities = character
            .abilities
            .iter()
            .map(|binding| binding.ability)
            .collect::<Vec<_>>();
        for family in abilities.clone() {
            abilities.extend(variant_ids.get(&family).into_iter().flatten().copied());
        }
        abilities.sort_unstable();
        abilities.dedup();
        let mut rules = character
            .innate_rule_bundles
            .iter()
            .copied()
            .chain(character.build_rule_bundles())
            .collect::<Vec<_>>();
        rules.sort_unstable();
        rules.dedup();
        let mut resources = character
            .resources
            .iter()
            .map(|resource| {
                CharacterResourceDefinition::new(
                    resource.stable_key.clone(),
                    resource.initial,
                    resource.maximum,
                )
                .ok_or_else(|| domain_fail("invalid compiled character resource"))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        resources.sort_unstable_by(|left, right| left.stable_key().cmp(right.stable_key()));
        builder.add_unit(
            UnitDefinition::new(character.id, abilities, rules).with_resources(resources),
        );
        unit_ids.insert(character.id);
    }
    for identity in identities.iter().filter(|identity| {
        (identity.enabled || mode != LoadMode::Production)
            && identity.kind == IdentityKind::Character
    }) {
        let id = UnitDefinitionId::new(identity.id).expect("identity ID is nonzero");
        if unit_ids.insert(id) {
            builder.add_unit(UnitDefinition::new(id, Vec::new(), Vec::new()));
        }
    }
    for enemy in &encounters.enemies {
        if unit_ids.insert(enemy.unit()) {
            builder.add_unit(UnitDefinition::new(
                enemy.unit(),
                enemy.abilities().to_vec(),
                Vec::new(),
            ));
        }
        builder.add_enemy(enemy.clone());
    }
    for graph in &encounters.ai_graphs {
        builder.add_ai_graph(graph.clone());
    }
    for encounter in &encounters.encounters {
        builder.add_encounter(encounter.clone());
    }
    for parameter in builds
        .characters
        .iter()
        .flat_map(|character| character.ability_parameters.iter())
    {
        let level = u8::try_from(parameter.effective_level)
            .map_err(|_| domain_fail("ability parameter level exceeds u8"))?;
        let ability = variant_id(parameter.ability, level)?;
        builder.add_ability_parameter(
            CombatAbilityParameterDefinition::new(
                ability,
                parameter.parameter_key.clone(),
                RuleValue::Scalar(parameter.value),
            )
            .ok_or_else(|| domain_fail("invalid compiled ability parameter"))?,
        );
    }
    builder.build().map_err(domain_fail)
}

fn compile_build(
    revision: &str,
    builds: &BuildDefinitions,
    combat: &CombatCatalog,
) -> Result<BuildCatalog, CatalogLoadError> {
    let build_revision = BuildCatalogRevision::new(revision)
        .ok_or_else(|| domain_fail("empty build catalog revision"))?;
    let mut builder = BuildCatalogBuilder::new(build_revision, combat.revision().as_str())
        .ok_or_else(|| domain_fail("empty combat compatibility revision"))?;
    for character in builds
        .characters
        .iter()
        .filter(|character| character.complete_progression_required)
    {
        builder.add_character(character.compile(combat.digest().bytes())?);
    }
    builder.build(combat).map_err(domain_fail)
}

impl CharacterDataDefinition {
    fn compile(&self, digest: [u8; 32]) -> Result<CharacterBuildDefinition, CatalogLoadError> {
        let source = source(self.id.get(), SourceClass::Unit, digest)?;
        let stats = self
            .stats
            .iter()
            .map(|row| {
                Ok(CharacterStatRow::new(
                    starclock_combat::UnitLevel::new(
                        u8::try_from(row.level)
                            .map_err(|_| domain_fail("character level exceeds u8"))?,
                    )
                    .ok_or_else(|| domain_fail("character level exceeds combat domain"))?,
                    PromotionStage::new(row.promotion)
                        .ok_or_else(|| domain_fail("character promotion exceeds build domain"))?,
                    Hp::from_scalar(row.hp, Rounding::NearestTiesEven).map_err(domain_fail)?,
                    Speed::from_scaled(row.speed.scaled()).map_err(domain_fail)?,
                )
                .with_attack_defense(
                    StatValue::from_scaled(row.attack.scaled()).map_err(domain_fail)?,
                    StatValue::from_scaled(row.defense.scaled()).map_err(domain_fail)?,
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        let first = *stats
            .first()
            .ok_or_else(|| domain_fail("empty character stat curve"))?;
        let mut abilities = self
            .abilities
            .iter()
            .map(|binding| binding.ability)
            .collect::<Vec<_>>();
        abilities.sort_unstable();
        abilities.dedup();
        let bindings = ResolvedDefinitionBindings::new(
            abilities,
            self.innate_rule_bundles.to_vec(),
            Vec::new(),
        )
        .map_err(domain_fail)?;
        let mut definition =
            CharacterBuildDefinition::new(self.id, build_path(self.path)?, source, first, bindings)
                .with_stat_rows(stats)
                .with_ability_levels(self.compile_ability_levels()?);
        if !self.traces.is_empty() {
            definition = definition.with_trace_graph(self.compile_traces(digest)?);
        }
        if !self.eidolons.is_empty() {
            definition = definition.with_eidolons(self.compile_eidolons(digest)?);
        }
        Ok(definition)
    }

    fn compile_ability_levels(&self) -> Result<Vec<AbilityLevelTable>, CatalogLoadError> {
        self.abilities
            .iter()
            .map(|binding| {
                let cap = u8::try_from(binding.invested_level_cap)
                    .ok()
                    .and_then(AbilityLevel::new)
                    .ok_or_else(|| domain_fail("ability cap exceeds build domain"))?;
                let effective_cap = u8::try_from(binding.effective_level_cap)
                    .ok()
                    .and_then(AbilityLevel::new)
                    .ok_or_else(|| domain_fail("effective ability cap exceeds build domain"))?;
                let rows = (1..=effective_cap.get())
                    .map(|level| {
                        Ok(AbilityLevelRow::new(
                            AbilityLevel::new(level).expect("bounded level is nonzero"),
                            variant_id(binding.ability, level)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, CatalogLoadError>>()?;
                Ok(AbilityLevelTable::new(binding.ability, cap, rows))
            })
            .collect()
    }

    fn compile_traces(&self, digest: [u8; 32]) -> Result<TraceGraphDefinition, CatalogLoadError> {
        let nodes = self
            .traces
            .iter()
            .map(|trace| {
                Ok(TraceNodeDefinition::new(
                    TraceNodeId::new(trace.id).expect("Trace ID is nonzero"),
                    source(trace.id, SourceClass::Progression, digest)?,
                    trace
                        .prerequisites
                        .iter()
                        .map(|id| TraceNodeId::new(*id).expect("Trace ID is nonzero"))
                        .collect(),
                    PromotionStage::new(trace.promotion_requirement)
                        .ok_or_else(|| domain_fail("Trace promotion exceeds build domain"))?,
                    trace.patches.iter().copied().map(build_patch).collect(),
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        Ok(TraceGraphDefinition::new(self.id, nodes))
    }

    fn compile_eidolons(&self, digest: [u8; 32]) -> Result<EidolonSetDefinition, CatalogLoadError> {
        let ranks = self
            .eidolons
            .iter()
            .map(|eidolon| {
                Ok(BuildEidolonDefinition::new(
                    EidolonDefinitionId::new(eidolon.id).expect("Eidolon ID is nonzero"),
                    source(eidolon.id, SourceClass::Progression, digest)?,
                    EidolonLevel::new(eidolon.rank)
                        .ok_or_else(|| domain_fail("Eidolon rank exceeds build domain"))?,
                    eidolon.patches.iter().copied().map(build_patch).collect(),
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        Ok(EidolonSetDefinition::new(self.id, ranks))
    }

    fn build_rule_bundles(&self) -> impl Iterator<Item = RuleBundleId> + '_ {
        self.traces
            .iter()
            .flat_map(|trace| trace.patches.iter())
            .chain(
                self.eidolons
                    .iter()
                    .flat_map(|eidolon| eidolon.patches.iter()),
            )
            .filter_map(|patch| match patch {
                DataBuildPatch::AddRule(id) | DataBuildPatch::RemoveRule(id) => Some(*id),
                _ => None,
            })
    }
}

fn add_combat_ability(
    builder: &mut CombatCatalogBuilder,
    id: AbilityId,
    source: &crate::catalog::AbilityDefinition,
    program: ProgramId,
    ultimate_cost: Option<starclock_combat::Scalar>,
    hit_plans: &[crate::catalog::HitPlanDefinition],
    ability_parameters: &BTreeMap<AbilityId, BTreeMap<Box<str>, starclock_combat::Scalar>>,
) -> Result<(), CatalogLoadError> {
    let selector = ability_selector(source.id)?;
    let mut action = AbilityActionDefinition::new(
        action_kind(source.kind),
        1,
        invalidation(source.retarget_policy),
        action_resources(source, ultimate_cost)?,
    )
    .ok_or_else(|| domain_fail("invalid structural ability action"))?
    .with_tags(&ability_tags(source.semantic_tags));
    if !source.hit_plan_bindings.is_empty() {
        let hits = source
            .hit_plan_bindings
            .iter()
            .map(|binding| {
                hit_plans
                    .binary_search_by_key(&binding.hit_plan_id, |plan| plan.id)
                    .ok()
                    .map(|index| (binding, &hit_plans[index]))
                    .ok_or_else(|| domain_fail("ability hit plan disappeared during compilation"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|(binding, plan)| plan.hits.iter().map(move |hit| (binding, hit)))
            .map(|(binding, hit)| {
                let mut operations = Vec::new();
                if let Some(key) = binding.damage_parameter_key.as_deref() {
                    let coefficient = ability_parameters
                        .get(&id)
                        .and_then(|parameters| parameters.get(key))
                        .ok_or_else(|| domain_fail("ability hit formula parameter is missing"))?;
                    let coefficient = Ratio::from_scaled(coefficient.scaled())
                        .checked_mul(hit.damage_ratio, Rounding::NearestTiesEven)
                        .map_err(domain_fail)?;
                    operations.push(HitOperationDefinition::ScalingDamage(
                        ScalingDamageDefinition::new(
                            binding.damage_scaling_stat.expect("validated damage stat"),
                            coefficient,
                            binding.damage_class.expect("validated damage class"),
                            binding.element.expect("validated damage element"),
                        )
                        .map_err(domain_fail)?,
                    ));
                }
                if let Some(base) = binding.base_toughness {
                    let scaled = hit
                        .toughness_ratio
                        .checked_apply(base, Rounding::NearestTiesEven)
                        .map_err(domain_fail)?;
                    let base =
                        RawToughness::from_scalar(scaled, Rounding::Floor).map_err(domain_fail)?;
                    operations.push(HitOperationDefinition::ReduceToughness(
                        toughness_reduction(
                            binding.element.expect("validated Toughness element"),
                            base,
                        ),
                    ));
                }
                Ok(ActionHitDefinition::new(operations).with_profile(
                    hit_target_group(hit.target_group),
                    hit.damage_ratio,
                    hit.toughness_ratio,
                    hit_crit_policy(hit.crit_policy),
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        action = action
            .with_hits(hits)
            .ok_or_else(|| domain_fail("compiled ability hit count exceeds combat bounds"))?;
    }
    let programs = source
        .phases
        .iter()
        .filter_map(|phase| phase.program.map(|program| (phase, program)))
        .map(|(phase, program)| {
            AbilityProgramBinding::new(
                phase.sequence,
                match phase.kind {
                    0 => AbilityProgramTiming::Entry,
                    1 => AbilityProgramTiming::BeforeHits,
                    2 => AbilityProgramTiming::Hits,
                    3 => AbilityProgramTiming::AfterHits,
                    4 => AbilityProgramTiming::Resolved,
                    _ => unreachable!("validated ability phase kind"),
                },
                program,
            )
            .expect("validated ability phase sequence")
        })
        .collect();
    builder.add_ability(
        CombatAbilityDefinition::new(id, program, selector, Vec::new())
            .with_action(action)
            .with_programs(programs),
    );
    Ok(())
}

fn hit_target_group(value: u8) -> HitTargetGroup {
    match value {
        0 => HitTargetGroup::Primary,
        1 => HitTargetGroup::Adjacent,
        2 => HitTargetGroup::Selected,
        3 => HitTargetGroup::All,
        4 => HitTargetGroup::BounceDraw,
        5 => HitTargetGroup::SelfTarget,
        _ => unreachable!("validated hit target group"),
    }
}

fn hit_crit_policy(value: u8) -> HitCritPolicy {
    match value {
        0 => HitCritPolicy::PerTarget,
        1 => HitCritPolicy::Shared,
        2 => HitCritPolicy::Never,
        _ => unreachable!("validated hit CRIT policy"),
    }
}

fn toughness_reduction(
    element: starclock_combat::formula::model::CombatElement,
    base: RawToughness,
) -> starclock_combat::ToughnessReductionDefinition {
    use starclock_combat::formula::toughness::{BreakDamageDefinition, ToughnessReductionContext};
    starclock_combat::ToughnessReductionDefinition {
        element,
        reduction: ToughnessReductionContext {
            base,
            additive: RawToughness::new(0).expect("zero is valid"),
            reduction_increase: starclock_combat::Ratio::ZERO,
            weakness_break_efficiency: starclock_combat::Ratio::ZERO,
            weakness_break_efficiency_cap: starclock_combat::Ratio::from_scaled(3_000_000),
            toughness_vulnerability: starclock_combat::Ratio::ZERO,
            ability_multiplier: starclock_combat::Ratio::ONE,
        },
        break_damage: BreakDamageDefinition {
            attacker_level_multiplier: starclock_combat::Scalar::ONE,
            ability_multiplier: starclock_combat::Ratio::ONE,
            break_effect: starclock_combat::Ratio::ZERO,
            break_damage_increase: starclock_combat::Ratio::ZERO,
            defense_multiplier: starclock_combat::Ratio::ONE,
            resistance_multiplier: starclock_combat::Ratio::ONE,
            vulnerability_multiplier: starclock_combat::Ratio::ONE,
            mitigation_multiplier: starclock_combat::Ratio::ONE,
            unbroken_multiplier: starclock_combat::Ratio::ONE,
        },
        break_effect_chance: starclock_combat::Probability::ONE,
    }
}

fn variant_map(
    builds: &BuildDefinitions,
) -> Result<BTreeMap<AbilityId, Vec<AbilityId>>, CatalogLoadError> {
    let mut output = BTreeMap::new();
    for binding in builds
        .characters
        .iter()
        .flat_map(|character| character.abilities.iter())
    {
        let levels = (2..=binding.effective_level_cap)
            .map(|level| {
                let level =
                    u8::try_from(level).map_err(|_| domain_fail("ability level exceeds u8"))?;
                variant_id(binding.ability, level)
            })
            .collect::<Result<Vec<_>, _>>()?;
        output.insert(binding.ability, levels);
    }
    Ok(output)
}

fn ability_parameter_map(
    builds: &BuildDefinitions,
) -> Result<BTreeMap<AbilityId, BTreeMap<Box<str>, starclock_combat::Scalar>>, CatalogLoadError> {
    let mut output = BTreeMap::<AbilityId, BTreeMap<Box<str>, starclock_combat::Scalar>>::new();
    for parameter in builds
        .characters
        .iter()
        .flat_map(|character| character.ability_parameters.iter())
    {
        let level = u8::try_from(parameter.effective_level)
            .map_err(|_| domain_fail("ability parameter level exceeds u8"))?;
        output
            .entry(variant_id(parameter.ability, level)?)
            .or_default()
            .insert(parameter.parameter_key.clone(), parameter.value);
    }
    Ok(output)
}

fn variant_id(family: AbilityId, level: u8) -> Result<AbilityId, CatalogLoadError> {
    if level == 1 {
        return Ok(family);
    }
    let raw = VARIANT_BASE
        .checked_add(
            family
                .get()
                .checked_mul(32)
                .ok_or_else(|| domain_fail("ability variant ID overflow"))?,
        )
        .and_then(|value| value.checked_add(u32::from(level)))
        .ok_or_else(|| domain_fail("ability variant ID overflow"))?;
    AbilityId::new(raw).ok_or_else(|| domain_fail("ability variant ID is zero"))
}

fn ability_program(ability: &crate::catalog::AbilityDefinition) -> ProgramId {
    ability
        .phases
        .iter()
        .find_map(|phase| phase.program)
        .unwrap_or_else(|| {
            ProgramId::new(DEFAULT_PROGRAM_BASE + ability.id.get())
                .expect("bounded ability ID produces nonzero program ID")
        })
}

fn ability_selector(ability: AbilityId) -> Result<SelectorId, CatalogLoadError> {
    SelectorId::new(
        ABILITY_SELECTOR_BASE
            .checked_add(ability.get())
            .ok_or_else(|| domain_fail("ability selector ID overflow"))?,
    )
    .ok_or_else(|| domain_fail("ability selector ID is zero"))
}

fn target_selector(pattern: u8) -> Result<UnitTargetSelector, CatalogLoadError> {
    let (relation, pattern, repeated) = match pattern {
        0 => (TargetRelation::Opposing, TargetPattern::Single, false),
        1 => (TargetRelation::Opposing, TargetPattern::Blast, false),
        2 => (TargetRelation::Opposing, TargetPattern::All, false),
        3 => (TargetRelation::Opposing, TargetPattern::Single, true),
        4..=6 => (TargetRelation::SelfUnit, TargetPattern::Single, false),
        7 => (TargetRelation::Opposing, TargetPattern::Single, false),
        _ => return Err(domain_fail("unknown target pattern")),
    };
    let selector = UnitTargetSelector::new(relation, pattern)
        .ok_or_else(|| domain_fail("invalid unit target selector"))?;
    Ok(if repeated {
        selector.with_repeated_targets()
    } else {
        selector
    })
}

fn action_kind(kind: u8) -> AbilityKind {
    match kind {
        0 | 5 => AbilityKind::Basic,
        1 | 6 => AbilityKind::Skill,
        2 => AbilityKind::Ultimate,
        7 => AbilityKind::FollowUp,
        8 => AbilityKind::Counter,
        9 => AbilityKind::Summon,
        10 => AbilityKind::Memosprite,
        13 => AbilityKind::Countdown,
        _ => AbilityKind::ExtraAction,
    }
}

fn invalidation(policy: u8) -> TargetInvalidationPolicy {
    match policy {
        0 => TargetInvalidationPolicy::KeepIfPresent,
        1 => TargetInvalidationPolicy::CancelRemainingForTarget,
        2 => TargetInvalidationPolicy::RetargetSamePool,
        3 => TargetInvalidationPolicy::RetargetPrimaryThenRebuildPattern,
        _ => TargetInvalidationPolicy::FailAction,
    }
}

fn ability_tags(tags: starclock_combat::catalog::action::AbilityTags) -> Vec<AbilityTag> {
    [
        AbilityTag::Attack,
        AbilityTag::Basic,
        AbilityTag::Skill,
        AbilityTag::Ultimate,
        AbilityTag::FollowUp,
        AbilityTag::Counter,
        AbilityTag::Summon,
        AbilityTag::Memosprite,
        AbilityTag::AdditionalDamage,
        AbilityTag::Joint,
        AbilityTag::ElationSkill,
    ]
    .into_iter()
    .filter(|tag| tags.contains(*tag))
    .collect()
}

fn action_resources(
    source: &crate::catalog::AbilityDefinition,
    ultimate_cost: Option<starclock_combat::Scalar>,
) -> Result<ActionResourcePolicy, CatalogLoadError> {
    let mut sp_cost = 0_u16;
    let mut sp_gain = 0_u16;
    let mut energy_cost = Energy::ZERO;
    let mut energy_gain = Energy::ZERO;
    for resource in &source.resources {
        match (resource.resource_kind, resource.delta_kind) {
            (1, 0) => sp_cost = scalar_u16(resource.amount)?,
            (1, 2) => sp_gain = scalar_u16(resource.amount)?,
            (0, 0) => {
                energy_cost = Energy::from_scaled(resource.amount.scaled()).map_err(domain_fail)?
            }
            (0, 2) => {
                energy_gain = Energy::from_scaled(resource.amount.scaled()).map_err(domain_fail)?
            }
            _ => {}
        }
    }
    if source.kind == 2 && energy_cost == Energy::ZERO {
        energy_cost =
            Energy::from_scaled(ultimate_cost.map_or(1_000_000, starclock_combat::Scalar::scaled))
                .map_err(domain_fail)?;
    }
    Ok(ActionResourcePolicy::new(
        sp_cost,
        sp_gain,
        energy_cost,
        energy_gain,
    ))
}

fn scalar_u16(value: starclock_combat::Scalar) -> Result<u16, CatalogLoadError> {
    u16::try_from(
        value
            .rounded_integer(Rounding::NearestTiesEven)
            .map_err(domain_fail)?,
    )
    .map_err(domain_fail)
}

fn build_path(path: u8) -> Result<CombatPath, CatalogLoadError> {
    Ok(match path {
        0 => CombatPath::Destruction,
        1 => CombatPath::Hunt,
        2 => CombatPath::Erudition,
        3 => CombatPath::Harmony,
        4 => CombatPath::Nihility,
        5 => CombatPath::Preservation,
        6 => CombatPath::Abundance,
        7 => CombatPath::Remembrance,
        8 => CombatPath::Elation,
        _ => return Err(domain_fail("unknown character path")),
    })
}

fn build_patch(patch: DataBuildPatch) -> BuildPatch {
    match patch {
        DataBuildPatch::AddRule(id) => BuildPatch::AddRuleBundle(id),
        DataBuildPatch::RemoveRule(id) => BuildPatch::RemoveRuleBundle(id),
        DataBuildPatch::AddModifier(id) => BuildPatch::AddModifier(id),
        DataBuildPatch::AddAbility(id) => BuildPatch::AddAbility(id),
        DataBuildPatch::ReplaceAbility { old, new } => BuildPatch::ReplaceAbility { old, new },
        DataBuildPatch::AdjustAbilityLevel {
            ability,
            bonus,
            cap_delta,
        } => BuildPatch::AdjustAbilityLevel {
            family: ability,
            bonus,
            cap_delta,
        },
    }
}

fn source(id: u32, class: SourceClass, digest: [u8; 32]) -> Result<RuleSource, CatalogLoadError> {
    Ok(RuleSource::new(
        SourceDefinitionId::new(id).ok_or_else(|| domain_fail("source ID is zero"))?,
        class,
        Vec::new(),
        digest,
    ))
}

#[cfg(test)]
mod tests {
    use starclock_build::{
        ability::{AbilityInvestment, AbilityLevel},
        catalog::{BuildCatalogBuilder, BuildCatalogRevision},
        compiler::LoadoutCompiler,
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    };
    use starclock_combat::{
        Energy, ModifierDefinitionId, ModifierStackingGroupId, ProgramId, RuleId, Scalar,
        SelectorId, UnitLevel,
        catalog::{
            action::{
                AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
                TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
            },
            definition::{
                AbilityDefinition, ProgramDefinition, RuleBundle, RuleDefinition,
                SelectorDefinition, UnitDefinition,
            },
        },
        modifier::model::{
            FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
            ModifierStackingGroup, SnapshotPolicy, StatKind,
        },
        rule::model::{RuleValue, ValueExpr},
    };

    use super::*;
    use crate::build_lower::{
        CharacterAbilityDefinition, CharacterStatDefinition, EidolonDefinition, TraceDefinition,
    };

    #[test]
    fn character_rows_compile_every_executable_patch_through_e0_and_e6() {
        let digest = [0x42; 32];
        let combat = combat_catalog(digest);
        let character = character_definition();
        let compiled = character.compile(digest).unwrap();
        let mut builder = BuildCatalogBuilder::new(
            BuildCatalogRevision::new("data-domain-test").unwrap(),
            "data-domain-test",
        )
        .unwrap();
        builder.add_character(compiled);
        let builds = builder.build(&combat).unwrap();

        let e0 = LoadoutCompiler
            .compile(&builds, &combat, &build_spec(0, false))
            .unwrap();
        assert_eq!(
            e0.combatant().abilities(),
            &[variant_id(ability(1), 2).unwrap()]
        );
        assert_eq!(e0.combatant().rule_bundles(), &[bundle(1)]);
        assert!(e0.combatant().modifiers().is_empty());

        let e6 = LoadoutCompiler
            .compile(&builds, &combat, &build_spec(6, true))
            .unwrap();
        assert_eq!(
            e6.combatant().abilities(),
            &[ability(7), variant_id(ability(1), 4).unwrap()]
        );
        assert_eq!(e6.combatant().rule_bundles(), &[bundle(2)]);
        assert_eq!(e6.combatant().modifiers(), &[modifier(1), modifier(2)]);
    }

    fn character_definition() -> CharacterDataDefinition {
        CharacterDataDefinition {
            id: form(1),
            rarity: 5,
            path: 3,
            element: 1,
            base_energy: Scalar::from_scaled(120_000_000),
            base_aggro: Scalar::from_scaled(100_000_000),
            stats: vec![CharacterStatDefinition {
                level: 80,
                promotion: 6,
                hp: Scalar::from_scaled(1_000_000_000),
                attack: Scalar::from_scaled(500_000_000),
                defense: Scalar::from_scaled(400_000_000),
                speed: Scalar::from_scaled(100_000_000),
            }]
            .into_boxed_slice(),
            abilities: vec![CharacterAbilityDefinition {
                sequence: 1,
                slot: 0,
                ability: ability(1),
                invested_level_cap: 2,
                effective_level_cap: 4,
            }]
            .into_boxed_slice(),
            innate_rule_bundles: vec![bundle(1)].into_boxed_slice(),
            resources: Box::new([]),
            ability_parameters: Box::new([]),
            traces: vec![
                TraceDefinition {
                    id: 10,
                    kind: 1,
                    promotion_requirement: 0,
                    prerequisites: Box::new([]),
                    patches: vec![
                        DataBuildPatch::AddAbility(ability(6)),
                        DataBuildPatch::ReplaceAbility {
                            old: ability(6),
                            new: ability(7),
                        },
                        DataBuildPatch::AddModifier(modifier(1)),
                    ]
                    .into_boxed_slice(),
                },
                TraceDefinition {
                    id: 20,
                    kind: 0,
                    promotion_requirement: 1,
                    prerequisites: vec![10].into_boxed_slice(),
                    patches: vec![DataBuildPatch::AddRule(bundle(2))].into_boxed_slice(),
                },
            ]
            .into_boxed_slice(),
            eidolons: (1..=6)
                .map(|rank| EidolonDefinition {
                    id: 100 + u32::from(rank),
                    rank,
                    patches: match rank {
                        1 => vec![DataBuildPatch::RemoveRule(bundle(1))],
                        3 | 5 => vec![DataBuildPatch::AdjustAbilityLevel {
                            ability: ability(1),
                            bonus: 1,
                            cap_delta: 1,
                        }],
                        6 => vec![DataBuildPatch::AddModifier(modifier(2))],
                        _ => Vec::new(),
                    }
                    .into_boxed_slice(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            complete_progression_required: true,
        }
    }

    fn combat_catalog(digest: [u8; 32]) -> Arc<CombatCatalog> {
        let mut builder = CombatCatalogBuilder::new("data-domain-test", digest);
        builder.add_selector(SelectorDefinition::new(selector(1)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            program(1),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
        let abilities = [ability(1), ability(6), ability(7)]
            .into_iter()
            .chain((2..=4).map(|level| variant_id(ability(1), level).unwrap()))
            .collect::<Vec<_>>();
        for id in &abilities {
            builder.add_ability(
                AbilityDefinition::new(*id, program(1), selector(1), Vec::new())
                    .with_action(basic_action()),
            );
        }
        for raw in 1..=2 {
            builder.add_rule(RuleDefinition::new(rule(raw), Vec::new(), Vec::new()));
            builder.add_rule_bundle(RuleBundle::new(bundle(raw), vec![rule(raw)]));
            builder.add_modifier_group(ModifierStackingGroup {
                id: ModifierStackingGroupId::new(raw).unwrap(),
                aggregation: ModifierAggregation::Sum,
            });
            builder.add_modifier(ModifierDefinition {
                id: modifier(raw),
                stat: StatKind::Atk,
                stage: FormulaStage::DamageBoost,
                purpose: FormulaPurpose::OrdinaryDamage,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
                stacking_group: ModifierStackingGroupId::new(raw).unwrap(),
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                filters: Box::new([]),
            });
        }
        builder.add_unit(UnitDefinition::new(
            form(1),
            abilities,
            vec![bundle(1), bundle(2)],
        ));
        builder.build().unwrap()
    }

    fn basic_action() -> AbilityActionDefinition {
        AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(vec![ActionHitDefinition::new(Vec::new())])
        .unwrap()
    }

    fn build_spec(eidolon: u8, traces: bool) -> CombatantBuildSpec {
        let spec = CombatantBuildSpec::new(
            form(1),
            UnitLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .with_ability_levels(vec![AbilityInvestment::new(
            ability(1),
            AbilityLevel::new(2).unwrap(),
        )])
        .unwrap()
        .with_eidolon(EidolonLevel::new(eidolon).unwrap());
        if traces {
            spec.with_traces(vec![
                TraceNodeId::new(10).unwrap(),
                TraceNodeId::new(20).unwrap(),
            ])
            .unwrap()
        } else {
            spec
        }
    }

    fn ability(raw: u32) -> AbilityId {
        AbilityId::new(raw).unwrap()
    }
    fn form(raw: u32) -> UnitDefinitionId {
        UnitDefinitionId::new(raw).unwrap()
    }
    fn modifier(raw: u32) -> ModifierDefinitionId {
        ModifierDefinitionId::new(raw).unwrap()
    }
    fn bundle(raw: u32) -> RuleBundleId {
        RuleBundleId::new(raw).unwrap()
    }
    fn rule(raw: u32) -> RuleId {
        RuleId::new(raw).unwrap()
    }
    fn program(raw: u32) -> ProgramId {
        ProgramId::new(raw).unwrap()
    }
    fn selector(raw: u32) -> SelectorId {
        SelectorId::new(raw).unwrap()
    }
}
