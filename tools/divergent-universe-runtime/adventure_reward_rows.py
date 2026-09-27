"""Current reviewed Adventure card; opaque challenges remain external inputs."""


def append_adventure_rewards(data: dict[str, list[list[object]]]) -> None:
    data["Sources"].append([
        72, "du.source.adventure-description", "https://gitlab.com/Dimbreath/turnbasedgamedata",
        "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-27",
        "TextMap/TextMapEN.json; hashes=3196724585769076860,12097919368463049754,10649614652576832174; RoguePersonaRoomCompType=9",
        "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
        "ExactStructured", "Current Adventure description bounds level-one rewards at three chests; exact payouts and challenge selection/scoring/timing are unavailable.",
    ])
    data["AdventureRewards"] = [[
        1, "du.adventure-reward.level-one-aggregate", "1011", 1, 3, "100",
        "ExternalEarnedChestCountAggregateFragmentsOnly",
        "External earned-chest count settles Fragments; challenge simulation remains unimplemented.",
        "外部输入已赢得宝箱数并结算碎片；挑战模拟仍未实现。",
        "VersionedProjectPolicy: explicit external earned count 0..3; 100 base Fragments per chest, "
        "aggregated before the current active Curio gain pipeline and its integer rounding. "
        "Exact reward composition, amounts and timing are unavailable after bounded released-source research. "
        "No RNG, automatic entry payout, guessed challenge pool, score/time simulation or higher-level reward. "
        "Alternatives: disable settlement or guess original mini-games; the explicit host result preserves "
        "the known level-one bound without inventing gameplay. Original-parity confidence is unproven. "
        "Production Sora, all-count settlement, overflow/rejection and fresh-definition fixtures freeze policy.",
        "Replace amount/composition/rounding/timing when current released chest programs or reproducible "
        "observations establish them. Implement source-selected challenges and Adventure-specific Curio "
        "time/extra-chest/rank effects separately; this row does not complete Adventure gameplay.",
        "68|69|72",
    ]]
