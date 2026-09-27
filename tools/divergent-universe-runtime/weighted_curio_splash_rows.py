"""Current Hex membership and generic MazeBuff facts; explicit copied-hit policy."""


def append_weighted_curio_splash(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (73, "hex", "ExcelOutput/RogueTournHex.json; HexID=1001; TournMode=Tourn3; MazeBuffID=633401; AvatarType=Rogue; DisplayID=1014",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (74, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633401; Lv=1; ParamList[0].Value=0.3; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633401",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (75, "text-en", "TextMap/TextMapEN.json; hash=8678628073262894449; Hunt attack damage copies thirty percent to adjacent targets",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (76, "text-zh", "TextMap/TextMapCHS.json; hash=8678628073262894449; Hunt attack damage copies thirty percent to adjacent targets",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-splash.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-27",
            locator, digest, "ExactStructured",
            "Exact current membership, generic MazeBuff join and operand. The missing RogueMazeBuff row is not absence of released MazeBuff data. Trigger cardinality and copied-damage execution are separate policy.",
        ])
    data["WeightedCurioSplashes"] = [[
        1, "du.weighted-curio-splash.ten-light-years", "divergent-universe.weighted-curio.1001",
        "633401", "Rogue", 1, "0.3", "HitCalculatedCopyAdjacentTrueDamage",
        "Hunt attack damage copies 30% of calculated damage to adjacent enemies.",
        "巡猎角色攻击造成伤害后，向相邻敌人复制计算伤害的 30%。",
        "VersionedProjectPolicy: bind only mapped player combatants whose immutable build Path is Hunt. Observe each positive ordinary Ability damage event with an action, once per event, at the shared AfterEvent phase. Copy its calculated integer damage, including shield-absorbed and overkill portions, multiplied by the exact 0.3 with checked fixed-point floor. Select alive present opposing units exactly one formation slot from the damaged target in current state, ordered by formation and stable ID. Emit shared TrueDamage operations, which can defeat and bypass source/target multipliers and critical rolls; no recursive splash, RNG or live Activity mutation. Multiple hits and multiple struck targets each retain their own event. A defeated primary remains the adjacency origin. Equipment lasts until replacement/unequip or Run teardown; each immutable battle assembles a fresh source-attributed rule. Linked or newly summoned unit inheritance is not implicit.",
        "Replace trigger cardinality, source-class/action eligibility, calculated versus effective-HP basis, damage category, shield/overkill handling, target presence/adjacency snapshot and summon inheritance independently when released executable programs or reproducible observations establish them. Alternatives include per-action aggregation and source-unboosted elemental additional damage, which would reapply target factors to already-calculated damage. Hidden timing/category confidence is low; this is not observed parity. Preserve released Hunt eligibility and thirty percent. Forge placement/offers and encoded equipment-command replay remain separate pending requirements.",
        "73|74|75|76",
    ]]
