# DiggerChain website

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Confirmed by the user: static HTML/CSS with a small amount of JavaScript. The playable browser demo uses the shared Rust simulation through WebAssembly. No project-operated backend is required.

## Users

Visitors discovering DiggerChain, learning how to play, trying the browser demo, and finding the downloadable terminal game. The full game also exposes a headless interface for external agents.

## Product Purpose

Explain the game clearly, let visitors try it, and help them start playing. The documentation must use ordinary English and avoid unnecessary implementation detail.

## Capabilities and Constraints

- Navigation: Home, Lore, Roadmap, Docs, Demo, Links. Lore is an eight-page illustrated book; the public roadmap has five selectable phases without dates.
- Every screen stays inside the viewport. Docs uses selectable topics and an independently scrollable reading pane, with the surrounding interface fixed.
- Provide Dark, Nord, and Solarized Light themes on every page. Save the choice locally and theme the browser map too.
- Provide llms.txt, complete machine-readable context, and Markdown exports of the actual player guide, lore, and roadmap.
- The planned $DCG token launches on PonsFamily on Robinhood Chain (4663). Its CA appears on Home, Docs, and machine-readable content from one public configuration file.
- The GitHub destination is https://github.com/vanb0mmel/DiggerChain. The CA, X profile, and PonsFamily token page have not been supplied. Display Coming Soon; do not invent destinations.
- Alpha and beta have no direct token payouts. Beta is planned to introduce points for a future airdrop. Direct $DCG and supported RWA-token rewards are a Phase III full-release goal. The browser demo is practice only, without a wallet or rewards.
- The accepted game scope is a human delver's three-floor expedition with the existing deterministic chamber generator and a 512-action limit. Unlimited depth is deferred.
- The game works offline after installation. Its browser demo needs a browser renderer; the native terminal executable is not embedded directly.
- Website copy, documentation, code, and comments are English.
- The user owns acceptance and has deferred further tests to accelerate launch. Do not report unrun checks as passed.
- Hosting uses Fly.io, with diggerchain.xyz as the selected domain and vanb0mmel/DiggerChain as the public repository. Final token configuration and platform artifact publication remain open.

## Brand Commitments

- Product name: DiggerChain.
- Approved candle logo: `CEO-FILES/logo.png`. Preserve the original asset.
- Primary reference: https://joelshajan.itch.io/static-hymn.
- Requested aesthetic: a restrained retro game screen, with strict, old-fashioned proportional book typography for UI and reading. The Home title is the explicit exception: original bitmap/character-art lettering that immediately conveys terminal-core.
- Impeccable guides UI design; Ponytail guides implementation.
- The user chose multiple quick visual mockups before implementation. Alternatives must differ fundamentally in composition and concept, rather than just palette.
- The selected boot-screen concept remains the composition reference: title, candle, caption, commands, and bottom navigation. The user's subsequent refinements require compact screens, neutral Dark, Nord, Solarized Light, proportional UI lettering, and an inline bitmap Home wordmark. Download opens the download guide rather than immediately fetching the archive. The browser title remains DiggerChain - A Terminal Mining Adventure on every route.
- Lore follows four friends from future Belgium into an immense fantasy castle. Four original illustrations in the user's chosen Boris Vallejo fantasy-oil direction accompany short pages. Study actual visual references rather than relying on an artist name alone. Artwork sits above text at every viewport, without a contrasting letterbox panel. The fictional castle does not change the alpha's three-floor scope.

## Evidence on Hand

The repository contains the terminal game, the shared Rust simulation, a local Linux candidate archive, a fixed-parameter proof tool, testnet contracts, and recorded checks. Full registration/payout and Windows/macOS builds are not confirmed. Website claims must reflect those limits.

## Product Principles

- Make trying and understanding the game straightforward.
- Keep the delivery lightweight and the game simulation shared.
- Preserve the selected terminal aesthetic and candle identity.
- Explain player actions in plain English.
- Present actual capabilities and availability accurately.
