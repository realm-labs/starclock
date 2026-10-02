"""Genius' Confusion operands and review policy; no battle-execution credit."""


def append_weighted_curio_excitation(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (114, "hex", "ExcelOutput/RogueTournHex.json; HexID=1006; TournMode=Tourn3; MazeBuffID=633406; AvatarDamageType=Quantum; DisplayID=1011",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (115, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633406; Lv=1; ParamList=2|1|2.5|0.5|1; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633406",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (116, "text-en", "TextMap/TextMapEN.json; hash=332970425552739522; Quantum Skill Point gain; team Excitation; Basic or Skill attack; Quantum Additional damage; Entanglement",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (117, "text-zh", "TextMap/TextMapCHS.json; hash=332970425552739522; Quantum Skill Point gain; team Excitation; Basic or Skill attack; Quantum Additional damage; Entanglement",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-excitation.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-02",
            locator, digest, "ExactStructured",
            "Exact Tourn3 Quantum eligibility and all five MazeBuff operands. Hidden gain attribution, timing, stacking and non-Break Entanglement semantics remain policy, not observed parity. Authoring does not implement this Curio.",
        ])
    data["WeightedCurioExcitations"] = [[
        1, "du.weighted-curio-excitation.genius-confusion", "divergent-universe.weighted-curio.1006",
        "633406", "Quantum", 1, 2, 2, 1, 3, "2.5", 4, "0.5", 5, 1, 65535,
        "EffectiveGainActionResolvedTeamConsumption",
        "Quantum allies gaining Skill Points grant two Excitation stacks to all allies. Basic or Skill attacks consume one team-wide stack to deal 250% ATK Quantum Additional damage and have a 50% base chance of one-turn Entanglement.",
        "量子属性我方获得战技点后，全体获得两层激发；普攻或战技攻击后消耗全体一层，造成250%攻击力量子附加伤害，并有50%基础概率施加一回合纠缠。",
        "VersionedProjectPolicy: original mapped party elements qualify Skill Point gain actors; exclude linked/shared actors even when Present. Observe positive effective Skill Point balance changes AfterMutation, once per resource event, granting two stacks per event rather than per point. Cap each living/present original party recipient at the authored policy maximum 65535; zero stacks at fresh battle entry, nondispellable, persist across waves and provider defeat, no Activity carry. At ActionResolved AfterAction priority zero, once per original party Basic/Skill action tagged Attack, require the living/present actor's positive Excitation; subtract up to one from each living/present original party recipient before damage. Resolve one actor-sourced Quantum Additional packet per surviving attacked target in formation order, using actor ATK queried at the trigger boundary times 2.5, ordinary target-dynamic stages and independent crit. Then attempt one Entanglement per surviving target through ordinary effect-hit/control-resistance and labeled battle RNG. Do not reclassify Additional damage as another action. Entanglement must retain delay, hit accumulation and delayed damage, not just a marker or action skip; use the reviewed shared Quantum base-effect calculation as replaceable policy without setting Weakness Broken, reducing Toughness, dealing initial Break damage or adding the universal Break delay. Refresh same-definition Entanglement across casters without repeating delay while active; cleanse removes it without expiry damage. This authoring batch does not implement or admit the effect.",
        "Low confidence for effective rather than attempted gains, cause-actor attribution, per-event rather than per-point gain, original-party scope, stack cap, gain-before-consumption ordering, actor stack admission, partial-team subtraction, source snapshots, crit, refresh and non-Break Entanglement internals. Pinned Git tree has no StageAbility_633406 program. Bounded 2026-10-02 public cross-check: https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Curio confirms current player-facing clauses only. Alternatives include overflow-triggered or per-point stacks, shared single counter, all-recipient positive-stack admission, independent caster Entanglement and repeat delay. Data fixtures weighted_curio_excitation verify operands, joins, cap-policy separation, provenance and rejection; they do not prove battle execution. Implement native Ordinary/Cyclical gain, cap, consumption, damage, control, cleanse, wave, provider-defeat and fresh-reconstruction fixtures before admission. Replace each unavailable field independently when released StageAbility_633406 or reproducible current observations establish it. Entanglement formula/lifecycle and equipment execution remain pending; Forge and full-run release credit remain pending.",
        "114|115|116|117",
    ]]
