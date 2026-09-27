"""Released Harmony shield operands; independently replaceable timing policy."""


def append_weighted_curio_shield(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (77, "hex", "ExcelOutput/RogueTournHex.json; HexID=1002; TournMode=Tourn3; MazeBuffID=633402; AvatarType=Shaman; DisplayID=1015",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (78, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633402; Lv=1; ParamList[0].Value=0.35; ParamList[1].Value=2; InBattleBindingKey=StageAbility_633402",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (79, "text-en", "TextMap/TextMapEN.json; hash=18303557854201536316; Harmony Basic/Skill/Ultimate on allies shields all allies using respective maximum HP for two turns",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (80, "text-zh", "TextMap/TextMapCHS.json; hash=18303557854201536316; Harmony Basic/Skill/Ultimate on allies shields all allies using respective maximum HP for two turns",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-shield.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-27",
            locator, digest, "ExactStructured",
            "Released current Hex join, Harmony eligibility, ally-directed action kinds, respective maximum-HP basis, 0.35 fraction and two-turn duration. Trigger phase, refresh, scope and coexistence are separate policy.",
        ])
    data["WeightedCurioShields"] = [[
        1, "du.weighted-curio-shield.silent-song", "divergent-universe.weighted-curio.1002",
        "633402", "Shaman", 1, "0.35", 2, 2, "AllyActionResolvedReplaceTargetTurnShield",
        "Harmony ally-directed Basic, Skill or Ultimate gives each living present ally a two-turn shield based on 35% of their own maximum HP.",
        "同谐角色对我方施放普攻、战技或终结技后，为在场存活队友提供各自生命上限 35% 的两回合护盾。",
        "VersionedProjectPolicy: bind action producers only to mapped player characters whose immutable build Path is Harmony. Observe ActionResolved at AfterAction once per action, require actor owner, Basic/Skill/Ultimate kind and nonempty committed allied targets; this includes self and team-wide actions without a controller primary. Affect current alive present team units in formation/stable-ID order, including the caster. Snapshot each recipient's current maximum HP at application; multiply by exact 0.35 with checked fixed-point floor through the shared shield formula (normal shield boosts remain applicable). Remove only this definition's prior shield/effect, then create one non-dispellable replacement shield with duration two at TargetTurnEnd; the shared clock includes an already-running recipient turn. Every mapped recipient owns expiry cleanup, independent of caster survival; delayed old-effect removal cannot clear a replacement. Other shields coexist using ConcurrentLargest absorption. No RNG, damage/heal-only inference, enemy-action activation or live Activity mutation. No implicit linked/new summon inheritance. Equipment remains Run-scoped; battle effects teardown with their battle/recipient.",
        "Hidden phase, committed-target versus declaration-target admission, current alive/present recipient scope, immediate target-turn tick, shield-boost handling, refresh across multiple Harmony casters and coexistence are low-confidence policy, not observed parity. Alternatives include declaration-time shield, independent per-caster layers or refresh-only duration. Replace each independently using released executable programs or reproducible current observations; preserve exact Harmony, action kinds, ally direction (including self/team-wide actions), per-recipient maximum HP, 0.35 and two turns. Forge admission and encoded equipment-command replay remain pending.",
        "77|78|79|80",
    ]]
