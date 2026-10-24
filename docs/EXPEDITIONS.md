# Playable bounded expeditions

Status: the accepted first-release format. Normal launch opens or resumes a human delver's expedition. `--expedition` remains a compatible explicit selector. The authored goblin prototype is independently preserved and available through `--prototype`. The three-floor simulation and proof relation are unchanged by the new protagonist.

## Play offline

```sh
cargo run --release --locked
```

The first run creates a private expedition secret using operating-system randomness. Its SHA-256 digest is the expedition identity; the first eight digest bytes, interpreted as an unsigned big-endian integer, select the world seed. This secret is not a wallet key. Subsequent runs load the same save. No wallet, RPC connection, project server, or internet access is needed after the binary has been built.

Arrows, WASD, or HJKL move. Bumping a soft or hard vein digs it for 2 or 3 charge; the next move enters the opened passage. Stand on a flask (`!`) or portal (`<` / `>`) and press E. A flask restores 8 charge up to capacity 12. Walking and portals cost no charge. G changes the view; Z/X zoom sprites. Fog stays enabled, and the camera never shakes.

Three floors contain increasingly valuable crystals: 1, 4, and 16 score. Deeper floors contain fewer crystals on average. Portals work in both directions; opened passages and consumed objects stay changed. The expedition permits 512 successful actions. Invalid actions consume no charge or actions. Exhaustion does not prevent walking or retreat.

Q exits with progress saved. Repeating the command resumes. F finishes the attempt at its current score and prevents further play in that save. Visiting every floor is optional; sealing after one crystal is an early finish, not full exploration. Reaching 512 actions prevents further core actions but still permits sealing and exit. Sealing is not a token claim or an on-chain operation.

## Saves and new attempts

Default save locations:

| Platform | Path |
| --- | --- |
| Linux | `$XDG_DATA_HOME/diggerchain/expedition.json`, or `$HOME/.local/share/diggerchain/expedition.json` |
| macOS | `$HOME/Library/Application Support/DiggerChain/expedition.json` |
| Windows | `%LOCALAPPDATA%\DiggerChain\expedition.json` |

Use `--save PATH` to choose a file. Each accepted action writes a temporary file, flushes it, and atomically replaces the save. A process lock prevents two game instances from writing the same save. The lock is released automatically when the process ends, including a crash. Linux and macOS save files are created with mode 0600; Windows uses the containing directory's access controls.

The save contains the private expedition secret and ordered action history. Back it up privately; do not publish it or send it with bug reports. Losing the secret loses control of that expedition's future reward proof. The game never stores wallet spending keys. Saves are unencrypted and rely on local account access controls. Loading replays and validates every action; unsupported versions and invalid histories fail without rewriting the file. Presentation fog is reconstructed from the history.

```sh
cargo run --release --locked -- --expedition --new
```

`--new` archives the prior save in an `archive` directory beside it before writing a fresh attempt. It does not delete the old expedition. To reopen an archive, pass its path with `--save`.

For a repeatable practice scenario:

```sh
cargo run --release --locked -- --expedition --new --seed 0 --save target/expedition-practice.json
```

Explicit seeds create practice runs without a private reward identity. This avoids claiming ownership of arbitrary chosen seeds. The v1 generator is a small seeded chamber layout, not a guarantee that every player's geometry is globally unique. Seed grinding and automation are allowed.

## External agents

```sh
cargo run --release --locked -- --expedition --headless
```

The initial observation and every response are one JSON line. Commands: `look`, `north`, `south`, `west`, `east`, `dig-north`, `dig-south`, `dig-west`, `dig-east`, `interact`, `portal`, `wait`, `save`, `finish`, and `quit`. Movement and digging are separate protocol actions. `look` and `save` do not advance time. EOF exits cleanly. The protocol exposes the current floor's visible objects and discovered map, never the private secret or hidden floor grids. The same save, action limit, and simulation serve human and agent play.

## Optional focused user check

The maintainer deferred additional test execution on 2026-10-22. This scenario remains available for later verification; it is not a launch prerequisite.

```sh
cargo test --workspace --locked -j 2
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --release --locked -- --expedition --new --seed 0 --save target/expedition-practice.json
```

In the practice world, start at `(2,2)`. Walk right to the soft vein at `(6,3)`, bump it once, then enter. Charge drops by 2 only when digging. Use a flask with E, collect a crystal, quit, and resume with the same command minus `--new --seed 0`. Position, charge, score, changed terrain, and discovery must survive. Find the descending portal in the southwest chamber, descend, and return through the arrival portal; no floor may reset. Test G, resizing, and Q. Finally press F and check that movement is rejected after resuming. `cargo run --release --locked -- --prototype` opens the authored playground.

The automated suite includes native history and identity checks, invalid-save rejection, save locking and archival, fog and secret boundaries, sealed-state behavior, presentation collision agreement, and both render modes. The assistant compiles and lints these targets; the user executes tests and owns visual acceptance. Native Windows and macOS checks remain pending.
