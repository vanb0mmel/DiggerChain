mod art;
mod screen;

use std::io::{self, Write};

use diggerchain_core::vault::{self, Expedition};
use diggerchain_core::{Crystal, Event, Game, Point, PropKind, Tile};

use art::{BLUE, BORDER, GOLD, GREEN, MUTED, PANEL, Rgb, TEXT, VOID};
use screen::Screen;

const ZOOMS: [i32; 4] = [2, 4, 6, 8];

#[derive(Clone, Copy)]
struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

struct Light {
    at: Point,
    color: Rgb,
    radius: f32,
}

struct Float {
    at: Point,
    age: f32,
    text: String,
    color: Rgb,
}

pub struct View {
    sprites: bool,
    zoom_index: usize,
    pub paused: bool,
    pub help: bool,
    time: f32,
    camera: Point,
    recenter: bool,
    dimensions: (u16, u16),
    previous: Option<Screen>,
    floats: Vec<Float>,
}

impl View {
    pub fn new(glyphs: bool) -> Self {
        Self {
            sprites: !glyphs,
            zoom_index: 1,
            paused: false,
            help: false,
            time: 0.0,
            camera: Point::new(0, 0),
            recenter: true,
            dimensions: (0, 0),
            previous: None,
            floats: Vec::new(),
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if !self.paused {
            self.time += dt;
            for label in &mut self.floats {
                label.age += dt;
            }
            self.floats.retain(|label| label.age < 2.0);
        }
    }

    pub fn toggle_mode(&mut self) {
        self.sprites = !self.sprites;
        self.recenter = true;
    }

    pub fn zoom(&mut self, direction: i32) {
        if self.sprites {
            self.zoom_index =
                (self.zoom_index as i32 + direction).clamp(0, ZOOMS.len() as i32 - 1) as usize;
            self.recenter = true;
        }
    }

    pub fn reset_camera(&mut self) {
        self.recenter = true;
        self.floats.clear();
    }

    pub fn on_event(&mut self, event: Event, at: Point) {
        let (text, color) = match event {
            Event::Collected(crystal) => {
                (format!("+ {}", crystal.name()), art::crystal_color(crystal))
            }
            Event::Completed => ("THE WAYSTONE AWAKENS".to_string(), GOLD),
            _ => return,
        };
        self.floats.push(Float {
            at,
            age: 0.0,
            text,
            color,
        });
    }

    pub fn on_expedition_event(&mut self, event: vault::Event, at: Point) {
        let (text, color) = match event {
            vault::Event::Collected { value } => (format!("+{value} CRYSTAL"), GOLD),
            vault::Event::Replenished { amount } => (format!("+{amount} CHARGE"), GREEN),
            vault::Event::Dug { cost } => (format!("-{cost}"), MUTED),
            _ => return,
        };
        self.floats.push(Float {
            at,
            age: 0.0,
            text,
            color,
        });
    }

    pub fn draw(
        &mut self,
        game: &Game,
        width: u16,
        height: u16,
        output: &mut impl Write,
    ) -> io::Result<()> {
        self.draw_inner(game, None, width, height, output)
    }

    pub fn draw_expedition(
        &mut self,
        game: &Game,
        expedition: &Expedition,
        finished: bool,
        width: u16,
        height: u16,
        output: &mut impl Write,
    ) -> io::Result<()> {
        self.draw_inner(game, Some((expedition, finished)), width, height, output)
    }

    fn draw_inner(
        &mut self,
        game: &Game,
        expedition: Option<(&Expedition, bool)>,
        width: u16,
        height: u16,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let mut screen = Screen::new(width, height);
        if width < 90 || height < 30 {
            screen.text(2, 2, "DIGGERCHAIN", GOLD);
            screen.clipped(
                2,
                4,
                width as i32 - 4,
                "Please enlarge the terminal to at least 90 x 30.",
                TEXT,
            );
            screen.clipped(
                2,
                6,
                width as i32 - 4,
                &format!("Current size: {width} x {height}. Q exits safely."),
                MUTED,
            );
            return screen.flush(&mut self.previous, output);
        }
        if self.dimensions != (width, height) {
            self.dimensions = (width, height);
            self.recenter = true;
        }
        let sidebar = if width >= 110 { 30 } else { 26 };
        let body = Area {
            x: 1,
            y: 3,
            w: width as i32 - sidebar - 3,
            h: height as i32 - 11,
        };
        let map = Area {
            x: body.x + 1,
            y: body.y + 1,
            w: body.w - 2,
            h: body.h - 2,
        };
        let panel = Area {
            x: body.x + body.w + 1,
            y: 3,
            w: sidebar,
            h: body.h,
        };
        screen.text(2, 0, "D I G G E R C H A I N", GOLD);
        screen.text(
            2,
            1,
            if expedition.is_some() {
                "A delver's journey beneath an impossible hill"
            } else {
                "A goblin tale beneath an impossible hill"
            },
            MUTED,
        );
        screen.text(
            width as i32 - 25,
            0,
            if expedition.is_some() {
                "THE HOLLOW CROWN"
            } else {
                "THE GLOAMING THRESHOLD"
            },
            GREEN,
        );
        let region = expedition.map_or(game.region().0, |(state, _)| {
            [
                "The Velvet Vestibule",
                "The Sleeping Court",
                "The Crown Below",
            ][state.floor()]
        });
        screen.frame(body.x, body.y, body.w, body.h, region);
        screen.frame(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            if expedition.is_some() {
                "THE DELVER"
            } else {
                "THE CRYSTAL SEEKERS"
            },
        );
        self.follow(game, map);
        let lights = collect_lights(game, self.time);
        let (player_sprite, player_color) = if expedition.is_some() {
            (&art::HUMAN, art::HUMAN_COLOR)
        } else {
            (&art::GOBLIN, GREEN)
        };
        if self.sprites {
            self.pixels(game, &mut screen, map, &lights, player_sprite, player_color);
        } else {
            self.glyphs(game, &mut screen, map, &lights, player_color);
        }
        self.atmosphere(game, &mut screen, map);
        self.draw_floats(&mut screen, map);
        if let Some((state, finished)) = expedition {
            draw_expedition_sidebar(state, finished, &mut screen, panel);
        } else {
            draw_sidebar(game, &mut screen, panel);
        }
        draw_journal(game, &mut screen, body.y + body.h, width as i32 - 2);
        screen.clipped(
            2,
            height as i32 - 2,
            width as i32 - 4,
            if expedition.is_some() {
                "WASD / arrows  Move / dig    E  Flask / portal    G  View    ?  Help    Q  Quit"
            } else {
                "WASD / arrows  Move    E  Inspect    G  View    Z/X  Zoom    ?  Help    Q  Quit"
            },
            TEXT,
        );
        screen.text(
            2,
            height as i32 - 1,
            if expedition.is_some() {
                "F  Finish expedition    Autosave on every action"
            } else {
                "Space  Stillness    R  Restart"
            },
            MUTED,
        );
        let status = format!(
            "{}  |  {}",
            if self.sprites { "PIXELS" } else { "GLYPHS" },
            if self.paused { "STILL" } else { "LIVING" }
        );
        screen.text(
            width as i32 - status.len() as i32 - 2,
            height as i32 - 1,
            &status,
            BLUE,
        );
        if self.help {
            draw_help(&mut screen, expedition.is_some());
        }
        screen.flush(&mut self.previous, output)
    }

    fn span(&self, area: Area) -> (i32, i32) {
        if self.sprites {
            let zoom = ZOOMS[self.zoom_index];
            ((area.w + zoom - 1) / zoom, (area.h * 2 + zoom - 1) / zoom)
        } else {
            ((area.w + 1) / 2, area.h)
        }
    }

    fn follow(&mut self, game: &Game, area: Area) {
        let (w, h) = self.span(area);
        if self.recenter {
            self.camera = Point::new(game.player.x - w / 2, game.player.y - h / 2);
            self.recenter = false;
        }
        self.camera.x = follow_axis(self.camera.x, game.player.x, w, game.world.width);
        self.camera.y = follow_axis(self.camera.y, game.player.y, h, game.world.height);
    }

    fn pixels(
        &self,
        game: &Game,
        screen: &mut Screen,
        area: Area,
        lights: &[Light],
        player_sprite: art::Sprite,
        player_color: Rgb,
    ) {
        let zoom = ZOOMS[self.zoom_index];
        let (tiles_w, tiles_h) = self.span(area);
        let pixel_h = area.h * 2;
        let mut pixels = vec![VOID; (area.w * pixel_h) as usize];
        for ty in 0..tiles_h {
            for tx in 0..tiles_w {
                let at = Point::new(self.camera.x + tx, self.camera.y + ty);
                if game.world.index(at).is_none() || !game.explored_at(at) {
                    continue;
                }
                let memory = !game.visible_at(at);
                let illumination = illumination(at, lights);
                for y in 0..zoom {
                    for x in 0..zoom {
                        let px = tx * zoom + x;
                        let py = ty * zoom + y;
                        if px >= area.w || py >= pixel_h {
                            continue;
                        }
                        let base = art::terrain(
                            game.world.tile(at),
                            at,
                            (x * 8 / zoom) as usize,
                            (y * 8 / zoom) as usize,
                            self.time,
                        );
                        pixels[(py * area.w + px) as usize] = shade(base, illumination, memory);
                    }
                }
            }
        }
        for prop in &game.world.props {
            if !prop.present() || !game.visible_at(prop.at) {
                continue;
            }
            let base = if prop.kind == PropKind::Waystone && game.completed {
                GOLD
            } else {
                art::prop_color(prop.kind)
            };
            let float = matches!(prop.kind, PropKind::Crystal(_) | PropKind::Companion);
            let bob = if float && (self.time * 2.1 + prop.at.x as f32).sin() > 0.4 {
                -1
            } else {
                0
            };
            let origin = Point::new(
                (prop.at.x - self.camera.x) * zoom,
                (prop.at.y - self.camera.y) * zoom + bob,
            );
            let glow = matches!(prop.kind, PropKind::Crystal(_) | PropKind::Window);
            let sprite = SpritePaint {
                origin,
                zoom,
                mask: art::sprite(prop.kind),
                color: base,
                light: illumination(prop.at, lights),
                emissive: glow,
                active: prop.active,
            };
            blit(&mut pixels, area.w, pixel_h, sprite);
        }
        let origin = Point::new(
            (game.player.x - self.camera.x) * zoom,
            (game.player.y - self.camera.y) * zoom,
        );
        blit(
            &mut pixels,
            area.w,
            pixel_h,
            SpritePaint {
                origin,
                zoom,
                mask: player_sprite,
                color: player_color,
                light: illumination(game.player, lights),
                emissive: false,
                active: true,
            },
        );
        for y in 0..area.h {
            for x in 0..area.w {
                let top = pixels[((y * 2) * area.w + x) as usize];
                let bottom = pixels[((y * 2 + 1) * area.w + x) as usize];
                screen.set(area.x + x, area.y + y, '▀', top, bottom);
            }
        }
    }

    fn glyphs(
        &self,
        game: &Game,
        screen: &mut Screen,
        area: Area,
        lights: &[Light],
        player_color: Rgb,
    ) {
        let (tiles_w, tiles_h) = self.span(area);
        for ty in 0..tiles_h {
            for tx in 0..tiles_w {
                let at = Point::new(self.camera.x + tx, self.camera.y + ty);
                if game.world.index(at).is_none() || !game.explored_at(at) {
                    continue;
                }
                let memory = !game.visible_at(at);
                let tile = game.world.tile(at);
                let light = illumination(at, lights);
                let base = art::terrain(tile, at, 3, 3, self.time);
                let background = shade(base.scale(0.65), light, memory);
                let ch = match tile {
                    Tile::Wall | Tile::Rock => ' ',
                    Tile::Water => '~',
                    Tile::Grass => {
                        if art::hash(at.x, at.y).is_multiple_of(3) {
                            ','
                        } else {
                            '·'
                        }
                    }
                    Tile::DoorClosed => '+',
                    Tile::DoorOpen => '/',
                    Tile::Carpet => '·',
                    Tile::Void => ' ',
                    _ => '·',
                };
                let foreground = shade(base.scale(1.7), light, memory);
                for dx in 0..2 {
                    let sx = area.x + tx * 2 + dx;
                    if sx < area.x + area.w {
                        screen.set(
                            sx,
                            area.y + ty,
                            if dx == 0 { ch } else { ' ' },
                            foreground,
                            background,
                        );
                    }
                }
            }
        }
        for prop in &game.world.props {
            if !prop.present() || !game.visible_at(prop.at) {
                continue;
            }
            let ch = match prop.kind {
                PropKind::Tree => '♠',
                PropKind::Fern => '♣',
                PropKind::Flowers => '*',
                PropKind::Mushroom => '⌂',
                PropKind::Pillar => '▥',
                PropKind::Window => '◊',
                PropKind::Bookshelf => '≡',
                PropKind::Coffin => '†',
                PropKind::Bench => '═',
                PropKind::Banner => '⚑',
                PropKind::Brazier => {
                    if prop.active {
                        'Ψ'
                    } else {
                        'v'
                    }
                }
                PropKind::Sconce => {
                    if prop.active {
                        'i'
                    } else {
                        '·'
                    }
                }
                PropKind::Crystal(_) => '♦',
                PropKind::Waystone => 'Ω',
                PropKind::Companion => 'g',
                PropKind::Flask => '!',
                PropKind::PortalUp => '<',
                PropKind::PortalDown => '>',
                PropKind::SoftVein => 'r',
                PropKind::HardVein => 'R',
            };
            let (x, y) = self.screen_position(prop.at, area);
            if inside(x, y, area) {
                screen.text(x, y, &ch.to_string(), art::prop_color(prop.kind));
            }
        }
        let (x, y) = self.screen_position(game.player, area);
        if inside(x, y, area) {
            screen.text(x, y, "@", player_color);
        }
    }

    fn screen_position(&self, at: Point, area: Area) -> (i32, i32) {
        if self.sprites {
            let zoom = ZOOMS[self.zoom_index];
            (
                area.x + (at.x - self.camera.x) * zoom + zoom / 2,
                area.y + (at.y - self.camera.y) * zoom / 2 + zoom / 4,
            )
        } else {
            (
                area.x + (at.x - self.camera.x) * 2,
                area.y + at.y - self.camera.y,
            )
        }
    }

    fn atmosphere(&self, game: &Game, screen: &mut Screen, area: Area) {
        for i in 0..25 {
            let outside = i < 15;
            let x = if outside {
                4 + i * 7 % 25
            } else {
                36 + i * 11 % 44
            };
            let y = if outside {
                10 + i * 11 % 26
            } else {
                4 + i * 7 % 29
            };
            let at = Point::new(x, y);
            if game.world.tile(at).opaque() || !game.visible_at(at) {
                continue;
            }
            let (mut sx, mut sy) = self.screen_position(at, area);
            sx += (self.time * 0.7 + i as f32 * 2.3).sin().round() as i32;
            sy += (self.time * 0.5 + i as f32).sin().round() as i32;
            let pulse = (self.time * 1.6 + i as f32 * 1.7).sin();
            if pulse > 0.25 && inside(sx, sy, area) {
                let color = if outside {
                    Rgb(211, 224, 153)
                } else {
                    Rgb(169, 161, 174)
                };
                screen.text(sx, sy, "·", color.scale(pulse * 0.65));
            }
        }
    }

    fn draw_floats(&self, screen: &mut Screen, area: Area) {
        for label in &self.floats {
            let (x, y) = self.screen_position(label.at, area);
            let y = y - 1 - (label.age * 1.5) as i32;
            let width = (label.text.len() as i32).min(area.w - 2);
            let x = (x - width / 2).clamp(area.x, area.x + area.w - width);
            if inside(x, y, area) {
                screen.clipped(
                    x,
                    y,
                    width,
                    &label.text,
                    label.color.scale((2.0 - label.age).min(1.0)),
                );
            }
        }
    }
}

fn inside(x: i32, y: i32, area: Area) -> bool {
    x >= area.x && y >= area.y && x < area.x + area.w && y < area.y + area.h
}

fn follow_axis(mut origin: i32, focus: i32, span: i32, extent: i32) -> i32 {
    if span >= extent {
        return -(span - extent) / 2;
    }
    let margin = (span / 4).max(1);
    if focus < origin + margin {
        origin = focus - margin;
    }
    if focus >= origin + span - margin {
        origin = focus - span + margin + 1;
    }
    origin.clamp(0, extent - span)
}

fn collect_lights(game: &Game, time: f32) -> Vec<Light> {
    let mut lights = vec![Light {
        at: game.player,
        color: Rgb(126, 110, 73),
        radius: 9.0,
    }];
    for prop in &game.world.props {
        if !prop.active {
            continue;
        }
        let phase = prop.at.x as f32 * 0.8 + prop.at.y as f32;
        let flicker = 0.88 + 0.08 * (time * 6.3 + phase).sin() + 0.04 * (time * 13.1 + phase).sin();
        let (color, radius) = match prop.kind {
            PropKind::Brazier => (Rgb(236, 139, 62).scale(flicker), 6.5),
            PropKind::Sconce => (Rgb(220, 174, 99).scale(flicker), 5.0),
            PropKind::Crystal(crystal) => (
                art::crystal_color(crystal).scale(0.7 + 0.12 * (time * 1.7).sin()),
                5.5,
            ),
            PropKind::Window => (Rgb(89, 112, 194), 6.0),
            PropKind::Mushroom => (Rgb(61, 110, 127).scale(0.8), 2.8),
            PropKind::Flask => (Rgb(176, 108, 52).scale(0.8), 2.5),
            PropKind::PortalUp | PropKind::PortalDown => (
                Rgb(111, 103, 205).scale(0.85 + 0.1 * (time * 1.5).sin()),
                4.5,
            ),
            PropKind::Waystone => (
                if game.completed {
                    GOLD
                } else {
                    Rgb(61, 115, 118)
                },
                if game.completed { 10.0 } else { 4.0 },
            ),
            _ => continue,
        };
        lights.push(Light {
            at: prop.at,
            color,
            radius,
        });
    }
    lights
}

fn illumination(at: Point, lights: &[Light]) -> Rgb {
    let mut color = Rgb(0, 0, 0);
    for light in lights {
        let dx = (at.x - light.at.x) as f32;
        let dy = (at.y - light.at.y) as f32;
        let falloff = (1.0 - (dx * dx + dy * dy).sqrt() / light.radius).max(0.0);
        color = color.add(light.color.scale(falloff * falloff * 0.42));
    }
    color
}

fn shade(base: Rgb, light: Rgb, memory: bool) -> Rgb {
    if memory {
        let grey = base.0 as f32 * 0.3 + base.1 as f32 * 0.5 + base.2 as f32 * 0.2;
        return Rgb(
            (grey * 0.20) as u8,
            (grey * 0.24) as u8,
            (grey * 0.34) as u8,
        );
    }
    base.scale(0.78).add(light)
}

struct SpritePaint {
    origin: Point,
    zoom: i32,
    mask: art::Sprite,
    color: Rgb,
    light: Rgb,
    emissive: bool,
    active: bool,
}

fn blit(pixels: &mut [Rgb], width: i32, height: i32, paint: SpritePaint) {
    for y in 0..paint.zoom {
        for x in 0..paint.zoom {
            let px = paint.origin.x + x;
            let py = paint.origin.y + y;
            if px < 0 || py < 0 || px >= width || py >= height {
                continue;
            }
            let sample_x = ((x * 2 + 1) * 8 / (paint.zoom * 2)) as usize;
            let sample_y = ((y * 2 + 1) * 8 / (paint.zoom * 2)) as usize;
            let code = paint.mask[sample_y].as_bytes()[sample_x];
            let Some(mut color) = art::sprite_pixel(code, paint.color) else {
                continue;
            };
            if !paint.active && matches!(code, b'h' | b'a' | b'w' | b'r') {
                color = Rgb(63, 55, 57);
            }
            if code != b'o' && !paint.emissive {
                color = shade(color, paint.light.scale(0.35), false);
            }
            pixels[(py * width + px) as usize] = color;
        }
    }
}

fn draw_portrait(screen: &mut Screen, x: i32, y: i32, mask: art::Sprite, color: Rgb) {
    for row in 0..4 {
        for col in 0..8 {
            let top = art::sprite_pixel(mask[row * 2].as_bytes()[col], color).unwrap_or(PANEL);
            let bottom =
                art::sprite_pixel(mask[row * 2 + 1].as_bytes()[col], color).unwrap_or(PANEL);
            screen.set(x + col as i32, y + row as i32, '▀', top, bottom);
        }
    }
}

fn draw_sidebar(game: &Game, screen: &mut Screen, area: Area) {
    let x = area.x + 2;
    let y = area.y + 2;
    draw_portrait(screen, x, y, &art::GOBLIN, GREEN);
    screen.text(x + 10, y, "PIP", GREEN);
    screen.text(x + 10, y + 1, "Goblin scout", TEXT);
    screen.text(x + 10, y + 3, &format!("Step {}", game.turns), MUTED);
    screen.text(x, area.y + 8, "THE THREE VOICES", GOLD);
    for (i, (kind, label)) in [
        (Crystal::Thorns, "THORNS"),
        (Crystal::Silence, "SILENCE"),
        (Crystal::Echoes, "ECHOES"),
    ]
    .iter()
    .enumerate()
    {
        let found = game.crystals.contains(kind);
        screen.text(
            x,
            area.y + 10 + i as i32,
            if found { "◆" } else { "◇" },
            if found {
                art::crystal_color(*kind)
            } else {
                BORDER
            },
        );
        screen.text(
            x + 3,
            area.y + 10 + i as i32,
            label,
            if found {
                art::crystal_color(*kind)
            } else {
                MUTED
            },
        );
        screen.text(
            x + 14,
            area.y + 10 + i as i32,
            if found { "found" } else { "..." },
            MUTED,
        );
    }
    if area.h >= 23 {
        let goal = if game.completed {
            "The waystone is awake. The forest is listening."
        } else if game.crystals.len() == 3 {
            "Return to the waystone in the clearing."
        } else {
            "Follow the crystal songs into the castle."
        };
        screen.wrap(
            x,
            area.y + 14,
            area.w - 4,
            3,
            goal,
            if game.completed { GREEN } else { TEXT },
        );
    }
    if area.h >= 28 {
        screen.wrap(x, area.y + 18, area.w - 4, 3, game.region().1, MUTED);
    }
    let hint_y = area.y + area.h - 4;
    if let Some(index) = game.interaction_target() {
        screen.text(x, hint_y, "E  INSPECT", GOLD);
        screen.clipped(
            x,
            hint_y + 1,
            area.w - 4,
            game.world.props[index].kind.name(),
            TEXT,
        );
    } else {
        screen.text(x, hint_y, "Listen. Look closer.", MUTED);
        screen.text(x, hint_y + 1, "The hill has secrets.", MUTED);
    }
}

fn draw_journal(game: &Game, screen: &mut Screen, y: i32, width: i32) {
    screen.frame(1, y, width, 6, "FIELD NOTES");
    let messages: Vec<_> = game.messages.iter().rev().take(2).collect();
    for (i, message) in messages.iter().rev().enumerate() {
        screen.wrap(
            3,
            y + 1 + i as i32 * 2,
            width - 4,
            2,
            message,
            if i + 1 == messages.len() { TEXT } else { MUTED },
        );
    }
}

fn draw_expedition_sidebar(state: &Expedition, finished: bool, screen: &mut Screen, area: Area) {
    let x = area.x + 2;
    let y = area.y + 2;
    screen.text(x, y, "HUMAN DELVER", art::HUMAN_COLOR);
    draw_portrait(screen, x + area.w - 12, y, &art::HUMAN, art::HUMAN_COLOR);
    screen.text(
        x,
        y + 2,
        &format!("Depth {} / {}", state.floor() + 1, vault::FLOORS),
        TEXT,
    );
    screen.text(x, y + 4, &format!("Crystal score: {}", state.score()), GOLD);
    screen.text(
        x,
        y + 6,
        &format!("Charge: {} / {}", state.charge(), vault::CAPACITY),
        BLUE,
    );
    let bar = format!(
        "[{}{}]",
        "|".repeat(state.charge() as usize),
        ".".repeat((vault::CAPACITY - state.charge()) as usize)
    );
    screen.text(
        x,
        y + 7,
        &bar,
        if state.charge() < 3 { GOLD } else { GREEN },
    );
    screen.text(
        x,
        y + 9,
        &format!("Actions: {} / {}", state.turns(), vault::MAX_ACTIONS),
        MUTED,
    );
    let hint = match state.cell(state.position()) {
        vault::Cell::Flask => "E  Drink ember flask",
        vault::Cell::Up => "E  Ascend",
        vault::Cell::Down => "E  Descend",
        _ => "Bump veins to dig",
    };
    screen.clipped(
        x,
        y + 11,
        area.w - 4,
        if finished { "EXPEDITION SEALED" } else { hint },
        if finished { GOLD } else { TEXT },
    );
    screen.clipped(
        x,
        y + 13,
        area.w - 4,
        if finished {
            "Q  Exit safely"
        } else {
            "F  Finish this attempt"
        },
        MUTED,
    );
    if area.h >= 24 {
        screen.wrap(
            x,
            y + 16,
            area.w - 4,
            3,
            "Digging spends charge. Walking and portals are free. Nothing grows back.",
            MUTED,
        );
    }
}

fn draw_help(screen: &mut Screen, expedition: bool) {
    let width = 70;
    let height = 23;
    let x = (screen.width - width) / 2;
    let y = (screen.height - height) / 2;
    screen.frame(x, y, width, height, "A SMALL GUIDE TO AN IMPOSSIBLE PLACE");
    screen.text(
        x + 3,
        y + 2,
        if expedition {
            "You are a human delver. The Hollow Crown is waiting."
        } else {
            "You are Pip. Moss is guarding the clearing."
        },
        if expedition { art::HUMAN_COLOR } else { GREEN },
    );
    screen.wrap(x + 3, y + 4, width - 6, 2, if expedition { "Explore three floors. Deeper crystals score more. F finishes this attempt early at its current score." } else { "Find three singing crystals in the castle. Bring them to the old waystone near the beginning, then press E." }, TEXT);
    for (i, (keys, action)) in [
        (
            "WASD / arrows / HJKL",
            if expedition {
                "Walk; bump a vein to dig it"
            } else {
                "Walk; bump a door to open it"
            },
        ),
        (
            "E / Enter",
            if expedition {
                "Use flask or portal underfoot"
            } else {
                "Inspect or interact nearby"
            },
        ),
        ("G", "Switch colored glyphs / pixel sprites"),
        ("Z / X", "Zoom pixel sprites in / out"),
        ("Space", "Pause ambient motion; walking still works"),
        (
            if expedition { "F" } else { "R" },
            if expedition {
                "Seal the saved expedition"
            } else {
                "Restart this handcrafted location"
            },
        ),
        (
            "Q / Escape",
            if expedition {
                "Exit; progress is already saved"
            } else {
                "Quit"
            },
        ),
    ]
    .iter()
    .enumerate()
    {
        screen.text(x + 3, y + 8 + i as i32, keys, GOLD);
        screen.clipped(x + 28, y + 8 + i as i32, width - 31, action, TEXT);
    }
    screen.text(
        x + 3,
        y + 18,
        if expedition {
            "512 actions. Invalid moves cost nothing. No time pressure."
        } else {
            "No combat or time pressure. Take your time."
        },
        MUTED,
    );
    screen.text(
        x + 3,
        y + 20,
        "Press ? or Enter to return to the game.",
        BLUE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_holds_still_inside_its_safe_area() {
        assert_eq!(follow_axis(10, 18, 20, 86), 10);
        assert_eq!(follow_axis(10, 26, 20, 86), 12);
        assert_eq!(follow_axis(0, 2, 100, 86), -7);
    }

    #[test]
    fn both_views_render_at_the_minimum_terminal_size() {
        let game = Game::new();
        for glyphs in [true, false] {
            let mut view = View::new(glyphs);
            let mut output = Vec::new();
            view.draw(&game, 90, 30, &mut output).unwrap();
            assert!(
                String::from_utf8(output)
                    .unwrap()
                    .contains("D I G G E R C H A I N")
            );
            view.zoom(2);
            view.draw(&game, 140, 50, &mut Vec::new()).unwrap();
        }
    }
}
