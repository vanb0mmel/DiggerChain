pub mod expedition;
pub mod record;
mod scene;
pub mod vault;

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Void,
    Rock,
    Wall,
    Grass,
    Path,
    Marble,
    Mosaic,
    Wood,
    Crypt,
    Carpet,
    Water,
    DoorClosed,
    DoorOpen,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(
            self,
            Self::Void | Self::Rock | Self::Wall | Self::Water | Self::DoorClosed
        )
    }

    pub fn opaque(self) -> bool {
        matches!(
            self,
            Self::Void | Self::Rock | Self::Wall | Self::DoorClosed
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Void => "darkness",
            Self::Rock => "cave rock",
            Self::Wall => "castle wall",
            Self::Grass => "moss and grass",
            Self::Path => "worn stone path",
            Self::Marble => "marble floor",
            Self::Mosaic => "garden mosaic",
            Self::Wood => "walnut floor",
            Self::Crypt => "crypt flagstones",
            Self::Carpet => "crimson runner",
            Self::Water => "reflecting pool",
            Self::DoorClosed => "closed oak door",
            Self::DoorOpen => "open oak door",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crystal {
    Thorns,
    Silence,
    Echoes,
}

impl Crystal {
    pub fn name(self) -> &'static str {
        match self {
            Self::Thorns => "Emerald of Thorns",
            Self::Silence => "Sapphire of Silence",
            Self::Echoes => "Amethyst of Echoes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Tree,
    Fern,
    Flowers,
    Mushroom,
    Pillar,
    Window,
    Bookshelf,
    Coffin,
    Bench,
    Banner,
    Brazier,
    Sconce,
    Crystal(Crystal),
    Waystone,
    Companion,
    Flask,
    PortalUp,
    PortalDown,
    SoftVein,
    HardVein,
}

impl PropKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Tree => "silverleaf tree",
            Self::Fern => "moon fern",
            Self::Flowers => "velvet roses",
            Self::Mushroom => "glowing mushrooms",
            Self::Pillar => "carved pillar",
            Self::Window => "stained-glass window",
            Self::Bookshelf => "old bookshelves",
            Self::Coffin => "sealed sarcophagus",
            Self::Bench => "stone bench",
            Self::Banner => "faded royal banner",
            Self::Brazier => "brazier",
            Self::Sconce => "candle stand",
            Self::Crystal(crystal) => crystal.name(),
            Self::Waystone => "forest waystone",
            Self::Companion => "Moss, your fellow goblin",
            Self::Flask => "ember flask",
            Self::PortalUp => "ascending portal",
            Self::PortalDown => "descending portal",
            Self::SoftVein => "soft vein: 2 charge",
            Self::HardVein => "hard vein: 3 charge",
        }
    }

    pub fn blocks_movement(self) -> bool {
        matches!(
            self,
            Self::Tree | Self::Pillar | Self::Bookshelf | Self::Coffin | Self::Bench
        )
    }

    pub fn interactive(self) -> bool {
        matches!(
            self,
            Self::Brazier
                | Self::Sconce
                | Self::Crystal(_)
                | Self::Waystone
                | Self::Companion
                | Self::Bookshelf
                | Self::Coffin
        )
    }
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub at: Point,
    pub kind: PropKind,
    pub active: bool,
}

impl Prop {
    pub fn present(&self) -> bool {
        self.active || !matches!(self.kind, PropKind::Crystal(_))
    }
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub props: Vec<Prop>,
}

impl World {
    pub fn index(&self, p: Point) -> Option<usize> {
        (p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height)
            .then(|| (p.y * self.width + p.x) as usize)
    }

    pub fn tile(&self, p: Point) -> Tile {
        self.index(p).map_or(Tile::Void, |i| self.tiles[i])
    }

    pub fn set(&mut self, p: Point, tile: Tile) {
        if let Some(i) = self.index(p) {
            self.tiles[i] = tile;
        }
    }

    pub fn blocked(&self, p: Point) -> bool {
        !self.tile(p).walkable()
            || self
                .props
                .iter()
                .any(|prop| prop.at == p && prop.present() && prop.kind.blocks_movement())
    }

