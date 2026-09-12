"""Released Sage's Leaf Robe acquisition values and explicit evolution policy."""


def append_sage_acquisitions(data: dict[str, list[list[object]]]) -> None:
    repository = "https://gitlab.com/Dimbreath/turnbasedgamedata"
    revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
    for source, name, locator, digest in [
        (32, "states", "ExcelOutput/RogueTournMiracle.json; MiracleID=9192,9193,9194; TournMode=Tourn3; effects=2192,2193,2194; base handbook=9154; upgrade handbook IDs absent", "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (33, "effects", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2192,2193,2194; ParamList[0]=1,2,3; acquisition counts only; ParamList[1] belongs to separate victory effects", "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (34, "text", "TextMap/TextMapEN.json; description hashes=18402293723530033109,6941038067972437758,1542688008986925663; acquisition rarities=1,2,3 respectively", "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (35, "names", "ExcelOutput/RogueMiracleDisplay.json; MiracleDisplayID=253,254,255; name hashes=3846558292214136969,13442696550144374710,904237867389681142", "5937214885a6f3c2f6fcc65870ab7bccd4e4493cffd682dbda75e63ac50ce739"),
    ]:
        data["Sources"].append([source, f"du.source.sage-{name}", repository, revision, "4.4", "2026-09-12", locator, digest,
                                "ExactStructured", "Exact released acquisition count, rarity and named form facts. Hidden pools and transition scheduling remain policy; victory effects and public Divine Treasure placement are not implemented by these rows."])
    for ordinal, state, rarity, name, name_zh in [
        (16, 9192, 1, "Dormant", "休眠"),
        (17, 9193, 2, "Awakened", "觉醒"),
        (18, 9194, 3, "Exalted", "升华"),
    ]:
        data["CurioAcquisitions"].append([
            ordinal, f"du.curio-acquisition.{state}", f"divergent-universe.curio-state.{state}", str(state - 7000), "RarityBlessings", 1, str(rarity),
            "FeasibleRarityAssignments",
            f"Sage's Leaf Robe ({name}) grants {rarity} random {rarity}-star Blessings on acquisition.",
            f"贤人叶袍（{name_zh}）获取时获得 {rarity} 个随机 {rarity} 星祝福。",
            "VersionedProjectPolicy: after inserting accepted Curio inventory, execute full acquisition rewards in stable state-key order. Use current unowned Blessing identities of the authored rarity with unit weights, without replacement across the complete batch. Before any draw, require a complete distinct assignment for all mandatory Path and rarity grants. At each draw, restrict candidates to those permitting the remaining mandatory grants, then sample uniformly in stable identity order. This is conditional sequential sampling, not uniform sampling of complete assignments. Coalesce mandatory Blessing grants after stable-order fragment effects, refresh Equation progress once, then execute separately authored bounded expansion rewards and merge allowance changes into final Curio holdings. Pending Blessing offers, dirty progress or exhausted mandatory pools reject atomically; never reduce the count or substitute another rarity. Victory grants and public evolution events have separate owners.",
            "Replace candidate membership, rarity weights, stable ordering, feasibility-conditioned sampling, exhaustion behavior and full versus difference-only evolution grants independently when released graphs or reproducible observations establish them. Alternatives include weighted draws, duplicate conversion and engine-defined reward order. Confidence in hidden scheduling and distribution is low; count and rarity remain exact released fields.",
            "32|33|34", None, rarity, rarity,
        ])
    for ordinal, before, after, event in [(3, 9192, 9193, 165), (4, 9193, 9194, 166)]:
        data["CurioEvolutions"].append([
            ordinal, f"du.curio-evolution.{before}.{after}", f"divergent-universe.curio-state.{before}", f"divergent-universe.curio-state.{after}",
            "divergent-universe.curio.9154", f"divergent-universe.occurrence.{event}", "ActiveSameOwnerResetAndAcquire",
            "Evolve Sage's Leaf Robe with one ownership identity and the successor's complete acquisition reward.",
            "升级贤人叶袍，保持单一持有身份并执行新形态完整获取奖励。",
            "VersionedProjectPolicy: map the released named Dormant/Awakened/Exalted tiers to consecutive event upgrades. Source-unbound target states receive separate runtime ownership aliases, never ordinary-pool membership. Require an active predecessor, replace all lifecycle counters with successor defaults, then grant the full new-state Blessing reward atomically. This trusted transition does not itself expose or authorize a public option; producers are separately authored in EvolutionEvents/EvolutionOptions. Original room reachability and the complete source event program remain pending.",
            "Replace current-profile event/edge binding, owner alias, active-only eligibility, counter reset and full acquisition-on-upgrade independently with released graphs or reproducible observations. Numeric adjacency and matching names are not exact membership evidence. Alternatives include standalone ownership, preserved counters and difference-only rewards; hidden transition confidence is low.",
            "26|32|33|34|35",
        ])
