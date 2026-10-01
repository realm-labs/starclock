"""Released Automated Experience operands and replaceable execution policy."""


def append_weighted_curio_attack_debuff(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (81, "hex", "ExcelOutput/RogueTournHex.json; HexID=1014; TournMode=Tourn3; MazeBuffID=633414; AvatarType=Warrior|Warlock; DisplayID=1021",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (82, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633414; Lv=1; ParamList[0].Value=0.2; ParamList[1].Value=0.3; ParamList[2].Value=1; InBattleBindingKey=StageAbility_633414",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (83, "text-en", "TextMap/TextMapEN.json; hash=4217722633224787979; Destruction or Nihility attack advances attacked enemy and reduces outgoing damage for one turn",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (84, "text-zh", "TextMap/TextMapCHS.json; hash=4217722633224787979; Destruction or Nihility attack advances attacked enemy and reduces outgoing damage for one turn",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-attack-debuff.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Released Tourn3 Hex join, both Paths, attack condition, attacked enemy direction, 0.2 advance, 0.3 reduction and one turn. Hidden phase, formula coverage, refresh and dispel remain separate policy.",
        ])
    data["WeightedCurioAttackDebuffs"] = [[
        1, "du.weighted-curio-attack-debuff.automated-experience",
        "divergent-universe.weighted-curio.1014", "633414", "Warrior|Warlock",
        1, "0.2", 2, "0.3", 3, 1, "AttackResolvedAdvanceAndTargetTurnFinalReduction",
        "After a Destruction or Nihility attack, attacked enemies advance 20% and deal 30% less damage for one turn.",
        "毁灭或虚无角色攻击后，受击敌人行动提前 20%，造成伤害降低 30%，持续一回合。",
        "VersionedProjectPolicy: bind only mapped Destruction/Nihility player producers. Observe ActionResolved at AfterAction with Attack tag, owner actor and committed opposing targets; once per action, not hit. Visit current living present targets in formation/stable-ID order. Advance each through shared AdvanceAction by exact 0.2; replace this definition's prior effect with one guaranteed non-dispellable Debuff, duration one at TargetTurnEnd (an already-running target turn counts). Six outgoing purposes OrdinaryDamage/Dot/AdditionalDamage/ElationDamage/Break/SuperBreak use a dynamic source-owned final product factor 1-0.3; no ATK or incoming-damage change. True damage, explicit damage overrides and deliberately unboosted operations retain their shared bypass semantics. Refresh replaces across casters, does not multiply reductions; unrelated modifiers coexist in separate groups. No resistance roll, RNG, recursive attack, implicit summon inheritance or live Activity mutation. Equipment is Run-scoped; effects and modifiers teardown with battle/target.",
        "Hidden phase, committed versus declared targets, dead/absent admission, formula position/category coverage, dispel, immediate turn tick and cross-caster replacement are low-confidence policy, not observed parity. Alternatives: declaration-time trigger, source Weaken aggregation (not final product), break exemption, dispellable debuff or independent caster layers. Chosen policy preserves both exact effects and bounded shared execution; affected tests weighted_curio_attack_debuff and its production data rejection fixtures. Replace each field independently with released executable programs or reproducible current traces; preserve both Paths, attack/enemy direction, 0.2/0.3/one turn. Forge menus and encoded equipment replay remain pending.",
        "81|82|83|84",
    ]]
