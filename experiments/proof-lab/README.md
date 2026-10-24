# Bounded gameplay proof experiment

Status: historical `lab-v0` experiment with a passing native proof and four local EVM tests (269,032 gas per verifier call). The user subsequently passed the generated 512-action world and Robinhood testnet verification; see [M2 results](GENERATED.md). The assistant has not executed proofs or tests. The new fixed-parameter `diggerchain-proof` binary shares the gameplay constraints through the package library; it has a separate [reward protocol and user validation flow](../../contracts/TESTNET.md). Its ownership and settlement checks must pass before release.

This candidate uses Groth16 over BN254 through arkworks 0.5. The circuit interprets legal `lab-v0` actions directly. It does not prove rendering or run a general-purpose virtual machine. The experiment has its own Cargo workspace and lockfile; ordinary game builds and workspace tests do not build its cryptographic dependencies.

## First measurement on the selected Linux laptop

From the main repository directory:

```sh
bash scripts/proof-lab.sh
```

Required: the existing Rust toolchain, Bash, and GNU coreutils. GNU time is optional: the experiment already measures stage durations and reads Linux's peak-resident-memory counter without another dependency. Node, Circom, a GPU, a wallet, and a blockchain connection are not required for this step. The initial Cargo build may download the locked Rust dependencies.

The wrapper first builds with two Cargo jobs. Compilation is outside the proving resource limits and its elapsed time is not proving time. It then runs the experiment with two Rayon worker threads, a 4 GiB virtual-address-space limit, and a 120-second timeout with a five-second termination grace period. These are conservative experiment guardrails, not accepted v1.0 performance budgets. Address space is different from physical resident memory; an allocation can fail before reaching the address-space limit because other mappings also count. The limits apply only to the child processes, not to the user's shell or system settings.

The experiment:

1. Constructs the native route and omits its three rejected commands. The statement concerns a legal reachable result, not an exact history of UI requests.
2. Checks the gameplay constraints and twelve deliberately invalid witnesses.
3. Generates a fresh local experiment-only setup, creates a real proof, and verifies it natively.
4. Alters each of the eleven public inputs and requires the original proof to fail verification.
5. Exports the proof, verifier, local EVM tests, and a measurement report into a fresh `target/proof-lab.XXXXXXXX` directory. Existing artifact files are never overwritten.

Success requires exit status 0, `native_verification: "passed"`, twelve rejected witness cases, eleven rejected public-input changes, and public result fields `score=5`, `charge=7`, `floor=0`, `x=3`, `y=1`, `turns=36`. The output supplies public input names and decimal values in matching order. Unused circuit slots are no-ops and do not advance turns. `timings_ms` separates setup, proving, verification, and the overall experiment time before report serialization; it excludes compilation. A phone stopwatch is unnecessary.

