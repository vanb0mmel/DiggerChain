//! Bounded gameplay fixture for proof feasibility; not the production generator.

use crate::Point;

pub const SIZE: usize = 7;
pub const CAPACITY: u8 = 12;
pub const REFILL: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
        }
    }

    pub fn mirrored(self) -> Self {
        match self {
            Self::West => Self::East,
            Self::East => Self::West,
            direction => direction,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Ground,
    SoftRock,
    HardRock,
    Crystal,
    Flask,
    Portal,
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
            Self::Portal => 'O',
        }
    }

    fn walkable(self) -> bool {
        !matches!(self, Self::Wall | Self::SoftRock | Self::HardRock)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move(Direction),
    Dig(Direction),
    Interact,
    Portal,
    Wait,
}

impl Action {
    pub fn code(self) -> u8 {
        match self {
            Self::Move(Direction::North) => 1,
            Self::Move(Direction::South) => 2,
            Self::Move(Direction::West) => 3,
            Self::Move(Direction::East) => 4,
            Self::Dig(Direction::North) => 5,
            Self::Dig(Direction::South) => 6,
            Self::Dig(Direction::West) => 7,
            Self::Dig(Direction::East) => 8,
            Self::Interact => 9,
            Self::Portal => 10,
            Self::Wait => 11,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::Move(Direction::North),
            2 => Self::Move(Direction::South),
            3 => Self::Move(Direction::West),
            4 => Self::Move(Direction::East),
            5 => Self::Dig(Direction::North),
            6 => Self::Dig(Direction::South),
            7 => Self::Dig(Direction::West),
            8 => Self::Dig(Direction::East),
            9 => Self::Interact,
            10 => Self::Portal,
            11 => Self::Wait,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self.code() {
            1 => "north",
            2 => "south",
            3 => "west",
            4 => "east",
            5 => "dig-north",
            6 => "dig-south",
            7 => "dig-west",
            8 => "dig-east",
            9 => "interact",
            10 => "portal",
            11 => "wait",
            _ => unreachable!(),
        }
    }

    pub fn parse(command: &str) -> Option<Self> {
        (1..=11)
            .filter_map(Self::from_code)
            .find(|action| action.name() == command)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Moved,
    Dug { cost: u8 },
    Collected { value: u32 },
    Replenished { amount: u8 },
    Travelled { floor: usize },
    Waited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Blocked,
    NotRock,
    InsufficientCharge,
    NoFlask,
    ChargeFull,
    NoPortal,
    TurnLimit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Blocked => "The destination is blocked.",
            Self::NotRock => "There is no mineable rock in that direction.",
            Self::InsufficientCharge => "There is not enough charge to dig that rock.",
            Self::NoFlask => "There is no unused flask under the player.",
            Self::ChargeFull => "Charge is full; the flask remains unused.",
            Self::NoPortal => "There is no portal under the player.",
            Self::TurnLimit => "This expedition has reached its action limit.",
        })
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expedition {
    seed: u32,
    floor: usize,
    position: Point,
    charge: u8,
    score: u32,
    turns: u32,
    floors: [[Cell; SIZE * SIZE]; 2],
}

impl Expedition {
    pub fn new(seed: u32) -> Self {
        let layouts = [
            [
                "#######", "#.rC..#", "###r###", "#..F..#", "###r###", "#..O..#", "#######",
            ],
            [
                "#######", "#Cr...#", "###r###", "#..F..#", "###R###", "#..O..#", "#######",
            ],
        ];
        let mut floors = [[Cell::Wall; SIZE * SIZE]; 2];
        for (floor, rows) in layouts.iter().enumerate() {
            for (y, row) in rows.iter().enumerate() {
                for (x, glyph) in row.chars().enumerate() {
                    let target_x = if seed & 1 == 1 { SIZE - 1 - x } else { x };
                    floors[floor][y * SIZE + target_x] = match glyph {
                        '.' => Cell::Ground,
                        'r' => Cell::SoftRock,
                        'R' => Cell::HardRock,
                        'C' => Cell::Crystal,
                        'F' => Cell::Flask,
                        'O' => Cell::Portal,
                        _ => Cell::Wall,
                    };
                }
            }
        }
        Self {
            seed,
            floor: 0,
            position: Point::new(if seed & 1 == 1 { 5 } else { 1 }, 1),
            charge: 4,
            score: 0,
            turns: 0,
            floors,
        }
    }

