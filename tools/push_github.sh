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
LAST="$(git log --format=%B -n 200 | grep -m1 -oE '^git-subtree-split: [0-9a-f]+' | awk '{print $2}' || true)"
BAD="$(git diff --name-only ${LAST:+$LAST} FETCH_HEAD 2>/dev/null | grep -iE '\.(exe|dll|pdb|i64|idb|bin|dat|pem|key|blob|mdf|ldf|rar)$' || true)"
[ -z "$BAD" ] || { echo "refused: binary or key files: $BAD"; exit 1; }
if [ -f "$KEYFILE" ] && git diff ${LAST:+$LAST} FETCH_HEAD | grep -qF -f "$KEYFILE"; then echo "refused: gateway key"; exit 1; fi
if git diff ${LAST:+$LAST} FETCH_HEAD | grep -qiE '^\+.*BEGIN [A-Z ]*PRIVATE KEY'; then echo "refused: private key"; exit 1; fi
git -c user.name=Claude -c user.email=noreply@anthropic.com subtree pull -q --prefix=bns-server "$SRC" main \
  -m "Merge bns-server $(git -C "$SRC" log -1 --format=%h)" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -q origin HEAD
echo "pushed $(git log -1 --format=%h) to $(git remote get-url origin)"
