"""Released passive values with explicit assembly and lifetime policy."""


def append_curio_battle_stats(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (43, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9068; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2068",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (44, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2068; ParamList[0]=0.35; ParamList[1]=5",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (45, "text", "TextMap/TextMapEN.json; hash=16574523273451283809; all-allies SPD and discard after five battles",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.curio-battle-stat.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact current state, magnitude and battle-count limit; stacking, assembly scope and terminal-result counting remain separately authored policy.",
        ])
    data["CurioBattleStats"] = [[
        1, "du.curio-battle-stat.9068", "divergent-universe.curio-state.9068", "2068",
        "Speed", 1, "0.35", 2, 5, "BasePercentVerifiedBattleLifetime",
        "All allies gain 35% SPD; discard after five battles.", "我方全体速度提高 35%，五场战斗后丢弃。",
        "VersionedProjectPolicy: active holding with positive allowance at immutable battle assembly adds a source-attributed, non-dispellable PercentOfBase speed modifier to every player combatant and its linked unit subjects, not timeline-only actors. Existing build fields, sources and modifiers are preserved; enemy inputs are unchanged. The modifier participates in shared stat/timeline resolution from battle initialization, does not change the locked build and does not mutate Activity during battle. After a verified Won or Lost result, consume one active remaining battle after victory grants and before ordinary Blessing offer generation; discard all holding counters at zero. A trusted zero allowance contributes nothing and is discarded at the next counted result. Faulted/rejected results consume nothing. Destroyed holdings neither contribute nor consume allowance; repair resumes, replacement/reacquisition refills. Domain selections never consume this battle counter.",
        "Replace percent-of-base stacking, linked-unit propagation, initial activation, defeat/fault counting and end-of-battle ordering independently when released execution programs or reproducible observations establish them. Alternatives include total-speed scaling, roster-only applicability and victory-only counting. These timing/scope choices have low parity confidence. Keep exact 35% and five battles. Other states of identity 9068 retain independent pending dispositions.",
        "43|44|45",
    ]]
    for ordinal, suffix, locator, digest in [
        (52, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9073; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2073",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (53, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2073; ParamList[0]=0.5; ParamList[1]=5",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (54, "text", "TextMap/TextMapEN.json; hash=7393919144864761857; final damage increase and discard after five battles",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.curio-final-damage.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact state/effect/operand joins. Damage-class scope, stacking, snapshot and terminal counting remain explicit policy, not observed parity.",
        ])
    data["CurioBattleStats"].append([
        2, "du.curio-battle-stat.9073", "divergent-universe.curio-state.9073", "2073",
        "FinalDamage", 1, "0.5", 2, 5, "OutgoingFinalMultiplierVerifiedBattleLifetime",
        "Increase final damage by 50%; discard after five battles.", "最终伤害提高 50%，五场战斗后丢弃。",
        "VersionedProjectPolicy: an active positive-allowance holding contributes one source-attributed non-dispellable factor of 1.5 at shared DamageFinalMultiply for player Direct, DoT, Additional, Elation, Break and SuperBreak purposes, including linked unit subjects. Each purpose has an independent Product group; other final groups multiply in stable order. The factor applies to unfloored raw damage after the existing damage factors; use checked nearest-ties-even scalar multiplication and floor the resulting applied integer damage. Do not alter locked builds, ATK, damage-boost sums, enemy inputs or timeline-only actors. True damage and other source-modifier-bypassing copied damage stay unamplified; positive absolute damage overrides stay absolute. Continuing Break effects retain their original applier. Verified Won/Lost results consume one allowance; Faulted/rejected/duplicate results do not. Destroy/repair pauses/resumes; replacement and reacquisition refill five; zero allowance does not contribute and is discarded at the next counted result. Domains never count.",
        "Replace eligible classes, linked-unit scope, independent multiplier stacking, dynamic snapshot, bypass/absolute-override treatment and terminal counting independently when released execution programs or reproducible observations establish them. Alternatives include ordinary-only scope, amplifying true/copied damage, additive final bonuses and victory-only counting. Confidence is low for these hidden choices; exact 50% and five battles are retained. The public Tawot Cards page indexed on 2026-09-12 cross-checks operands but explicitly requests Arcadian Chronicles verification; it is not exact execution evidence. Tests must cover source/target separation, real HP damage, all admitted purposes, lifetime, destroy/repair/replacement and paid-service replay.",
        "52|53|54",
    ])
