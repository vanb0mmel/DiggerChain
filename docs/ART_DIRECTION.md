# DiggerChain art direction

## Current expedition

The Hollow Crown follows a human delver into a castle concealed inside a forest cave. The protagonist wears a pale slate-blue cloak and crimson scarf, with warm skin tones, brown hair and boots, and a brass pickaxe. The glyph view uses a pale `@`; the sidebar portrait and optional eight-by-eight sprite share the same original human mask. Character appearance belongs entirely to terminal presentation and does not change simulation, saves, or proofs.

The first release has three small seeded floors. Additional depth and biomes are deferred. The original Gloaming Threshold fiction and green goblin appearances below belong to the preserved `--prototype` mode.

## Preserved pilot fiction

Pip and Moss are goblin scavengers lost beneath an unfamiliar canopy. A cave mouth leads into architecture far too grand to fit inside the hill. Three crystals call from the rooms beyond. The goblins intend to bring them to an old waystone before deciding whether the castle is a discovery or a trap.

All names, lore, and setting details are provisional.

## Visual baseline

Use the visual language studied in Abyssal: colored glyphs, programmable foreground and background colors, half-block pixel sprites, tiny shaded material patterns, floating labels, and restrained ambient particles. Implement original shapes and code. Do not assemble unrelated reference systems without a demonstrated benefit.

- Forest: deep petrol greens, silver foliage, warm fireflies, violet flowers.
- Threshold: wet slate, moss, a warm seam of candlelight inside the cave.
- Nave: plum-black stone, antique gold, warm sconces, a crimson runner, high stained-glass windows.
- Conservatory: jade tiles, rose beds, a reflecting pool, cyan moonlight.
- Archive: walnut shelves, parchment, cool glass, a hanging lamp.
- Crypt: blue-grey stone, dark iron, pale roots, an amethyst crystal.
- Human delver: rounded head, readable pale cloak, warm face, crimson scarf, brass tool, and dark boots.
- Prototype goblins: readable green silhouettes, pointed ears, warm eyes, a scarf accent.

Keep most surfaces dark. Reserve strong saturation for light, crystals, flowers, and character accents. Shading should reveal material without turning every tile into noise. The interface uses quiet borders, ample spacing, and readable English prose.

## Boundaries

The authored location remains the preserved visual baseline. The current expedition adds a small deterministic chamber generator and a separate proof/reward workflow; the prototype itself has no procedural world or economy. Neither mode adds combat or audio. Presentation animation runs independently of turn-based simulation. Agent mode does not initialize rendering.

Camera shake and automatic camera effects are excluded. The camera follows only when the player leaves its central safe area. The default view uses top-down colored glyphs; pixel sprites remain available through `G`. Visual calibration controls include adjustable sprite zoom and paused ambient animation. Fog of war is always enabled. Scene reveal is disabled until explicitly requested again.

## Visual review

Ask the user to review the forest entrance, the first view into the nave, the conservatory pool, the archive, the crypt crystal, and the completed waystone. Collect feedback on readability, palette, scale, motion, terminal compatibility, and whether the castle feels unexpectedly luxurious.

The renderer is a baseline for direct comparison. Consider extra lighting or color techniques only after a specific scene demonstrates a deficiency.