`memory.peak_rss_kib_before_report` records the kernel-reported resident-memory high-water mark before final report serialization, including setup and rejection checks but excluding compilation. Linux provides this process-wide counter as [`VmHWM` in `/proc/self/status`](https://docs.kernel.org/filesystems/proc.html). RSS accounting is approximate; this is a practical hardware measurement, not a byte-exact allocation audit. If the counter cannot be read, the report contains `null`, not zero; memory feasibility then remains unmeasured. When `/usr/bin/time` is installed, the wrapper also writes `resources.txt` with an external full-process report, which can remain available after an early failure or timeout. Both paths retain the same resource limits.

Send the complete output, including a failure or timeout. Exit 124 normally indicates the time limit. A memory limit can produce an allocation failure or nonzero signal exit instead of a report. Do not remove the limits to obtain a passing result. Partial artifact directories can be retained for diagnosis or deleted; the wrapper creates a new directory on each run.

## Statement and rejection cases

Public inputs, in order: rules version, seed, settlement domain, fixture epoch, fixture expedition ID, final score, charge, floor, x, y, and successful-turn count. The seed is constrained to 32 bits; the three context fields are constrained to 64 bits. The demo context is domain `46630`, epoch `0`, expedition `1`. These are explicit test labels, not authenticated live-chain data, wallet ownership, or a seed-uniqueness policy. Their inclusion binds the proof to its declared context; a settlement protocol must separately check that context is authorized.

The canonical start, both authored layouts, initial resource balance, mining costs, collectible identities, and scoring rules are part of the circuit. The seed's low bit mirrors the fixture. Higher seed bits do not create additional layouts. Each step selects exactly one action, validates its preconditions, and computes the next state. Opened rocks, consumed flasks, and collected crystals stay recorded across both floors. No supplied intermediate state or score is trusted. The native core supplies the honest witness and expected result; the circuit independently constrains the reward-relevant transitions.

The twelve runtime rejection cases are six fabricated final fields plus walking into rock, mining opened rock, skipping a required refill, consuming a flask twice, forgetting an opened rock after backtracking, and an unknown action. The initial balance and starting floor are fixed constants, not client-supplied witness fields. A single circuit computes the whole route from that start, so separate history branches cannot be combined by importing their states.

## Artifacts and local EVM check

- `report.json`: native result, stage timings, Linux peak resident memory, sizes, and rejection results. EVM and Robinhood testnet status are `not_run` when the native process writes this file; the later local EVM result is recorded separately in [EVM.md](EVM.md#recorded-user-result).
- `resources.txt`: optional external proof-process report, created only when GNU time is installed. Without it, a process killed before writing `report.json` may leave no memory measurement.
- `source-sha256.txt`: source and lockfile fingerprints captured after compilation. Keep this with the working tree used for the measurement; the fixture has no release tag yet.
- `proof.bin`, `verifying-key.bin`: arkworks compressed encodings. The proving key is measured but not saved.
- `evm.json`: eight uncompressed proof-coordinate words and eleven public-input words as decimal strings.
- `src/Verifier.sol`: a verifier with this run's key fixed in its source. Callers cannot substitute a verification key.
- `test/Verifier.t.sol`, `foundry.toml`: four local EVM tests, requiring no third-party Solidity test library.

The EVM adapter uses point addition, multiplication, and pairing precompiles, validates canonical coordinates and scalars, and reverses the Fq2 coefficient order required by EIP-197. It does not contain a token or reward contract. The generated seed-0 verifier passed its four local EVM tests; other generated fixtures and live-chain behavior still require validation.

The [local EVM handoff](EVM.md) covers installing Foundry, the offline compiler command, and the successful user-run result. Foundry 1.8.5 is installed. After an automatic Solidity download timed out, a SHA-256-verified local Solidity 0.8.28 binary was prepared; the user then compiled the package and passed all four EVM tests.

After a successful native run, a separately installed Foundry toolchain can run the generated package. Replace `RUN_DIRECTORY` with that run's full artifact path:

```sh
forge test --root "RUN_DIRECTORY" -vv
```

Foundry may fetch the pinned Solidity 0.8.28 compiler. Expected: four passing tests, covering the original proof, each altered public input, a noncanonical scalar, and a noncanonical proof coordinate. The emitted gas value measures a local verifier call, excluding contract deployment and a complete reward transaction. It is not a Robinhood Chain fee or testnet measurement. Do not deploy this experiment's key to any funded reward system.

## Scaling and focused development checks

Once the first result is reviewed, the same wrapper accepts seed, circuit step bound, and additional round trips:

```sh
bash scripts/proof-lab.sh 1 64 0
bash scripts/proof-lab.sh 0 128 5
bash scripts/proof-lab.sh 0 256 12
```

The first mirrors the original route. Each added round trip uses eighteen successful moves and portal actions while preserving both floors. The longer examples finish with the same score, charge, and position but have 126 and 252 successful actions. Each step bound has a different circuit and setup. These are representative bounded measurements, not proof composition for unlimited expeditions.

Prepared checks, all run by the user:

```sh
cargo test --manifest-path experiments/proof-lab/Cargo.toml --release --locked -j 2 -- --test-threads=1
cargo clippy --manifest-path experiments/proof-lab/Cargo.toml --all-targets --locked -j 2 -- -D warnings
cargo fmt --manifest-path experiments/proof-lab/Cargo.toml -- --check
```

The focused test command checks constant inputs, start-state preconditions, honest and forged routes, and identical constraint matrices in setup and proving modes with different seeds. It does not generate a proof or run a blockchain transaction. The user later supplied 14 passing tests including the generated-world circuit; ownership and fixed-parameter additions bring the current candidate to 18. No repeat of the old native proof is required merely to record the historical result. Test compilation and execution are separate from the wrapper's proof-process limits.

## Limits that remain open

The local setup is single-party and not a production trust solution. Generating one's own verifier key cannot establish honest rewards. A real deployment would need a fixed approved verifier and an acceptable setup process; this experiment deliberately measures the setup separately from proving. It performs no setup ceremony and makes no audited-security claim.

This circuit proves one bounded route from a canonical initial state. It does not expose a final world-state commitment for continuation, compose proofs, implement procedural generation, authenticate drand or epochs, establish wallet ownership, prevent repeated claims, or distribute tokens. Native success and local EVM success are separate gates; both still leave Robinhood testnet behavior and long-run feasibility to measure.

## Recorded user execution

The first user run wrote source fingerprints to `target/proof-lab.uMdDpmrc` and exited with status 1 and `unsatisfiable constraint system` before setup or proof generation. Its circuit source SHA-256 was `29142c95758847f0c6d094f5db37d4901198291679d89accdb9e1714ea13d402`. The reported shell duration of approximately 34 seconds does not establish proving time; compilation and the failed preflight were not separated in that observation. No final report or proof was produced.

Static inspection identified the failure: arkworks 0.5's Boolean conditional equality rejects unequal constant operands before consulting the condition. At the start, flask and portal availability are constant false; the old conditional assertions therefore rejected the route even with those actions disabled. Preconditions now forbid the conjunction of an active action and an unmet requirement. A regression test covers padding and waiting at the start, while still rejecting interaction and portal travel there. Expected `Unsatisfiable` errors count as rejected forged witnesses; other synthesis errors remain failures. Progress and errors now identify canonical-route synthesis and individual rejection cases. The correction was included in the next user build, which encountered the separate failure below before completing route validation.

The second user run wrote source fingerprints to `target/proof-lab.nxDKnbNG` and exited with status 101. Its circuit source SHA-256 was `87cadd9180a2f6bdbfb92887e04b9e121682721a2869d9f100436a8b924edf6f`. Compilation completed in 9.97 seconds. The runtime panicked in arkworks' `AllocatedFp::add_many` because it received no allocated variables after constants were filtered out. The triggering multi-input OR evaluated five constant comparisons against the initial charge. There was no successful proof or performance result.

All circuit multi-input OR calls and the explicit field sum now use sequential folds. New regression tests cover constant-only replenishment masks and matrix equality between setup and a mirrored witness. The user then compiled and ran the corrected proof wrapper successfully, as recorded below. The assistant has not compiled or executed it.

The successful run is `target/proof-lab.psxh3L7W`, with circuit source SHA-256 `be53f85a497921a893a27495f1071c65dd781a4665041e5466aa18767501b4b8`. Its [native report](measurements/native-seed0-steps64.json) and [source fingerprints](measurements/native-seed0-steps64.sources.sha256) are copied outside the ignored build directory; fingerprint paths are repository-relative. The measured machine is the selected Ryzen 7 5800H Linux laptop described in the [rules draft](../../docs/REWARD_RULES.md#initial-test-machine).

| Measurement | User-run result |
| --- | --- |
| Route | Seed 0, 36 successful actions in 64 circuit slots |
| Result | Score 5, charge 7, floor 0, position `(3, 1)` |
| Constraint and rejection checks | 619 ms |
| Fresh experiment-only setup | 864 ms |
| Proving | 398 ms |
| Native verification including key preparation | 2 ms |
| Total before report serialization, excluding compilation | 1,896 ms |
| Kernel-reported peak resident memory | 91,184 KiB, approximately 89.05 MiB |
| Compressed proof / verifying key | 128 / 616 bytes |
| Compressed proving-key size, not saved | 5,980,720 bytes |
| EVM proof / complete static verifier calldata | 256 / 612 bytes |
| Rejections | 12 forged witnesses and 11 altered public inputs |
| Native verification / exit status | Passed / 0 |
| Subsequent local EVM tests | 4 passed, 0 failed, 0 skipped |
| Local verifier call, excluding deployment | 269,032 gas |

The reported 9.93 seconds is compilation time and is separate from those runtime measurements. Twelve `Constraint trace requires enabling ConstraintLayer` messages accompanied the deliberately invalid witnesses: arkworks reported that detailed constraint traces were not registered while returning unsatisfied results. They do not invalidate this successful run. No tracing dependency or source change is needed to accept the result.

This is the first successful bounded native measurement, followed by a passing [local EVM check](EVM.md#recorded-user-result). It is not a production performance target, independent security audit, or proof of unlimited-world feasibility. Robinhood testnet verification and longer-route measurements remain open.

## Primary references

- [arkworks Groth16 0.5 source and APIs](https://github.com/arkworks-rs/groth16/tree/v0.5.0).
- [arkworks constraint gadgets 0.5](https://github.com/arkworks-rs/r1cs-std/tree/v0.5.0).
- [EIP-196: BN254 point addition and multiplication](https://eips.ethereum.org/EIPS/eip-196).
- [EIP-197: pairing checks and Fq2 encoding](https://eips.ethereum.org/EIPS/eip-197).

Related: [fixture rules](../../docs/REWARD_RULES.md), [native lab](../../crates/diggerchain-lab/README.md), [roadmap](../../docs/ROADMAP.md).
