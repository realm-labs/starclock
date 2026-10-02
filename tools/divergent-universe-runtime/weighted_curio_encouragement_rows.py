"""Current Encouragement for You operands; no battle execution credit."""


def append_weighted_curio_encouragement(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (118, "hex", "ExcelOutput/RogueTournHex.json; HexID=1008; TournMode=Tourn3; MazeBuffID=633408; AvatarType=Elation; DisplayID=1029",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (119, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633408; Lv=1; ParamList=1.5; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633408",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (120, "text-en", "TextMap/TextMapEN.json; hash=2693397682820933078; Elation character damage classification and Follow-Up CRIT DMG",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (121, "text-zh", "TextMap/TextMapCHS.json; hash=2693397682820933078; Elation character damage classification and Follow-Up CRIT DMG",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-encouragement.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-02",
            locator, digest, "ExactStructured",
            "Exact current Tourn3 eligibility and generic MazeBuff operand. Hidden damage-label routing, source ownership and formula interaction remain policy, not observed parity. Loading does not implement or admit this effect.",
        ])
    data["WeightedCurioEncouragements"] = [[
        1, "du.weighted-curio-encouragement.encouragement-for-you",
        "divergent-universe.weighted-curio.1008", "633408", "Elation", 1, "1.5",
        "OriginalElationFollowUpDamage",
        "Elation characters retain their Elation formula while their Elation damage also qualifies as follow-up damage; their follow-up critical damage gains an additive 150%.",
        "欢愉角色的欢愉伤害保留欢愉公式并同时满足追加攻击伤害判定，其追加攻击伤害的暴击伤害额外提高150%。",
        "VersionedProjectPolicy: original mapped party forms determine Elation Path eligibility before battle assembly. At BattleStarted/AfterEvent, once per battle per original owner, apply one permanent NonDispellable Replace effect with PersistByScope teardown. Ownership links, not presence, exclude linked recipients. Add FollowUp damage semantics to eligible owners' Elation hits without changing DamageClass::Elation, the Elation calculator, action kind/origin, hit count, costs or RNG grouping. Native follow-up hits also qualify. Add the canonical 1.5 ratio once to the qualifying damage operation's CritDamage stat query before ordinary or Elation critical multiplication. Both classification and critical-stat filters require an actual original-unit producer. Summons, memosprites, shared actors and unitless timeline actors cannot borrow eligibility through forms, bundles, forced effect attachment or formula-owner fallback. No bonus to noncritical damage, CritRate, Elation stat or ordinary DMG Boost. Classification creates no extra action, hit, damage event or reaction. Reconstruct ownership from accepted equipment for each battle, persist through waves and never write Activity state. Non-Elation owners, DoT, Break, Super Break and True damage receive no bonus. Data loading alone does not implement or admit the effect. Forge placement, equipment replay and complete-run release remain pending.",
        "Low confidence for original-owner scope, additive label propagation to selectors/events/modifiers, preservation of action identity, and scoped critical-stat timing. Released StageAbility_633408 is absent from the pinned Git object tree. Bounded public cross-check on 2026-10-02: https://honkai-star-rail.fandom.com/wiki/Encouragement_for_You supports only the current Arcadian Chronicles clauses; its older Remembrance/memosprite variant is excluded. Alternatives: replace rather than add labels, reclassify the whole action, inherit owner eligibility for linked actors, or apply a formula-stage bonus instead of a scoped critical stat. Data tests weighted_curio_encouragement validate canonical operands, policy/provenance and exact joins; they do not prove battle execution. Replace each hidden field independently when released programs or reproducible current observations establish it. Forge, equipment replay and full-run release remain pending.",
        "118|119|120|121",
    ]]
