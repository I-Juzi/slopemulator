#!/usr/bin/env python3
"""Run one Sol job with a turn limit and a token budget. Append its usage to a ledger.

Usage: sol_job.py --slot N --turns N --prompt FILE --out FILE [--resume ID] [--ida] [--write] [--emu]
       sol_job.py --report
"""
import argparse, json, os, signal, subprocess, sys, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LEDGER = ROOT / "notes" / "sol-usage.jsonl"
BUDGET_5H = 30_000_000  # Report/warning threshold only. It never blocks a job.
WINDOW = 5 * 3600
JOB_TIMEOUT = 4 * 3600  # Wall-clock safety only. --max-turns is the real job limit (user, 2026-11-08: never cut jobs short).
MAX_RESUME_CONTEXT = 60_000  # Over this, start a new session. A resumed session resends its context on each request.


def rows():
    if not LEDGER.is_file():
        return []
    return [json.loads(line) for line in LEDGER.read_text().splitlines() if line.strip()]


def used(slot):
    cutoff = time.time() - WINDOW
    return sum(r["total"] for r in rows() if r["slot"] == slot and r["ts"] >= cutoff)


def last_context(session):
    """Context size of the newest request in the session log. 0 if unknown."""
    base = Path.home() / ".local/state/nl-parts-pipeline/claude-code-gateway"
    size = 0
    for path in base.glob(f"sol-slot-*/projects/**/{session}.jsonl"):
        for line in path.read_text(errors="replace").splitlines():
            try:
                usage = (json.loads(line).get("message") or {}).get("usage")
            except (ValueError, AttributeError):
                continue
            if usage:
                size = sum(usage.get(k, 0) for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens"))
    return size


def subagent_total(session):
    """Tokens in all sub-agent logs of a session. The job's JSON usage leaves these out."""
    base = Path.home() / ".local/state/nl-parts-pipeline/claude-code-gateway"
    seen = {}
    for path in base.glob(f"sol-slot-*/projects/**/{session}/subagents/*.jsonl"):
        for line in path.read_text(errors="replace").splitlines():
            try:
                msg = json.loads(line).get("message") or {}
            except ValueError:
                continue
            if msg.get("usage"):
                seen[msg.get("id") or len(seen)] = msg["usage"]
    keys = ("input_tokens", "cache_read_input_tokens", "output_tokens")
    return sum(u.get(k, 0) for u in seen.values() for k in keys)


def report():
    for slot in (1, 2):
        print(f"slot {slot}: {used(slot):,} of {BUDGET_5H:,} tokens in the last 5 h")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slot", type=int, choices=(1, 2))
    ap.add_argument("--turns", type=int)
    ap.add_argument("--prompt")
    ap.add_argument("--out")
    ap.add_argument("--resume")
    ap.add_argument("--ida", action="store_true")
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--emu", action="store_true", help="allow the venv Python, only for scripts in bns-server/Program/Tools/Emu/")
    ap.add_argument("--report", action="store_true")
    a = ap.parse_args()
    if a.report:
        return report()
    if not (a.slot and a.turns and a.prompt and a.out):
        ap.error("--slot, --turns, --prompt and --out are required")
    if used(a.slot) >= BUDGET_5H:
        # Warning only (user, 2026-11-08): the local guard must never stop the pipeline.
        # The launcher's real quota check (exit 75) is the only stop rule.
        print(f"note: slot {a.slot} used {used(a.slot):,} tokens in 5 h (local estimate); job sent anyway", file=sys.stderr)
    if a.resume:
        last = [r for r in rows() if r["session"] == a.resume]
        if last and last[-1].get("last_context", 0) > MAX_RESUME_CONTEXT:
            print(f"session context is over {MAX_RESUME_CONTEXT:,}; start a new session with a handoff", file=sys.stderr)
            return 75
    cmd = ["claude-sol", "--slot", str(a.slot), "--max-turns", str(a.turns), "--output-format", "json"]
    if a.resume:
        cmd += ["--resume", a.resume]
    allowed = ["Read", "Grep", "Glob", "Bash(rg:*)"]
    if a.ida:
        allowed.append("mcp__ida__*")
    if a.write:
        abs_root = f"/{ROOT}/bns-server/**"  # "//" prefix = absolute path rule
        allowed += ["Write(bns-server/**)", "Edit(bns-server/**)",
                    f"Write({abs_root})", f"Edit({abs_root})", "Bash(cargo:*)"]
    if a.emu:
        allowed.append("Bash(bns-server/.venv/bin/python bns-server/Program/Tools/Emu/*)")
    cmd += ["--allowedTools", ",".join(allowed)]
    cmd += ["-p", Path(a.prompt).read_text()]
    proc = subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        out, err = proc.communicate(timeout=JOB_TIMEOUT)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGTERM)  # whole group: no orphan Sol session keeps editing files
        out, err = proc.communicate()
        print(f"job timed out after {JOB_TIMEOUT} s; process group stopped", file=sys.stderr)
        return 124
    run = subprocess.CompletedProcess(cmd, proc.returncode, out, err)
    if run.returncode != 0 and not run.stdout.strip():
        print(run.stderr[-500:], file=sys.stderr)
        return run.returncode
    data = json.loads(run.stdout)
    if isinstance(data, list):
        data = [x for x in data if x.get("type") == "result"][-1]
    Path(a.out).write_text(data.get("result") or "")
    u = data.get("usage", {})
    row = {
        "ts": int(time.time()), "slot": a.slot, "session": data.get("session_id"),
        "turns": data.get("num_turns"), "ms": data.get("duration_ms"), "subtype": data.get("subtype"),
        "input": u.get("input_tokens", 0), "cached": u.get("cache_read_input_tokens", 0),
        "output": u.get("output_tokens", 0),
    }
    row["last_context"] = last_context(row["session"])
    earlier = max((r.get("subagent_cum", 0) for r in rows() if r["session"] == row["session"]), default=0)
    row["subagent_cum"] = subagent_total(row["session"])
    row["total"] = row["input"] + row["cached"] + row["output"] + max(0, row["subagent_cum"] - earlier)
    with LEDGER.open("a") as f:
        f.write(json.dumps(row) + "\n")
    print(json.dumps(row))
    return 0


if __name__ == "__main__":
    sys.exit(main())
