#!/usr/bin/env python3
"""Start the real client on the laptop, record the traffic, collect results.

Usage: tools/client_run.py [--secs 60] [--shots 20,40,60] [--log FILE ...]

Steps: tcpdump on tailscale0 -> start.bat through a /it scheduled task ->
screenshots -> list new client files -> kill Client.exe -> short summary.
Output: logs/client-runs/<time>/ (cap.pcap, shot-*.png, summary.txt).
"""
import argparse, os, pathlib, subprocess, sys, time

REPO = pathlib.Path(__file__).resolve().parent.parent
LAPTOP = "katie@100.99.212.54"
SSH = ["ssh", "-q", "-o", "ConnectTimeout=8", "-i", os.path.expanduser("~/.ssh/bns_laptop"), "-p", "33222", LAPTOP]
SCP = ["scp", "-q", "-i", os.path.expanduser("~/.ssh/bns_laptop"), "-P", "33222"]
HOME = r"C:\Users\Katie"
CLIENT_DIR = r"C:\Users\Katie\OneDrive\Documents\College work\Desktop\BNS_Client"

RUN_CMD = f'@echo off\r\ncd /d "{CLIENT_DIR}"\r\ncall start.bat\r\n'
SHOT_PS1 = r"""Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type 'using System.Runtime.InteropServices; public class D { [DllImport("user32.dll")] public static extern bool SetProcessDPIAware(); }'
[D]::SetProcessDPIAware() | Out-Null
$b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp=New-Object System.Drawing.Bitmap $b.Width,$b.Height
$g=[System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size)
$bmp.Save("$env:USERPROFILE\bns-shot.png")
"""
# Types the login. Reads the file, deletes it at once, focuses the client, types.
LOGIN_PS1 = r"""$f="$env:USERPROFILE\bns-login.txt"
$id,$pw=(Get-Content $f -Raw).Trim() -split "`t"
Remove-Item $f -Force
function Esc($s){ ($s -replace '([+^%~(){}\[\]])','{$1}') }
$w=New-Object -ComObject WScript.Shell
if(-not $w.AppActivate('Blade & Soul')){ exit 2 }
Start-Sleep -Milliseconds 700
$w.SendKeys((Esc $id)); Start-Sleep -Milliseconds 300
$w.SendKeys('{TAB}'); Start-Sleep -Milliseconds 300
$w.SendKeys((Esc $pw)); Start-Sleep -Milliseconds 300
$w.SendKeys('{ENTER}')
"""
NEW_FILES_PS1 = r"""param([string]$Since)
$t=[datetime]::Parse($Since)
$dirs=@("{client}","$env:USERPROFILE\Documents\BnS","$env:LOCALAPPDATA\CrashDumps")
foreach($d in $dirs){{ if(Test-Path $d){{ Get-ChildItem $d -Recurse -File -EA SilentlyContinue |
  Where-Object {{ $_.LastWriteTime -gt $t }} | ForEach-Object {{ "{{0}}`t{{1}}" -f $_.Length,$_.FullName }} }} }}
""".format(client=CLIENT_DIR)


def ssh(cmd, check=False):
    r = subprocess.run(SSH + [cmd], capture_output=True, text=True, timeout=60)
    if check and r.returncode:
        raise SystemExit(f"ssh failed: {cmd}\n{r.stderr}")
    return r.stdout.strip()


def put(name, text):
    p = OUT / name
    p.write_text(text, newline="")
    subprocess.run(SCP + [str(p), f"{LAPTOP}:{name}"], check=True, timeout=60)


def task(name, tr):
    # /rl highest: Client.exe needs admin. Without it, a UAC prompt blocks the run.
    ssh(f'schtasks /create /f /tn {name} /tr "{tr}" /sc once /st 23:59 /it /rl highest >nul', check=True)
    ssh(f"schtasks /run /tn {name} >nul", check=True)


def login():
    """Copy the login file (secrets/, never logged), run the typing task."""
    cred = REPO / "secrets" / "client-login.tsv"
    tmp = REPO / "secrets" / "bns-login.txt"
    tmp.write_text(cred.read_text().strip())
    try:
        subprocess.run(SCP + [str(tmp), f"{LAPTOP}:bns-login.txt"], check=True, timeout=60)
    finally:
        tmp.unlink()
    task("bns-login", rf"powershell -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File {HOME}\bns-login.ps1")


def client_alive():
    return "Client.exe" in ssh('tasklist /fi "imagename eq Client.exe" /nh')


def shot(label):
    task("bns-shot", rf"powershell -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File {HOME}\bns-shot.ps1")
    time.sleep(4)
    dst = OUT / f"shot-{label}.png"
    r = subprocess.run(SCP + [f"{LAPTOP}:bns-shot.png", str(dst)], capture_output=True, timeout=60)
    return dst.name if r.returncode == 0 else "failed"


