#!/usr/bin/env bash
set -euo pipefail

cache_root="${1:-.cache/content-reference}"
seed_root="${2:-}"
turn_remote="https://gitlab.com/Dimbreath/turnbasedgamedata.git"
res_remote="https://github.com/Mar-7th/StarRailRes.git"
turn_revision="fd978d6ef09f941fba644c731ab54abd6f7c3568"
res_revision="7b349e39ee0f6f3bf814567995829b99c95e7a93"
turn_target="${cache_root}/turnbasedgamedata"
res_target="${cache_root}/StarRailRes"

mkdir -p "${cache_root}"

if [[ ! -d "${turn_target}/.git" ]]; then
  if [[ -n "${seed_root}" ]]; then
    turn_seed="${seed_root}/turnbasedgamedata"
    [[ "$(git -C "${turn_seed}" rev-parse HEAD)" == "${turn_revision}" ]]
    [[ "$(git -C "${turn_seed}" remote get-url origin)" == "${turn_remote}" ]]
    [[ -z "$(git -C "${turn_seed}" status --porcelain)" ]]
    git -C "${turn_seed}" fsck --connectivity-only --no-dangling
    mkdir -p "${turn_target}"
    if [[ "$(uname -s)" == "Darwin" ]]; then
      cp -cR "${turn_seed}/." "${turn_target}"
    else
      cp -a --reflink=auto "${turn_seed}/." "${turn_target}"
    fi
  else
    git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
      "${turn_remote}" "${turn_target}"
  fi
fi
git -C "${turn_target}" config http.version HTTP/1.1
[[ "$(git -C "${turn_target}" remote get-url origin)" == "${turn_remote}" ]]
if [[ "$(git -C "${turn_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${turn_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${turn_target}" >&2
  exit 1
fi
if ! git -C "${turn_target}" cat-file -e "${turn_revision}^{commit}"; then
  git -C "${turn_target}" fetch origin "${turn_revision}" --depth 1
fi
git -C "${turn_target}" sparse-checkout init --no-cone
git -C "${turn_target}" sparse-checkout set --no-cone \
  '/README.md' \
  '/ExcelOutput/StageConfig.json' \
  '/ExcelOutput/Rogue*.json' \
  '/TextMap/TextMapEN.json' \
  '/TextMap/TextMapCHS.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Tourn1.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Tourn1.layout.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S1.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S1.layout.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.json' \
  '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.layout.json'
git -C "${turn_target}" checkout --detach "${turn_revision}"

if [[ ! -d "${res_target}/.git" ]]; then
  if [[ -n "${seed_root}" ]]; then
    res_seed="${seed_root}/StarRailRes"
    [[ "$(git -C "${res_seed}" rev-parse HEAD)" == "${res_revision}" ]]
    [[ "$(git -C "${res_seed}" remote get-url origin)" == "${res_remote}" ]]
    [[ -z "$(git -C "${res_seed}" status --porcelain)" ]]
    git -C "${res_seed}" fsck --connectivity-only --no-dangling
    mkdir -p "${res_target}"
    if [[ "$(uname -s)" == "Darwin" ]]; then
      cp -cR "${res_seed}/." "${res_target}"
    else
      cp -a --reflink=auto "${res_seed}/." "${res_target}"
    fi
  else
    git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
      "${res_remote}" "${res_target}"
  fi
fi
git -C "${res_target}" config http.version HTTP/1.1
[[ "$(git -C "${res_target}" remote get-url origin)" == "${res_remote}" ]]
if [[ "$(git -C "${res_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${res_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${res_target}" >&2
  exit 1
fi
if ! git -C "${res_target}" cat-file -e "${res_revision}^{commit}"; then
  git -C "${res_target}" fetch origin "${res_revision}" --depth 1
fi
git -C "${res_target}" sparse-checkout init --no-cone
git -C "${res_target}" sparse-checkout set --no-cone \
  '/README.md' '/LICENSE' '/info.json' '/index_new/'
git -C "${res_target}" checkout --detach "${res_revision}"

[[ "$(git -C "${turn_target}" rev-parse HEAD)" == "${turn_revision}" ]]
[[ "$(git -C "${res_target}" rev-parse HEAD)" == "${res_revision}" ]]
[[ -z "$(git -C "${turn_target}" status --porcelain)" ]]
[[ -z "$(git -C "${res_target}" status --porcelain)" ]]
git -C "${turn_target}" fsck --connectivity-only --no-dangling
git -C "${res_target}" fsck --connectivity-only --no-dangling
echo "Pinned Goal 11 source cache is ready at ${cache_root}"
