#!/usr/bin/env python3
"""Run one slice in a new session, gate it, and merge it into main.

Usage: slice_runner.py status
       slice_runner.py run [ID] [--dry-run] [--no-push]

Queue: tools/slices/queue.tsv (tab-separated)
  id     slice id, also the card name: Docs/Plan/cards/<id>.md
  kind   rust (Sonnet builder, no IDA) or spec (Opus, IDA)
  model  sonnet or opus
  crate  cargo package to check (rust), or - (spec)
  paths  comma-separated path prefixes the session may change
  deps   comma-separated slice ids that must be done first, or -
  turns  turn limit
  status ready | running | done | blocked
  title  short title (commit subject)

Flow: account check -> worktree outside private-project -> session -> gate
      -> on fail: 1 repair (same session, error tail only) -> gate
      -> pass: commit, fast-forward main, push; fail: blocked (worktree kept).
Ledger: notes/slice-usage.jsonl (one row per session).
Exit codes: 0 done, 1 blocked, 2 nothing to run, 75 no account with quota.
"""
import argparse, json, os, re, subprocess, sys, time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]           # bns-server
PROJECT = REPO.parent                                # private-project
QUEUE = REPO / "tools/slices/queue.tsv"
LEDGER = PROJECT / "notes/slice-usage.jsonl"
WT_ROOT = PROJECT.parent / "bns-wt"                   # outside private-project: no supervisor CLAUDE.md or memory
TARGET = Path.home() / ".cache/bns-wt-target"         # one builder at a time, so one shared target dir
MCP = PROJECT / ".mcp.json"
GATEWAY_KEY = Path.home() / ".local/state/nl-parts-pipeline/operator-access/home-gateway/codex-handover/worker-api-key"
TOOLS = "Read,Edit,Write,Bash,Grep,Glob"
ALLOW = {
    "rust": "Read,Grep,Glob,Bash(cargo:*),Bash(rg:*),Bash(git diff:*),Bash(git status:*),Bash(python3 tools/spec_index.py:*),Bash(python3 tools/rust_api.py:*)",
    "spec": "Read,Grep,Glob,mcp__ida__*,Bash(rg:*),Bash(git diff:*),Bash(git status:*),Bash(python3 tools/spec_index.py:*),Bash(python3 tools/rust_api.py:*)",
}
RULES = {"rust": REPO / "tools/slices/builder-rules.md", "spec": REPO / "tools/slices/spec-rules.md"}
BAD_EXT = re.compile(r"\.(exe|dll|pdb|i64|idb|bin|dat|pem|key|blob|mdf|ldf|rar|pfx|cer|crt)$", re.I)
TAIL = 40
# Extra gate commands per crate (cross builds).
EXTRA_GATE = {"clientdbg": [["cargo", "build", "-q", "-p", "clientdbg", "--release", "--target", "i686-pc-windows-gnu"],
                           ["bash", "-c", "i686-w64-mingw32-objdump -p target/i686-pc-windows-gnu/release/clientdbg.dll | grep -q GPluginInfo"]]}
LIMIT_5H, LIMIT_7D = 90, 99   # cswap rule (memory: cswap-account-switching)


# ---------- queue ----------
def load():
    lines = QUEUE.read_text().splitlines()
    head = lines[0].split("\t")
    return head, [dict(zip(head, l.split("\t"))) for l in lines[1:] if l.strip()]


def save(head, rows):
    QUEUE.write_text("\n".join(["\t".join(head)] + ["\t".join(r[h] for h in head) for r in rows]) + "\n")


def set_status(sid, status):
    head, rows = load()
    for r in rows:
        if r["id"] == sid:
            r["status"] = status
    save(head, rows)


def pick(rows, sid):
    done = {r["id"] for r in rows if r["status"] == "done"}
    for r in rows:
        if sid and r["id"] != sid:
            continue
        deps = [d for d in r["deps"].split(",") if d and d != "-"]
        if r["status"] == "ready" and all(d in done for d in deps):
            return r
    return None


# ---------- accounts ----------
def accounts():
    out = subprocess.run(["cswap", "list"], capture_output=True, text=True, timeout=60).stdout
    accts, cur = [], None
    for line in out.splitlines():
        m = re.match(r"\s*(\d+): .*?(\(active\))?", line)
        if m and "·" in line:
            cur = {"n": int(m.group(1)), "active": "(active)" in line, "5h": None, "7d": None}
            accts.append(cur)
        m = re.search(r"(5h|7d):\s+(\d+)%", line)
        if m and cur:
            cur[m.group(1)] = int(m.group(2))
    return accts


