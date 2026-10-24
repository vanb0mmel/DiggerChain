//! Silent campaign films composed in fframes from original art and real PTY footage.
use fframes::{
    AudioMap, Color, Duration, FFramesContext, Frame, Overlap, Scene, Scenes, Svgr, Transform,
    Video, animation::Easing, include_media_dir,
};
use serde_json::Value;
use std::sync::Arc;

include_media_dir!(pub struct CampaignMedia, "media");
const FPS: usize = 30;
const PAPER: &str = "#f3dfb3";
const INK: &str = "#080b10";
const GREEN: &str = "#76dbaa";
const FONT: &str = "Libre Baskerville";
const WORDMARK: &str = include_str!("../../../website/assets/wordmark.svg");

#[derive(Debug)]
struct Run {
    x: usize,
    y: usize,
    cells: usize,
    foreground: String,
    background: String,
    text: String,
}

#[derive(Debug)]
struct Tape {
    frames: Vec<Vec<Run>>,
    descent: f32,
}

fn color(value: vt100::Color, foreground: bool) -> String {
    match value {
        vt100::Color::Default => if foreground { "#e1d7c4" } else { INK }.to_owned(),
        vt100::Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        vt100::Color::Idx(index) => {
            let palette = [
                "#080b10", "#cc5555", "#76dbaa", "#d2ac6c", "#719cce", "#aa84bc", "#6bbab7",
                "#dfd9cd", "#656b77", "#f27e7e", "#a1e5b6", "#edce8f", "#a0bce0", "#ceb0dd",
                "#9ddcdb", "#f5efdf",
            ];
            if index < 16 {
                return palette[index as usize].to_owned();
            }
            let (r, g, b) = if index >= 232 {
                let grey = 8 + 10 * (index - 232);
                (grey, grey, grey)
            } else {
                let n = index - 16;
                let levels = [0, 95, 135, 175, 215, 255];
                (
                    levels[(n / 36) as usize],
                    levels[((n / 6) % 6) as usize],
                    levels[(n % 6) as usize],
                )
            };
            format!("#{r:02x}{g:02x}{b:02x}")
        }
    }
}

impl Tape {
    fn load() -> Result<Self, String> {
        let capture: Value = serde_json::from_str(include_str!("../recordings/gameplay.json"))
            .map_err(|e| e.to_string())?;
        let rows = capture["rows"].as_u64().ok_or("Missing capture height")? as u16;
        let cols = capture["cols"].as_u64().ok_or("Missing capture width")? as u16;
        let events = capture["events"]
            .as_array()
            .ok_or("Missing captured output")?;
        let duration = capture["duration"]
            .as_f64()
            .ok_or("Missing capture duration")?;
        let descent = capture["markers"]
            .as_array()
            .and_then(|items| items.iter().find(|m| m["floor"] == 1))
            .and_then(|m| m["time"].as_f64())
            .ok_or("Capture has no descent")? as f32;
        let mut parser = vt100::Parser::new(rows, cols, 0);
        let mut cursor = 0;
        let mut frames = Vec::new();
        for frame in 0..(duration * FPS as f64).ceil() as usize {
            let t = frame as f64 / FPS as f64;
            while cursor < events.len()
                && events[cursor]["time"]
                    .as_f64()
                    .ok_or("Invalid event time")?
                    <= t
            {
                parser.process(
                    events[cursor]["data"]
                        .as_str()
                        .ok_or("Invalid event data")?
                        .as_bytes(),
                );
                cursor += 1;
            }
            let mut runs = Vec::new();
            for y in 0..rows {
                let mut x = 0;
                while x < cols {
                    let cell = parser.screen().cell(y, x).ok_or("Missing terminal cell")?;
                    let mut fg = color(cell.fgcolor(), true);
                    let mut bg = color(cell.bgcolor(), false);
                    if cell.inverse() {
                        std::mem::swap(&mut fg, &mut bg);
                    }
                    let mut text = String::new();
                    let start = x;
                    while x < cols {
                        let c = parser.screen().cell(y, x).ok_or("Missing terminal cell")?;
                        let mut c_fg = color(c.fgcolor(), true);
                        let mut c_bg = color(c.bgcolor(), false);
                        if c.inverse() {
                            std::mem::swap(&mut c_fg, &mut c_bg);
                        }
                        if c_fg != fg || c_bg != bg {
                            break;
                        }
                        if c.contents().is_empty() {
                            text.push(' ');
                        } else {
                            text.push_str(c.contents());
                        }
                        x += 1;
                    }
                    if !text.trim().is_empty() || bg != INK {
                        runs.push(Run {
                            x: start as usize,
                            y: y as usize,
                            cells: (x - start) as usize,
                            foreground: fg,
                            background: bg,
                            text,
                        });
                    }
                }
            }
            frames.push(runs);
        }
        if frames.is_empty() {
            return Err("Recording contains no frames".into());
        }
        Ok(Self { frames, descent })
    }

