# DiggerChain silent campaign films

Two 18-second films, landscape 1920×1080 and portrait 1080×1920, at 30 fps. fframes 1.3.0 composes original fantasy paintings, the website's own bitmap wordmark, and actual native terminal footage. Skia renders through Vulkan. The audio map is empty; final exports contain no audio stream.

This is a standalone production workspace. Its dependencies, media, render caches, and codec compatibility files do not ship in the game archive or website.

## Edit plan

| Shot | Main range | Treatment |
| --- | --- | --- |
| TheKeeper | 0–3 s | Immediate keeper/delver painting, slow push, original bitmap title. |
| CarveAPassage | 3–9 s | A real recorded glyph-view passage, including crystals and charge; original speed. |
| FollowThePortals | 9–15 s | The same real take, switching to sprites and descending through portals; 1.95× source speed. |
| EnterTheCastle | 15–18 s | Painted invitation and an offline-alpha call to play. |

Adjacent shots overlap by 0.3 seconds. Portrait crops the actual map into a larger phone-friendly viewport; landscape preserves the whole native window. Neither format invents game objects, combat, rewards, or gameplay animations. Slow movement belongs to the painted backgrounds and window entrance, with no camera shake.

## Sources

- `recordings/gameplay.json`: 42.03 seconds of native ANSI terminal output, captured in a 100×34 PTY from a task-owned seed-0 practice expedition. All 275 successful actions were saved by the real executable. No private expedition save is included.
- `capture.py` and `src/bin/capture-route.rs`: the repeatable capture procedure. Capture refuses to overwrite an existing take. The route uses `diggerchain-core`; it does not reimplement the game.
- `media/`: four original campaign paintings, Libre Baskerville, and DejaVu Sans Mono. Artwork PNG metadata retains its exact generation prompt. Font licenses are in `LICENSES/`. Visual reference works are not bundled.
- `src/lib.rs`: scenes, cached terminal snapshots, and motion. `src/main.rs`: the fframes CLI and Vulkan backend.

## Build and export

Install the prerequisites from the official [fframes skill](https://fframes.studio/) and use two Cargo jobs:

```sh
cargo build --release --locked -j 2 --manifest-path tools/promo/Cargo.toml
tools/promo/target/release/diggerchain-promo timeline
tools/promo/target/release/diggerchain-promo inspect --pops
tools/promo/target/release/diggerchain-promo render -o target/promo-landscape-master.mp4
tools/promo/target/release/diggerchain-promo --portrait render -o target/promo-portrait-master.mp4
```

The development laptop's x264 ABI is 165 while fframes' prebuilt FFmpeg references ABI 163. For this production tool only, an Ubuntu package was extracted into `target/tool-setup/x264-163`; the system codec was not replaced. Its URL and SHA-256 are recorded in that folder's `SOURCE.json`. Set `DIGGERCHAIN_PROMO_X264` to the absolute directory containing `libx264.so.163` when compiling on this laptop. The build script adds only that tool-local link search path and runtime path. Other machines with compatible codec libraries do not need this override. The upstream package is [libx264-163 from Ubuntu](https://archive.ubuntu.com/ubuntu/pool/universe/x/x264/libx264-163_0.163.3060+git5db6aa6-2build1_amd64.deb).

Remux a rendered master with the system FFmpeg to remove any audio and move MP4 metadata to the front:

```sh
ffmpeg -i target/promo-landscape-master.mp4 -map 0:v:0 -c:v copy -an -movflags +faststart OUTPUT.mp4
```

Use a new output path rather than overwriting an accepted export. Check the actual export with `ffprobe`; a source configuration alone does not establish silence or codec details.

## Review

Use the fframes `timeline`, `inspect`, `strip`, and `frame` commands for production inspection. There is no video test suite. Pop warnings confined to gameplay reflect real discrete terminal moves, counters, floating labels, and view/portal changes; they are not interpolated into fabricated game motion. The maintainer owns creative acceptance.

Final assets live in the ignored `CEO-FILES/promo/2026-10-24/` folder. No social post or external announcement is sent by these tools.
