"""Explicit shop stock policy; source links establish rewards, never merchant prices."""


def append_shop_stocks(data: dict[str, list[list[object]]]) -> None:
    data["ShopStocks"] = [[
        1, "du.shop-stock.acquisition-policy", "ExplicitFixedStockAndPriceBaseRewards",
        "Fixed policy stock demonstrating base Blessing and immediate Curio acquisition.",
        "固定项目策略库存，包含基础祝福及会触发即时获得效果的奇物。",
        "VersionedProjectPolicy: explicit three-item stock and positive fixed Fragment prices; "
        "one purchase per owner per logical room, no sampling, refresh or discount. "
        "Merchant membership, original prices and stock weights are unavailable. "
        "Reward source links prove identities only. Prices 50/70/40 preserve deterministic "
        "payment and mandatory reward coverage, not observed parity. Alternatives: disable "
        "shops or reuse unproven catalog pools; explicit stock keeps behavior replaceable "
        "without asserting membership. Original-parity confidence is unproven; data admission "
        "and Ordinary/Cyclical shop controller tests freeze this policy.",
        "Replace stock, price and admission fields when released merchant selectors or "
        "reproducible current-version observations establish exact membership, weights, "
        "costs, discounts, quantities and refresh. No default profile placement is implied.",
    ]]
    # Canonical integer strings are authoritative prices, never Excel floating cells.
    data["ShopItems"] = [
        [1, "du.shop-item.acquisition-policy.blessing", 1, 1, "Blessing",
         "divergent-universe.blessing.615130", "50", "15"],
        [2, "du.shop-item.acquisition-policy.wax", 1, 2, "Curio",
         "divergent-universe.curio-state.9043", "70", "7|8|9"],
        [3, "du.shop-item.acquisition-policy.fragments", 1, 3, "Curio",
         "divergent-universe.curio-state.9053", "40", "4|5|6"],
    ]