def ensure_account():
    """Apply the cswap rule. Return the active account row, or None if no account has quota."""
    accts = accounts()
    ok = lambda a: a["5h"] is not None and a["7d"] is not None and a["5h"] < LIMIT_5H and a["7d"] < LIMIT_7D
    active = next((a for a in accts if a["active"]), None)
    if active and ok(active):
        return active
    target = next((a for a in sorted(accts, key=lambda a: a["n"]) if ok(a)), None)
    if not target:
        return None
    subprocess.run(["cswap", "switch", str(target["n"])], check=True, capture_output=True, timeout=60)
    return target


# ---------- session ----------
def session_cmd(row, prompt, resume=None):
    cmd = ["claude", "-p", "--model", row["model"], "--max-turns", row["turns"], "--output-format", "json",
           "--disable-slash-commands", "--tools", TOOLS, "--permission-mode", "dontAsk",
           "--allowedTools", ALLOW[row["kind"]] + f",Edit(/{WT_ROOT}/{row['id']}/**),Write(/{WT_ROOT}/{row['id']}/**)", "--append-system-prompt-file", str(RULES[row["kind"]])]
    cmd += ["--mcp-config", str(MCP), "--strict-mcp-config"] if row["kind"] == "spec" else ["--strict-mcp-config"]
    if resume:
        cmd += ["--resume", resume]
    return cmd + [prompt]


def run_session(row, wt, prompt, resume=None):
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET))
    t0 = time.time()
    p = subprocess.run(session_cmd(row, prompt, resume), cwd=wt, capture_output=True, text=True, env=env, timeout=4 * 3600)
    try:
        data = json.loads(p.stdout)
        if isinstance(data, list):
            data = [x for x in data if x.get("type") == "result"][-1]
    except (ValueError, IndexError):
        data = {"subtype": "no_json", "result": (p.stderr or p.stdout)[-500:]}
    u = data.get("usage", {})
    rec = {"ts": int(t0), "slice": row["id"], "kind": row["kind"], "model": row["model"],
           "repair": bool(resume), "session": data.get("session_id"), "subtype": data.get("subtype"),
           "turns": data.get("num_turns"), "s": int(time.time() - t0),
           "input": u.get("input_tokens", 0), "cache_write": u.get("cache_creation_input_tokens", 0),
           "cache_read": u.get("cache_read_input_tokens", 0), "output": u.get("output_tokens", 0),
           "cost_usd_equiv": data.get("total_cost_usd")}
    return rec


# ---------- gate ----------
def sh(cmd, cwd):
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=str(TARGET)))
    return p.returncode, (p.stdout + p.stderr)


def changed(wt):
    _, out = sh(["git", "status", "--porcelain", "--untracked-files=all"], wt)
    return [l[3:].split(" -> ")[-1] for l in out.splitlines() if l.strip()]


def gate(row, wt):
    """Return '' when all checks pass, else the failure text (tail only)."""
    files = changed(wt)
    handoff = f"Docs/Plan/handoffs/{row['id']}.md"
    if handoff not in files:
        return f"missing {handoff}"
    allowed = [p.strip() for p in row["paths"].split(",") if p.strip()] + ["Docs/Plan/handoffs/", "Docs/Plan/cards/"]
    outside = [f for f in files if not any(f.startswith(a) for a in allowed)]
    if outside:
        return "changed files outside the card paths:\n" + "\n".join(outside)
    bad = [f for f in files if BAD_EXT.search(f)]
    if bad:
        return "binary or key files:\n" + "\n".join(bad)
    sh(["git", "add", "-A"], wt)
    _, diff = sh(["git", "diff", "--cached"], wt)
    sh(["git", "reset", "-q"], wt)
    if re.search(r"^\+.*BEGIN [A-Z ]*PRIVATE KEY", diff, re.M):
        return "private key in diff"
    if GATEWAY_KEY.is_file():
        key = GATEWAY_KEY.read_text().strip()
        if key and key in diff:
            return "gateway key in diff"
    if row["kind"] == "rust":
        c = row["crate"]
        for cmd in (["cargo", "fmt", "-p", c, "--", "--check"], ["cargo", "test", "-q", "-p", c],
                    ["cargo", "clippy", "-q", "-p", c, "--all-targets", "--", "-D", "warnings"],
                    *EXTRA_GATE.get(c, [])):
            rc, out = sh(cmd, wt)
            if rc:
                return f"$ {' '.join(cmd)}\n" + "\n".join(out.splitlines()[-TAIL:])
    return ""


