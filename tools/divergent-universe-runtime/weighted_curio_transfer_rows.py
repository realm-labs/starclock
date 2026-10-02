"""Released Dignity and Passion operands and separately replaceable timing."""


def append_weighted_curio_transfer(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (122, "hex", "ExcelOutput/RogueTournHex.json; HexID=1009; TournMode=Tourn3; MazeBuffID=633409; AvatarType=Knight; DisplayID=1002",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (123, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633409; Lv=1; ParamList=0.75,0.3,0.9,0.1; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633409",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (124, "text-en", "TextMap/TextMapEN.json; hash=14114601434821166552; Preservation team admission, other-shield transfer, owner-turn excess decay and healing from lost capacity",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (125, "text-zh", "TextMap/TextMapCHS.json; hash=14114601434821166552; Preservation team admission, other-shield transfer, owner-turn excess decay and healing from lost capacity",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-transfer.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-02",
            locator, digest, "ExactStructured",
            "Current Tourn3 joins and four exact decimal operands. Hidden ordering, birth scope, overlap, rounding and formula-modifier treatment are independent policy, not observed parity.",
        ])
    data["WeightedCurioTransfers"] = [[
        1, "du.weighted-curio-transfer.dignity-and-passion",
        "divergent-universe.weighted-curio.1009", "633409", "Knight",
        1, "0.75", 2, "0.3", 3, "0.9", 4, "0.1",
        "OtherShieldAppliedOwnerTurnExcessDecay",
        "A Preservation teammate enables an ally special shield. Other shields add 75% of their applied capacity. At its owner's turn start, 90% of capacity above 30% maximum HP is lost, healing for 10% of the actual loss.",
        "队伍中有存护角色时，我方获得特殊护盾；其他护盾实际获得量的75%转存至此护盾。本人回合开始时，超过生命上限30%的部分衰减90%，按实际衰减量的10%回复生命。",
        "VersionedProjectPolicy: immutable mapped original party Paths admit the whole party if any is Preservation, irrespective of later caster death or absence. Bind only original player recipients; no automatic linked/new summon inheritance. At BattleStarted/AfterEvent install a permanent NonDispellable Replace marker with initially zero capacity; retain through waves with PersistByScope. Each positive ordinary Shield Applied event targeting a living present owner transfers its actual post-formula applied capacity once at AfterMutation, including independent repeated or replacement grants, without reapplying shield bonuses. Ensure the marker exists before a transfer. Exact effect-owned capacity uses ConcurrentLargest coexistence with ordinary shields. At recipient TurnStarted/AfterEvent once per turn, read current special capacity and current maximum HP from one immutable snapshot; decrease floor(0.9 * max(capacity - 0.3 * maximum HP, 0)), retaining fractional threshold until the final integer boundary. A separate AfterMutation reaction filters Adjusted events for this definition and a negative actual delta, healing floor(0.1 * actual loss) without healing modifiers. Absorption, removal, other definitions, increases, actions and other recipients' turns cannot heal. Explicit marker removal clears only its shield when no replacement marker exists. Alive/present selectors prevent death from producing gains/decay/healing. Stable participant/event order, checked fixed-point arithmetic, no RNG and no Activity mutation. Equipment changes affect future battle snapshots only.",
        "Low confidence, not observed parity: original-party eligibility snapshot, birth/linked scope, event ordering, replacement-grant interpretation, ConcurrentLargest overlap, dynamic maximum HP, threshold precision, shield/healing modifier handling and marker teardown. Pinned StageAbility_633409 is absent from the filename tree; bounded public cross-check cannot establish hidden timing. Alternatives include additive absorption, per-caster layers, initial HP-based capacity, inheritance by summons, threshold flooring before decay, or boosted healing. Replace each independently with released programs or reproducible current traces. Data and both-family battle fixtures must cover exact operands, eligibility, repeated grants, zero/threshold/fractional decay, actual-loss-only healing, absorption, teardown and fresh deterministic commands. Original Forge admission, equipment replay and full-run release remain pending.",
        "122|123|124|125",
    ]]
