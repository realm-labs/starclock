"""Reviewed Wealth presets; chest amount/count/timing remain explicit project policy."""


def append_coin_rewards(data: dict[str, list[list[object]]]) -> None:
    data["Sources"].append([
        70, "du.source.wealth-description", "https://gitlab.com/Dimbreath/turnbasedgamedata",
        "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-27",
        "TextMap/TextMapEN.json; hashes=6523492130013154265,12602106085340263013,12397529171261210903; RoguePersonaRoomCompType=6",
        "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
        "ExactStructured", "Current Wealth description establishes chests and facilities, plus increased rewards at higher levels; no exact chest amounts, counts or selectors.",
    ])
    data["CoinRewards"] = []
    for ordinal, (preset, level, amount) in enumerate([(1008, 1, "100"), (1024, 2, "200"), (9008, 1, "100")], 1):
        data["CoinRewards"].append([
            ordinal, f"du.coin-reward.preset-{preset}", str(preset), level, amount,
            "FixedSingleChestCreditNoAmusementFacilities",
            "One explicitly collected Wealth chest reward; amusement facilities remain unimplemented.",
            "显式领取一次财富域宝箱碎片；游乐设施仍未实现。",
            "VersionedProjectPolicy: one aggregate chest pickup per reviewed logical room, "
            "100 base Fragments for selected level-one presets and 200 for the selected level-two preset. "
            "Exact amounts, chest counts, refresh and collection timing are unavailable. "
            "Explicit Collect credits through the existing active Curio gain pipeline; Leave grants nothing. "
            "No RNG, automatic entry payout, guessed original chest selector or facility outcome. "
            "Alternatives: disable pickup or substitute battle rewards; explicit fixed operands preserve "
            "a replaceable chest boundary and the known level-two increase. Original-parity confidence "
            "is unproven; data, atomic pickup and Ordinary/Cyclical controller fixtures freeze this policy.",
            "Replace amount, count and timing when current released chest/level programs or reproducible "
            "observations establish them. Add facilities separately; this row does not complete Wealth "
            "gameplay or infer mask/room/NPC admission.", "68|69|70",
        ])
