"""Released Self-Amusement operands with explicit targeting/reaction policy."""


def append_weighted_curio_retaliation(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (93, "hex", "ExcelOutput/RogueTournHex.json; HexID=1013; TournMode=Tourn3; MazeBuffID=633413; AvatarDamageType=Physical; DisplayID=1026",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (94, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633413; Lv=1; ParamList[0].Value=4; ParamList[1].Value=0.3; InBattleBindingKey=StageAbility_633413",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (95, "text-en", "TextMap/TextMapEN.json; hash=6457248194266440334; Physical attacked-target weight increase and owner-ATK additional damage that cannot defeat enemies",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (96, "text-zh", "TextMap/TextMapCHS.json; hash=6457248194266440334; Physical attacked-target weight increase and owner-ATK additional damage that cannot defeat enemies",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
        (97, "avatar-path", "ExcelOutput/AvatarConfig.json; AvatarID=1001:Knight|1002:Rogue|1003:Mage|1004:Warlock|1008:Warrior|1009:Shaman|1105:Priest|1402:Memory|1501:Elation; released representative path joins",
         "c14584519e2feda70e9932501f7b79d67242d700585fe44119bbce9da9678e98"),
        (98, "avatar-aggro", "ExcelOutput/AvatarPromotionConfig.json; AvatarID=1001:150|1002:75|1003:75|1004:100|1008:125|1009:100|1105:100|1402:100|1501:100; Promotion=0..6; BaseAggro.Value; representative path weights",
         "4453f206d6b79658128f22ce4d923e2e608f92b48175ba5c913ed2be322d24c5"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-retaliation.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Released current Hex join, Physical eligibility, exact 4 and 0.3 operands and representative path BaseAggro values. Mapping the second operand, baseline targeting admission, trigger cardinality/timing, damage element/crit and linked inheritance remain field-level policy.",
        ])
    data["WeightedCurioRetaliations"] = [[
        1, "du.weighted-curio-retaliation.self-amusement", "divergent-universe.weighted-curio.1013",
        "633413", "Physical", 1, "4", 2, "0.3",
        "125", "75", "75", "100", "100", "150", "100", "100", "100",
        "PhysicalAggroAndOwnerAdditional",
        "Physical characters gain target weight; attacked survivors deal additional damage equal to 400% of their own ATK to the attacker, without defeating it.",
        "物理角色受击权重提高；受击后存活的角色对攻击者造成等同于自身攻击力 400% 的附加伤害，不能击败敌方。",
        "VersionedProjectPolicy: qualify mapped player characters by their immutable canonical Basic scaling element, with no linked/summon inheritance. Interpret the exact second operand 0.3 as a +30% percent-of-base Aggro modifier. While equipped, explicitly bind each mapped character's path baseline as one Aggro-purpose final multiplier after the normalized factor; weight is BaseAggro times (1 plus the Curio bonus and other applicable modifiers), never a second inferred multiplier. Representative path values are exact; applying the path-wide model is policy. Bind materialized enemies' unbound normal Basic/Skill Opposing Single/Blast abilities to the shared current-state weighted single-primary selector, formation order and aggro-target battle RNG. Preserve ability choice/order, explicitly bound selectors, All/self actions, queued/forced targeting and each later-hit invalidation policy. This does not rewrite other modes or assert unreconstructed enemy target-lock parity. Ordinary DamageApplied with Attack tag and an action, target equal to alive/present Physical owner and alive/present opposing actor: AfterEvent, priority zero, once per owner per Action, including fully shielded and zero-loss hits. Emit DamageFromOwner with trigger-snapshot effective owner ATK times 4, Physical Additional, no crit, no Toughness, cannot defeat. No extra action/turn, no attack proc on DoT, Break, Additional or reflected damage, and no retaliation from down/departed victims. Unequip removes future battle bindings through fresh assembly; no battle mutation of Activity.",
        "Low confidence for hidden mapping of 0.3, path-wide exceptions, normal-primary admission, hit/action cardinality, phase, shield/zero-hit admission, snapshot, Physical element, crit and linked inheritance; not observed parity. Alternatives include multiplicative weight bonus, per-hit reactions, after-action timing, effective-loss-only admission, crit-enabled or attacker-element damage and retaining unsupported first-target behavior. Selected policy executes both released clauses with the shared sampler, owner-sourced Additional formula and explicit nonlethal floor. Tests weighted_curio_retaliation and data rejection cases cover these fields. Replace each field independently when released executable selectors/programs or reproducible current traces establish it. Forge offers, equipment-command replay and complete original runs remain pending.",
        "93|94|95|96|97|98",
    ]]