# ---------- main flow ----------
def run(sid, dry, push):
    head, rows = load()
    row = pick(rows, sid)
    if not row:
        print("nothing to run (no ready slice with done deps)")
        return 2
    card = REPO / f"Docs/Plan/cards/{row['id']}.md"
    prompt = card.read_text()
    wt = WT_ROOT / row["id"]
    if dry:
        print(" ".join(session_cmd(row, f"<{card.name}>")))
        print(f"cwd={wt}")
        return 0
    acct = ensure_account()
    if not acct:
        print("no account under 5h 90 % and 7d 99 %; stop")
        return 75
    WT_ROOT.mkdir(exist_ok=True)
    branch = f"slice/{row['id']}"
    if not wt.exists():
        rc, out = sh(["git", "worktree", "add", "-q", "-b", branch, str(wt), "main"], REPO)
        if rc:
            print(out)
            return 1
    set_status(row["id"], "running")
    log = []
    rec = run_session(row, wt, prompt)
    rec["account"], rec["acct_5h"], rec["acct_7d"] = acct["n"], acct["5h"], acct["7d"]
    log.append(rec)
    fail = gate(row, wt)
    if fail and rec.get("session"):
        rep = run_session(row, wt, "The gate failed. Fix only this. Then run the checks again and update the handoff.\n\n" + fail,
                          resume=rec["session"])
        log.append(rep)
        fail = gate(row, wt)
    with LEDGER.open("a") as f:
        for r in log:
            r["gate"] = "fail" if fail else "pass"
            f.write(json.dumps(r) + "\n")
    tokens = sum(r["input"] + r["cache_write"] + r["cache_read"] + r["output"] for r in log)
    if fail:
        set_status(row["id"], "blocked")
        print(f"BLOCKED {row['id']} ({tokens:,} tokens). Worktree kept: {wt}\n{fail}")
        return 1
    sh(["git", "add", "-A"], wt)
    msg = f"{row['id']}: {row['title']}\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
    rc, out = sh(["git", "-c", "user.name=Claude", "-c", "user.email=noreply@anthropic.com", "commit", "-q", "-m", msg], wt)
    if rc:
        set_status(row["id"], "blocked")
        print("commit failed:\n" + out)
        return 1
    # main may have moved while the slice ran (queue or card commits). Rebase first.
    rc, out = sh(["git", "rebase", "-q", "main"], wt)
    if rc:
        sh(["git", "rebase", "--abort"], wt)
    rc, out = sh(["git", "merge", "-q", "--ff-only", branch], REPO)
    if rc:
        set_status(row["id"], "blocked")
        print(f"merge into main failed; branch {branch} kept:\n{out}")
        return 1
    set_status(row["id"], "done")
    sh(["git", "add", "tools/slices/queue.tsv"], REPO)
    sh(["git", "-c", "user.name=Claude", "-c", "user.email=noreply@anthropic.com", "commit", "-q", "-m",
        f"queue: {row['id']} done\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>", "--", "tools/slices/queue.tsv"], REPO)
    sh(["git", "worktree", "remove", "--force", str(wt)], REPO)
    sh(["git", "branch", "-q", "-D", branch], REPO)
    if row["kind"] == "spec":
        chain_rust(row)
    if push:
        rc, out = sh(["bash", "tools/push_github.sh"], REPO)
        print(out.strip().splitlines()[-1] if out.strip() else f"push rc={rc}")
    print(f"DONE {row['id']} ({tokens:,} tokens, {sum(r['s'] for r in log)} s)")
    return 0


