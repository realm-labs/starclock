"""Released Sapient Pen operands; authoring does not grant execution credit."""


def append_weighted_curio_elation(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (107, "hex", "ExcelOutput/RogueTournHex.json; HexID=1015; TournMode=Tourn3; MazeBuffID=633415; AvatarType=Elation; DisplayID=1027",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (108, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633415; Lv=1; ParamList=2|0.5|2; InBattleBindingType=StageAbilityBeforeCharacterBorn; InBattleBindingKey=StageAbility_633415",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (109, "text-en", "TextMap/TextMapEN.json; hash=7976287788830292485; Elation party presence; non-Elation Basic or Skill; Punchline and timed Elation bonus",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (110, "text-zh", "TextMap/TextMapCHS.json; hash=7976287788830292485; Elation party presence; non-Elation Basic or Skill; Punchline and timed Elation bonus",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-elation.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-02",
            locator, digest, "ExactStructured",
            "Exact current Tourn3 eligibility and MazeBuff operands. Hidden trigger, recipient, refresh and meter lifecycle semantics are not observed parity. This row does not implement the Curio.",
        ])
    data["WeightedCurioElations"] = [[
        1, "du.weighted-curio-elation.sapient-pen", "divergent-universe.weighted-curio.1015",
        "633415", "Elation", 1, 2, 2, "0.5", 3, 2,
        "ActionResolvedOriginalPartyRefresh",
        "With an Elation character in the party, non-Elation Basic or Skill use grants two Punchlines and adds 50% Elation to Elation characters for two turns.",
        "队伍含欢愉角色时，非欢愉角色施放普攻或战技后获得两点笑点，并使欢愉角色的欢愉度提高50%，持续两回合。",
        "VersionedProjectPolicy, authored but not yet executed: original party combat forms determine Path membership; any original Elation party form satisfies presence, including a defeated member. Eligible original non-Elation owners trigger at ActionResolved AfterAction, priority zero, once per action for Basic or Skill regardless of Attack tag or hit count. Grant two to the side's single shared.punchline binding, then apply one dispellable additive StatKind::Elation bonus of 0.5 to each living/present original Elation recipient in formation order. TargetTurnEnd duration two, replace and refresh across casters rather than stack; no Ultimate, follow-up, summon, memosprite or shared-actor trigger. No RNG or Activity mutation. Shared-meter initial value, maximum, overflow and wave/battle carry belong to the separate BattleTeamResources assembly policy, not Curio operands or defaults; implementing the meter does not implement this Curio.",
        "Low confidence for party presence while defeated, original-form membership, action observation point, cause ownership, ordering, dispel and cross-caster refresh. Bounded 2026-10-02 public cross-check: https://honkai-star-rail.fandom.com/wiki/Sapient_Pen confirms only the current Arcadian Chronicles player-facing clauses, not hidden scheduling; its older Quantum/Wind variant is excluded. Alternatives: active-only presence, per-hit triggers, caster-specific stacking, nondispellable bonus and source-turn duration. Data tests weighted_curio_elation validate exact operands, joins, policy labels and all four sources; they do not prove battle execution. Replace fields independently when released StageAbility_633415 or reproducible current observations establish trigger/recipient/refresh semantics. Runtime admission still requires real-command fixtures for both run families and explicit shared-meter assembly, overflow, lifecycle and reconstruction tests.",
        "107|108|109|110",
    ]]
