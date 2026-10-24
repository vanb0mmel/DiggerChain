//! Deterministic bounded castle simulation shared by human play, agents, and proofs.

use std::collections::VecDeque;

use crate::Point;
pub use crate::expedition::{Action, Direction, Error, Event};

pub const RULES_VERSION: u64 = 2;
pub const WIDTH: usize = 19;
pub const HEIGHT: usize = 13;
pub const FLOORS: usize = 3;
pub const MAX_ACTIONS: usize = 512;
pub const CAPACITY: u8 = 12;
pub const INITIAL_CHARGE: u8 = 8;
pub const REFILL: u8 = 8;
pub const ENTRY: Point = Point::new(2, 2);
pub const EXIT: Point = Point::new(2, 10);
pub const ROOMS: [Point; 6] = [
    Point::new(3, 3),
    Point::new(9, 3),
    Point::new(15, 3),
    Point::new(15, 9),
    Point::new(9, 9),
    Point::new(3, 9),
];
pub const GATES: [Point; 7] = [
    Point::new(6, 3),
    Point::new(12, 3),
    Point::new(15, 6),
    Point::new(12, 9),
    Point::new(6, 9),
    Point::new(3, 6),
    Point::new(9, 6),
];
pub const CRYSTAL_ROOMS: [usize; 3] = [1, 2, 4];
pub const OFFSETS: [i32; 4] = [-2, -1, 1, 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Ground,
    SoftRock,
    HardRock,
    Crystal,
    Flask,
    Up,
    Down,
}

impl Cell {
    pub fn glyph(self) -> char {
        match self {
            Self::Wall => '#',
            Self::Ground => '.',
            Self::SoftRock => 'r',
            Self::HardRock => 'R',
            Self::Crystal => 'C',
            Self::Flask => 'F',
            Self::Up => '<',
            Self::Down => '>',
        }
    }

    pub fn dig_cost(self) -> Option<u8> {
        match self {
            Self::SoftRock => Some(2),
            Self::HardRock => Some(3),
            _ => None,
        }
    }

    pub fn walkable(self) -> bool {
        !matches!(self, Self::Wall | Self::SoftRock | Self::HardRock)
    }
}

pub fn seed_bit(seed: u64, index: usize) -> bool {
    (seed >> index) & 1 != 0
}

pub fn crystal_at(seed: u64, floor: usize, slot: usize) -> Point {
    let shift = 27 + floor * 12 + slot * 4;
    let center = ROOMS[CRYSTAL_ROOMS[slot]];
    Point::new(
        center.x + OFFSETS[((seed >> shift) & 3) as usize],
        center.y + OFFSETS[((seed >> (shift + 2)) & 3) as usize],
    )
}

pub fn crystal_present(seed: u64, floor: usize, slot: usize) -> bool {
    match (floor, slot) {
        (0, _) | (_, 0) => true,
        (1, slot) => seed_bit(seed, 7 + slot),
        (_, slot) => seed_bit(seed, 14 + slot * 2) && seed_bit(seed, 15 + slot * 2),
    }
}

pub fn crystal_value(floor: usize) -> u32 {
    1 << (2 * floor)
}

pub fn gate_present(seed: u64, floor: usize, gate: usize) -> bool {
    gate < 5 || seed_bit(seed, 21 + floor * 2 + gate - 5)
}