def chain_rust(row):
    """After a spec slice: queue its Rust card (<id>-rust) if the card exists."""
    rid = f"{row['id']}-rust"
    card = REPO / f"Docs/Plan/cards/{rid}.md"
    _, rows = load()
    if not card.is_file() or any(r["id"] == rid for r in rows):
        return
    text = card.read_text()
    crate = re.search(r"cargo (?:test|clippy) -p ([\w-]+)", text)
    files = re.search(r"^## Files\n(.*?)(?=^## |\Z)", text, re.S | re.M)
    paths = [m.split()[0] for m in re.findall(r"^- `([^`]+)`", files.group(1), re.M)] if files else []
    title = re.search(r"^# (.+)$", text, re.M)
    if not crate or not paths:
        print(f"chain: {rid} not queued (no crate or Files list in card)")
        return
    paths.append("Cargo.lock")
    t = (title.group(1).split(":", 1)[-1].strip() if title else rid).replace("\t", " ")
    with QUEUE.open("a") as f:
        f.write("\t".join([rid, "rust", "sonnet", crate.group(1), ",".join(paths), row["id"], "120", "ready", t]) + "\n")
    sh(["git", "add", "tools/slices/queue.tsv"], REPO)
    sh(["git", "-c", "user.name=Claude", "-c", "user.email=noreply@anthropic.com", "commit", "-q", "-m",
        f"queue: add {rid}\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>", "--", "tools/slices/queue.tsv"], REPO)
    print(f"chain: queued {rid} ({crate.group(1)})")


def salvage():
    """Merge blocked slices whose gate passed but whose ff merge failed (main moved)."""
    recs = [json.loads(l) for l in LEDGER.read_text().splitlines() if l.strip()] if LEDGER.is_file() else []
    last = {r["slice"]: r.get("gate") for r in recs}
    _, rows = load()
    for row in rows:
        wt, branch = WT_ROOT / row["id"], f"slice/{row['id']}"
        if row["status"] != "blocked" or last.get(row["id"]) != "pass" or not wt.is_dir():
            continue
        _, ahead = sh(["git", "rev-list", "--count", f"main..{branch}"], REPO)
        if ahead.strip() in ("", "0"):
            continue
        rc, _ = sh(["git", "rebase", "-q", "main"], wt)
        if rc:
            sh(["git", "rebase", "--abort"], wt)
            print(f"salvage: {row['id']} rebase conflict, left blocked")
            continue
        rc, out = sh(["git", "merge", "-q", "--ff-only", branch], REPO)
        if rc:
            print(f"salvage: {row['id']} merge failed:\n{out}")
            continue
        set_status(row["id"], "done")
        sh(["git", "-c", "user.name=Claude", "-c", "user.email=noreply@anthropic.com", "commit", "-q", "-m",
            f"queue: {row['id']} done (salvaged)\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>", "--", "tools/slices/queue.tsv"], REPO)
        sh(["git", "worktree", "remove", "--force", str(wt)], REPO)
        sh(["git", "branch", "-q", "-D", branch], REPO)
        if row["kind"] == "spec":
            chain_rust(row)
        print(f"salvage: {row['id']} merged")


def loop(push, max_slices):
    """Run ready slices one by one. Skip past blocked ones. Stop on empty queue or no quota."""
    salvage()
    for _ in range(max_slices):
        rc = run(None, False, push)
        salvage()
        if rc in (2, 75):
            return rc
    return 0


def status():
    _, rows = load()
    for r in rows:
        print(f"{r['status']:8} {r['id']:16} {r['kind']:4} {r['model']:6} {r['title']}")
    if LEDGER.is_file():
        recs = [json.loads(l) for l in LEDGER.read_text().splitlines() if l.strip()]
        tot = sum(r["input"] + r["cache_write"] + r["cache_read"] + r["output"] for r in recs)
        waste = sum(r["input"] + r["cache_write"] + r["cache_read"] + r["output"] for r in recs if r.get("gate") == "fail" or r.get("repair"))
        print(f"ledger: {len(recs)} sessions, {tot:,} tokens, of which repair or failed: {waste:,}")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=("status", "run", "loop"))
    ap.add_argument("id", nargs="?")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--no-push", action="store_true")
    ap.add_argument("--max", type=int, default=8, help="loop: maximum slices")
    a = ap.parse_args()
    if a.cmd == "loop":
        return loop(not a.no_push, a.max)
    return status() if a.cmd == "status" else run(a.id, a.dry_run, not a.no_push)


if __name__ == "__main__":
    sys.exit(main())
