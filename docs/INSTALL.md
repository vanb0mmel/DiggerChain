# DiggerChain native candidate

For the public game-only alpha, use [ALPHA_INSTALL.md](ALPHA_INSTALL.md). This document preserves the separate development proof/reward package.

This archive contains the terminal game and the separate reward-proof tool. It is a candidate for user acceptance, not confirmation of a funded or deployed v1.0 economy. Check the archive and internal SHA-256 checksums against the release record before use.

## Run

Extract both executables into one directory. Linux and macOS users may need `chmod +x diggerchain diggerchain-proof`; Windows binaries have the `.exe` suffix. Use a UTF-8 TrueColor terminal with at least 90 columns and 30 rows. Windows Terminal is recommended on Windows.

```sh
./diggerchain
```

On Windows PowerShell, use `./diggerchain.exe`. Normal launch opens or resumes a human delver's three-floor expedition; `--expedition` remains supported. No Rust compiler is needed for a native binary. The game runs offline, without a connected wallet. Do not use a downloaded file if the operating system reports an unexpected architecture.

Move with arrows/WASD/HJKL. Bump a vein to dig; E uses a flask or portal underfoot. G changes view, Z/X zoom sprites, ? opens help, and Q exits. Every accepted action saves. Run the same command to resume. F finishes the attempt at its current score; visiting all three floors is optional. A new attempt uses `--new` and archives the previous save. Explicit `--seed N` creates a practice attempt without a reward identity.

Use `--prototype` for the preserved goblin playground. That mode has a separate three-crystal objective and no persistent expedition.

## Saves and agents

Default save: Linux `$XDG_DATA_HOME/diggerchain/expedition.json` or `~/.local/share/diggerchain/expedition.json`; macOS `~/Library/Application Support/DiggerChain/expedition.json`; Windows `%LOCALAPPDATA%\DiggerChain\expedition.json`. Override with `--save PATH`. Back up saves privately: they contain an expedition secret, not a wallet spending key. Never attach a save to a public issue.

```sh
./diggerchain --headless
```

Agents exchange one JSON line per command through stdin/stdout. Commands: look, north/south/west/east, dig-north/dig-south/dig-west/dig-east, interact, portal, wait, save, finish, quit. Movement and digging are separate actions. No LLM subscription, OAuth provider, or agent manager is embedded in the game.

## Rewards

The matching fixed parameter bundle and reviewed wallet handoff are separate public artifacts. Ordinary play does not load the proving key. `diggerchain-proof --help` describes offline proof generation for a sealed non-practice save. The proof tool checks parameter fingerprints and never generates substitute parameters while proving. Do not run `setup` as an ordinary player.

Extract the approved public bundle into the executable directory; it adds `parameters/` and `wallet.html`. Then run:

```sh
./diggerchain --wallet wallet.html
```

Open the printed localhost address in the browser with MetaMask/Rabby. A player page already contains the reviewed contract addresses and has no deployment controls. It displays native game and proof commands for the selected wallet and shell (including Windows PowerShell). The game serves public HTML only; it never receives a private save through HTTP or asks for wallet spending keys. Rust, Python, Node.js, and Foundry are not runtime requirements for these native player binaries.

Keep wallet spending keys in the external wallet. Only public proof output is shared for registration. Submission must use an open epoch, and each expedition can register once across the deployment's epochs. Claims become available after the deadline and require a separate wallet signature. Live contract addresses, launch schedule, token budget, and final setup assumptions must come from the concrete approved deployment record; this candidate archive supplies none by implication.

## Platform limits

Record actual platform builds separately from runtime checks. The prepared manual build matrix targets Linux x86-64, Windows x86-64, and the native macOS runner architecture. Other architectures can build from the same source with Rust 1.89 or newer; they are not certified by a build for a different architecture. The CI Linux candidate targets glibc 2.35 or newer; a locally built archive uses its build host's libc and is labeled accordingly. macOS and Windows builds have not yet been recorded.

The authenticated fixed-key proof run used approximately 1.29 GiB peak resident memory and 54.579 seconds overall, including 46.416 seconds for key loading/validation and 5.858 seconds for proving. TUI/headless play does not require that proving working set.

The maintainer deferred remaining runtime and reward-flow tests on 2026-10-22 to accelerate launch. The previous 62 checks, fixed setup, saved-game proof, and testnet deployments passed; a complete registration/payout and the new presentation changes have not been verified end to end. This decision does not change save validation, proof checks, contract rules, or wallet signing. Publication and any funded deployment remain explicit maintainer actions.
