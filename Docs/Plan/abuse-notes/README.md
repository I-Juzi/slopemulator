# Abuse notes

Agents write a note here (`<slice id>.md`) when they see a way that players can abuse the copied original behaviour. Nothing is fixed before full parity. See `../known-exploits.md`.

**The notes never go to GitHub.** The slice runner moves each note to the local folder `notes/abuse-notes/` (outside the repo) before the commit. `tools/push_github.sh` refuses a push that contains a note.

Why: the repo is public. A note can also apply to a live game (NEO, UE4) with the same code. Never test a note on a live server. If a note looks like it works on live, report it to NCSoft in private.
