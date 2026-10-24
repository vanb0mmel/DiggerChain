# DiggerChain v1.0 Roadmap

Public launch plan updated on 2026-10-24: alpha and beta have no direct token payouts. Beta is planned to introduce points for a future airdrop. Direct $DCG and supported RWA-token rewards are a Phase III full-release goal. The five public phases without dates are maintained in [website/content/roadmap.json](../website/content/roadmap.json). The engineering milestones and measurements below preserve the implementation history; completed proof work does not activate alpha rewards.

Status: the user authorized the accelerated scope and implementation, targeting an initial audience of approximately 10-20 players. On 2026-10-22 the user confirmed three floors, the existing generator, and a human protagonist, and explicitly deferred remaining tests to accelerate launch. Cross-platform builds remain a release deliverable. Publication, deployment, and funding still require the concrete review package and user-controlled signing.

Implementation completion and executed verification are recorded separately. Check an implementation item when its deliverable exists; do not infer an unrun test from it. The user owns tests and benchmarks. Additional runtime, reward-flow, recovery, and platform smoke checks are deferred by the user's decision, not prerequisites for this launch. Existing runtime validation and cryptographic checks remain enabled. The optional test workflow is manual so publication does not silently start a new test campaign.


## Current alpha handoff — 2026-10-24

- [x] Freeze `0.1.0-alpha.1`: human delver, three seeded floors, 512 actions, offline play, automatic saves, headless agent interface, and no token payouts.
- [x] Keep proof and settlement development separate from the game-only alpha archive.
- [x] Receive the maintainer's website acceptance, including Lore, Roadmap, Docs, Demo, Links, and the final bitmap-title/art refinement. Preview is stopped.
- [x] Prepare game-only packaging and a four-target build workflow: Linux x86-64, Windows x86-64, macOS Apple silicon, and macOS Intel.
- [x] Receive the public destination: https://github.com/vanb0mmel/DiggerChain.
- [ ] Publish the clean source snapshot and trigger the native build workflow.
- [ ] Publish actual native archives and checksums, then replace the website's Coming Soon download entries.
- [x] Publish the static site at https://diggerchain-alpha.fly.dev/ on one 256 MB Fly machine.
- [ ] Complete DNS and HTTPS for diggerchain.xyz.
- [x] Add the official GitHub destination to the single website configuration.
- [ ] Receive the token CA and official X/PonsFamily destinations from the maintainer.
- [ ] Have the maintainer select the prepared promo images and silent gameplay films, then announce alpha.

The engineering history below documents the earlier rewarded-release track. Its remaining proof, parameter, and funded-settlement tasks belong to Phase III preparation; they are not alpha launch prerequisites. Completed historical measurements remain intact, and deferred checks remain unrun.

## Already available

- [x] Rust workspace with a shared simulation core and separate terminal presentation.
- [x] An accepted authored visual playground, top-down colored glyphs, optional half-block sprites, fog of war, and no camera shake or scene reveal.
- [x] A headless JSON interface for external agents.
- [x] Preserved baseline: commit `7acf8d6`, tag `prototype-v0.1.0`, and independent worktree `../DCG-PROTOTYPE`. See [prototype instructions](PROTOTYPE.md).
- [x] Separate native two-floor fixture with digging, replenishment, crystals, backtracking, and persistent collections. The user supplied 17 passing game-workspace tests.
- [x] A real Groth16/BN254 proof for one 36-action route, followed by four passing local EVM tests.

