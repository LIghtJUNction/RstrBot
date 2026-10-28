#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
skills=(rstrbot-development rstrbot-plugin-development)
commits=()

for skill in "${skills[@]}"; do
  source_dir="$repo_root/.agents/skills/$skill"
  if [[ ! -f "$source_dir/SKILL.md" ]]; then
    printf 'Missing %s/SKILL.md; initialize the submodules first.\n' "$source_dir" >&2
    exit 1
  fi
  if [[ -n "$(git -C "$source_dir" status --porcelain)" ]]; then
    printf 'Submodule %s has uncommitted changes; commit or preserve them before syncing.\n' "$skill" >&2
    exit 1
  fi
  commits+=("$(git -C "$source_dir" rev-parse HEAD)")
done

for skill in "${skills[@]}"; do
  destination="$repo_root/recipes/rstrbot/skills/$skill"
  mkdir -p "$destination"
  cp "$repo_root/.agents/skills/$skill/SKILL.md" "$destination/SKILL.md"
done

source_list="$repo_root/recipes/rstrbot/skills/SOURCES.md"
temp_file="$(mktemp)"
trap 'rm -f "$temp_file"' EXIT
{
  cat <<'EOF'
# 开发技能来源提交

配方中的`SKILL.md`是对应Git子模块提交的完整快照。主仓库以`.gitmodules`记录远端，以gitlink固定提交；本清单让独立打包的Recipkg配方也保留来源身份。

| 技能 | 仓库 | 固定提交 |
| --- | --- | --- |
EOF
  for index in "${!skills[@]}"; do
    skill="${skills[$index]}"
    printf '| `%s` | https://github.com/LIghtJUNction/%s | `%s` |\n' \
      "$skill" "$skill" "${commits[$index]}"
  done
} > "$temp_file"
mv "$temp_file" "$source_list"
trap - EXIT

printf 'Synchronized %d skills into recipes/rstrbot/skills/.\n' "${#skills[@]}"