    pub fn seed(&self) -> u32 {
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

    pub fn turns(&self) -> u32 {
        self.turns
    }

    pub fn floors(&self) -> &[[Cell; SIZE * SIZE]; 2] {
        &self.floors
    }

    fn index(at: Point) -> Option<usize> {
        (at.x >= 0 && at.y >= 0 && at.x < SIZE as i32 && at.y < SIZE as i32)
            .then(|| at.y as usize * SIZE + at.x as usize)
    }

    pub fn apply(&mut self, action: Action) -> Result<Event, Error> {
        if self.turns == u32::MAX {
            return Err(Error::TurnLimit);
        }
        let event = match action {
            Action::Move(direction) | Action::Dig(direction) => {
                let (dx, dy) = direction.delta();
                let next = Point::new(self.position.x + dx, self.position.y + dy);
                let index = Self::index(next).ok_or(Error::Blocked)?;
                let cell = self.floors[self.floor][index];
                if matches!(action, Action::Dig(_)) {
                    let cost = match cell {
                        Cell::SoftRock => 2,
                        Cell::HardRock => 3,
                        _ => return Err(Error::NotRock),
                    };
                    if self.charge < cost {
                        return Err(Error::InsufficientCharge);
                    }
                    self.charge -= cost;
                    self.floors[self.floor][index] = Cell::Ground;
                    Event::Dug { cost }
                } else {
                    if !cell.walkable() {
                        return Err(Error::Blocked);
                    }
                    self.position = next;
                    if cell == Cell::Crystal {
                        let value = if self.floor == 0 { 1 } else { 4 };
                        self.score += value;
                        self.floors[self.floor][index] = Cell::Ground;
                        Event::Collected { value }
                    } else {
                        Event::Moved
                    }
                }
            }
            Action::Interact => {
                let index = Self::index(self.position).expect("fixture position stays in bounds");
                if self.floors[self.floor][index] != Cell::Flask {
                    return Err(Error::NoFlask);
                }
                if self.charge == CAPACITY {
                    return Err(Error::ChargeFull);
                }
                let before = self.charge;
                self.charge = self.charge.saturating_add(REFILL).min(CAPACITY);
                self.floors[self.floor][index] = Cell::Ground;
                Event::Replenished {
                    amount: self.charge - before,
                }
            }
            Action::Portal => {
                let index = Self::index(self.position).expect("fixture position stays in bounds");
                if self.floors[self.floor][index] != Cell::Portal {
                    return Err(Error::NoPortal);
                }
                self.floor = 1 - self.floor;
                self.position = Point::new(3, 5);
                Event::Travelled { floor: self.floor }
            }
            Action::Wait => Event::Waited,
        };
        self.turns += 1;
        Ok(event)
    }
}

/// Includes deliberate rejected actions to expose reset and duplicate-pickup errors.
pub fn demonstration(seed: u32) -> Vec<Action> {
    use Action::{Dig, Interact, Move, Portal};
    use Direction::{East, North, South, West};

    let route = [
        Dig(East),
        Move(East),
        Move(East),
        Dig(South),
        Move(South),
        Move(South),
        Interact,
        Dig(South),
        Move(South),
        Move(South),
        Portal,
        Dig(North),
        Move(North),
        Move(North),
        Interact,
        Dig(North),
        Move(North),
        Move(North),
        Dig(West),
        Move(West),
        Move(West),
        Move(East),
        Move(East),
        Move(South),
        Move(South),
        Interact,
        Move(South),
        Move(South),
        Portal,
        Move(North),
        Dig(North),
        Move(North),
        Interact,
        Move(North),
        Move(North),
        Move(West),
        Move(West),
        Move(East),
        Move(East),
    ];
    route
        .into_iter()
        .map(|action| {
            if seed & 1 == 0 {
                return action;
            }
            match action {
                Move(direction) => Move(direction.mirrored()),
                Dig(direction) => Dig(direction.mirrored()),
                action => action,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_preserves_digging_and_collections_across_revisits() {
        for seed in [0, 1, 42, u32::MAX] {
            let mut expedition = Expedition::new(seed);
            let mut errors = Vec::new();
            for action in demonstration(seed) {
                let before = expedition.clone();
                if let Err(error) = expedition.apply(action) {
                    assert_eq!(expedition, before);
                    errors.push(error);
                }
            }
            assert_eq!(errors, [Error::NoFlask, Error::NotRock, Error::NoFlask]);
            assert_eq!(expedition.floor(), 0);
            assert_eq!(expedition.score(), 5);
            assert_eq!(expedition.charge(), 7);
            assert_eq!(expedition.turns(), 36);
            assert_eq!(expedition.position(), Point::new(3, 1));
        }
    }

    #[test]
    fn rejected_actions_are_atomic_and_wait_does_not_refill() {
        let mut expedition = Expedition::new(0);
        for action in [
            Action::Move(Direction::North),
            Action::Move(Direction::East),
            Action::Dig(Direction::West),
            Action::Interact,
            Action::Portal,
        ] {
            let before = expedition.clone();
            assert!(expedition.apply(action).is_err());
            assert_eq!(expedition, before);
        }
        expedition.apply(Action::Wait).unwrap();
        assert_eq!(expedition.charge(), 4);
        assert_eq!(expedition.turns(), 1);
    }

    #[test]
    fn insufficient_charge_does_not_break_rock() {
        let mut expedition = Expedition::new(0);
        expedition.charge = 1;
        let before = expedition.clone();
        assert_eq!(
            expedition.apply(Action::Dig(Direction::East)),
            Err(Error::InsufficientCharge)
        );
        assert_eq!(expedition, before);
    }

    #[test]
    fn empty_charge_allows_retreat_and_portal_travel() {
        let mut expedition = Expedition::new(0);
        expedition.position = Point::new(3, 5);
        expedition.charge = 0;
        let terrain = expedition.floors;
        assert_eq!(
            expedition.apply(Action::Move(Direction::West)),
            Ok(Event::Moved)
        );
        expedition.apply(Action::Move(Direction::East)).unwrap();
        assert_eq!(
            expedition.apply(Action::Portal),
            Ok(Event::Travelled { floor: 1 })
        );
        assert_eq!(
            expedition.apply(Action::Portal),
            Ok(Event::Travelled { floor: 0 })
        );
        assert_eq!(expedition.floors, terrain);
        assert_eq!(expedition.charge(), 0);
        assert_eq!(expedition.turns(), 4);
    }

    #[test]
    fn full_charge_keeps_flask_and_partial_refills_are_capped() {
        let mut expedition = Expedition::new(0);
        expedition.position = Point::new(3, 3);
        expedition.charge = CAPACITY;
        let before = expedition.clone();
        assert_eq!(expedition.apply(Action::Interact), Err(Error::ChargeFull));
        assert_eq!(expedition, before);
        expedition.charge = 10;
        assert_eq!(
            expedition.apply(Action::Interact),
            Ok(Event::Replenished { amount: 2 })
        );
        assert_eq!(expedition.charge(), CAPACITY);
        assert_eq!(expedition.apply(Action::Interact), Err(Error::NoFlask));
    }
}