pub fn room_ground(at: Point) -> bool {
    (1..=17).contains(&at.x) && at.x != 6 && at.x != 12 && (1..=11).contains(&at.y) && at.y != 6
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expedition {
    seed: u64,
    floor: usize,
    position: Point,
    charge: u8,
    score: u32,
    turns: usize,
    floors: [[Cell; WIDTH * HEIGHT]; FLOORS],
}

impl Expedition {
    pub fn new(seed: u64) -> Self {
        let mut floors = [[Cell::Wall; WIDTH * HEIGHT]; FLOORS];
        for (floor, cells) in floors.iter_mut().enumerate() {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if room_ground(Point::new(x as i32, y as i32)) {
                        cells[y * WIDTH + x] = Cell::Ground;
                    }
                }
            }
            for (gate, at) in GATES.iter().enumerate() {
                if gate_present(seed, floor, gate) {
                    cells[index(*at).unwrap()] = if seed_bit(seed, floor * 7 + gate) {
                        Cell::HardRock
                    } else {
                        Cell::SoftRock
                    };
                }
            }
            for (room, at) in ROOMS.iter().enumerate() {
                if room != 0 || !seed_bit(seed, 63) {
                    cells[index(*at).unwrap()] = Cell::Flask;
                }
            }
            for slot in 0..CRYSTAL_ROOMS.len() {
                if crystal_present(seed, floor, slot) {
                    cells[index(crystal_at(seed, floor, slot)).unwrap()] = Cell::Crystal;
                }
            }
            if floor > 0 {
                cells[index(ENTRY).unwrap()] = Cell::Up;
            }
            if floor + 1 < FLOORS {
                cells[index(EXIT).unwrap()] = Cell::Down;
            }
        }
        Self {
            seed,
            floor: 0,
            position: ENTRY,
            charge: INITIAL_CHARGE,
            score: 0,
            turns: 0,
            floors,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn floor(&self) -> usize {
        self.floor
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn charge(&self) -> u8 {
        self.charge
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn turns(&self) -> usize {
        self.turns
    }

    pub fn floors(&self) -> &[[Cell; WIDTH * HEIGHT]; FLOORS] {
        &self.floors
    }

    pub fn cell(&self, at: Point) -> Cell {
        index(at).map_or(Cell::Wall, |i| self.floors[self.floor][i])
    }

    pub fn apply(&mut self, action: Action) -> Result<Event, Error> {
        if self.turns >= MAX_ACTIONS {
            return Err(Error::TurnLimit);
        }
        let event = match action {
            Action::Move(direction) | Action::Dig(direction) => {
                let (dx, dy) = direction.delta();
                let next = Point::new(self.position.x + dx, self.position.y + dy);
                let target = index(next).ok_or(Error::Blocked)?;
                let cell = self.floors[self.floor][target];
                if matches!(action, Action::Dig(_)) {
                    let cost = cell.dig_cost().ok_or(Error::NotRock)?;
                    if self.charge < cost {
                        return Err(Error::InsufficientCharge);
                    }
                    self.charge -= cost;
                    self.floors[self.floor][target] = Cell::Ground;
                    Event::Dug { cost }
                } else {
                    if !cell.walkable() {
                        return Err(Error::Blocked);
                    }
                    self.position = next;
                    if cell == Cell::Crystal {
                        let value = crystal_value(self.floor);
                        self.score += value;
                        self.floors[self.floor][target] = Cell::Ground;
                        Event::Collected { value }
                    } else {
                        Event::Moved
                    }
                }
            }
            Action::Interact => {
                let at = index(self.position).expect("the player stays inside the floor");
                if self.floors[self.floor][at] != Cell::Flask {
                    return Err(Error::NoFlask);
                }
                if self.charge == CAPACITY {
                    return Err(Error::ChargeFull);
                }
                let before = self.charge;
                self.charge = self.charge.saturating_add(REFILL).min(CAPACITY);
                self.floors[self.floor][at] = Cell::Ground;
                Event::Replenished {
                    amount: self.charge - before,
                }
            }
            Action::Portal => {
                match self.cell(self.position) {
                    Cell::Up => {
                        self.floor -= 1;
                        self.position = EXIT;
                    }
                    Cell::Down => {
                        self.floor += 1;
                        self.position = ENTRY;
                    }
                    _ => return Err(Error::NoPortal),
                }
                Event::Travelled { floor: self.floor }
            }
            Action::Wait => Event::Waited,
        };
        self.turns += 1;
        Ok(event)
    }
}

fn index(at: Point) -> Option<usize> {
    (at.x >= 0 && at.y >= 0 && at.x < WIDTH as i32 && at.y < HEIGHT as i32)
        .then(|| at.y as usize * WIDTH + at.x as usize)
}

fn record(game: &mut Expedition, actions: &mut Vec<Action>, action: Action) -> Result<(), String> {
    game.apply(action).map_err(|error| error.to_string())?;
    actions.push(action);
    Ok(())
}

fn walk_to(game: &mut Expedition, actions: &mut Vec<Action>, goal: Point) -> Result<(), String> {
    let mut previous = [None; WIDTH * HEIGHT];
    let mut seen = [false; WIDTH * HEIGHT];
    let mut queue = VecDeque::from([game.position]);
    seen[index(game.position).unwrap()] = true;
    while let Some(at) = queue.pop_front() {
        if at == goal {
            break;
        }
        for direction in [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let (dx, dy) = direction.delta();
            let next = Point::new(at.x + dx, at.y + dy);
            let Some(i) = index(next) else { continue };
            if !seen[i] && game.cell(next) != Cell::Wall {
                seen[i] = true;
                previous[i] = Some((at, direction));
                queue.push_back(next);
            }
        }
    }
    if !seen[index(goal).ok_or("Invalid route target.")?] {
        return Err("The generated floor has no route to the target.".into());
    }
    let mut cursor = goal;
    let mut path = Vec::new();
    while cursor != game.position {
        let (from, direction) = previous[index(cursor).unwrap()].ok_or("Incomplete route.")?;
        path.push((cursor, direction));
        cursor = from;
    }
    for (next, direction) in path.into_iter().rev() {
        if game.cell(next).dig_cost().is_some() {
            record(game, actions, Action::Dig(direction))?;
        }
        record(game, actions, Action::Move(direction))?;
    }
    Ok(())
}

/// Visits every chamber, collects all generated crystals, and returns upstairs.
pub fn demonstration(seed: u64, max_steps: usize, fill: bool) -> Result<Vec<Action>, String> {
    if max_steps == 0 || max_steps > MAX_ACTIONS {
        return Err(format!(
            "The generated step bound must be between 1 and {MAX_ACTIONS}."
        ));
    }
    let mut game = Expedition::new(seed);
    let mut actions = Vec::new();
    for floor in 0..FLOORS {
        for (room, center) in ROOMS.iter().enumerate() {
            walk_to(&mut game, &mut actions, *center)?;
            if game.cell(*center) == Cell::Flask && game.charge < CAPACITY {
                record(&mut game, &mut actions, Action::Interact)?;
            }
            for (slot, crystal_room) in CRYSTAL_ROOMS.iter().enumerate() {
                let target = crystal_at(seed, floor, slot);
                if *crystal_room == room && game.cell(target) == Cell::Crystal {
                    walk_to(&mut game, &mut actions, target)?;
                }
            }
        }
        walk_to(&mut game, &mut actions, EXIT)?;
        if floor + 1 < FLOORS {
            record(&mut game, &mut actions, Action::Portal)?;
        }
    }
    while game.floor > 0 {
        walk_to(&mut game, &mut actions, ENTRY)?;
        record(&mut game, &mut actions, Action::Portal)?;
    }
    walk_to(&mut game, &mut actions, ENTRY)?;
    if actions.len() > max_steps {
        return Err(format!(
            "The {}-action route exceeds the {max_steps}-step bound.",
            actions.len()
        ));
    }
    if fill {
        while actions.len() + 1 < max_steps {
            record(&mut game, &mut actions, Action::Move(Direction::North))?;
            record(&mut game, &mut actions, Action::Move(Direction::South))?;
        }
        if actions.len() < max_steps {
            record(&mut game, &mut actions, Action::Wait)?;
        }
    }
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_routes_revisit_every_floor_without_resetting_items() {
        for seed in [0, 1, 42, 0x1234_5678_9abc_def0, u64::MAX] {
            let initial = Expedition::new(seed);
            assert_eq!(initial, Expedition::new(seed));
            let expected: u32 = initial
                .floors
                .iter()
                .enumerate()
                .map(|(floor, cells)| {
                    cells.iter().filter(|cell| **cell == Cell::Crystal).count() as u32
                        * crystal_value(floor)
                })
                .sum();
            let mut game = initial;
            for action in demonstration(seed, MAX_ACTIONS, true).unwrap() {
                game.apply(action).unwrap();
            }
            assert_eq!(game.floor(), 0);
            assert_eq!(game.position(), ENTRY);
            assert_eq!(game.turns(), MAX_ACTIONS);
            assert_eq!(game.score(), expected);
            assert!(
                game.floors
                    .iter()
                    .flatten()
                    .all(|cell| *cell != Cell::Crystal)
            );
            let before = game.clone();
            assert_eq!(game.apply(Action::Wait), Err(Error::TurnLimit));
            assert_eq!(game, before);
        }
        assert_ne!(Expedition::new(0).floors, Expedition::new(u64::MAX).floors);
    }

    #[test]
    fn blocked_actions_and_insufficient_charge_are_atomic() {
        let mut game = Expedition::new(0);
        for action in [
            Action::Portal,
            Action::Interact,
            Action::Dig(Direction::North),
        ] {
            let before = game.clone();
            assert!(game.apply(action).is_err());
            assert_eq!(game, before);
        }
        game.position = Point::new(5, 3);
        game.charge = 0;
        let before = game.clone();
        assert_eq!(
            game.apply(Action::Move(Direction::East)),
            Err(Error::Blocked)
        );
        assert_eq!(
            game.apply(Action::Dig(Direction::East)),
            Err(Error::InsufficientCharge)
        );
        assert_eq!(game, before);
        assert_eq!(game.apply(Action::Move(Direction::West)), Ok(Event::Moved));
    }

    #[test]
    fn portals_work_without_charge_and_full_flasks_stay_available() {
        let mut game = Expedition::new(0);
        game.position = ROOMS[0];
        game.charge = CAPACITY;
        let before = game.clone();
        assert_eq!(game.apply(Action::Interact), Err(Error::ChargeFull));
        assert_eq!(game, before);
        game.charge = 10;
        assert_eq!(
            game.apply(Action::Interact),
            Ok(Event::Replenished { amount: 2 })
        );
        assert_eq!(game.apply(Action::Interact), Err(Error::NoFlask));
        game.position = EXIT;
        game.charge = 0;
        assert_eq!(
            game.apply(Action::Portal),
            Ok(Event::Travelled { floor: 1 })
        );
        assert_eq!(game.position(), ENTRY);
        assert_eq!(
            game.apply(Action::Portal),
            Ok(Event::Travelled { floor: 0 })
        );
        assert_eq!(game.position(), EXIT);
        assert_eq!(game.charge(), 0);
    }
}
