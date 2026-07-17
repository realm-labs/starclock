mod generated;

use generated::{SoraConfig, effect::Effect, element::Element};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle_path = std::env::args().nth(1).ok_or("missing bundle path")?;
    let bytes = std::fs::read(bundle_path)?;
    let bundle = generated::runtime::SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;

    let ability = config.ability().get(&1001).ok_or("missing ability 1001")?;
    let indexed = config
        .ability()
        .get_by_stable_name("ability.golden-flare")
        .ok_or("missing unique-index result")?;
    assert_eq!(indexed.id, ability.id);
    assert!(matches!(ability.element, Element::Fire));
    assert!(matches!(
        ability.primary_effect,
        Effect::Damage { amount: 1250 }
    ));
    assert_eq!(ability.steps.len(), 2);
    assert_eq!(ability.steps[0].sequence, 1);
    assert!(matches!(
        ability.steps[0].effect,
        Effect::Damage { amount: 750 }
    ));
    assert_eq!(ability.steps[1].sequence, 2);
    assert!(matches!(
        ability.steps[1].effect,
        Effect::Heal { amount: 250 }
    ));
    assert_eq!(config.step_row().len(), 2);
    assert_eq!(config.excel_probe().len(), 0);

    println!(
        "loaded Sora golden ability {} with {} ordered steps",
        ability.id,
        ability.steps.len()
    );
    Ok(())
}
