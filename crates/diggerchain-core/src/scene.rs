use crate::{Crystal, Point, Prop, PropKind, Tile, World};

pub fn build() -> World {
    let mut world = World {
        width: 86,
        height: 46,
        tiles: vec![Tile::Void; 86 * 46],
        props: Vec::new(),
    };
    fill(&mut world, (2, 8, 31, 38), Tile::Grass);
    fill(&mut world, (26, 17, 36, 29), Tile::Rock);
    room(&mut world, (34, 19, 52, 30), Tile::Marble);
    room(&mut world, (52, 12, 83, 35), Tile::Marble);
    room(&mut world, (34, 2, 51, 17), Tile::Mosaic);
    room(&mut world, (54, 2, 81, 11), Tile::Wood);
    room(&mut world, (36, 33, 53, 44), Tile::Crypt);

    fill(&mut world, (10, 22, 55, 25), Tile::Path);
    fill(&mut world, (42, 16, 45, 20), Tile::Marble);
    fill(&mut world, (65, 10, 68, 13), Tile::Marble);
    fill(&mut world, (42, 29, 45, 34), Tile::Crypt);
    fill(&mut world, (55, 22, 81, 25), Tile::Carpet);
    fill(&mut world, (13, 30, 21, 35), Tile::Water);
    fill(&mut world, (40, 8, 46, 12), Tile::Water);
    for (x, y) in [
        (34, 22),
        (34, 24),
        (52, 22),
        (52, 24),
        (42, 16),
        (44, 16),
        (65, 10),
        (67, 10),
        (42, 33),
        (44, 33),
    ] {
        world.set(Point::new(x, y), Tile::Wall);
    }
    for p in [
        Point::new(34, 23),
        Point::new(52, 23),
        Point::new(43, 16),
        Point::new(66, 10),
        Point::new(43, 33),
    ] {
        world.set(p, Tile::DoorClosed);
    }

    for (x, y) in [
        (3, 9),
        (7, 9),
        (12, 10),
        (17, 8),
        (23, 9),
        (28, 10),
        (3, 14),
        (8, 14),
        (20, 14),
        (25, 15),
        (4, 19),
        (8, 18),
        (24, 19),
        (29, 19),
        (3, 26),
        (7, 28),
        (25, 28),
        (28, 31),
        (3, 34),
        (8, 35),
        (11, 37),
        (23, 37),
        (29, 36),
    ] {
        add(&mut world, x, y, PropKind::Tree);
    }
    for (x, y) in [
        (6, 12),
        (15, 13),
        (22, 17),
        (6, 30),
        (24, 33),
        (30, 26),
        (36, 5),
        (48, 13),
    ] {
        add(&mut world, x, y, PropKind::Fern);
    }
    for (x, y) in [
        (11, 18),
        (19, 27),
        (9, 33),
        (36, 6),
        (37, 6),
        (47, 6),
        (48, 6),
        (37, 13),
        (47, 13),
    ] {
        add(&mut world, x, y, PropKind::Flowers);
    }
    for (x, y) in [(24, 21), (29, 25), (5, 31), (40, 41), (50, 41)] {
        add(&mut world, x, y, PropKind::Mushroom);
    }
    for (x, y) in [(57, 17), (65, 17), (74, 17), (57, 29), (65, 29), (74, 29)] {
        add(&mut world, x, y, PropKind::Pillar);
    }
    for (x, y) in [
        (57, 12),
        (65, 12),
        (74, 12),
        (82, 17),
        (82, 29),
        (38, 2),
        (46, 2),
        (59, 2),
        (68, 2),
        (76, 2),
    ] {
        add(&mut world, x, y, PropKind::Window);
    }
    for (x, y) in [
        (57, 5),
        (58, 5),
        (61, 5),
        (62, 5),
        (68, 7),
        (69, 7),
        (72, 7),
        (73, 7),
        (77, 5),
        (78, 5),
    ] {
        add(&mut world, x, y, PropKind::Bookshelf);
    }
    for (x, y) in [(39, 36), (48, 36), (39, 40), (49, 40)] {
        add(&mut world, x, y, PropKind::Coffin);
    }
    for (x, y) in [(37, 10), (48, 10), (60, 20), (70, 20), (60, 26), (70, 26)] {
        add(&mut world, x, y, PropKind::Bench);
    }
    for (x, y) in [(53, 16), (53, 30), (80, 16), (80, 30)] {
        add(&mut world, x, y, PropKind::Banner);
    }
    for (x, y) in [
        (29, 21),
        (36, 21),
        (49, 27),
        (55, 20),
        (55, 26),
        (79, 20),
        (79, 26),
        (38, 34),
        (50, 42),
    ] {
        add(&mut world, x, y, PropKind::Brazier);
    }
    for (x, y) in [(37, 15), (48, 4), (64, 4), (76, 9), (44, 35)] {
        add(&mut world, x, y, PropKind::Sconce);
    }
    add(&mut world, 44, 5, PropKind::Crystal(Crystal::Thorns));
    add(&mut world, 74, 5, PropKind::Crystal(Crystal::Silence));
    add(&mut world, 45, 40, PropKind::Crystal(Crystal::Echoes));
    add(&mut world, 8, 23, PropKind::Waystone);
    add(&mut world, 14, 25, PropKind::Companion);
    world
}

fn fill(world: &mut World, (left, top, right, bottom): (i32, i32, i32, i32), tile: Tile) {
    for y in top..bottom {
        for x in left..right {
            world.set(Point::new(x, y), tile);
        }
    }
}

fn room(world: &mut World, rect: (i32, i32, i32, i32), floor: Tile) {
    let (left, top, right, bottom) = rect;
    fill(world, rect, Tile::Wall);
    fill(world, (left + 1, top + 1, right - 1, bottom - 1), floor);
}

fn add(world: &mut World, x: i32, y: i32, kind: PropKind) {
    world.props.push(Prop {
        at: Point::new(x, y),
        kind,
        active: true,
    });
}

pub fn region(p: Point) -> (&'static str, &'static str) {
    match (p.x, p.y) {
        (34..=51, 2..=16) => (
            "The Moon Conservatory",
            "Moonlight pools between roses. The garden has never seen the sky.",
        ),
        (54..=80, 2..=11) => (
            "The Hushed Archive",
            "Velvet dust. Locked books. A blue note trembling between the shelves.",
        ),
        (36..=51, 31..=44) => (
            "The Gardener's Rest",
            "Pale roots cradle the tombs. Something beautiful grew down here.",
        ),
        (52..=84, 12..=35) => (
            "The Velvet Nave",
            "Gold catches the candlelight. This hall is larger than the hill outside.",
        ),
        (34..=51, 17..=30) => (
            "The Gloaming Threshold",
            "The cave becomes a vestibule. Someone has polished the impossible marble.",
        ),
        (26..=33, _) => (
            "The Candlelit Seam",
            "Between the roots: cut stone, a brass hinge, and the smell of candle wax.",
        ),
        _ => (
            "The Silverleaf Clearing",
            "Silver leaves keep their own moonlight. The cave waits beyond the moss.",
        ),
    }
}
