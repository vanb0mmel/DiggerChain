# DiggerChain 0.1.0-alpha.1

A mining adventure in your terminal. This alpha contains the game executable, dependency notices, and checksums. It has three seeded castle floors and a 512-action expedition limit. Alpha has no token payouts and needs no wallet, proving key, account, or internet connection to play.

## Start

Extract the ZIP and open a terminal in its folder. Linux and macOS:

```sh
chmod +x diggerchain
./diggerchain
```

Windows PowerShell:

```powershell
./diggerchain.exe
```

Use a UTF-8 terminal with full color and at least 90 columns by 30 rows. Choose an archive matching your operating system and processor. Windows Terminal is suitable on Windows. A native binary does not require Rust, Python, Node.js, or Foundry.

## Play

Move with arrows, WASD, or HJKL. Bump into a vein to dig; move again to step through. Stand on a flask or portal and press E. G changes between top-down glyphs and sprites; Z/X changes sprite zoom. Press ? for help.

Walking and portals do not use charge. Soft rock costs 2, hard rock costs 3, and a flask restores up to 8 charge, capped at 12. Crystals score 1, 4, and 16 on the three depths. Previously collected items and mined passages stay changed when you return upstairs.

Every successful action saves. Q leaves the expedition resumable. F finishes at the current score. To start another expedition:

```sh
./diggerchain --new
```

The previous save is archived. Add `--seed 42` for a repeatable practice world. `--prototype` opens the independently preserved goblin playground.

## Saves

Linux: `$XDG_DATA_HOME/diggerchain/expedition.json`, or `~/.local/share/diggerchain/expedition.json`.
macOS: `~/Library/Application Support/DiggerChain/expedition.json`.
Windows: `%LOCALAPPDATA%\DiggerChain\expedition.json`.

Use `--save PATH` to select a different file. Back up saves privately; do not attach a save to a public issue.

## External agents

```sh
./diggerchain --headless
```

The game immediately writes one JSON observation. Send one plain-text command per line; receive one JSON response per line. Commands: `look`, `north`, `south`, `west`, `east`, `dig-north`, `dig-south`, `dig-west`, `dig-east`, `interact`, `portal`, `wait`, `save`, `finish`, and `quit`. Movement and digging are separate actions. Use the observation's `legal_actions` and discovered map. Your external agent supplies its own model connection.

## Build availability

The local Linux archive uses its build machine's libc. Hosted Linux builds target Ubuntu 22.04 or newer compatible glibc environments. macOS archives are separate for Apple Silicon and Intel; Windows targets x86-64. Download availability and executed runtime checks are recorded separately. An unbuilt platform is not a published binary.

The maintainer owns runtime acceptance and has deferred additional tests. No broader platform certification is implied by a successful compilation. The archive's SHA-256 and its internal `SHA256SUMS` identify the actual files supplied.
