"""Released reaction operands with separately replaceable trigger policy."""


def append_curio_battle_reactions(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (46, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9069; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2069",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (47, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2069; ParamList[0]=0.2; ParamList[1]=5",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (48, "text", "TextMap/TextMapEN.json; hash=7916500230569937913; allied attacked target heals twenty percent of Max HP; discard after five battles",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.curio-battle-reaction.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact current-state healing magnitude and battle allowance. Released text does not establish multi-hit timing, shield-only hits, healing modifiers or source-owner attribution.",
        ])
    data["CurioBattleReactions"] = [[
        1, "du.curio-battle-reaction.9069", "divergent-universe.curio-state.9069", "2069",
        1, "0.2", 2, 5, "FirstSurvivingOrdinaryAttackPerTargetAction",
        "After an ally is attacked, restore 20% of their Max HP; discard after five battles.",
        "我方目标受到攻击后回复自身生命上限的 20%，五场战斗后丢弃。",
        "VersionedProjectPolicy: active positive-allowance holding installs one Mode-source Rule IR observer anchored to the lowest-formation player; anchor defeat does not disable the run-owned effect. At the shared AfterEvent drain, the first surviving ordinary Ability damage event from the opposing side for each target within an action heals that active allied target, including linked units. Shield-only hits count. DoT, Break, additional, true, HP-cost and friendly damage do not. Restore floor(current Max HP * 0.2) through fixed healing without outgoing/incoming healing modifiers; never revive or heal on battle entry. No RNG. End-of-battle allowance uses the same verified Won/Lost active-only settlement as battle stats; Faulted/rejected/duplicate results do not count. Destroy/repair pauses/resumes; replacement/reacquisition refills; domain entry never counts.",
        "Replace first-surviving-hit timing, eligible attack classes, shield handling, fixed-heal modifiers, stable player anchor and terminal-result counting independently with released execution evidence or reproducible observations. Alternatives include last-hit/after-action healing, only positive HP damage and formula-modified healing. These choices have low parity confidence. Bounded public cross-check https://honkai-star-rail.fandom.com/wiki/Tawot_Cards accessed 2026-09-12 confirms current operands but does not resolve timing; older entry-heal/domain-expiry variants are excluded. Original state producer remains separately pending.",
        "46|47|48",
    ]]
