#!/usr/bin/env bash
# Run the slice queue in tmux, outside any Claude chat session.
# A chat session that polls the loop reads its whole context again on each poll.
#
#   tools/run_queue.sh start [max]   start the loop (default max 200 cards)
#   tools/run_queue.sh status        queue, cards that need attention, last log lines
#   tools/run_queue.sh stop          stop after the current card (safe)
#   tools/run_queue.sh attach        watch the loop live (detach: Ctrl-b d)
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
NOTES="$REPO/../notes"
LOG="$NOTES/slices-loop.log"
STOP="$HOME/.cache/bns-slices/stop"
S=slices

case "${1:-status}" in
  start)
    if tmux has-session -t "$S" 2>/dev/null; then echo "already running (tmux session '$S')"; exit 0; fi
    rm -f "$STOP"
    tmux new-session -d -s "$S" -c "$REPO" \
      "python3 -u tools/slice_runner.py loop --max ${2:-200} 2>&1 | tee -a '$LOG'; echo; echo 'loop ended; press Enter to close'; read"
    echo "started in tmux session '$S'; log: $LOG"
    ;;
  stop)
    mkdir -p "$(dirname "$STOP")"; touch "$STOP"
    echo "the loop stops after the current card"
    ;;
  attach)
    exec tmux attach -t "$S"
    ;;
  status)
    tmux has-session -t "$S" 2>/dev/null && echo "loop: running" || echo "loop: not running"
    python3 "$REPO/tools/slice_runner.py" status | grep -vE '^done ' || true
    echo; sed -n '5,200p' "$NOTES/slices-blocked.md" 2>/dev/null | head -40 || true
    echo; echo "--- last log lines"; tail -n 8 "$LOG" 2>/dev/null || true
    ;;
  *) sed -n '2,9p' "$0"; exit 1 ;;
esac
