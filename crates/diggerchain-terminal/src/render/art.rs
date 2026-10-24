use diggerchain_core::{Crystal, Point, PropKind, Tile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn scale(self, factor: f32) -> Self {
        Self(
            (self.0 as f32 * factor).clamp(0.0, 255.0) as u8,
            (self.1 as f32 * factor).clamp(0.0, 255.0) as u8,
            (self.2 as f32 * factor).clamp(0.0, 255.0) as u8,
        )
    }

    pub fn add(self, other: Self) -> Self {
        Self(
            self.0.saturating_add(other.0),
            self.1.saturating_add(other.1),
            self.2.saturating_add(other.2),
        )
    }

    pub fn mix(self, other: Self, amount: f32) -> Self {
        let t = amount.clamp(0.0, 1.0);
        Self(
            (self.0 as f32 * (1.0 - t) + other.0 as f32 * t) as u8,
            (self.1 as f32 * (1.0 - t) + other.1 as f32 * t) as u8,
            (self.2 as f32 * (1.0 - t) + other.2 as f32 * t) as u8,
        )
    }
}

pub const VOID: Rgb = Rgb(5, 7, 13);
pub const PANEL: Rgb = Rgb(11, 14, 23);
pub const TEXT: Rgb = Rgb(204, 204, 211);
pub const MUTED: Rgb = Rgb(123, 132, 150);
pub const BORDER: Rgb = Rgb(63, 71, 91);
pub const GOLD: Rgb = Rgb(222, 186, 116);
pub const GREEN: Rgb = Rgb(130, 214, 147);
pub const BLUE: Rgb = Rgb(122, 190, 236);
pub const HUMAN_COLOR: Rgb = Rgb(176, 194, 215);

pub type Sprite = &'static [&'static str; 8];

pub const GOBLIN: [&str; 8] = [
    " b    b ", "bbbhhbbb", " obbbbo ", " obewbo ", "  bbbb  ", " orrrro ", " odbbdo ", "  o  o  ",
];
pub const HUMAN: [&str; 8] = [
    "  tttt  ", "  tsssoa", "  sosoat", " orrrrot", " bbbbd t", " obbbott", "  td dt ", "  oo oo ",
];
const TREE: [&str; 8] = [
    "   hh   ", " bhhhh  ", "bbhhbbdb", " bhbbbdb", "bbbbbddb", "  bttb  ", "   tt   ", "  tttt  ",
];
const FERN: [&str; 8] = [
    "        ", " h    h ", "  h  h  ", "b hbh b ", " bbbbb  ", "  bbb   ", "   t    ", "  dt    ",
];
const FLOWERS: [&str; 8] = [
    "        ", "  p  r  ", " php rhr", "  p  r  ", "  b  b  ", " bbbb b ", "  bbb   ", "   b    ",
];
const MUSHROOM: [&str; 8] = [
    "        ", "  bhb   ", " bhhhb  ", "bbhbbdb ", "   w    ", "  bw  bb", "  db bbh", "     dd ",
];
const PILLAR: [&str; 8] = [
    " bbbbb  ", " hhhhh  ", "  hbdd  ", "  hbdd  ", "  hbdd  ", "  hbdd  ", " bhbbdd ", " dddddd ",
];
const WINDOW: [&str; 8] = [
    "   oo   ", "  oaco  ", " ocapco ", "occapcco", "oaacoaco", "occapcco", "occapcco", "oooooooo",
];
const BOOKS: [&str; 8] = [
    "tttttttt", "tracpptt", "tracpptt", "tttttttt", "tcpraact", "tcpraact", "tttttttt", "dddddddd",
];
const COFFIN: [&str; 8] = [
    "  bbbb  ", " bhhhdb ", " bhahdb ", " bhaadb ", " bhahdb ", " bhhhdb ", "  bddb  ", "  dddd  ",
];
const BENCH: [&str; 8] = [
    "        ", "        ", " hhhhhh ", " bbbbbb ", " dddddd ", "  d  d  ", "  d  d  ", "        ",
];
const BANNER: [&str; 8] = [
    " aaaaaa ", " obbbbo ", " obabdo ", " obabdo ", " obaado ", " obdddo ", "  bddo  ", "   do   ",
];
const FIRE: [&str; 8] = [
    "   h    ", "  aha   ", " ahhha  ", "  ahra  ", " bbbbb  ", "  ddd   ", "   d    ", "  ddd   ",
];
const CANDLE: [&str; 8] = [
    "   h    ", "   a    ", "   w    ", "   w    ", "   w    ", "  aaa   ", "   t    ", "  ttt   ",
];
const CRYSTAL: [&str; 8] = [
    "   h    ", "  hhb   ", " hhbbd  ", "hhbbddd ", " hhbbd  ", "  bbdd  ", "   dd   ", "  oooo  ",
];
const WAYSTONE: [&str; 8] = [
    "  bbbb  ", " bhhhdb ", " bhccdb ", " bhccdb ", " bhccdb ", " bhhhdb ", "bbbbbbbb", " dddddd ",
];
const FLASK: [&str; 8] = [
    "   tt   ", "   ww   ", "   bb   ", "  bhhb  ", " bhahdb ", " bhaadb ", " bbbddb ", "  oooo  ",
];

