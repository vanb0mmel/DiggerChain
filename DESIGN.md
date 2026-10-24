---
name: DiggerChain website
description: A compact candlelit game screen with three palettes, a bitmap Home title, and proportional book typography.
colors:
  accent: "#a7db86"
  ground: "#090b0e"
  ink: "#e5e9ef"
  muted: "#a5adba"
  rule: "#5d6777"
  surface: "#12161c"
  hover: "#1d252b"
  crystal: "#7dd7d0"
  portal: "#bca4ec"
  flask: "#e3b979"
  map-ground: "#0b0f14"
typography:
  headline:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "clamp(25px, 3.3vw, 36px)"
    fontWeight: 400
    lineHeight: 1.2
  title:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "24px"
    fontWeight: 400
    lineHeight: 1.3
  body:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.55
  article:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.7
  command:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "clamp(18px, 2vw, 23px)"
    fontWeight: 400
    lineHeight: 1.35
  code:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.7
  label:
    fontFamily: "'Libre Baskerville', Georgia, serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.55
rounded:
  square: "0"
spacing:
  small: "8px"
  medium: "16px"
  large: "32px"
components:
  text-action:
    backgroundColor: "transparent"
    textColor: "{colors.accent}"
    typography: "{typography.body}"
    padding: "0"
  home-command:
    backgroundColor: "transparent"
    textColor: "{colors.accent}"
    typography: "{typography.command}"
    padding: "3px 8px"
  theme-choice:
    backgroundColor: "transparent"
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "4px 8px"
  topic-select:
    backgroundColor: "{colors.ground}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.square}"
    padding: "4px 8px"
  touch-control:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    width: "44px"
    height: "44px"
  topic-link:
    backgroundColor: "transparent"
    textColor: "{colors.muted}"
    padding: "6px 8px"
  reader-action:
    backgroundColor: "transparent"
    textColor: "{colors.accent}"
    typography: "{typography.label}"
    padding: "4px 0"
  footer-navigation:
    textColor: "{colors.muted}"
    width: "min(720px, 100%)"
  destination:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    padding: "18px 20px"
---

# Design System: DiggerChain website

## Overview

**Creative North Star: "The Candlelit Boot Screen"**

An original candle, an original bitmap Home wordmark, strict proportional book typography, and direct text controls form a compact game screen. Three palettes share flat geometry. The fixed viewport surrounds illustrated reading, selected Docs topics, and a themed glyph map.

Scope: `website/` only. Sources: `site.css`, `index.html`, `app.js`, `demo.js`, Lore and Roadmap content, `PRODUCT.md`, and the website surface brief. This source-derived record contains no computed-style, screenshot, runtime, gameplay, or visual-acceptance evidence. The native terminal font is outside its scope.

**Key Characteristics:**

- Dark, Nord, and Solarized Light palettes.
- Proportional Libre Baskerville UI and reading, an inline bitmap Home wordmark, and the approved candle asset.
- Fixed viewport with paginated illustrated Lore, phase-based Roadmap, and topic-based Docs.
- Flat controls and visible keyboard states.

## Colors

Frontmatter records Dark; sidecar maps record all three palettes. CSS custom properties remain authoritative.

### Primary

- **Phosphor selection** (`accent`): current navigation, commands, focus, and actions; frost blue in Nord and deep teal in Solarized Light.

### Secondary

- **Crystal**, **portal**, and **flask** colors identify map objects and legend entries.

### Neutral

- **Ground**, **ink**, and **muted** separate background, text, and support text.
- **Rule**, **surface**, and **hover** define boundaries and states. Map-specific tones distinguish floors, memory, rock, and player background.

**The Shared Palette Rule.** Theme changes update both interface and map through CSS tokens.

## Typography

Self-hosted Libre Baskerville uses Georgia and serif fallbacks. One normal variable WOFF2 provides weights (400–700) with font-display swap and a font preload. Ordinary headings remain regular weight. Buttons, selectors, code, keyboard labels, and browser-map glyphs use the same proportional family. Home alone uses original bitmap/character-art lettering: `assets/wordmark.svg` is embedded inline by `scripts/website_content.py`, with no wordmark font loading or swapping. The semantic h1 has `aria-label="DiggerChain"`; its SVG is aria-hidden, uses currentColor paths, and etches with the theme ground.

Frontmatter records the shared type roles. Lore headings use (clamp(21px, 2.3vw, 29px)/1.25), and Lore passages use (16px/1.7) within a (min(72ch, 100%)) copy width; passage headings have a (12px) bottom margin. Roadmap headings use (clamp(24px, 3vw, 34px)/1.2), summaries use the article scale with a (62ch) measure, and lists use (14px/1.6). Docs copy caps at (68ch). Home caption uses (clamp(14px, 1.7vw, 19px)/1.4). Footer labels use (clamp(12px, 1.6vw, 15px)) with a (38px) line box. Theme and reader labels use the label token; CA text uses (12px). Statistics and reader counters use tabular figures.

Narrow Docs copy uses (15px) and headings (22px). Narrow Lore passages use (14px/1.65); Roadmap summaries use (14px/1.6). Compact and short-height overrides are recorded in Layout and the sidecar.

**The Proportional Type Rule.** Website text, code, UI, and map glyphs share the proportional book family. Only the Home wordmark uses bitmap/character-art lettering; this rule does not alter the user's terminal font.

## Layout

