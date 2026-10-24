# DiggerChain website

A compact game-screen site: Home, an illustrated Lore book, a five-phase Roadmap, a topic-based player guide, the practice demo, and Links. Every screen fits the viewport. Lore has eight short pages and four original illustrations; Roadmap shows one milestone at a time, without dates. Docs is the exception for reading: its article scrolls inside a fixed topic reader, keeping chapter controls, themes, and site navigation in place.

There is no framework, npm dependency, project backend, wallet connection, analytics, or runtime CDN. JavaScript handles presentation; shared Rust/WebAssembly handles expedition rules. Dark, Nord, and Solarized Light palettes cover the whole interface and map. The saved palette applies before the first paint. Self-hosted Libre Baskerville supplies proportional book typography for UI and reading. The Home wordmark is original inline bitmap SVG with an accessible heading label; it is present at first paint without a font or JavaScript replacement. The browser tab title stays DiggerChain - A Terminal Mining Adventure across routes. Home's Download command opens `#docs/download`; the archive is downloaded from that guide.

## Build and preview

Development needs the existing Rust toolchain, Python 3.11+, the `wasm32-unknown-unknown` Rust target, and `wasm-bindgen-cli` 0.2.129. These are already installed on the development laptop. From the monorepo root:

```sh
python3 scripts/build-website.py
python3 scripts/serve-website.py
```

Open `http://127.0.0.1:8787`. Only `target/website` is served, on loopback. The repository and private saves are not exposed. Do not open the source HTML directly: it contains build placeholders, and the demo needs HTTP to load WebAssembly.

For website-only edits, reuse the unchanged compiled demo:

```sh
python3 scripts/build-website.py --skip-wasm
```

Download the Linux ZIP from the public alpha release into `target/dist` before building a fresh checkout. The native download defaults to `target/dist/diggerchain-0.1.0-alpha.1-linux-x86_64.zip`. Supply another Linux alpha archive with `--native-archive PATH`. The builder stages a clean public tree and generates the renamed archive's matching checksum. Public output is `target/website/`; its host-ready copy is `target/dist/diggerchain-website-preview.zip`. Serve WASM as `application/wasm` and provide HTTPS when publishing. Hash navigation needs no rewrite rules. The site is deployed at https://diggerchain.xyz/ (also https://diggerchain-alpha.fly.dev/), with active HTTPS for the apex and www domains. The isolated Fly build context and update procedure are documented in `deploy/fly/README.md`.

## Token and project links

Edit `website/site.json` when the maintainer supplies the real values. The GitHub destination is https://github.com/vanb0mmel/DiggerChain. The $DCG CA, X profile, and PonsFamily token page remain `null` and display Coming Soon. The LLMs control is active.

The builder validates CA shape and HTTPS URLs, then puts the same values in Home, Docs, public `site.json`, and the machine-readable files. Do not use the previously supplied wallet address or testnet contracts as the token CA. Download platforms and release version also live in this configuration; unavailable platform URLs remain null. Copy controls appear only when a complete token address exists. Updating an address does not activate a reward deployment or make a live launch claim.

Alpha and beta do not pay tokens. The public roadmap plans beta points for a future airdrop. Direct $DCG and supported RWA-token rewards belong to Phase III. Existing proof and testnet work remains development evidence, not an active alpha claim flow.

## Machine-readable site

- `llms.txt`: a concise Markdown index for agents.
- `llms-full.txt`: full site context, native agent protocol, scope, availability, the player guide, lore, and roadmap.
- `docs.md`: the actual human-readable guide exported from the same HTML source; it includes all topics, regardless of the selected UI chapter.
- `lore.md` and `roadmap.md`: complete exports from `content/lore.json` and `content/roadmap.json`, which also generate the visible readers.
- `site.json`: published address/link values; null means Coming Soon.

These are generated into public output by `scripts/website_content.py`. The source `.txt` files are templates. Native agent commands are plain newline-delimited text on stdin, with newline-delimited JSON observations on stdout; they are not a browser-demo API.

## User acceptance

The user accepted the current website visuals on 2026-10-24 and owns browser and gameplay acceptance. Additional tests remain deferred. Builds and source checks do not certify browser rendering. Focused review scenarios:

- Reload Home: check the immediate bitmap title, candle home link, caption, and compact CA strip. Download should open the download guide. The browser tab title stays fixed across routes. The page itself should not scroll.
- Read all eight Lore pages and all five Roadmap phases. Lore artwork is always above the text and has no contrasting letterbox panel. Previous/Next, phase selection, and browser Back should restore the relevant page. Check narrow portrait and short landscape screens, including complete text and uncropped artwork.
- Switch Dark, Nord, and Solar. Check all six screens, browser reload, and an active demo map. Theme changes must not reset progress.
- In Docs, choose topics through the sidebar or mobile selector. Previous/Next changes the chapter. Only the reading pane scrolls; text is not discarded. Browser Back should restore topic navigation.
- Check Home and the token topic: CA says Coming Soon. Links shows X, PonsFamily, and GitHub as Coming Soon; LLMs opens `llms.txt` and its detailed links resolve.
- Start Demo and use arrows/WASD/HJKL or the on-screen controls. E uses a flask or portal; F finishes. The map is centered at its available resolution with square cells, including after resizing. Check a narrow portrait and short landscape viewport without page scrolling. Switching sections keeps the attempt; reloading or New descent resets it.

## Evidence and assets

The initial Rust/WASM build, WASM-target Clippy, formatting, and static source checks passed on 2026-10-23. This refinement changes no Rust rules and reuses that module. Public packaging, frontend syntax/format checks, Python syntax inspection, and source review are recorded separately from user acceptance. No browser, screenshot, test suite, gameplay, or proof generation was run for the website implementation.

The original user-supplied candle is visually unchanged. PonsFamily's logo comes from its official site and is displayed through a monochrome mask. The GitHub mark comes from Primer Octicons; its license ships with the site. X geometry, the robot, chapter icons, and bitmap title were authored for this interface. Libre Baskerville is self-hosted under its included SIL Open Font License. The retired pixel body font is removed; only the title uses bitmap lettering.

Current Lore originals and exact built-in image-generation prompts are saved in `CEO-FILES/lore/2026-10-24-v2/`. The first version remains archived in `CEO-FILES/lore/2026-10-24/`. The revision uses actual official Boris Vallejo paintings as visual style references, with original DiggerChain scenes and characters. Reference works are not shipped as site assets. Compact WebP copies ship in `assets/lore/`, with prompt sidecars; original PNGs retain embedded prompt metadata. Lore artwork is lazy-loaded. Sources and hashes are recorded in `assets/provenance.json`.

The published Downloads link Linux, portable Windows, macOS Apple silicon, and macOS Intel archives. Native build/package jobs passed; additional platform runtime acceptance remains deferred. Open Graph and X card metadata use the original campaign keeper painting, delivered as an optimized JPEG with its prompt sidecar.
