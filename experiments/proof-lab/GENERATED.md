# Generated expedition feasibility check

Status: the user passed all 20 workspace tests and 14 proof tests, generated and verified the 512-action proof, and verified it on Robinhood testnet. The M2 feasibility gate passed. This remains an isolated experiment; it is not a reward contract or release parameter setup.

## User command

From the monorepo:

```sh
bash scripts/check-generated.sh
```

This runs the current workspace tests, the separate proof tests, and one generated-world proof measurement. The recorded M2 run had 20 workspace tests and 14 proof tests; the M3-M4 candidate adds save, wallet-page, and ownership tests (currently 26 and 18). The wrapper retains two Cargo jobs, two proof workers, the 4 GiB virtual-address-space limit, and the 120-second proof timeout. Compilation and unit tests are outside that proof-process timeout. No signing, wallet access, transaction, or network deployment occurs. Cargo may fetch missing locked dependencies.

No repeat is needed for the recorded passing run. A future change to proof constraints requires the relevant checks again.

## Measured candidate

- Three 19 by 13 floors, with six connected chambers on each floor.
- A 64-bit seed controls passage availability, soft or hard rock, crystal coordinates, crystal presence, and entry-room flasks.
- Five mineable passages form a connected backbone; two optional passages per floor vary with the seed. Structural castle walls cannot be mined.
- Charge starts at 8, capacity is 12, soft rock costs 2, hard rock costs 3, and a flask restores 8 up to capacity.
- Crystal values are 1, 4, and 16 by depth. Deeper floors contain fewer crystals on average; one crystal per floor is guaranteed.
- Portals support descent and ascent and retain all changed terrain, flasks, and crystals.
- The benchmark visits all chambers on all floors, collects all generated crystals, returns upstairs, and fills the bound with legal backtracking. It proves 512 successful actions.

For seed 0, the expected result is score 23, charge 12, floor 0, position `(2, 2)`, and 512 turns. A successful report must name `vault-v1-feasibility-groth16-bn254`, state `generation_enforced_by_circuit: true`, pass native verification, reject six fabricated public results and five invalid action histories, and reject all eleven altered public inputs during cryptographic verification.

The circuit derives room geometry, mineable passages, resource costs, collectibles, and portals from the constrained seed. It never accepts a client-supplied floor grid or intermediate state. Setup and proving must generate identical constraint matrices for different seeds; a focused test checks this.

This candidate is a simple chamber generator with many seeded variations, not a claim that every possible seed yields a globally unique geometry or a finished launch balance. Seed grinding and automated solving remain permitted.

## Scope and remaining work

The old `lab-v0` CLI and measurement remain available. The generated mode can also be run alone:

```sh
bash scripts/proof-lab.sh --generated 0 512
```

This experiment still creates disposable single-party parameters and uses unauthenticated fixture identity and epoch labels. It does not implement ownership, production parameter setup, saves, or token payouts, and must not be used for funded rewards.

The Rust simulation and circuit use portable APIs. The shell benchmark wrapper and Linux peak-RSS reporting are measurement tools, not runtime requirements for the cross-platform game.

## Recorded user execution

The user ran `scripts/check-generated.sh` successfully and supplied `target/proof-lab.fsXVTtO3`. All 20 workspace tests and 14 proof tests passed. The eleven `ConstraintLayer` notices accompanied deliberately invalid witnesses; all eleven were rejected.

The [native report](measurements/generated-seed0-steps512.json) records 444,095 constraints, 5,653 ms proving time, 11,948 ms setup time, 2 ms native verification, and 1,258,676 KiB peak resident memory (approximately 1.20 GiB). Setup is separate from the normal player proving operation. The compressed proving key would occupy 87,177,680 bytes; this experiment did not save it.

The user then connected the funded testnet wallet and supplied the [browser-wallet result](measurements/testnet-generated-seed0-steps512.json). On chain 46630, the deployed bytecode matched, the valid proof returned true, and an altered score returned false. The successful verification transaction used 293,535 gas, excluding deployment and any reward settlement logic.

- Verifier: `0x69284f178a82f439acd50682263664d7c664df35`.
- [Deployment transaction](https://explorer.testnet.chain.robinhood.com/tx/0xbf693e9f7f3c9a75372550c2f968b68c359bb268f7321573fa6ee0ac5889f532): 651,815 gas.
- [Verification transaction](https://explorer.testnet.chain.robinhood.com/tx/0x8c5bb23cc4889337e47026921461438c38a84de3f21872116af41f7e717b5b34): 293,535 gas.

These are user-supplied test results and receipts, preserved with local artifact fingerprint checks. They establish feasibility for this bounded world, not production security or final game balance.
