# Card perf-1: DXVK trial on the client (operator task, not a slice)

## Goal
Measure if DXVK (DirectX 9 → Vulkan) makes the 381 client faster. Keep it only if it is faster and stable.

## Context
- The client is 32-bit UE3, DirectX 9 (`d3dx9_35.dll`, `D3DX9_43.dll` in `bin`). The engine is `bsengine_Shipping.dll`. Large Address Aware is already set in `Client.exe`.
- DXVK for a 32-bit DX9 game: put the 32-bit (x32) `d3d9.dll` from a DXVK release next to `Client.exe` in `bin`.
- Run on the laptop with `tools/client_run.py`, and later on the GPU host.

## Work
1. Baseline without DXVK: same scene (character select, then one busy town), 120 s, record FPS (DXVK HUD cannot run without DXVK: use the in-game FPS display or PresentMon), CPU and GPU use.
2. Add the DXVK x32 `d3d9.dll` and `dxvk.conf` (`dxvk.hud = fps,frametimes`, `d3d9.maxFrameLatency = 1`). Same test.
3. Compare: average FPS, 1 % low, load time, crashes. Write the result in `Docs/Client/RE/perf.md`.
4. Remove the DLL if it is not better. Keep a backup of `bin` before the change.

## Out of scope
- No change to `Client.exe` or `bsengine_Shipping.dll`.
