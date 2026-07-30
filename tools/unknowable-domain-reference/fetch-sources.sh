#!/usr/bin/env bash
set -euo pipefail

cache_root="${1:-.cache/content-reference}"
turn_remote="https://gitlab.com/Dimbreath/turnbasedgamedata.git"
res_remote="https://github.com/Mar-7th/StarRailRes.git"
turn_revision="fd978d6ef09f941fba644c731ab54abd6f7c3568"
res_revision="7b349e39ee0f6f3bf814567995829b99c95e7a93"
turn_target="${cache_root}/turnbasedgamedata"
res_target="${cache_root}/StarRailRes"

mkdir -p "${cache_root}"

if [[ ! -d "${turn_target}/.git" ]]; then
  git clone --filter=blob:none --no-checkout \
    "${turn_remote}" "${turn_target}"
fi
[[ "$(git -C "${turn_target}" remote get-url origin)" == "${turn_remote}" ]]
if [[ "$(git -C "${turn_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${turn_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${turn_target}" >&2
  exit 1
fi
git -C "${turn_target}" fetch origin "${turn_revision}" --depth 1
git -C "${turn_target}" sparse-checkout init --no-cone
git -C "${turn_target}" sparse-checkout set --no-cone \
  '/README.md' \
  '/ExcelOutput/AvatarConfig.json' \
  '/ExcelOutput/AvatarPromotionConfig.json' \
  '/ExcelOutput/AvatarRankConfig.json' \
  '/ExcelOutput/AvatarSkillConfig.json' \
  '/ExcelOutput/AvatarSkillTreeConfig.json' \
  '/ExcelOutput/EquipmentConfig.json' \
  '/ExcelOutput/EquipmentPromotionConfig.json' \
  '/ExcelOutput/EquipmentSkillConfig.json' \
  '/ExcelOutput/MonsterConfig.json' \
  '/ExcelOutput/MonsterTemplateConfig.json' \
  '/ExcelOutput/MonsterSkillConfig.json' \
  '/ExcelOutput/MonsterStatusConfig.json' \
  '/ExcelOutput/StageConfig.json' \
  '/ExcelOutput/ActivityRogue*.json' \
  '/ExcelOutput/ConstValueRogue.json' \
  '/ExcelOutput/FinishWayRogue.json' \
  '/ExcelOutput/GuideRogue*.json' \
  '/ExcelOutput/Rogue*.json' \
  '/ExcelOutput/ScheduleDataRogue.json' \
  '/TextMap/TextMapEN.json' \
  '/TextMap/TextMapCHS.json' \
  '/Config/ConfigAI/' \
  '/Config/ConfigAdventureModifier/AdventureModifier_Rogue_RogueMagic.json' \
  '/Config/ConfigAbility/Avatar/' \
  '/Config/ConfigAbility/Monster/' \
  '/Config/ConfigAbility/BattleEvent/*Rogue*.json' \
  '/Config/ConfigAbility/Level/Level_*Rogue*.json' \
  '/Config/ConfigCharacter/Avatar/' \
  '/Config/ConfigCharacter/Monster/' \
  '/Config/ConfigCharacter/BattleEvent/Avatar_RogueMagic_*.json' \
  '/Config/Level/GroupTemplateGraph/03_Rogue/RogueMagic260/' \
  '/Config/Level/Maze/MazeRogue/Rogue260/' \
  '/Config/Level/Rogue/' \
  '/Config/Level/RogueDialogue/'
git -C "${turn_target}" checkout --detach "${turn_revision}"

if [[ ! -d "${res_target}/.git" ]]; then
  git clone --filter=blob:none --no-checkout \
    "${res_remote}" "${res_target}"
fi
[[ "$(git -C "${res_target}" remote get-url origin)" == "${res_remote}" ]]
if [[ "$(git -C "${res_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${res_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${res_target}" >&2
  exit 1
fi
git -C "${res_target}" fetch origin "${res_revision}" --depth 1
git -C "${res_target}" sparse-checkout init --no-cone
git -C "${res_target}" sparse-checkout set --no-cone \
  '/README.md' '/LICENSE' '/info.json' '/index_new/'
git -C "${res_target}" checkout --detach "${res_revision}"

[[ "$(git -C "${turn_target}" rev-parse HEAD)" == "${turn_revision}" ]]
[[ "$(git -C "${res_target}" rev-parse HEAD)" == "${res_revision}" ]]
[[ -z "$(git -C "${turn_target}" status --porcelain)" ]]
[[ -z "$(git -C "${res_target}" status --porcelain)" ]]
git -C "${turn_target}" fsck --connectivity-only
git -C "${res_target}" fsck --connectivity-only
echo "Pinned Goal 10 source cache is ready at ${cache_root}"
