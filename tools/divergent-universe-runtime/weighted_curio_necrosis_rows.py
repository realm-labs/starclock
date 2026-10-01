"""Released Mock Crimson Moon operands and replaceable timing policy."""


def append_weighted_curio_necrosis(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (103, "hex", "ExcelOutput/RogueTournHex.json; HexID=1003; TournMode=Tourn3; MazeBuffID=633403; AvatarType=Priest; DisplayID=1016",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (104, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633403; Lv=1; ParamList=1.5|6|3|2; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633403",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (105, "text-en", "TextMap/TextMapEN.json; hash=14457895563947008111; Abundance attacks inflict Necrosis, turn-start Fire DoT and other Burn immediate damage",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (106, "text-zh", "TextMap/TextMapCHS.json; hash=14457895563947008111; Abundance attacks inflict Necrosis, turn-start Fire DoT and other Burn immediate damage",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-necrosis.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Exact released eligibility and four MazeBuff operands. Attack cardinality, chance scheduling, snapshot, replacement and the immediate Burn clause's observation point remain policy, not observed parity.",
        ])
    data["WeightedCurioNecroses"] = [[
        1, "du.weighted-curio-necrosis.mock-crimson-moon", "divergent-universe.weighted-curio.1003",
        "633403", "Priest", 1, "1.5", 2, "6", 3, 3, 4, "2",
        "AttackResolvedNecrosisAndDotDamageBurnDetonation",
        "Abundance attacks can inflict three-turn Necrosis: 600% ATK Fire DoT at turn start, with other Burns producing 200% immediate damage.",
        "丰饶角色攻击可施加持续三回合的坏死：回合开始造成攻击力六倍的火属性持续伤害，并使其他灼烧立即造成两倍伤害。",
        "VersionedProjectPolicy: ActionResolved AfterAction priority zero once per action, original Abundance party owners only; distinct living/present opposing EventTargets in formation order, one resistible 1.5-base-chance roll per target with live EffectHitRate and EffectResistance, no authored specific-resistance stat. Successful applications replace the target's previous Necrosis even across casters, capture six times effective applier ATK with NearestTiesEven, one stack, dispellable, SourceSnapshotTargetDynamic, target TurnStart tick and TargetTurnEnd three-turn duration. On each Necrosis DamageApplied AfterEvent priority zero once per event, including externally detonated Necrosis, the original applier's bound rule detonates all other classified Burns at fraction two; ordinary and periodic Break stores share a stable effect-instance pool, exclude every Necrosis instance, retain original Burn sources, and floor only after full formula and fraction. No Burn detonation on application, resisted attempts, nonattack/allied/inactive targets or unrelated DoT; no recursive Necrosis/Burn chain, duration refresh or Activity mutation.",
        "Low confidence for attack cardinality, chance ordering, cross-caster replacement, dispel, source snapshot and the Burn clause's placement on Necrosis damage rather than application or turn-boundary notification. Public text and released operands do not expose the executable stage program. Bounded 2026-10-01 cross-checks: BWIKI oldid=81596 and Fandom Mock_Crimson_Moon; older 400% text is not current. Alternatives: per-hit application, source-independent stacking, dynamic ATK, application-only detonation or natural-tick-only detonation. Tests weighted_curio_necrosis cover both run families, multihit/multitarget, probabilities, refresh, turn ticks, mixed-store Burns, exclusions, independent sources, deterministic reconstruction, rejection and unequip. Replace fields independently when released StageAbility_633403 or reproducible current traces establish timing, lifetime, membership or snapshot; equipment Forge and full original-run coverage remain separate pending work.",
        "103|104|105|106",
    ]]
