#!/usr/bin/env bash
# Change bad subtree merge messages on main. Do not push.
set -euo pipefail

BAD_TITLE='Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
BACKUP_REF='refs/backup/main-before-reword'

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

new_message() {
  local commit="$1" parents p1 p2 work subject
  local -a parent_fields work_lines=()

  parents="$(git rev-list --parents -n 1 "$commit")" || fail "Cannot read commit $commit."
  read -r -a parent_fields <<< "$parents"
  [[ ${#parent_fields[@]} -eq 3 ]] ||
    fail "Commit $commit must have exactly two parents."
  # Use the original parents, not the new commit map.
  p1="$(git rev-parse --verify "$commit^1")" || fail "Cannot read parent 1 of $commit."
  p2="$(git rev-parse --verify "$commit^2")" || fail "Cannot read parent 2 of $commit."
  work="$(git log --no-merges --format=%s "$p2" "^$p1" | sed '/^queue: /d')" ||
    fail "Cannot read the work subjects for $commit."

  if [[ -n "$work" ]]; then
    mapfile -t work_lines <<< "$work"
    subject="${work_lines[0]}"
    if [[ ${#work_lines[@]} -gt 1 ]]; then
      subject+=" (+$((${#work_lines[@]} - 1)) more)"
    fi
  else
    subject="$(git log -1 --format=%s "$p2")" || fail "Cannot read the subject of $p2."
  fi

  printf 'bns-server: %s\n\n' "$subject"
  if [[ ${#work_lines[@]} -gt 0 ]]; then
    printf -- '- %s\n' "${work_lines[@]}"
  fi
  printf '\n%s\n' "$BAD_TITLE"
}

# Git runs this mode with the original commit in GIT_COMMIT.
if [[ ${1:-} == '--message-filter' ]]; then
  [[ $# -eq 1 && -n ${GIT_COMMIT:-} ]] || fail 'No filter commit is set.'
  title="$(git log -1 --format=%s "$GIT_COMMIT")"
  if [[ "$title" == "$BAD_TITLE" ]] &&
    git rev-parse --verify --quiet "$GIT_COMMIT^2" > /dev/null; then
    # Read the full input before this process writes the new message.
    cat > /dev/null
    new_message "$GIT_COMMIT"
  else
    # Keep all other messages without changes.
    cat
  fi
  exit 0
fi

DRY_RUN=0
case "${1:-}" in
  '') [[ $# -eq 0 ]] || fail 'Use no option or --dry-run.' ;;
  --dry-run)
    [[ $# -eq 1 ]] || fail 'Use no option or --dry-run.'
    DRY_RUN=1
    ;;
  *) fail 'Use no option or --dry-run.' ;;
esac

# Keep status from writing an index refresh, including in dry-run mode.
export GIT_OPTIONAL_LOCKS=0
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
export REWORD_MERGES_SCRIPT="$SCRIPT_DIR/$(basename -- "${BASH_SOURCE[0]}")"
MONO="${SLOPEMULATOR_DIR:-$HOME/Documents/Codex/slopemulator}"
cd -- "$MONO"
[[ "$(git rev-parse --is-inside-work-tree)" == true ]] || fail 'No work tree is open.'
[[ "$(git symbolic-ref --quiet --short HEAD)" == main ]] || fail 'The branch must be main.'
STATUS="$(git status --porcelain --untracked-files=all --ignore-submodules=none)"
[[ -z "$STATUS" ]] || fail 'The work tree and index must be clean.'

OLD_MAIN="$(git rev-parse --verify refs/heads/main)"
OLD_COUNT="$(git rev-list --count main)"
MERGES="$(git rev-list --merges --reverse main)"
BAD_COUNT=0
# Check each new message before any ref changes.
while IFS= read -r commit; do
  [[ -n "$commit" ]] || continue
  title="$(git log -1 --format=%s "$commit")"
  [[ "$title" == "$BAD_TITLE" ]] || continue
  message="$(new_message "$commit")"
  BAD_COUNT=$((BAD_COUNT + 1))
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '%s: %s → %s\n' "$commit" "$title" "${message%%$'\n'*}"
  fi
done <<< "$MERGES"

if [[ "$DRY_RUN" -eq 1 ]]; then
  printf 'Dry run: %s bad merges. No refs or messages changed.\n' "$BAD_COUNT"
  exit 0
fi

# Read the local remote-tracking ref. Do not fetch or push.
OLD_ORIGIN_MAIN="$(git rev-parse --verify 'refs/remotes/origin/main^{commit}')"
# An empty old value requires a new ref. Never replace an earlier backup.
git update-ref "$BACKUP_REF" "$OLD_MAIN" ''
printf 'Backup: %s -> %s\n' "$BACKUP_REF" "$OLD_MAIN"
printf 'Bad merges: %s\n' "$BAD_COUNT"

export FILTER_BRANCH_SQUELCH_WARNING=1
# The message filter does not set author, committer, date, tree or parent filters.
git filter-branch -f --msg-filter \
  'bash "$REWORD_MERGES_SCRIPT" --message-filter' -- main

CHECKS_FAILED=0
if git diff --exit-code --quiet "$BACKUP_REF" main --; then
  printf 'PASS: The tree diff is empty.\n'
else
  printf 'FAIL: The tree diff is not empty, or the check failed.\n'
  CHECKS_FAILED=1
fi

if BAD_LEFT="$(git log main --format=%s | awk '/^Co-Authored-By/ { n++ } END { print n+0 }')"; then
  if [[ "$BAD_LEFT" -eq 0 ]]; then
    printf 'PASS: No title starts with Co-Authored-By.\n'
  else
    printf 'FAIL: %s titles start with Co-Authored-By.\n' "$BAD_LEFT"
    CHECKS_FAILED=1
  fi
else
  printf 'FAIL: The title check failed.\n'
  CHECKS_FAILED=1
fi

if NEW_COUNT="$(git rev-list --count main)"; then
  if [[ "$NEW_COUNT" == "$OLD_COUNT" ]]; then
    printf 'PASS: The commit count is the same (%s).\n' "$NEW_COUNT"
  else
    printf 'FAIL: The commit count changed from %s to %s.\n' "$OLD_COUNT" "$NEW_COUNT"
    CHECKS_FAILED=1
  fi
else
  printf 'FAIL: The commit count check failed.\n'
  CHECKS_FAILED=1
fi

[[ "$CHECKS_FAILED" -eq 0 ]] || fail 'Do not push. Inspect main and the backup ref.'
printf 'No push was run. Run this command from the monorepo:\n'
printf 'git push --force-with-lease=main:%s origin main\n' "$OLD_ORIGIN_MAIN"