def pcap_summary(pcap):
    r = subprocess.run(["tcpdump", "-r", str(pcap), "-nn", "-q", "tcp"], capture_output=True, text=True)
    flows = {}
    for line in r.stdout.splitlines():
        parts = line.split()
        if len(parts) < 6 or parts[1] != "IP":
            continue
        src, dst = parts[2], parts[4].rstrip(":")
        try:
            n = int(parts[-1])
        except ValueError:
            continue
        laptop_to_srv = src.startswith("100.99.")
        port = dst.rsplit(".", 1)[1] if laptop_to_srv else src.rsplit(".", 1)[1]
        cport = src.rsplit(".", 1)[1] if laptop_to_srv else dst.rsplit(".", 1)[1]
        f = flows.setdefault((port, cport), [0, 0, 0])
        f[0 if laptop_to_srv else 1] += n
        f[2] += 1
    rows = [f"server port {p} client port {c}: client->server {a} B, server->client {b} B, {k} packets"
            for (p, c), (a, b, k) in sorted(flows.items())]
    return rows or ["no TCP traffic from the laptop"]


def main():
    global OUT
    ap = argparse.ArgumentParser()
    ap.add_argument("--secs", type=int, default=60)
    ap.add_argument("--shots", default="20,40,60")
    ap.add_argument("--login-at", type=int, default=0, help="type the login at this second (0 = no)")
    ap.add_argument("--log", action="append", default=[], help="server log to tail (new lines only)")
    a = ap.parse_args()

    OUT = REPO / "logs" / "client-runs" / time.strftime("%Y%m%d-%H%M%S")
    OUT.mkdir(parents=True)
    logs = {p: (os.path.getsize(p) if os.path.exists(p) else 0) for p in a.log}

    if client_alive():
        raise SystemExit("Client.exe already runs on the laptop. Stop it first.")
    since = ssh('powershell -NoProfile -Command "Get-Date -Format o"', check=True)
    put("run-bns.cmd", RUN_CMD)
    put("bns-shot.ps1", SHOT_PS1)
    put("bns-newfiles.ps1", NEW_FILES_PS1)
    put("bns-login.ps1", LOGIN_PS1)

    pcap = OUT / "cap.pcap"
    dump = subprocess.Popen(["sudo", "-n", "tcpdump", "-i", "tailscale0", "-nn", "-U", "-w", str(pcap),
                             "host", LAPTOP.split("@")[1], "and", "not", "port", "33222"],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(2)

    t0 = time.time()
    task("bns-run", rf"{HOME}\run-bns.cmd")
    events, shots, died = [], [], None
    marks = sorted(int(x) for x in a.shots.split(",") if x)
    while time.time() - t0 < a.secs:
        el = int(time.time() - t0)
        if marks and el >= marks[0]:
            shots.append(f"{marks.pop(0)} s: {shot(str(el))}")
        if a.login_at and el >= a.login_at:
            login()
            events.append(f"login typed at {el} s")
            a.login_at = 0
        if el > 8 and died is None and not client_alive():
            died = el
            events.append(f"Client.exe not running at {el} s")
            shots.append(f"{el} s (after exit): {shot('exit')}")
            break
        time.sleep(3)

    new_files = ssh(f'powershell -NoProfile -ExecutionPolicy Bypass -File {HOME}\\bns-newfiles.ps1 -Since "{since}"')
    alive = client_alive()
    ssh("taskkill /im Client.exe /f >nul 2>&1")
    ssh("schtasks /delete /f /tn bns-run >nul 2>&1 & schtasks /delete /f /tn bns-shot >nul 2>&1 & schtasks /delete /f /tn bns-login >nul 2>&1 & del /q bns-login.txt >nul 2>&1")
    subprocess.run(["sudo", "-n", "pkill", "-INT", "-f", f"tcpdump -i tailscale0 -nn -U -w {pcap}"])
    dump.wait(timeout=10)

    out = [f"run dir: {OUT}", f"duration: {int(time.time() - t0)} s",
           f"client alive at end: {alive}", *events, "", "traffic:", *pcap_summary(pcap), "",
           "screenshots:", *shots, "", "new client files:", *(new_files.splitlines()[:25] or ["none"])]
    for p, off in logs.items():
        with open(p, "rb") as fh:
            fh.seek(off)
            lines = fh.read().decode(errors="replace").splitlines()
        out += ["", f"log {p}: {len(lines)} new lines, last 12:", *[l[:220] for l in lines[-12:]]]
    text = "\n".join(out)
    (OUT / "summary.txt").write_text(text + "\n")
    print(text)


if __name__ == "__main__":
    main()
