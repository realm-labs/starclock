"""Current public Divine Treasure event policies; not original room membership."""


def append_evolution_events(data: dict[str, list[list[object]]]) -> None:
    for source_id, part, title, revision, digest in [
        (30, "ii", "%E7%A5%9E%E8%B5%90%E7%8F%8D%E5%AE%9D%EF%BC%88%E5%85%B6%E4%BA%8C%EF%BC%89", "95613", "ee8bbf735b0288f50d3859538d5af1bfe9a3caabe3fdca10ec78aa1a7d5f57d9"),
        (31, "iii", "%E7%A5%9E%E8%B5%90%E7%8F%8D%E5%AE%9D%EF%BC%88%E5%85%B6%E4%B8%89%EF%BC%89", "77058", "4a6ef56cb0e117806fa416d85dd5f73b3e1b91a8b907064d25aaebd98d9bd73a"),
    ]:
        data["Sources"].append([
            source_id, f"du.source.divine-treasures-{part}-options", f"https://wiki.biligame.com/sr/{title}", revision, "4.4", "2026-09-10",
            f"Divine Treasures {part.upper()}; three alternative mechanical outcomes; fetched HTML bytes", digest, "ObservedCommunity",
            "Public generic Divine Treasure option values; application to current Green Miracle and Sage's Leaf Robe edges is explicit policy. This does not establish exact current-profile room placement, other Divine Treasure mechanics, random-pool weights or hidden trigger ordering.",
        ])
    data["EvolutionEvents"] = [
        [event, f"du.evolution-event.{family}.{part}", event, layer, "ActiveTreasuresAtLayerEntryStableSequence",
         "VersionedProjectPolicy: at this authored logical layer in either baseline, process at most three distinct-owner events in stable event-key order, each once. Re-evaluate eligibility after prior choices: an active predecessor and at least one executable option are required. A later event may qualify through a prior reward, but skipped earlier events are not revisited. Skip ineligible events without RNG or player action. Costs precede evolution; the successor's full acquisition grant precedes the separate event grant. Both guaranteed and chance upgrades require the complete mandatory successor reward to be available before RNG; exhaustion disables them, never grants a free failed roll. Chance uses one bounded Reward draw, preserves the old form on failure and consumes the event. Random Curios use unowned canonical identities, unit weights, no replacement, reviewed acquisition preflight and explicit rarity bounds. Snapshot eligibility before sacrifice. Inventory, rewards, all gains, dialogue handoff and next-node pumping commit atomically. No free leave alternative, reroll, healing or extra reward. Original topology and remaining Treasure effects are gaps.",
         "Replace layer/profile placement, multiple-treasure order and eligibility snapshots, active-only gate, success-reward preflight, default rarity range, weights, pool membership, sacrifice snapshot, grant ordering and probability mapping independently with released graphs or reproducible current-profile observations. Alternatives include a single treasure per event, simultaneous eligibility, random placement, weighted rarity, retained sacrifice eligibility and interleaved rewards. Hidden fields have low confidence; published numeric outcomes are independent community evidence, not exact reachability.",
         f"26|{source}" + ("|32|34|35" if family == "sage" else "")]
        for event, family, part, layer, source in [(1, "green", "ii", 2, 30), (2, "green", "iii", 3, 31), (3, "sage", "ii", 2, 30), (4, "sage", "iii", 3, 31)]
    ]
    data["EvolutionOptions"] = [
        [1, "du.evolution-option.green.ii.evolve", 1, 1, "Evolve", "Accept the power", "接受力量", 0, 200, 0, 1, 3, 1, 1, "30"],
        [2, "du.evolution-option.green.ii.curios", 1, 2, "RandomCurios", "Request Curios", "请求奇物", 0, 0, 2, 1, 3, 0, 1, "30"],
        [3, "du.evolution-option.green.ii.sacrifice", 1, 3, "Sacrifice", "Give up the treasure", "放弃珍宝", 0, 150, 2, 1, 3, 0, 1, "30"],
        [4, "du.evolution-option.green.iii.evolve", 2, 1, "Evolve", "Pay fragments to evolve", "支付碎片升级", 100, 0, 0, 1, 3, 1, 1, "31"],
        [5, "du.evolution-option.green.iii.chance", 2, 2, "ChanceEvolution", "Attempt evolution", "尝试升级", 0, 0, 0, 1, 3, 1, 2, "31"],
        [6, "du.evolution-option.green.iii.sacrifice", 2, 3, "Sacrifice", "Exchange the treasure", "交换珍宝", 0, 0, 3, 2, 3, 0, 1, "31"],
    ]
    for original in list(data["EvolutionOptions"]):
        row = list(original)
        row[0] += 6
        row[1] = row[1].replace(".green.", ".sage.")
        row[2] += 2
        data["EvolutionOptions"].append(row)
