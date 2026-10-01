"""Released The Story Presently operands, independent of hidden execution policy."""


def append_weighted_curio_support_attack(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (85, "hex", "ExcelOutput/RogueTournHex.json; HexID=1017; TournMode=Tourn3; MazeBuffID=633417; AvatarType=Shaman|Priest|Knight; DisplayID=1020",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (86, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633417; Lv=1; ParamList[0].Value=0.15; ParamList[1].Value=0.3; ParamList[2].Value=1; InBattleBindingKey=StageAbility_633417",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (87, "text-en", "TextMap/TextMapEN.json; hash=18107300873516931386; Harmony Abundance Preservation roster count grants their characters crit and attack additional damage from Max HP plus DEF plus ATK",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (88, "text-zh", "TextMap/TextMapCHS.json; hash=18107300873516931386; Harmony Abundance Preservation roster count grants their characters crit and attack additional damage from Max HP plus DEF plus ATK",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-support-attack.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Released Tourn3 join: three Paths; each roster character gives 0.15 crit rate and 0.3 crit damage to eligible characters; attack additional damage is 1 times Max HP plus DEF plus ATK. Hidden lifecycle and damage details remain policy.",
        ])
    data["WeightedCurioSupportAttacks"] = [[
        1, "du.weighted-curio-support-attack.the-story-presently",
        "divergent-universe.weighted-curio.1017", "633417", "Shaman|Priest|Knight",
        1, "0.15", 2, "0.3", 3, "1", "RosterCountCritAndAttackResolvedAdditional",
        "Each Harmony, Abundance or Preservation teammate grants those Paths 15% CRIT Rate and 30% CRIT DMG; their attacks add damage equal to Max HP plus DEF plus ATK.",
        "每名同谐、丰饶或存护队员使上述命途角色暴击率提高 15%、暴击伤害提高 30%；攻击后造成生命上限、防御力与攻击力之和的附加伤害。",
        "VersionedProjectPolicy: count the immutable mapped player character roster at battle assembly, including down/absent characters but excluding linked actors and summons. Bind only Harmony/Abundance/Preservation players. Two battle-lifetime flat Stat modifiers add count times exact crit operands, with unique-per-source groups and no extra cap (shared crit sampling bounds apply). Owner ActionResolved at AfterAction with Attack tag, once per action, visits committed opposing alive/present targets in formation/stable-ID order. Additional damage uses the owner's dynamic effective HP stat plus DEF plus ATK, multiplied by exact 1, shared nearest-ties-even arithmetic and final damage floor. Use actor Basic element, shared AdditionalDamage formula modifiers, no crit or toughness reduction, can defeat. No recursive action, new summon inheritance or live Activity mutation. Unequip affects later snapshots only.",
        "Hidden roster life/presence counting, phase, target cardinality, stat snapshot, element, critical eligibility and toughness are low-confidence policy, not observed parity. Alternatives: live eligible-count updates, once per hit, only primary target, application snapshot, attack-event element, crit-capable additional damage or toughness damage. Selected behavior preserves all displayed operands and bounds shared execution; affected tests weighted_curio_support_attack and production data rejection fixtures. Replace each field independently with released programs or reproducible current traces. Do not substitute Human Comedy 0.25/0.75 or Remembrance variants for this Tourn3 row; Forge admission and equipment replay remain pending.",
        "85|86|87|88",
    ]]