    pub fn line_of_sight(&self, from: Point, to: Point) -> bool {
        let mut p = from;
        let dx = (to.x - from.x).abs();
        let dy = -(to.y - from.y).abs();
        let sx = if from.x < to.x { 1 } else { -1 };
        let sy = if from.y < to.y { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            if p == to {
                return true;
            }
            if p != from && self.tile(p).opaque() {
                return false;
            }
            let twice = 2 * error;
            if twice >= dy {
                error += dy;
                p.x += sx;
            }
            if twice <= dx {
                error += dx;
                p.y += sy;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move(i32, i32),
    Interact,
    Wait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Moved,
    Blocked,
    Opened,
    Collected(Crystal),
    Interacted,
    Completed,
    Waited,
}

pub struct Game {
    pub world: World,
    pub player: Point,
    pub crystals: Vec<Crystal>,
    pub turns: u64,
    pub completed: bool,
    pub visible: Vec<bool>,
    pub explored: Vec<bool>,
    pub messages: VecDeque<String>,
    last_region: &'static str,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let world = scene::build();
        let mut game = Self::from_world(world, Point::new(12, 23));
        game.log("Pip: That cave has windows. Caves should not have windows.");
        game.log("Find three crystals. Bring them back to the forest waystone.");
        game
    }

    pub fn from_world(world: World, player: Point) -> Self {
        let len = world.tiles.len();
        let mut game = Self {
            world,
            player,
            crystals: Vec::new(),
            turns: 0,
            completed: false,
            visible: vec![false; len],
            explored: vec![false; len],
            messages: VecDeque::new(),
            last_region: "The Silverleaf Clearing",
        };
        game.update_visibility();
        game
    }

    pub fn region(&self) -> (&'static str, &'static str) {
        scene::region(self.player)
    }

    pub fn visible_at(&self, p: Point) -> bool {
        self.world.index(p).is_some_and(|i| self.visible[i])
    }

    pub fn explored_at(&self, p: Point) -> bool {
        self.world.index(p).is_some_and(|i| self.explored[i])
    }

    pub fn interaction_target(&self) -> Option<usize> {
        self.world
            .props
            .iter()
            .enumerate()
            .filter(|(_, prop)| {
                prop.present() && prop.kind.interactive() && prop.at.distance(self.player) <= 1
            })
            .min_by_key(|(_, prop)| prop.at.distance(self.player))
            .map(|(i, _)| i)
    }

    pub fn apply(&mut self, action: Action) -> Event {
        let event = match action {
            Action::Move(dx, dy) => {
                if !matches!((dx, dy), (-1, 0) | (1, 0) | (0, -1) | (0, 1)) {
                    return Event::Blocked;
                }
                let next = Point::new(self.player.x + dx, self.player.y + dy);
                if self.world.tile(next) == Tile::DoorClosed {
                    self.world.set(next, Tile::DoorOpen);
                    self.log("The old door yields. Warm dust drifts through the frame.");
                    Event::Opened
                } else if self.world.blocked(next) {
                    return Event::Blocked;
                } else {
                    self.player = next;
                    self.collect_here().unwrap_or(Event::Moved)
                }
            }
            Action::Interact => self.interact(),
            Action::Wait => Event::Waited,
        };
        self.turns += 1;
        let (region, description) = self.region();
        if region != self.last_region {
            self.last_region = region;
            self.log(description);
        }
        self.update_visibility();
        event
    }

    fn collect_here(&mut self) -> Option<Event> {
        let index = self.world.props.iter().position(|prop| {
            prop.at == self.player && prop.active && matches!(prop.kind, PropKind::Crystal(_))
        })?;
        self.collect(index)
    }

    fn collect(&mut self, index: usize) -> Option<Event> {
        let PropKind::Crystal(crystal) = self.world.props[index].kind else {
            return None;
        };
        self.world.props[index].active = false;
        self.crystals.push(crystal);
        self.log(format!(
            "Found the {}. Its song settles into your satchel.",
            crystal.name()
        ));
        if self.crystals.len() == 3 {
            self.log("Three voices, one chord. Return to the waystone in the clearing.");
        }
        Some(Event::Collected(crystal))
    }

    fn interact(&mut self) -> Event {
        let Some(index) = self.interaction_target() else {
            self.log("Pip listens. Somewhere in the stone, a crystal is singing.");
            return Event::Interacted;
        };
        let prop = &self.world.props[index];
        match prop.kind {
            PropKind::Crystal(_) => self.collect(index).unwrap_or(Event::Interacted),
            PropKind::Waystone if self.crystals.len() == 3 => {
                if self.completed {
                    return Event::Interacted;
                }
                self.completed = true;
                self.log(
                    "The waystone blooms with light. For a moment, the forest remembers the stars.",
                );
                self.log("Moss: Three shiny rocks. One impossible castle. A very good evening.");
                Event::Completed
            }
            PropKind::Waystone => {
                self.log(format!(
                    "The waystone has three empty settings. Crystals found: {}/3.",
                    self.crystals.len()
                ));
                Event::Interacted
            }
            PropKind::Brazier | PropKind::Sconce => {
                let active = !prop.active;
                self.world.props[index].active = active;
                self.log(if active {
                    "A small flame wakes. The shadows make room."
                } else {
                    "You snuff the flame. Blue shadows return."
                });
                Event::Interacted
            }
            PropKind::Companion => {
                self.log("Moss: I will guard the way back. You investigate the suspiciously expensive cave.");
                Event::Interacted
            }
            PropKind::Bookshelf => {
                self.log("Every book has the same title: A Guest Who Never Left. You put it back.");
                Event::Interacted
            }
            PropKind::Coffin => {
                self.log("The inscription reads: LET THE GARDENER SLEEP. Something smells faintly of roses.");
                Event::Interacted
            }
            _ => Event::Interacted,
        }
    }

    pub fn update_visibility(&mut self) {
        self.visible.fill(false);
        for y in (self.player.y - 15).max(0)..=(self.player.y + 15).min(self.world.height - 1) {
            for x in (self.player.x - 15).max(0)..=(self.player.x + 15).min(self.world.width - 1) {
                let p = Point::new(x, y);
                let dx = x - self.player.x;
                let dy = y - self.player.y;
                if dx * dx + dy * dy <= 225 && self.world.line_of_sight(self.player, p) {
                    let i = self
                        .world
                        .index(p)
                        .expect("visibility coordinates are in bounds");
                    self.visible[i] = true;
                    self.explored[i] = true;
                }
            }
        }
    }

    fn log(&mut self, text: impl Into<String>) {
        self.messages.push_back(text.into());
        if self.messages.len() > 8 {
            self.messages.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(game: &Game, goal: Point) -> Vec<Point> {
        let mut queue = VecDeque::from([game.player]);
        let mut previous = vec![None; game.world.tiles.len()];
        previous[game.world.index(game.player).unwrap()] = Some(game.player);
        while let Some(p) = queue.pop_front() {
            if p == goal {
                let mut path = vec![goal];
                while *path.last().unwrap() != game.player {
                    path.push(previous[game.world.index(*path.last().unwrap()).unwrap()].unwrap());
                }
                path.reverse();
                return path;
            }
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let next = Point::new(p.x + dx, p.y + dy);
                let Some(i) = game.world.index(next) else {
                    continue;
                };
                if previous[i].is_none()
                    && (!game.world.blocked(next) || game.world.tile(next) == Tile::DoorClosed)
                {
                    previous[i] = Some(p);
                    queue.push_back(next);
                }
            }
        }
        panic!("authored objective is unreachable: {goal:?}");
    }

    #[test]
    fn the_pilot_can_be_completed_through_normal_actions() {
        let mut game = Game::new();
        let mut goals: Vec<_> = game
            .world
            .props
            .iter()
            .filter(|p| matches!(p.kind, PropKind::Crystal(_)))
            .map(|p| p.at)
            .collect();
        goals.push(
            game.world
                .props
                .iter()
                .find(|p| p.kind == PropKind::Waystone)
                .unwrap()
                .at,
        );
        for goal in goals {
            for next in route(&game, goal).into_iter().skip(1) {
                let action = Action::Move(next.x - game.player.x, next.y - game.player.y);
                if game.apply(action) == Event::Opened {
                    game.apply(action);
                }
                assert_eq!(game.player, next);
            }
        }
        assert_eq!(game.crystals.len(), 3);
        assert_eq!(game.apply(Action::Interact), Event::Completed);
        assert!(game.completed);
        game.apply(Action::Interact);
        assert_eq!(game.crystals.len(), 3);
    }

    #[test]
    fn invalid_moves_do_not_advance_the_world() {
        let mut game = Game::new();
        let start = game.player;
        for action in [
            Action::Move(0, 0),
            Action::Move(10, 0),
            Action::Move(1, 1),
            Action::Move(i32::MIN, 0),
        ] {
            assert_eq!(game.apply(action), Event::Blocked);
        }
        game.world.set(Point::new(start.x + 1, start.y), Tile::Wall);
        assert_eq!(game.apply(Action::Move(1, 0)), Event::Blocked);
        assert_eq!(game.player, start);
        assert_eq!(game.turns, 0);
    }

    #[test]
    fn doors_block_visibility_until_opened() {
        let mut game = Game::new();
        let door = Point::new(game.player.x + 1, game.player.y);
        let beyond = Point::new(game.player.x + 2, game.player.y);
        game.world.set(door, Tile::DoorClosed);
        assert!(!game.world.line_of_sight(game.player, beyond));
        assert_eq!(game.apply(Action::Move(1, 0)), Event::Opened);
        assert_eq!(game.player, Point::new(12, 23));
        assert!(game.world.line_of_sight(game.player, beyond));
    }

    #[test]
    fn crystals_are_collected_only_once() {
        let mut game = Game::new();
        let p = Point::new(game.player.x + 1, game.player.y);
        game.world.props.push(Prop {
            at: p,
            kind: PropKind::Crystal(Crystal::Thorns),
            active: true,
        });
        assert_eq!(
            game.apply(Action::Move(1, 0)),
            Event::Collected(Crystal::Thorns)
        );
        game.apply(Action::Move(-1, 0));
        game.apply(Action::Move(1, 0));
        assert_eq!(game.crystals, [Crystal::Thorns]);
    }
}