    fn draw<'a>(&'a self, time: f32) -> Svgr<'a> {
        let index = ((time.max(0.) * FPS as f32) as usize).min(self.frames.len() - 1);
        let nodes: Vec<Svgr<'a>> = self.frames[index].iter().map(|run| {
            fframes::svgr!(<g>
                <rect x={run.x * 18} y={run.y * 32} width={run.cells * 18} height="32" fill={run.background.as_str()} />
                <text x={run.x * 18} y={run.y * 32 + 26} font-family="DejaVu Sans Mono" font-size="30" fill={run.foreground.as_str()} textLength={run.cells * 18} lengthAdjust="spacingAndGlyphs">{run.text.as_str()}</text>
            </g>)
        }).collect();
        fframes::svgr!(<g><rect width="1800" height="1088" fill={INK}/>{nodes}</g>)
    }
}

fn logo<'a>(x: f32, y: f32, width: f32) -> Svgr<'a> {
    let paths: Vec<&str> = WORDMARK
        .split("d=\"")
        .skip(1)
        .map(|s| s.split('"').next().unwrap_or(""))
        .collect();
    let first = paths.first().copied().unwrap_or("");
    let second = paths.get(1).copied().unwrap_or("");
    fframes::svgr!(<svg x={x} y={y} width={width} height={width * 19. / 123.} viewBox="0 0 123 19">
        <path d={first} fill={PAPER}/><path d={second} fill={INK}/>
    </svg>)
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Keeper,
    Carve,
    Descend,
    Invitation,
}

#[derive(Debug)]
struct Shot {
    kind: Kind,
    width: f32,
    height: f32,
    tape: Arc<Tape>,
}

