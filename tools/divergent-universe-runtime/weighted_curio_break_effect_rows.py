"""Released Converse of Entropy with explicit entry snapshot policy."""


def append_weighted_curio_break_effect(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (99, "hex", "ExcelOutput/RogueTournHex.json; HexID=1005; TournMode=Tourn3; MazeBuffID=633405; AvatarDamageType=Wind|Thunder; DisplayID=1018",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (100, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633405; Lv=1; ParamList[0].Value=1.2; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633405",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (101, "text-en", "TextMap/TextMapEN.json; hash=6003923322476481427; entry raises Wind/Lightning allies Break Effect to 120 percent of highest team ally Break Effect",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (102, "text-zh", "TextMap/TextMapCHS.json; hash=6003923322476481427; entry raises Wind/Lightning allies Break Effect to 120 percent of highest team ally Break Effect",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-break-effect.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Exact current Tourn3 Hex/MazeBuff join, Wind/Thunder eligibility, 1.2 operand and highest-team raise-to clause. Hidden entry ordering, inactive roster, element mapping, rounding and capture lifetime remain field-level policy.",
        ])
    data["WeightedCurioBreakEffects"] = [[
        1, "du.weighted-curio-break-effect.converse-of-entropy", "divergent-universe.weighted-curio.1005",
        "633405", "Wind|Thunder", 1, "1.2", "EntryHighestTeamBreakEffectCapture",
        "At battle entry, raise Wind and Lightning allies' Break Effect to 120% of the highest ally Break Effect in the team.",
        "进入战斗时，将风、雷属性角色的击破特攻提高至队伍最高击破特攻的 120%。",
        "VersionedProjectPolicy: BattleStarted AfterEvent, priority zero, once per battle. Qualify current locked player forms by immutable canonical Basic scaling element; no linked, summon or transformed-form inheritance. Bind all party owners but admit only the first alive/present member in formation order. Snapshot highest effective Break Effect among alive/present original party forms, including ineligible elements. One ApplyEffect operation resolves all eligible recipients from one immutable input before any insertion. Capture max(1.2 times highest minus recipient entry Break Effect, 0) as modifier-local Scalar, NearestTiesEven multiplication, permanent non-dispellable Flat/Stat Break Effect addition with OnApplication snapshot. Later modifiers remain live without recomputing this entry bonus; no dynamic aura, absolute override, per-owner compounding, later-wave recapture or Activity mutation. Unequip changes only future assembled battles.",
        "Low confidence for hidden before-character-born equivalence, other entry-effect ordering, inactive eligibility, canonical Basic element mapping, linked inheritance, snapshot/teardown, rounding and negative input handling; not observed parity. Alternatives include authored before-birth scheduling, inactive-roster maxima, dynamic recomputation and absolute stat override. Tests weighted_curio_break_effect execute both eligible elements, unequal/tied/zero stats, ineligible maxima, downed first owner, later modifiers, real Break damage, fresh reconstruction, rejection and unequip in both families. Replace each field independently when released executable programs or reproducible current traces establish it. Forge admission, equipment-command replay and complete original runs remain pending.",
        "99|100|101|102",
    ]]
