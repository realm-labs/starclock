#!/usr/bin/env bash
set -euo pipefail

destination="${1:-.cache/pure-fiction/turnbasedgamedata}"
revision="fd978d6ef09f941fba644c731ab54abd6f7c3568"

if [[ ! -d "$destination/.git" ]]; then
  git init "$destination"
  git -C "$destination" remote add origin https://gitlab.com/Dimbreath/turnbasedgamedata.git
  git -C "$destination" config extensions.partialClone origin
  git -C "$destination" config remote.origin.promisor true
  git -C "$destination" config remote.origin.partialclonefilter blob:none
fi

git -C "$destination" fetch --depth=1 --filter=blob:none origin "$revision"
git -C "$destination" sparse-checkout init --no-cone
git -C "$destination" sparse-checkout set \
  '/ExcelOutput/ChallengeStory*.json' \
  '/ExcelOutput/ScheduleDataChallengeStory.json' \
  '/ExcelOutput/Challenge*.json' \
  '/ExcelOutput/ConstValueChallenge*.json' \
  '/ExcelOutput/MapEntrance*.json' \
  '/ExcelOutput/MappingInfo.json' \
  '/ExcelOutput/MazeBuff.json' \
  '/ExcelOutput/BattleEventConfig.json' \
  '/ExcelOutput/StageConfig.json' \
  '/ExcelOutput/Monster*.json' \
  '/TextMap/TextMapCHS.json' \
  '/TextMap/TextMapEN.json' \
  '/Config/ConfigAbility/BattleEvent/FantasticStory*.json' \
  '/Config/ConfigAbility/Level/Level_MazeChallengeBuff_Ability.json' \
  '/Config/ConfigAbility/StageBattleEventAbility.json' \
  '/Config/Level/StageCommonTemplate.json' \
  '/Config/ConfigCharacter/Monster/**' \
  '/Config/ConfigAbility/Monster/**' \
  '/Config/ConfigAI/**'
git -C "$destination" checkout --detach FETCH_HEAD
test "$(git -C "$destination" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$destination" status --porcelain)"