The native proof took 398 ms; the experiment reported approximately 89.05 MiB peak resident memory. The local verifier call measured 269,032 gas, excluding deployment and a complete reward transaction. These results apply to the small fixture. See the [native measurement](../experiments/proof-lab/README.md#recorded-user-execution) and [EVM record](../experiments/proof-lab/EVM.md#recorded-user-result).

The generated three-floor candidate passed 20 workspace tests, 14 proof tests, native verification, and Robinhood testnet verification. Its 512-action proof took 5,653 ms and approximately 1.20 GiB peak resident memory; the testnet verifier transaction used 293,535 gas. See the [recorded generated-world results](../experiments/proof-lab/GENERATED.md#recorded-user-execution).

The user then passed all 62 candidate checks: 26 workspace, 18 proof-package, seven accounting, and eleven Python tests. Dedicated fixed parameters, a real saved-game ownership proof, and both testnet deployments passed. The proof took 5.858 seconds; the complete fixed-key process took 54.579 seconds and approximately 1.29 GiB peak RSS. Registration missed the epoch deadline; no score or payout was recorded. See the [partial reward smoke record](../contracts/measurements/2026-10-21-smoke/README.md). Complete reward-flow verification is now deferred; it has not been reported as passed.

## Earlier rewarded-release boundaries (development history)

- A human delver, one castle biome, the existing simple deterministic generator, and a small content set using the established visual techniques.
- A bounded reward expedition: a fixed maximum number of floors and successful actions, selected through M2 and displayed before play. Freely created attempts remain allowed.
- One proof for a complete expedition from its canonical start. V1 has no continuation proofs, recursion, aggregation, or unlimited rewarded descent.
- Bidirectional portals, digging costs, replenishment, and persistent terrain and item changes remain part of the actual game and its proof.
- Worlds use personal expedition seeds with a defined identity and seed-derivation rule. Seed grinding and automated solving remain allowed. No external randomness beacon is required for v1.
- An expedition can begin offline without a connected wallet. Submission assigns its one final result to an eligible open epoch; the proof does not establish when the player actually played. Each expedition can earn a reward only once across all epochs.
- One token, one settlement chain, one fixed-budget proportional reward policy, and one external-wallet signing flow. Claims must bind the recipient, expedition, rules, epoch, and settlement domain.
- Target Linux, Windows, and macOS from the shared Rust codebase. Use portable runtime APIs, native builds, and small platform smoke checks. The current Linux laptop is the first proof benchmark machine, not a restriction on supported game platforms.
- No project-operated backend, required proving service, embedded LLM integration, or agent fleet manager. The game keeps a headless interface for external agents.
- The original authored prototype remains independently runnable.

The user accepted the bounded first release, personal seed policy, submission-time epoch assignment, and cross-platform target. The passing M2 measurement selects three floors and 512 successful actions for v1.

## M0 - Preserve the accepted prototype

- [x] Keep the baseline commit, tag, and separate worktree available.
- [x] Record the current game and proof results without treating the fixture as a finished reward system.

No separate preservation project or repeated visual acceptance cycle is required. The baseline remains available for optional comparison.

## M1 - Fix the v1 scope and rules

- [x] Accept the bounded release scope, seed and epoch policy, and cross-platform target.
- [x] Fix one set of digging, resource, crystal, and portal rules, plus expedition identity and deterministic seed derivation.
- [x] Specify the simple epoch schedule, submission and claim rules, fixed reward budget, and owner authorization. State the behavior of late submissions and empty epochs.
- [x] Fix save compatibility and the proof wrapper limits for the selected laptop: two workers, 4 GiB virtual address space, and 120 seconds. Use one v1 save format; reject unsupported versions clearly.

Exit: a short implementable specification. No additional reference-project survey, alternative proving-stack comparison, or broad content design is required before the next implementation step.

## M2 - Close the remaining feasibility gate

- [x] Produce and verify the existing fixture proof natively.
- [x] Compile the exported verifier and pass its four local EVM tests.
- [x] Validate the generated fixture: three 19 by 13 floors, seeded passages and crystals, and a 512-action bound. The user passed 20 workspace tests and 14 proof tests.
- [x] Have the user run one representative measurement at the selected bound, including backtracking. Native verification and all witness and public-input rejection cases passed.
- [x] Have the user verify a valid proof and an altered-input rejection on Robinhood Chain testnet; preserve receipts and gas measurements.
- [x] Select the production proof approach and parameter-generation process: Groth16/BN254 with a dedicated fixed single-operator setup on the user's machine. Implement it after freezing the authenticated circuit. See [setup assumptions](SETUP_MODEL.md).

Exit: one measured, affordable v1 configuration and a viable production setup plan. No open-ended benchmark campaign, recursion prototype, or large-world experiment is required.

## M3 - Build one complete playable expedition

Implemented handoff: [offline play and saves](EXPEDITIONS.md). The user passed all 26 workspace tests and supplied a sealed, positive-score expedition for proving. General user feedback is positive. The original SHA-256 test-vector typo is fixed. Normal launch now selects the human expedition; `--prototype` retains the goblin scene and original headless protocol. Presentation and launch changes pass static Rust checks; additional runtime checks are deferred.

- [x] Integrate digging, resource accounting, crystals, and bidirectional portals into the shared core and human terminal controls.
- [x] Implement the one versioned generator and bounded descent. Preserve all terrain and item changes through revisits.
- [x] Apply the existing art direction to the generated rooms, with a human protagonist, fog enabled, and no camera shake.
- [x] Add atomic local saves, restart recovery, and the declared expedition end condition.
- [x] Expose the same actions and state through the existing headless interface.

Exit: the user can start offline, explore, dig, descend, return, replenish, save, resume, and finish an expedition. The agent interface reaches the same result through the same core.

## M4 - Complete proof and reward settlement

Implemented handoff: [protocol](REWARD_PROTOCOL.md), [setup model](SETUP_MODEL.md), and [testnet walkthrough](../contracts/TESTNET.md). The user passed the 18-test proof package, seven accounting tests, fixed setup, real saved-game proof, and deployment checks. Actual registration/payout, recovery, and the clarified wallet UI remain unverified; the user deferred those checks. The final parameter bundle and concrete funded deployment configuration still need to be prepared for publication.

The native game can serve the public wallet page locally. The separate proof executable fixes its worker pool at two threads, validates published parameter fingerprints, and leaves spending keys in the browser wallet. Native packaging, public-parameter packaging, dependency notices, and a manually triggered three-platform candidate build workflow are prepared; hosted platform execution and publication have not occurred.

- [x] Generate a real proof for the actual M3 expedition, including its generator, resource rules, persistent changes, and final result.
- [ ] Freeze the approved circuit and verifier parameters through the accepted production setup process. Never use the experiment's disposable keys for funded rewards.
- [x] Implement one token and the fixed-budget epoch contract: register valid scores and distribute proportional rewards after the immutable deadline with defined rounding. No separate finalization transaction or closing bot is required.
- [x] Enforce owner authorization, recipient binding, rules and chain binding, submission deadlines, globally single-use expedition results, and single-use payouts.
- [x] Provide one external-wallet signing flow with clear progress and errors. Keep spending keys outside the game and agent protocol.
- [x] Keep every protocol operation callable by participants without a project-operated service.

Implementation: actual offline gameplay produces a proof for the deployed verifier and epoch contract. Complete on-chain registration and payout remain unobserved, with further verification deferred by the user. No proof aggregation, gas sponsorship, multi-wallet framework, or upgrade framework is required.

## M5 - User acceptance and release v1.0

- [x] Record the maintainer's 2026-10-22 decision to defer further tests while preserving the actual results and all runtime validation.
- [ ] Resolve any discovered reward-integrity, save-loss, or packaging blocker. Record available measurements and keep unobserved settlement stages explicit.
- [ ] Prepare native Linux, Windows, and macOS release binaries, concise install and play instructions, external-agent commands, wallet and backup steps, checksums, and known limits. Keep platform checks focused and record which checks actually ran.
- [ ] Receive the GitHub destination and prepare the concrete publication, deployment, and funding package. The user reviews it and controls signing and funds.
- [ ] Complete only the approved publication and deployment. Record the actual resulting addresses, transactions, and deferred verification status.

Exit: an installable, bounded release with real offline play, the implemented reward path, and an accurate record of executed and deferred verification. There is no known reward-integrity or save-loss blocker. No remote configuration, publication, or live funding is authorized by this roadmap alone.

## Website preview - 2026-10-23

- [x] Implement the approved boot-screen composition with the original candle logo, local fonts, and static HTML/CSS/JavaScript.
- [x] Add plain-English player documentation and a practice-only WebAssembly demo over the shared Rust expedition simulation.
- [x] Build the illustrated eight-page Lore and five-phase public Roadmap; prepare native downloads and a static hosting bundle.
- [x] Refine the site into fixed-viewport screens with topic-based Docs, immediate pixel lettering, three themes, Links, CA placeholders, and machine-readable content. Actual token/project URLs remain Coming Soon as confirmed by the user.
- [x] Record the user's visual acceptance of the final site on 2026-10-24. This acceptance does not imply new automated or cross-platform tests.
- [ ] Publish to the maintainer's selected host and domain after receiving the destination. No remote repository or deployment is configured by this website work.

Build and static-check results are recorded separately from player acceptance in `website/README.md`. This adds no new test prerequisite to the maintainer's accelerated release decision.

## Deferred verification

The following checks remain available but do not block launch under the maintainer's decision:

- [ ] Complete a two-participant testnet epoch, proportional payouts, an empty epoch, and wallet transaction recovery.
- [ ] Repeat the focused rejection cases through the complete wallet flow. The existing native, constraint, and accounting checks retain their recorded passing results.
- [ ] Check the final binaries for offline play, save/resume, interrupted operations, terminal restoration, and headless play on supported platforms.

The [smoke-test guide](SMOKE_TEST.md) is an optional future procedure, not a request to restart setup or testing now. Missing platform builds are still listed as missing artifacts; a deferred runtime check does not create a Windows or macOS executable.

## Deferred until after v1.0

- Unlimited rewarded depth, resumable proof segments, recursion, and aggregation.
- Multiple biomes and settings, a large content library, and extensive art variants.
- drand or another authenticated external randomness source, including round-selection and outage handling. Reconsider it only for a concrete rule that needs external unpredictability.
- Additional wallet integrations, embedded wallet UX, and sponsored transactions.
- Automatic save migrations and custom installation or update tooling.
- Batch settlement and further gas or throughput optimization beyond the observed launch workload.
- Agent orchestration: the existing headless interface is sufficient for external scripts and AI agents.
- Broad hardware certification and repeated benchmark matrices. Keep the representative v1 limit check and focused correctness tests.

These items are a backlog, not prerequisites or a promise to implement every feature. They must not silently expand the first release.

## Integrity requirements for future funded rewards

Proof correctness includes deterministic world generation and every reward-relevant state change. A client assertion, replay file, or save checksum cannot replace it. Contract authorization, replay prevention, the fixed payout budget, and acceptable verifier parameters remain mandatory even with only a few players.

The user owns testing and acceptance. All persisted project content stays English. Keep simulation separate from rendering, preserve the original prototype and Codex configuration, and do not publish until the user supplies the destination and authorizes the concrete action.

Dark Forest, Dark Forest Aztec, Loot Survivor, Pearl, and the visual references remain sources of hypotheses. No reference engine or dependency is mandatory.

The fixed epoch budget limits issuance, not token value or equal earnings. Public deterministic worlds can be inspected and solved by modified clients. V1 deliberately permits grinding and automation and does not claim that a proof establishes human effort or play during a specific time window.