impl Scene for Shot {
    fn name(&self) -> &'static str {
        match self.kind {
            Kind::Keeper => "TheKeeper",
            Kind::Carve => "CarveAPassage",
            Kind::Descend => "FollowThePortals",
            Kind::Invitation => "EnterTheCastle",
        }
    }
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(match self.kind {
            Kind::Keeper | Kind::Invitation => 3.,
            _ => 6.,
        })
    }
    fn overlap(&self) -> Overlap {
        match self.kind {
            Kind::Keeper => Overlap::None,
            _ => Overlap::Previous(0.3),
        }
    }
    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let (w, h) = (self.width, self.height);
        let portrait = h > w;
        let gameplay = matches!(self.kind, Kind::Carve | Kind::Descend);
        let art_name = match self.kind {
            Kind::Keeper => {
                if portrait {
                    "delver.png"
                } else {
                    "keeper.png"
                }
            }
            Kind::Invitation => {
                if portrait {
                    "delver.png"
                } else {
                    "invitation.png"
                }
            }
            _ => "castle.png",
        };
        let Some(art) = ctx.get_image(art_name) else {
            return Svgr::empty();
        };
        let title_x = if portrait { 90. } else { 170. };
        let title_y = if gameplay {
            if portrait { 290. } else { 52. }
        } else {
            if portrait { 1260. } else { 735. }
        };
        let title_w = if portrait {
            900.
        } else if gameplay {
            620.
        } else {
            1580.
        };
        let title = logo(title_x, title_y, title_w);
        let art_opacity = if gameplay { 0.22 } else { 1. };
        let content = if gameplay {
            let seconds = match self.kind {
                Kind::Carve => 2.1 + frame.seconds(),
                _ => self.tape.descent - 0.4 + frame.seconds() * 1.95,
            };
            let capture = self.tape.draw(seconds);
            let (x, y, tw, th, viewbox) = if portrait {
                (90., 600., 900., 900. * 672. / 792., "270 128 792 672")
            } else {
                (280., 198., 1360., 1360. * 1088. / 1800., "0 0 1800 1088")
            };
            let caption = if matches!(self.kind, Kind::Carve) {
                "Carve a passage."
            } else {
                "Follow the portals."
            };
            fframes::svgr!(<g>
                <text x={if portrait {w/2.} else {1750.}} y={if portrait {525.} else {112.}} text-anchor={if portrait {"middle"} else {"end"}} font-family={FONT} font-size={if portrait {46} else {38}} fill={PAPER}>{caption}</text>
                <g transform={frame.animate(fframes::timeline!(at 0.0 => 0.5, animate Transform::translate(0,28) => Transform::translate(0,0), Easing::EaseOut))}>
                    <rect x={x-2.} y={y-44.} width={tw+4.} height={th+48.} rx="2" fill={INK} stroke="#655c46" stroke-width="2"/>
                    <path d={format!("M{x},{}H{}",y-4.,x+tw)} stroke="#655c46" stroke-width="1"/>
                    <text x={x+18.} y={y-16.} font-family="DejaVu Sans Mono" font-size="22" fill={GREEN}>"diggerchain — alpha"</text>
                    <svg x={x} y={y} width={tw} height={th} viewBox={viewbox} overflow="hidden">{capture}</svg>
                </g>
                <text x={w/2.} y={if portrait {1450.} else {1048.}} text-anchor="middle" font-family={FONT} font-size="28" fill="#c6bfaf">{if portrait {"Recorded in the terminal."} else {"Native terminal gameplay · Alpha expedition"}}</text>
            </g>)
        } else {
            let caption = if matches!(self.kind, Kind::Keeper) {
                "A mining adventure in your terminal."
            } else {
                "Play the alpha. Explore offline."
            };
            fframes::svgr!(<g>
                <text x={w/2.} y={if portrait {1480.} else {1050.}} text-anchor="middle" font-family={FONT} font-size={if portrait {38} else {44}} fill={PAPER} opacity={frame.animate(fframes::timeline!(at 0.2 => 0.75, animate 0.0_f32 => 1.0, Easing::EaseOut))}>{caption}</text>
            </g>)
        };
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={w} height={h}>
            <defs>
                <linearGradient id="shade" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color={INK} stop-opacity="0.05"/>
                    <stop offset={if portrait {"0.48"} else {"0.54"}} stop-color={INK} stop-opacity="0.06"/>
                    <stop offset={if portrait {"0.73"} else {"0.92"}} stop-color={INK} stop-opacity="0.95"/>
                    <stop offset="1" stop-color={INK}/>
                </linearGradient>
            </defs>
            <rect width={w} height={h} fill={INK}/>
            <g opacity={art_opacity} transform={frame.animate(fframes::timeline!(at 0.0 => 6.3, animate Transform { scale: 1.0.into(), ..Default::default() } => Transform { translate_x: -24., translate_y: -18., scale: 1.035.into(), ..Default::default() }, Easing::EaseInOut))}>
                <image href={art.href()} width={w} height={h} preserveAspectRatio="xMidYMid slice"/>
            </g>
            <rect width={w} height={h} fill="url(#shade)"/>
            <g opacity={frame.animate(fframes::timeline!(at 0.0 => 0.45, animate 0.85_f32 => 1.0, Easing::EaseOut))}>{title}</g>
            {content}
        </svg>)
    }
}

#[derive(Debug)]
pub struct Campaign<const W: usize, const H: usize> {
    shots: Vec<Shot>,
}

impl<const W: usize, const H: usize> Campaign<W, H> {
    pub fn new() -> Result<Self, String> {
        let tape = Arc::new(Tape::load()?);
        Ok(Self {
            shots: [Kind::Keeper, Kind::Carve, Kind::Descend, Kind::Invitation]
                .into_iter()
                .map(|kind| Shot {
                    kind,
                    width: W as f32,
                    height: H as f32,
                    tape: tape.clone(),
                })
                .collect(),
        })
    }
}
impl<const W: usize, const H: usize> Video for Campaign<W, H> {
    const FPS: usize = FPS;
    const WIDTH: usize = W;
    const HEIGHT: usize = H;
    const BACKGROUND_COLOR: Color = Color::BLACK;
    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }
    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }
    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(
            self.shots
                .iter()
                .map(|s| s as &dyn Scene)
                .collect::<Vec<_>>(),
        )
    }
    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={W} height={H}>{ctx.render_scenes(&frame)}</svg>)
    }
}
