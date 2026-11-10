#!/usr/bin/env bash
# Sync bns-server commits into the slopemulator monorepo and push to GitHub.
# Refuses to push if the new range contains binaries, game data or secrets.
set -euo pipefail
SRC="$(cd "$(dirname "$0")/.." && pwd)"
MONO="${SLOPEMULATOR_DIR:-$HOME/Documents/Codex/slopemulator}"
KEYFILE="$HOME/.local/state/nl-parts-pipeline/operator-access/home-gateway/codex-handover/worker-api-key"
cd "$MONO"
git fetch -q "$SRC" main
# Check every file changed in bns-server since the last subtree merge.
# LAST = the bns-server commit that the newest merge brought in (its second parent).
LAST=""
for c in $(git log --merges --format=%P -n 50 | awk '{print $2}'); do
  if git merge-base --is-ancestor "$c" FETCH_HEAD 2>/dev/null; then LAST="$c"; break; fi
done
[ -n "$LAST" ] || LAST="$(git log --format=%B | grep -m1 -oE '^git-subtree-split: [0-9a-f]+' | awk '{print $2}' || true)"
BAD="$(git diff --name-only ${LAST:+$LAST} FETCH_HEAD 2>/dev/null | grep -iE '\.(exe|dll|pdb|i64|idb|bin|dat|pem|key|blob|mdf|ldf|rar)$' || true)"
[ -z "$BAD" ] || { echo "refused: binary or key files: $BAD"; exit 1; }
if [ -f "$KEYFILE" ] && git diff ${LAST:+$LAST} FETCH_HEAD | grep -qF -f "$KEYFILE"; then echo "refused: gateway key"; exit 1; fi
if git diff ${LAST:+$LAST} FETCH_HEAD | grep -qiE '^\+.*BEGIN [A-Z ]*PRIVATE KEY'; then echo "refused: private key"; exit 1; fi
# git subtree keeps only the last -m. Give one -m with subject, body and trailer.
# The subject tells what was merged (the real work, not the queue bookkeeping).
WORK="$(git log --no-merges --format=%s ${LAST:+$LAST..}FETCH_HEAD | grep -vE '^queue: ' || true)"
N="$(printf '%s' "$WORK" | grep -c . || true)"
if [ "$N" -eq 0 ]; then SUBJ="$(git log -1 --format=%s FETCH_HEAD)"
elif [ "$N" -eq 1 ]; then SUBJ="$WORK"
else SUBJ="$(printf '%s\n' "$WORK" | head -1) (+$((N - 1)) more)"; fi
MSG="$(printf 'bns-server: %s\n\n%s\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>' "$SUBJ" "$(printf '%s\n' "$WORK" | sed 's/^/- /')")"
git -c user.name=Claude -c user.email=noreply@anthropic.com subtree pull -q --prefix=bns-server "$SRC" main -m "$MSG"
git push -q origin HEAD
echo "pushed $(git log -1 --format=%h) to $(git remote get-url origin)"
