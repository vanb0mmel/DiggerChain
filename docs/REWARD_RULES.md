# Reward rules: first feasibility fixture

Status: historical `lab-v0` fixture specification. The implemented bounded v1 candidate is described in [REWARD_PROTOCOL.md](REWARD_PROTOCOL.md). The open decisions and pending checks below refer to the earlier fixture phase, not the current milestone status.

## Requirements retained

- Expeditions are individual and deterministic, and attempts can be created freely.
- Players may inspect or solve their own world with modified clients and external agents.
- Portals allow backtracking. Terrain changes and consumed items survive revisits.
- Digging consumes a replenishable resource. Walking and portal travel remain possible when that resource is exhausted.
- Reward evidence must describe a legal run from the declared initial state, including resource accounting and unique collections.
- Only reward-relevant state belongs in the proof. Rendering, presentation fog, frame timing, and the solver's reasoning are outside it.
- A valid run establishes a reachable result; it does not establish human participation, wall-clock effort, or that the submitted path was the exact path first played.

## Bounded fixture: `lab-v0`

The fixture has two 7 by 7 floors. It is deliberately small enough to inspect by hand. It is not an implementation of the unlimited procedural world.

| Rule | Fixture value |
| --- | --- |
| Seed | Unsigned 32-bit integer; odd seeds mirror the authored layout horizontally |
| Initial floor | 0 |
| Initial position | `(1, 1)`, mirrored when required |
| Initial charge | 4 |
| Charge capacity | 12 |
| Soft rock dig cost | 2 |
| Hard rock dig cost | 3 |
| Ember flask replenishment | 8, capped at capacity |
| Shallow crystal score | 1 |
| Deep crystal score | 4 |
| Floor 0 portal | Descends to floor 1 |
| Floor 1 portal | Returns to floor 0 |

Seed parity tests layout determinism only. It does not provide cryptographic randomness, seed uniqueness, a production rarity distribution, or authenticated epoch data.

### Actions

- `move(direction)`: move one cardinal cell if it is walkable. Entering an uncollected crystal cell collects that crystal and adds its score.
- `dig(direction)`: break one adjacent rock cell if sufficient charge is available. Deduct the cost and turn the rock into walkable ground. Digging does not move the player.
- `interact`: consume the flask under the player and replenish charge. At full charge, reject the interaction without consuming the flask.
- `portal`: travel to the paired portal on the other floor, retaining charge, score, terrain changes, and collection state.
- `wait`: consume one action without restoring charge or changing the world.
- `look`: observe the state without consuming an action.

Successful core actions increment the turn counter once. Invalid actions leave every state field unchanged and return an error. Walking and portals cost no charge. Exhaustion creates no automatic regeneration; the player may walk, retreat, find an accessible flask, or abandon the attempt.

Collected crystals and consumed flasks turn into ground. Breaking rock twice is invalid. Moving away and back, using a portal, or looking again must not recreate any object. New attempts start from the declared initial state; they cannot import charge, score, or changes from another attempt.

The `demo` command follows one legal route, collects both crystals, consumes both flasks, returns to floor 0, and revisits the first flask and crystal. The final result is floor 0, score 5, charge 7, and 36 successful actions out of 39 submitted commands. Its three expected errors are a second use of each flask and another attempt to dig already broken rock. These errors must not change state.

## Evidence boundary

The native lab exports an initial seed, ordered commands, per-command state, and the final result. This is an inspectable replay fixture, not a cryptographic proof and not an authorization to receive tokens.

A later proof must independently enforce:

1. The initial state is the declared start for the same expedition and rules version.
2. All terrain and item identities follow the declared generator.
3. Every transition is valid, and resource and score arithmetic stays within bounds.
4. Mining and collection changes persist across floor revisits.
5. All segments belong to one compatible history. State from different branches or attempts cannot be merged.
6. The public result matches the proven final state and the intended epoch and settlement domain.

A settlement contract must separately enforce owner authorization and one final accepted reward result per expedition. A replay file, save checksum, locally generated signature, or native test pass does not replace that proof.

## Decisions still open

- Final resource name, capacity, costs, replenishment distribution, and exhaustion experience.
- Unlimited-world generation, actual crystal rarity, score weights, and representational bounds.
- Epoch duration, submission and proof deadlines, and continuation after an expedition's reward result is finalized.
- Production expedition identity, key backup, seed derivation, and wallet recipient binding.
- drand network, fixed round assignment, signature verification, and outage policy.
- Proving implementation and long-run proof composition.
- Accepted memory, time, artifact size, and transaction cost budgets.

## Initial test machine

The user selected the current laptop. Read-only system inspection reports CachyOS on x86_64, an AMD Ryzen 7 5800H with 8 cores / 16 threads, and approximately 13.5 GiB of RAM visible to the operating system. This is a hardware inventory, not a benchmark result.

The native fixture uses only the existing Rust dependencies. Its handoff runs no prover, external service, wallet, or transaction. Use two Cargo build jobs in the provided commands. The separate [proof experiment](../experiments/proof-lab/README.md) supplies a user-run wrapper with two Rayon worker threads, a 4 GiB virtual-address-space limit, and a 120-second proof-process timeout. Compilation runs separately with two Cargo jobs. The first successful user measurement produced a proof in 398 ms, with approximately 89.05 MiB peak resident memory for the process and 1,896 ms before report serialization, including setup and rejection checks. A subsequent local EVM run passed all four tests and measured 269,032 gas for the verifier call, excluding deployment and a complete reward transaction. These are measurements for one bounded route, not accepted release budgets or evidence about unlimited expeditions. Robinhood testnet verification and scaling measurements remain pending.

The first experiment may use explicit fixture epoch values; it must not claim live epoch authenticity. M1 remains open until the required choices and budgets are accepted. Native proving and local EVM verification now have user measurements. M2 remains open pending Robinhood testnet validation, longer-route measurements, and the remaining budget and proof-assumption decisions.