pub fn sprite(kind: PropKind) -> Sprite {
    match kind {
        PropKind::Tree => &TREE,
        PropKind::Fern => &FERN,
        PropKind::Flowers => &FLOWERS,
        PropKind::Mushroom => &MUSHROOM,
        PropKind::Pillar => &PILLAR,
        PropKind::Window => &WINDOW,
        PropKind::Bookshelf => &BOOKS,
        PropKind::Coffin => &COFFIN,
        PropKind::Bench => &BENCH,
        PropKind::Banner => &BANNER,
        PropKind::Brazier => &FIRE,
        PropKind::Sconce => &CANDLE,
        PropKind::Crystal(_) => &CRYSTAL,
        PropKind::Waystone => &WAYSTONE,
        PropKind::Companion => &GOBLIN,
        PropKind::Flask => &FLASK,
        PropKind::PortalUp | PropKind::PortalDown => &WAYSTONE,
        PropKind::SoftVein | PropKind::HardVein => &CRYSTAL,
    }
}

pub fn crystal_color(crystal: Crystal) -> Rgb {
    match crystal {
        Crystal::Thorns => Rgb(87, 239, 167),
        Crystal::Silence => Rgb(109, 188, 255),
        Crystal::Echoes => Rgb(207, 139, 255),
    }
}

pub fn prop_color(kind: PropKind) -> Rgb {
    match kind {
        PropKind::Tree => Rgb(48, 103, 90),
        PropKind::Fern => Rgb(79, 143, 112),
        PropKind::Flowers => Rgb(97, 129, 91),
        PropKind::Mushroom => Rgb(110, 178, 202),
        PropKind::Pillar | PropKind::Coffin | PropKind::Bench => Rgb(105, 105, 124),
        PropKind::Window => Rgb(106, 158, 208),
        PropKind::Bookshelf => Rgb(103, 67, 48),
        PropKind::Banner => Rgb(138, 48, 77),
        PropKind::Brazier | PropKind::Sconce => Rgb(168, 118, 58),
        PropKind::Crystal(crystal) => crystal_color(crystal),
        PropKind::Waystone => Rgb(103, 137, 143),
        PropKind::Companion => Rgb(128, 181, 102),
        PropKind::Flask => Rgb(225, 153, 77),
        PropKind::PortalUp | PropKind::PortalDown => Rgb(156, 137, 235),
        PropKind::SoftVein => Rgb(167, 139, 114),
        PropKind::HardVein => Rgb(183, 166, 200),
    }
}

pub fn sprite_pixel(code: u8, base: Rgb) -> Option<Rgb> {
    Some(match code {
        b'o' => Rgb(10, 13, 23),
        b'd' => base.scale(0.55),
        b'b' => base,
        b'h' => base.mix(Rgb(245, 236, 204), 0.45),
        b'w' => Rgb(238, 232, 205),
        b'e' => Rgb(255, 199, 88),
        b'r' => Rgb(169, 64, 83),
        b'p' => Rgb(176, 112, 184),
        b'c' => Rgb(103, 188, 205),
        b'a' => Rgb(232, 171, 82),
        b't' => Rgb(90, 64, 52),
        b's' => Rgb(216, 166, 127),
        _ => return None,
    })
}

pub fn hash(x: i32, y: i32) -> u32 {
    let mut value = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    value = (value ^ (value >> 13)).wrapping_mul(1_274_126_177);
    value ^ (value >> 16)
}

