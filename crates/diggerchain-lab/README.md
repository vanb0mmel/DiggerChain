# DiggerChain feasibility fixture

This is the native gameplay part of M2: a small two-floor experiment for checking the proposed reward rules. It is not a cryptographic proof, production world generator, or replacement for the authored visual prototype. It performs no network requests or blockchain transactions and uses no new third-party dependencies.

Simulation lives in `diggerchain-core::expedition`. This executable provides a line-based adapter, an inspectable demonstration, and a JSON trace. It does not initialize a terminal renderer or change the normal game command.

## First user check

From the main workspace:

```sh
cargo run --release --locked -j 2 -p diggerchain-lab -- demo
```

Expected final fields:

```json
{
  "cryptographic_proof": false,
  "seed": 0,
  "floor": 0,
  "position": { "x": 3, "y": 1 },
  "charge": 7,
  "capacity": 12,
  "score": 5,
  "turn": 36
}
```

There are 39 requested actions. Three deliberate rejections occur in this order: an already consumed flask on floor 1, an already opened ground cell targeted by digging on floor 0, and an already consumed flask on floor 0. These actions must not advance the turn or change any other state. The demonstration also returns to the shallow crystal cell without receiving its score again.

Run the mirrored layout:

```sh
cargo run --release --locked -j 2 -p diggerchain-lab -- demo 1
```

Its final fields are identical except for the seed. The route mirrors horizontal directions automatically. Seed parity is only an inspectable fixture variation; many seeds intentionally produce the same map.

## Manual commands

```sh
cargo run --release --locked -j 2 -p diggerchain-lab -- play
```

Type one command per line: `look`, `north`, `south`, `west`, `east`, `dig-north`, `dig-south`, `dig-west`, `dig-east`, `interact`, `portal`, `wait`, or `quit`. Each command returns JSON. EOF exits cleanly. Observe both full fixture maps in `floors`; `standing_on` preserves the terrain hidden by the player glyph. Full map exposure is limited to this lab; the normal game retains fog of war and disabled reveal.

At the start, `east` must fail because rock blocks movement. `dig-east` spends 2 charge without moving. `east` then enters the opened cell. Use `interact` while standing on a flask to replenish charge. `wait` does not regenerate charge.

## Tests and trace export

The user runs these checks; they have not been executed by the assistant:

```sh
cargo test --workspace --locked -j 2
cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings
cargo fmt --all -- --check
```

Export the route and resulting states for inspection:

```sh
cargo run --release --locked -j 2 -p diggerchain-lab -- trace 0 > target/lab-trace-0.json
```

The trace contains the initial state, 39 command outcomes, and the final state. A trace is replay data, not proof of a valid run. No save import, owner authentication, epoch authentication, proof generation, or EVM verification is implemented yet.

Provide the final demo output and any build, test, or unexpected-action errors. A passing native fixture check does not close M2: actual proving, rejected forged witnesses, and EVM verification measurements are still required.

## Recorded user validation

The user supplied a successful release build and complete seed-0 demo output for `lab-v0` in the uncommitted working tree; no fixture commit hash was recorded. The final state matched the expected floor 0, position `(3, 1)`, charge 7, score 5, and turn 36. All three deliberate rejections appeared in the expected order. The final maps retained opened terrain and consumed items after the return trip, and revisiting the shallow crystal did not increase score.

The user subsequently supplied a passing `cargo test --workspace --locked -j 2` result: 9 core tests, 2 lab tests, and 6 terminal tests, with no failures. The core checks include mirrored seeds, atomic rejection, capped replenishment, zero-charge retreat, and persistent changes across revisits. Terminal checks include both views, camera stability, and hidden-crystal observations. No documentation tests were present.

This validates the supplied native demonstration and automated checks only. The displayed 2.46 seconds measured compilation, not execution or proving performance. The standalone mirrored demo, lint, and format checks remain unreported. Cryptographic proof generation and EVM verification have not been validated.

## Related documents

- [Fixture rules](../../docs/REWARD_RULES.md)
- [Roadmap](../../docs/ROADMAP.md)
- [Preserved prototype](../../docs/PROTOTYPE.md)
