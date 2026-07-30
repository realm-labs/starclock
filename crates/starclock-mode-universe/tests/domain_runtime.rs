use std::{collections::BTreeMap, sync::Arc};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    definition::{ActivityDomainDecision, DomainDecisionPolicy, DomainKind},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

#[derive(Clone, Copy)]
struct ExpectedDomain {
    key: &'static str,
    source_type: u32,
    kind: DomainKind,
    policy: DomainDecisionPolicy,
    terminal: bool,
    decision: ActivityDomainDecision,
    name_en: &'static str,
    name_zh_cn: &'static str,
}

const EXPECTED: [ExpectedDomain; 9] = [
    ExpectedDomain {
        key: "universe.domain.adventure",
        source_type: 9,
        kind: DomainKind::Adventure,
        policy: DomainDecisionPolicy::ExternalCommand,
        terminal: false,
        decision: ActivityDomainDecision::ExternalOutcome,
        name_en: "Domain — Adventure",
        name_zh_cn: "区域 - 冒险",
    },
    ExpectedDomain {
        key: "universe.domain.boss",
        source_type: 7,
        kind: DomainKind::Boss,
        policy: DomainDecisionPolicy::BattleHandoff,
        terminal: true,
        decision: ActivityDomainDecision::BattleCommand,
        name_en: "Domain — Boss",
        name_zh_cn: "区域 - 首领",
    },
    ExpectedDomain {
        key: "universe.domain.combat-primary",
        source_type: 1,
        kind: DomainKind::CombatPrimary,
        policy: DomainDecisionPolicy::BattleHandoff,
        terminal: false,
        decision: ActivityDomainDecision::BattleCommand,
        name_en: "Domain — Combat",
        name_zh_cn: "区域 - 战斗",
    },
    ExpectedDomain {
        key: "universe.domain.combat-secondary",
        source_type: 2,
        kind: DomainKind::CombatSecondary,
        policy: DomainDecisionPolicy::BattleHandoff,
        terminal: false,
        decision: ActivityDomainDecision::BattleCommand,
        name_en: "Domain — Combat",
        name_zh_cn: "区域 - 战斗",
    },
    ExpectedDomain {
        key: "universe.domain.elite",
        source_type: 6,
        kind: DomainKind::Elite,
        policy: DomainDecisionPolicy::BattleHandoff,
        terminal: false,
        decision: ActivityDomainDecision::BattleCommand,
        name_en: "Domain — Elite",
        name_zh_cn: "区域 - 精英",
    },
    ExpectedDomain {
        key: "universe.domain.encounter",
        source_type: 4,
        kind: DomainKind::Encounter,
        policy: DomainDecisionPolicy::ExternalCommand,
        terminal: false,
        decision: ActivityDomainDecision::RunCommand,
        name_en: "Domain — Encounter",
        name_zh_cn: "区域 - 遭遇",
    },
    ExpectedDomain {
        key: "universe.domain.occurrence",
        source_type: 3,
        kind: DomainKind::Occurrence,
        policy: DomainDecisionPolicy::ExternalCommand,
        terminal: false,
        decision: ActivityDomainDecision::RunCommand,
        name_en: "Domain — Occurrence",
        name_zh_cn: "区域 - 事件",
    },
    ExpectedDomain {
        key: "universe.domain.respite",
        source_type: 5,
        kind: DomainKind::Respite,
        policy: DomainDecisionPolicy::ExternalCommand,
        terminal: false,
        decision: ActivityDomainDecision::RunCommand,
        name_en: "Domain — Respite",
        name_zh_cn: "区域 - 休整",
    },
    ExpectedDomain {
        key: "universe.domain.transaction",
        source_type: 8,
        kind: DomainKind::Transaction,
        policy: DomainDecisionPolicy::ExternalCommand,
        terminal: false,
        decision: ActivityDomainDecision::RunCommand,
        name_en: "Domain — Transaction",
        name_zh_cn: "区域 - 交易",
    },
];

fn catalog() -> Arc<UniverseCatalog> {
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
}

#[test]
fn goal07_p5_m15_s19_materializes_all_nine_exact_domain_definitions() {
    let catalog = catalog();
    assert_eq!(catalog.domains().len(), EXPECTED.len());
    for (index, (domain, expected)) in catalog.domains().iter().zip(EXPECTED).enumerate() {
        assert_eq!(domain.id().get(), u32::try_from(index + 1).unwrap());
        assert_eq!(domain.stable_key(), expected.key);
        assert_eq!(domain.source_type(), expected.source_type);
        assert_eq!(domain.kind(), expected.kind);
        assert_eq!(domain.decision_policy(), expected.policy);
        assert_eq!(domain.is_terminal(), expected.terminal);
        assert_eq!(domain.text().name_en(), expected.name_en);
        assert_eq!(domain.text().name_zh_cn(), expected.name_zh_cn);
        let kind = expected.key.trim_start_matches("universe.domain.");
        assert_eq!(
            domain.text().summary_en(),
            format!(
                "Standard run domain kind {kind}; room content determines its concrete decision or battle."
            )
        );
        assert_eq!(
            domain.text().summary_zh_cn(),
            format!("标准运行区域类型“{kind}”，具体决策或战斗由房间内容决定。")
        );
    }
}

#[test]
fn goal07_p5_m15_s19_binds_each_domain_to_its_exact_activity_command() {
    let catalog = catalog();
    let activity = catalog.activity_binding();
    assert_eq!(
        activity.stable_key(),
        "universe.activity-binding.standard-main-world.v1"
    );
    assert_eq!(
        activity.activity_key(),
        "activity.standard-simulated-universe.v1"
    );
    assert_eq!(
        activity.battle_handoff_contract(),
        "activity.battle-handoff.rule-bundle.v1"
    );
    assert_eq!(
        activity.external_outcome_contract(),
        "activity.external-outcome.command.v1"
    );
    let bindings = activity
        .domains()
        .iter()
        .map(|binding| (binding.domain(), binding.decision()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(bindings.len(), EXPECTED.len());
    for expected in EXPECTED {
        let domain = catalog
            .domains()
            .iter()
            .find(|domain| domain.stable_key() == expected.key)
            .expect("frozen domain");
        assert_eq!(bindings.get(&domain.id()), Some(&expected.decision));
    }
}