pub fn terrain(tile: Tile, at: Point, x: usize, y: usize, time: f32) -> Rgb {
    let grain = hash(at.x * 8 + x as i32, at.y * 8 + y as i32);
    let fleck = (grain % 17) as f32 / 100.0;
    match tile {
        Tile::Void => VOID,
        Tile::Grass => {
            let base = if grain % 11 < 2 {
                Rgb(31, 59, 44)
            } else {
                Rgb(14, 34, 32)
            };
            base.scale(0.9 + fleck)
        }
        Tile::Path => {
            if (x + at.y as usize % 2 * 4).is_multiple_of(8) || y == 7 {
                Rgb(24, 29, 34)
            } else {
                Rgb(53, 51, 52).scale(0.85 + fleck)
            }
        }
        Tile::Rock => {
            let ledge = (x + y * 2 + at.x as usize) % 7;
            Rgb(42, 43, 57).scale(if ledge == 0 {
                0.6
            } else if ledge == 1 {
                1.15
            } else {
                0.9 + fleck
            })
        }
        Tile::Wall => {
            let base = Rgb(57, 49, 66);
            if y == 0 {
                Rgb(111, 94, 112)
            } else if y == 1 {
                Rgb(77, 67, 86)
            } else if y == 4 || (x + at.y as usize % 2 * 4).is_multiple_of(8) {
                base.scale(0.48)
            } else {
                base.scale(0.75 + fleck)
            }
        }
        Tile::Marble => {
            let base = if (at.x + at.y) % 2 == 0 {
                Rgb(44, 45, 62)
            } else {
                Rgb(30, 33, 47)
            };
            if x == 7 || y == 7 {
                base.scale(0.63)
            } else if (x + y * 2 + at.x as usize).is_multiple_of(13) {
                base.scale(1.3)
            } else {
                base
            }
        }
        Tile::Mosaic => {
            if (x == 1 || x == 6) && (y == 1 || y == 6) {
                Rgb(101, 111, 91)
            } else if x == 7 || y == 7 {
                Rgb(14, 34, 39)
            } else {
                Rgb(29, 62, 64).scale(0.9 + fleck)
            }
        }
        Tile::Wood => {
            if x == 0 || (y + at.x as usize * 3).is_multiple_of(8) {
                Rgb(26, 21, 29)
            } else {
                Rgb(66, 43, 42).scale(0.9 + fleck)
            }
        }
        Tile::Crypt => {
            if y == 0 || (x + at.y as usize * 2).is_multiple_of(8) {
                Rgb(19, 26, 39)
            } else {
                Rgb(47, 56, 71).scale(0.85 + fleck)
            }
        }
        Tile::Carpet => {
            if (at.y == 22 && y < 2) || (at.y == 24 && y > 5) {
                Rgb(150, 107, 68)
            } else if (x + y + at.x as usize).is_multiple_of(9) {
                Rgb(119, 48, 70)
            } else {
                Rgb(77, 27, 51)
            }
        }
        Tile::Water => {
            let phase = time * 1.5
                + (at.x * 8 + x as i32) as f32 * 0.7
                + (at.y * 8 + y as i32) as f32 * 1.1;
            let glint = phase.sin().powi(12) * 0.4;
            Rgb(15, 40, 58).mix(Rgb(66, 116, 140), glint)
        }
        Tile::DoorClosed => {
            if x == 1 && (y == 2 || y == 5) {
                GOLD
            } else if x.is_multiple_of(3) {
                Rgb(42, 27, 30)
            } else {
                Rgb(100, 65, 46)
            }
        }
        Tile::DoorOpen => {
            if x == 0 {
                Rgb(106, 76, 50)
            } else {
                Rgb(42, 40, 51)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sprite_is_an_eight_by_eight_mask() {
        for mask in [
            &GOBLIN, &HUMAN, &TREE, &FERN, &FLOWERS, &MUSHROOM, &PILLAR, &WINDOW, &BOOKS, &COFFIN,
            &BENCH, &BANNER, &FIRE, &CANDLE, &CRYSTAL, &WAYSTONE, &FLASK,
        ] {
            for row in mask {
                assert_eq!(row.len(), 8, "invalid sprite row: {row:?}");
            }
        }
    }
}
