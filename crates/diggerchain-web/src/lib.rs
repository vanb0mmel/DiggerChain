//! A practice-only browser adapter over the native expedition simulation.

use diggerchain_core::vault::{self, Action, Cell, Event, Expedition};
use diggerchain_core::{Point, Tile, World};
use serde_json::json;
use wasm_bindgen::prelude::*;

const CELLS: usize = vault::WIDTH * vault::HEIGHT;

#[wasm_bindgen]
pub struct Demo {
    expedition: Expedition,
    explored: [[bool; CELLS]; vault::FLOORS],
    visible: [bool; CELLS],
    finished: bool,
    message: String,
}

#[wasm_bindgen]
impl Demo {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &str) -> Result<Demo, JsValue> {
        let seed = u64::from_str_radix(seed, 16)
            .map_err(|_| JsValue::from_str("The practice seed must be hexadecimal."))?;
        let mut demo = Self {
            expedition: Expedition::new(seed),
            explored: [[false; CELLS]; vault::FLOORS],
            visible: [false; CELLS],
            finished: false,
            message: "You step beneath the forest. Move onto a flask and press E to refill.".into(),
        };
        demo.refresh_visibility();
        Ok(demo)
    }

    pub fn state(&self) -> String {
        let state = &self.expedition;
        let floor = state.floor();
        let cells: Vec<_> = state.floors()[floor]
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                if self.visible[i] {
                    cell.glyph().to_string()
                } else if self.explored[floor][i] {
                    // Remember architecture, but do not reveal unseen items.
                    match cell {
                        Cell::Wall => "#",
                        Cell::SoftRock | Cell::HardRock => ":",
                        _ => ".",
                    }
                    .into()
                } else {
                    " ".into()
                }
            })
            .collect();
        json!({
            "width": vault::WIDTH,
            "height": vault::HEIGHT,
            "floor": floor + 1,
            "floors": vault::FLOORS,
            "position": {"x": state.position().x, "y": state.position().y},
            "charge": state.charge(),
            "capacity": vault::CAPACITY,
            "score": state.score(),
            "turns": state.turns(),
            "max_turns": vault::MAX_ACTIONS,
            "finished": self.finished,
            "cells": cells,
            "visible": self.visible.as_slice(),
            "standing_on": state.cell(state.position()).glyph().to_string(),
            "message": self.message,
        })
        .to_string()
    }

    pub fn act(&mut self, command: &str) -> String {
        if self.finished {
            return self.state();
        }
        if command == "finish" {
            self.finished = true;
            self.message = format!(
                "Expedition finished. Score: {}. Start a new descent to explore another seed.",
                self.expedition.score()
            );
            return self.state();
        }
        let Some(mut action) = Action::parse(command) else {
            self.message = "Unknown command. Use the movement keys or the controls below.".into();
            return self.state();
        };
        if let Action::Move(direction) = action {
            let (dx, dy) = direction.delta();
            let at = self.expedition.position();
            if self
                .expedition
                .cell(Point::new(at.x + dx, at.y + dy))
                .dig_cost()
                .is_some()
            {
                action = Action::Dig(direction);
            }
        }
        if action == Action::Interact
            && matches!(
                self.expedition.cell(self.expedition.position()),
                Cell::Up | Cell::Down
            )
        {
            action = Action::Portal;
        }
        self.message = match self.expedition.apply(action) {
            Ok(Event::Moved) => match self.expedition.cell(self.expedition.position()) {
                Cell::Flask => "A flask. Press E to restore up to 8 charge.".into(),
                Cell::Down => "A way deeper. Press E to descend.".into(),
                Cell::Up => "The passage back. Press E to ascend.".into(),
                _ => "Your footsteps echo through the castle.".into(),
            },
            Ok(Event::Dug { cost }) => {
                format!("The vein gives way. {cost} charge spent; step forward to enter.")
            }
            Ok(Event::Collected { value }) => {
                format!("A crystal catches the candlelight. +{value} score.")
            }
            Ok(Event::Replenished { amount }) => format!("Flask used. +{amount} charge."),
            Ok(Event::Travelled { floor }) => {
                format!("Depth {}. The way back remains open.", floor + 1)
            }
            Ok(Event::Waited) => "You wait. The castle keeps its secrets.".into(),
            Err(error) => error.to_string(),
        };
        if self.expedition.turns() == vault::MAX_ACTIONS {
            self.finished = true;
            self.message = format!(
                "All 512 actions spent. Final score: {}. Start a new descent when ready.",
                self.expedition.score()
            );
        }
        self.refresh_visibility();
        self.state()
    }
}

impl Demo {
    fn refresh_visibility(&mut self) {
        let state = &self.expedition;
        let world = World {
            width: vault::WIDTH as i32,
            height: vault::HEIGHT as i32,
            tiles: state.floors()[state.floor()]
                .iter()
                .map(|cell| match cell {
                    Cell::Wall => Tile::Wall,
                    Cell::SoftRock | Cell::HardRock => Tile::Rock,
                    _ => Tile::Marble,
                })
                .collect(),
            props: Vec::new(),
        };
        for (i, visible) in self.visible.iter_mut().enumerate() {
            let at = Point::new((i % vault::WIDTH) as i32, (i / vault::WIDTH) as i32);
            let dx = at.x - state.position().x;
            let dy = at.y - state.position().y;
            *visible = dx * dx + dy * dy <= 225 && world.line_of_sight(state.position(), at);
            self.explored[state.floor()][i] |= *visible;
        }
    }
}
