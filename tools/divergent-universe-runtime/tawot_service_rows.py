"""Current service binding and independently replaceable purchase policy."""


def append_tawot_services(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (49, "handbook", "ExcelOutput/RogueTournHandBookEvent.json; EventHandbookID=181; UnlockNPCProgressIDList[0].FDOELDMEBPE=700108",
         "15e0e448186b799687a1ef4582e96b40da43888fee5a0e31278eb6cbc29106b9"),
        (50, "variant", "ExcelOutput/RogueTournNPC.json; RogueNPCID=700108; NPCJsonPath=Config/Level/Rogue/RogueNPC/RogueNPC_410/RogueNPC700108.json",
         "e3ef2da06848f45704310e1a35ae06390febf7d4e5a6027591912929b4e4c478"),
        (51, "states", "ExcelOutput/RogueTournMiracle.json; MiracleID=9068..9079; each explicitly Tourn3 and HandbookMiracleID=9068",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.tawot-service.{suffix}", "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12", locator, digest,
            "ExactStructured", "Exact released binding only. The missing option graph does not establish price, Forge eligibility, offer membership, weights or purchase/replacement timing.",
        ])
    data["TawotServices"] = [
        [level - 1, f"du.tawot-service.forge-{level}", "divergent-universe.occurrence.181",
         "divergent-universe.occurrence-variant.700108", "divergent-universe.curio.9068",
         level, 100, 3, limit, "UniformDistinctCurrentStatesPaidSelection",
         f"Forge level {level}: buy one of three Tawot states for 100 fragments, up to {limit} purchase(s).",
         f"{level} 级铸造区域：支付 100 碎片从三种塔沃特牌状态中选一，可购买 {limit} 次。",
         "VersionedProjectPolicy: explicit owning Forge service admission at the authored level; no random-room membership inferred. A purchase presents three distinct current states of the bound handbook with uniform integer weights and no replacement. All twelve current states remain candidates regardless of effect implementation coverage. Buying requires 100 fragments and remaining room-local purchase allowance; charge and count commit with the selected state's replacement/acquisition, not merely opening the offer. Cancel is free and retains the same sampled offer; no free reroll. Existing same-owner holdings may be replaced, including with the same state, resetting its allowance. All mandatory acquisition effects are preflighted. Room exit discards the offer and purchase count; repeated visits are separate service instances. Original Forge topology and Towat boss-card service are not implemented by this definition.",
         "Price 100, width three, levels 2..5 and limits 1/1/2/2 are policy operands informed by indexed public https://honkai-star-rail.fandom.com/wiki/Divergent_Universe/Occurrences and https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Domains accessed 2026-09-12; robots denial prevented complete page hashing, so no exact or independently verified gameplay claim is made. Replace these fields, candidate membership, weights, charge/selection timing, cancel persistence and replacement independently with released option graphs or reproducible observations. Alternatives include charging before display, excluding owned states and forfeiting cancellation. Hidden behavior has low confidence; handwritten service execution must not treat incomplete Curio effects as inert.",
         "49|50|51"]
        for level, limit in [(2, 1), (3, 1), (4, 2), (5, 2)]
    ]
