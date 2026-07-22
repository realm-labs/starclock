import { decimal, sha256 } from "./common.mjs";

const OUTCOME_PATTERNS = [
  ["Obtain", /obtain|gain|receive/iu], ["Lose", /lose|remove/iu],
  ["Consume", /consume|spend|pay/iu], ["Discard", /discard|destroy/iu],
  ["Enhance", /enhance|upgrade/iu], ["Repair", /repair|fix/iu],
  ["Restore", /restore|recover|heal/iu], ["Select", /select|choose/iu],
  ["Battle", /enter battle|fight|defeat/iu], ["NoOp", /nothing occurs|event ends|do nothing/iu],
];
const TARGET_PATTERNS = [
  ["CosmicFragments", /cosmic fragment/iu], ["Blessing", /blessing/iu],
  ["Curio", /curio/iu], ["HP", /\bhp\b/iu], ["Energy", /energy/iu],
  ["TechniquePoints", /technique point/iu], ["SkillPoints", /skill point/iu],
  ["Enemy", /enemy/iu], ["Character", /character|allies|team member/iu],
];

function mechanicSummary(text) {
  const kinds = OUTCOME_PATTERNS.filter(([, pattern]) => pattern.test(text)).map(([kind]) => kind);
  const targets = TARGET_PATTERNS.filter(([, pattern]) => pattern.test(text)).map(([target]) => target);
  const numbers = [...text.matchAll(/(?<![#\w])-?\d+(?:\.\d+)?%?/gu)].map((match) => match[0]);
  const parameterRefs = [...text.matchAll(/#(\d+)\[[^\]]+\]/gu)].map((match) => Number(match[1]));
  const chancePercentages = [...text.matchAll(/(\d+(?:\.\d+)?)%/gu)].map((match) => decimal(match[1]));
  return { kinds: kinds.length ? kinds : ["Special"], targets, numeric_literals: numbers, parameter_refs: [...new Set(parameterRefs)].sort((a, b) => a - b), chance_percentages: [...new Set(chancePercentages)] };
}

function isUnspecifiedRandom(text) {
  return /random|chance/iu.test(text) && !/\d+(?:\.\d+)?%/u.test(text);
}

export async function occurrences(ctx) {
  const handbook = await ctx.table("RogueHandBookEvent");
  const npcs = await ctx.table("RogueNPC");
  const optionDisplays = await ctx.table("RogueDialogueOptionDisplay");
  const optionValues = await ctx.table("RogueDialogueOption");
  const standard = handbook.filter(({ row }) => row.EventTypeList.includes(100)).sort((left, right) => left.row.Order - right.row.Order);
  const npcById = new Map(npcs.map((entry) => [entry.row.RogueNPCID, entry]));
  const optionDisplayById = new Map(optionDisplays.map((entry) => [entry.row.OptionDisplayID, entry]));
  const optionValuesByDisplay = Map.groupBy(optionValues, ({ row }) => row.OptionDisplayID);

  const occurrenceRows = [];
  const variantRows = [];
  const choiceRows = [];

  for (const entry of standard) {
    const row = entry.row;
    const occurrenceId = `universe.occurrence.${row.EventHandbookID}`;
    const nameEn = ctx.text(row.EventTitle, "en");
    const nameZh = ctx.text(row.EventTitle, "zh_cn");
    const baseNpcIds = [...new Set(row.UnlockNPCProgressIDList.map((value) => value.FDOELDMEBPE).filter((id) => id < 100000 && npcById.has(id)))].sort((a, b) => a - b);
    occurrenceRows.push({
      ...ctx.envelope({
        id: occurrenceId,
        nameEn,
        nameZh,
        summaryEn: baseNpcIds.length ? `Standard Occurrence with ${baseNpcIds.length} base-mode choice-graph variant${baseNpcIds.length === 1 ? "" : "s"}.` : "Standard index Occurrence without a released base-mode NPC choice graph.",
        summaryZh: baseNpcIds.length ? `标准事件，包含${baseNpcIds.length}个主模式选择图变体。` : "标准图鉴事件，已发布数据中没有主模式NPC选择图。",
        entry,
        coverageState: baseNpcIds.length ? "DataReady" : "DataReady",
        note: baseNpcIds.length ? "" : "Index-only record; no base-mode NPC graph exists at the frozen snapshot.",
        sourceIds: [row.EventHandbookID],
      }),
      variant_ids: baseNpcIds.map((id) => `${occurrenceId}.variant.${id}`),
      choice_graph_id: baseNpcIds.length ? `${occurrenceId}.graph` : "",
      pool_tags: ["mode:standard", `event-type:${ctx.text(row.EventType, "en") || "unknown"}`],
      index_only: baseNpcIds.length === 0,
    });

    for (const npcId of baseNpcIds) {
      const npcEntry = npcById.get(npcId);
      const npcConfig = await ctx.readSource(npcEntry.row.NPCJsonPath);
      const npcConfigEntry = { relativePath: npcEntry.row.NPCJsonPath, sourceKey: "root", row: npcConfig };
      const variantId = `${occurrenceId}.variant.${npcId}`;
      const dialogueChoices = [];
      const unlockIds = new Set();
      let choiceIndex = 0;
      for (const dialogue of npcConfig.DialogueList ?? []) {
        if (dialogue.UnlockID !== undefined) unlockIds.add(dialogue.UnlockID);
        if (!dialogue.OptionPath) continue;
        const optionConfig = await ctx.readSource(dialogue.OptionPath);
        const optionConfigEntry = { relativePath: dialogue.OptionPath, sourceKey: "root", row: optionConfig };
        for (const option of optionConfig.OptionList ?? []) {
          choiceIndex += 1;
          const displayEntry = optionDisplayById.get(option.DisplayID);
          if (!displayEntry) throw new Error(`missing option display ${option.DisplayID} for NPC ${npcId}`);
          const titleEn = ctx.text(displayEntry.row.OptionTitle, "en");
          const titleZh = ctx.text(displayEntry.row.OptionTitle, "zh_cn");
          const resultEn = ctx.text(displayEntry.row.OptionDesc, "en");
          const resultZh = ctx.text(displayEntry.row.OptionDesc, "zh_cn");
          const outcome = mechanicSummary(resultEn);
          const parameterRows = optionValuesByDisplay.get(option.DisplayID) ?? [];
          const choiceId = `${variantId}.choice.${String(choiceIndex).padStart(2, "0")}`;
          const approximate = isUnspecifiedRandom(resultEn);
          const record = {
            ...ctx.envelope({
              id: choiceId,
              nameEn: `${nameEn} — Choice ${choiceIndex}`,
              nameZh: `${nameZh}·选择${choiceIndex}`,
              summaryEn: `${outcome.kinds.join("/")} ${outcome.targets.length ? outcome.targets.join(", ") : "special state"} outcome.`,
              summaryZh: `${outcome.kinds.join("/")}：${outcome.targets.length ? outcome.targets.join("、") : "特殊状态"}结果。`,
              entry: displayEntry,
              mechanismQuality: approximate ? "ProjectPolicy" : "ExactPublicText",
              note: approximate ? "Released text leaves random weights unspecified; runtime must use a versioned deterministic policy until exact weights are found." : "Choice/result text is exact public TextMap evidence; prose is represented by digests and structured outcome facts.",
              sourceIds: [npcId, option.OptionID, option.DisplayID],
            }),
            variant_id: variantId,
            condition_ids: dialogue.UnlockID === undefined ? [] : [`universe.unlock.source-${dialogue.UnlockID}`],
            costs: outcome.kinds.filter((kind) => ["Lose", "Consume", "Discard"].includes(kind)).map((kind) => ({ kind, targets: outcome.targets })),
            outcomes: [{ ...outcome, unspecified_random_policy: approximate ? "StableUniformOrderedCandidates" : "" }],
            next_node_id: "",
            choice_label_sha256_en: sha256(titleEn),
            choice_label_sha256_zh_cn: sha256(titleZh),
            result_sha256_en: sha256(resultEn),
            result_sha256_zh_cn: sha256(resultZh),
            parameter_vectors: parameterRows.map((parameterEntry) => ({
              source_option_id: String(parameterEntry.row.OptionID),
              values: (parameterEntry.row.ParamList ?? []).map((value, parameterIndex) => ({ index: parameterIndex + 1, value: decimal(value) })),
            })),
          };
          record.provenance_ids.push(ctx.provenance(optionConfigEntry), ctx.provenance(npcConfigEntry));
          for (const parameterEntry of parameterRows) record.provenance_ids.push(ctx.provenance(parameterEntry));
          choiceRows.push(record);
          dialogueChoices.push(choiceId);
        }
      }
      const variant = {
        ...ctx.envelope({
          id: variantId,
          nameEn: `${nameEn} — Variant ${npcId}`,
          nameZh: `${nameZh}·变体${npcId}`,
          summaryEn: `Base-mode occurrence variant with ${dialogueChoices.length} ordered mechanical choices.`,
          summaryZh: `主模式事件变体，包含${dialogueChoices.length}个有序机制选择。`,
          entry: npcEntry,
          sourceIds: [npcId],
        }),
        occurrence_id: occurrenceId,
        entry_node_id: `${variantId}.entry`,
        condition_ids: [...unlockIds].sort((a, b) => a - b).map((id) => `universe.unlock.source-${id}`),
        choice_ids: dialogueChoices,
        source_dialogue_type: npcConfig.DialogueType ?? "",
      };
      variant.provenance_ids.push(ctx.provenance(npcConfigEntry));
      variantRows.push(variant);
    }
  }

  return new Map([
    ["occurrences.json", occurrenceRows],
    ["occurrence-variants.json", variantRows],
    ["occurrence-choices.json", choiceRows],
  ]);
}
