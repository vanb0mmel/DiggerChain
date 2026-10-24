# DiggerChain alpha release preparation

Version: **0.1.0-alpha.1**. The maintainer accepted a human delver, three seeded castle floors, a 512-successful-action limit, offline play, automatic saves, and an external-agent interface. The original goblin playground remains accessible with `--prototype`.

Alpha has no direct token payouts. Players do not install a proof tool or connect a wallet to play. The existing proof, ownership, and testnet settlement work remains available as future rewarded-release development. Do not bundle experiment parameters or treat its deferred settlement checks as alpha launch requirements.

## Available now

- Game-only Linux x86-64 archive: `target/dist/diggerchain-0.1.0-alpha.1-linux-x86_64-local.zip`, with an adjacent SHA-256 file. Installation instructions, dependency notices, and internal checksums are included; private saves and the proof executable are excluded.
- Live site: https://diggerchain-alpha.fly.dev/. The selected domain is `diggerchain.xyz`; DNS and HTTPS issuance are pending the maintainer's DNS update.
- Accepted static site: `target/website`, with its host-ready ZIP in `target/dist/diggerchain-website-preview.zip`. Downloads and official destinations are controlled by `website/site.json`.
- Fly deployment context: `target/fly-website`, prepared by `python3 scripts/prepare-fly.py`. Source configuration and the publication procedure are in [deploy/fly](../deploy/fly/README.md).
- Native build workflow: `.github/workflows/native-candidates.yml`, targeting Linux x86-64, Windows x86-64, macOS Apple silicon, and macOS Intel. It uploads game-only artifacts; an alpha-tag push publishes them as a prerelease after all four builds succeed. It does not run tests or create proof parameters.
- Promo production source: `tools/promo`. Original campaign images and final films are kept in the maintainer's ignored `CEO-FILES/promo/2026-10-24` folder.

## Remaining publication inputs

- Source is published at https://github.com/vanb0mmel/DiggerChain. All four native builds and the prerelease job passed: https://github.com/vanb0mmel/DiggerChain/releases/tag/v0.1.0-alpha.1. No tests were run by these jobs. A portable Windows variant is being added with a static C runtime.
- Publish native ZIPs and their matching checksums as release assets. Replace Coming Soon only for artifacts that exist. A successful cross-compilation does not establish runtime acceptance on that platform.
- Fly login and initial deployment are complete. The app is `diggerchain-alpha`, one shared CPU and 256 MB RAM, with idle autostop. The maintainer is configuring Dynadot DNS; complete certificate issuance and record the actual domain status.
- The maintainer launches $DCG and supplies its CA and PonsFamily URL, plus the official X/GitHub destinations. Enter them in `website/site.json` and rebuild; do not substitute a wallet address or a testnet contract.
- The maintainer selects the promo assets and publishes the announcement. No social posting is delegated to the development agent.

## Evidence and limits

The Linux release compilation, archive generation, and static website build completed locally. Native gameplay was recorded only for the requested promo footage, using a task-owned practice save. This is production capture, not a new regression suite or cross-platform acceptance run.

The maintainer owns tests and has deferred additional runs. Newly prepared focused packaging/download tests are available for later execution; they have not been run by the assistant. Existing recorded results remain unchanged. Windows/macOS archives and the public prerelease now exist; platform runtime acceptance remains deferred. Custom-domain DNS and token addresses are recorded only when their actual results exist.

The current local Linux executable requires glibc 2.39 or newer, as read from its ELF version requirements. Prefer the Ubuntu 22.04 workflow artifact for public Linux distribution after it exists. Unsigned macOS and Windows binaries may trigger operating-system publisher checks; no signing identity has been supplied.