The body uses (100dvh), toolbar/content/footer grid tracks, and hidden outer overflow, with a small-viewport-height fallback. Pages cap at (1160px). The toolbar has a (48px) candle home mark and theme controls; six footer links occupy equal columns within (720px). The spacing primitives remain the shared small, medium, and large steps in frontmatter.

Home preserves title, contained candle, caption, three commands, and CA within a centered composition capped at (660px) high. The wordmark spans (min(1120px, 100%)); its SVG uses (100%) width, automatic height, and a maximum height of (clamp(48px, 15dvh, 144px)). Its caption is "A mining adventure in your terminal." Download opens the guide at `#docs/download`; the archive download remains inside Docs. The toolbar mark also returns Home. The browser title stays "DiggerChain - A Terminal Mining Adventure" on every route. Home has no network-symbol row. Secondary captions are omitted on Docs, Demo, Roadmap, and Links.

Docs pairs a (190px) topic column and reader with a (32px) gap. One of nine chapters displays at a time. Its article is the only scrolling reading region; code blocks inside it may scroll horizontally for long commands. Topic and pagination controls stay stationary. At width ≤(760px) **or** height ≤(540px), a labeled selector replaces the topic column, the reader gains a selector row with a (10px) gap, and copy uses the narrow type sizes.

Lore presents eight pages using four paintings, one illustration shared by each pair of passages. At every viewport, including short landscape, a single column places the illustration above the text. The full artwork uses contain sizing without cropping, sits at center bottom, and has no contrasting panel background. The grid has a (20px) gap; copy is centered below it. The shipped v2 paintings are wide (1774×887), generated after studying actual Boris Vallejo references; originals and prompts remain in `CEO-FILES/lore/2026-10-24-v2`. Earlier artwork is not shipped. Roadmap presents one of five phases without dates; a (240px) selector column and a (48px) gap accompany the current phase. Both readers retain Previous/Next controls below their content.

Demo pairs a flexible map and (190px) tools with a (24px) gap. The map uses centered integer square cells capped at (32 CSS px), with backing-store device-pixel ratio capped at (3). Glyph size is the larger of (8px) and rounded (70%) of the cell size. Resize, font readiness, and theme changes repaint the canvas; fog remains enabled.

Responsive rules follow the stylesheet's order:

- At width ≤(760px), pages use (20px) horizontal padding and (25px) page headings. Lore retains artwork above copy with a (16px) gap. Roadmap shows a horizontal phase selector with numeral-only links. Demo tools move below the map; help and map legend hide, and five (44px) controls form a row.
- At width ≤(440px), pages use (16px) horizontal padding. Theme controls become (12px) text with (48px) minimum width. Home caption becomes (13px), commands (19px), and CA text (11px). Demo statistics become (15px) values; tools may wrap, and the demo note hides. Destination names become (14px), icons (28px), and the Lore page heading becomes (21px).
- At height ≤(780px), Demo help hides.
- At height ≤(660px), the toolbar and candle mark become (42px), footer line boxes (32px), Home commands (18px/1.15), and page headings (24px). Secondary Demo detail hides. Topic links shrink to (32px) minimum height. Demo tools form a column, restored to a row when width is also ≤(760px).
- At height ≤(420px), Home places candle and commands side by side beneath the spanning bitmap title, capped at (54px) high; caption becomes (12px) and commands (16px). Reader controls use (32px) minimum height. Lore retains one column with artwork above copy and a (10px) gap, hides its outer page heading, and uses (19px) passage headings with (12px/1.5) copy. Roadmap hides its outer heading and status, uses (20px) phase headings and (12px/1.4) copy, and reduces phase-index height to (28px). Links uses four columns.
- At height ≤(420px) and width ≥(480px), Demo restores a (144px) sidebar, a two-row (44px) touch pad, and a (36px) message row. Statistics use (13px) values; messages use (12px/1.35).

## Elevation & Depth

No box-shadow system is used. Thin borders and tonal contrast separate regions; the candle, paintings, and map tones provide depth. States change immediately, without animation or transitions.

## Shapes

Square controls, thin rules, and square map cells define the form. Candle and Lore artwork use contain sizing. Compact topic selection explicitly uses zero radius. Docs topic symbols are restrained stroke SVGs (17px); the current topic changes tone and surface rather than icon shape.

## Components

Accent text actions underline and switch to ink on hover. Selected Home commands add a cursor; focused commands underline. Theme choices expose pressed state through border and accent, with hover surface and ink. Keyboard focus uses an accent outline (1px), offset (3px).

Topics expose current location; phases expose the current step. Lore, Roadmap, and Docs show one selected item and a tabular counter; boundaries hide unavailable Previous or Next. Six footer links expose the current page. Bordered (44px) touch controls reverse on hover or press and dim when disabled. Destination tiles pair logos, names, and status; enabled destinations change border and surface on hover. CA and unavailable destinations currently say Coming Soon; LLMs is available. Availability is product truth, not a permanent style rule.

## Do's and Don'ts

### Do:

- **Do** share palette tokens with the demo.
- **Do** keep topic and chapter controls outside the scrolling article.
- **Do** preserve semantic title text and keyboard states.
- **Do** display the complete Lore painting above its passage at every viewport.

### Don't:

- **Don't** extend bitmap or character-art lettering beyond the Home wordmark.
- **Don't** add outer page scrolling or animated transitions.
- **Don't** apply this specification to the native game.
