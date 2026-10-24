use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as Input, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use diggerchain_core::record::{Record, hex};
use diggerchain_core::vault::{self, Action, Cell, Direction, Event, Expedition};
use diggerchain_core::{Crystal, Game, Point, Prop, PropKind, Tile, World};
use serde_json::{Value, json};

use crate::{TerminalSession, render, save::Save};

pub struct Session {
    pub state: Expedition,
    pub record: Record,
    pub picture: Game,
    save: Save,
    memories: Vec<Vec<bool>>,
}

fn world(state: &Expedition) -> World {
    let mut world = World {
        width: vault::WIDTH as i32,
        height: vault::HEIGHT as i32,
        tiles: vec![Tile::Wall; vault::WIDTH * vault::HEIGHT],
        props: Vec::new(),
    };
    let crystal = [Crystal::Thorns, Crystal::Silence, Crystal::Echoes][state.floor()];
    for y in 0..vault::HEIGHT {
        for x in 0..vault::WIDTH {
            let at = Point::new(x as i32, y as i32);
            let cell = state.cell(at);
            let room = if y < 6 { x / 6 } else { 5 - x / 6 };
            let tile = match cell {
                Cell::Wall => Tile::Wall,
                Cell::SoftRock | Cell::HardRock => Tile::Rock,
                _ => [
                    Tile::Marble,
                    Tile::Wood,
                    Tile::Mosaic,
                    Tile::Crypt,
                    Tile::Carpet,
                    Tile::Grass,
                ][room.min(5)],
            };
            world.set(at, tile);
            let kind = match cell {
                Cell::Crystal => Some(PropKind::Crystal(crystal)),
                Cell::Flask => Some(PropKind::Flask),
                Cell::Up => Some(PropKind::PortalUp),
                Cell::Down => Some(PropKind::PortalDown),
                Cell::SoftRock => Some(PropKind::SoftVein),
                Cell::HardRock => Some(PropKind::HardVein),
                Cell::Wall if y == 0 && [3, 9, 15].contains(&x) => Some(PropKind::Window),
                Cell::Wall if y == 12 && [3, 9, 15].contains(&x) => Some(PropKind::Banner),
                Cell::Wall if x == 18 && [3, 9].contains(&y) => Some(PropKind::Sconce),
                Cell::Wall if x == 0 && y == 3 => Some(PropKind::Bookshelf),
                Cell::Wall if x == 0 && y == 9 => Some(PropKind::Coffin),
                Cell::Ground if [Point::new(16, 1), Point::new(1, 11)].contains(&at) => {
                    Some(PropKind::Mushroom)
                }
                Cell::Ground if at == Point::new(4, 11) => Some(PropKind::Fern),
                _ => None,
            };
            if let Some(kind) = kind {
                world.props.push(Prop {
                    at,
                    kind,
                    active: true,
                });
            }
        }
    }
    world
}

impl Session {
    pub fn new(save: Save, record: Record) -> io::Result<Self> {
        record
            .replay()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let state = Expedition::new(record.seed);
        let picture = Game::from_world(world(&state), state.position());
        let mut session = Self {
            state,
            record,
            picture,
            save,
            memories: vec![vec![false; vault::WIDTH * vault::HEIGHT]; vault::FLOORS],
        };
        session.refresh();
        for code in session.record.actions.clone() {
            session
                .state
                .apply(Action::from_code(code).expect("validated action"))
                .expect("validated history");
            session.refresh();
        }
        session.log("Every floor has a way back. Remember where you came from.");
        session.log(if session.record.finished {
            "Expedition sealed. Your history is saved; start another with --new."
        } else {
            "Bump veins to dig. E uses a flask or portal underfoot. F ends this attempt at its current score."
        });
        if session.record.secret.is_none() {
            session.log("Practice world: the same seed recreates the same expedition.");
        }
        Ok(session)
    }

    fn refresh(&mut self) {
        self.picture.world = world(&self.state);
        self.picture.player = self.state.position();
        self.picture.turns = self.state.turns() as u64;
        self.picture.completed = self.record.finished;
        self.picture
            .explored
            .clone_from(&self.memories[self.state.floor()]);
        self.picture.update_visibility();
        self.memories[self.state.floor()].clone_from(&self.picture.explored);
    }

    fn log(&mut self, text: impl Into<String>) {
        self.picture.messages.push_back(text.into());
        while self.picture.messages.len() > 8 {
            self.picture.messages.pop_front();
        }
    }

    fn apply(&mut self, action: Action) -> io::Result<Result<Event, String>> {
        if self.record.finished {
            return Ok(Err(
                "This expedition is sealed. Start another with --new.".into()
            ));
        }
        let mut state = self.state.clone();
        let event = match state.apply(action) {
            Ok(event) => event,
            Err(error) => return Ok(Err(error.to_string())),
        };
        let mut record = self.record.clone();
        record.actions.push(action.code());
        self.save.write(&record)?;
        self.state = state;
        self.record = record;
        self.refresh();
        match event {
            Event::Dug { cost } => self.log(format!("The vein breaks. Spent {cost} charge.")),
            Event::Collected { value } => {
                self.log(format!("A crystal sings: +{value} score. Saved."))
            }
            Event::Replenished { amount } => {
                self.log(format!("Ember returns to the drill: +{amount} charge."))
            }
            Event::Travelled { floor } => self.log(format!(
                "Depth {}. The passage behind you remains open.",
                floor + 1
            )),
            _ => {}
        }
        if self.state.turns() == vault::MAX_ACTIONS {
            self.log("Action limit reached. Press F to seal your result; Q saves and exits.");
        }
        Ok(Ok(event))
    }

    fn finish(&mut self) -> io::Result<()> {
        if !self.record.finished {
            let mut record = self.record.clone();
            record.finished = true;
            self.save.write(&record)?;
            self.record = record;
            self.picture.completed = true;
            self.log(format!(
                "Expedition finished with {} score. Your progress is saved; start another with --new.",
                self.state.score()
            ));
        }
        Ok(())
    }

    fn human_move(&self, direction: Direction) -> Action {
        let (dx, dy) = direction.delta();
        let at = self.state.position();
        if self
            .state
            .cell(Point::new(at.x + dx, at.y + dy))
            .dig_cost()
            .is_some()
        {
            Action::Dig(direction)
        } else {
            Action::Move(direction)
        }
    }

    fn observation(&self) -> Value {
        let mut map = Vec::new();
        let mut objects = Vec::new();
        for y in 0..vault::HEIGHT {
            let mut row = String::new();
            for x in 0..vault::WIDTH {
                let at = Point::new(x as i32, y as i32);
                let cell = self.state.cell(at);
                row.push(if at == self.state.position() {
                    '@'
                } else if self.picture.visible_at(at) {
                    cell.glyph()
                } else if self.picture.explored_at(at) {
                    match cell {
                        Cell::Wall | Cell::SoftRock | Cell::HardRock => '#',
                        _ => '.',
                    }
                } else {
                    '?'
                });
                if self.picture.visible_at(at) && !matches!(cell, Cell::Wall | Cell::Ground) {
                    objects.push(json!({"x": x, "y": y, "kind": cell.glyph().to_string(), "dig_cost": cell.dig_cost()}));
                }
            }
            map.push(row);
        }
        let mut legal = vec!["look", "save", "finish", "quit"];
        if !self.record.finished {
            for action in (1..=11).filter_map(Action::from_code) {
                if self.state.clone().apply(action).is_ok() {
                    legal.push(action.name());
                }
            }
        }
        json!({
            "ok": true, "type": "observation", "game": "DiggerChain", "mode": "expedition",
            "rules_version": vault::RULES_VERSION, "seed": self.state.seed(),
            "expedition_id": self.record.id().map(|id| hex(&id)),
            "practice": self.record.secret.is_none(), "finished": self.record.finished,
            "turn": self.state.turns(), "max_actions": vault::MAX_ACTIONS,
            "floor": self.state.floor(), "floors": vault::FLOORS,
            "position": {"x": self.state.position().x, "y": self.state.position().y},
            "score": self.state.score(), "charge": self.state.charge(), "capacity": vault::CAPACITY,
            "standing_on": self.state.cell(self.state.position()).glyph().to_string(),
            "legal_actions": legal, "map": map, "objects": objects,
            "map_legend": "@ player, ? undiscovered, # wall or remembered rock, . ground, r soft vein, R hard vein, C crystal, F flask, < up portal, > down portal",
            "messages": self.picture.messages,
        })
    }
}

fn respond(output: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    output.flush()
}

pub fn agent(mut session: Session, input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    respond(&mut output, &session.observation())?;
    for line in input.lines() {
        let command = line?.trim().to_ascii_lowercase();
        let result = match command.as_str() {
            "quit" => { respond(&mut output, &json!({"ok": true, "type": "goodbye"}))?; break; }
            "look" => Ok(()),
            "save" => { session.save.write(&session.record)?; Ok(()) }
            "finish" => { session.finish()?; Ok(()) }
            _ => match Action::parse(&command) {
                Some(action) => session.apply(action)?.map(|_| ()),
                None => Err("Unknown command. Use look, north, south, west, east, dig-north, dig-south, dig-west, dig-east, interact, portal, wait, save, finish, or quit.".into()),
            }
        };
        let mut observation = session.observation();
        if let Err(error) = result {
            observation["ok"] = false.into();
            observation["error"] = error.into();
        }
        respond(&mut output, &observation)?;
    }
    Ok(())
}

pub fn play(mut session: Session, glyphs: bool) -> io::Result<()> {
    let _terminal = TerminalSession::enter()?;
    let mut view = render::View::new(glyphs);
    let mut stdout = io::stdout().lock();
    let mut last = Instant::now();
    loop {
        let started = Instant::now();
        view.tick(started.duration_since(last).as_secs_f32().min(0.1));
        last = started;
        let (width, height) = terminal::size()?;
        view.draw_expedition(
            &session.picture,
            &session.state,
            session.record.finished,
            width,
            height,
            &mut stdout,
        )?;
        if !event::poll(Duration::from_millis(33).saturating_sub(started.elapsed()))? {
            continue;
        }
        let Input::Key(key) = event::read()? else {
            continue;
        };
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }
        if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
            || matches!(key.code, KeyCode::Char('q' | 'Q') | KeyCode::Esc)
        {
            break;
        }
        if key.code == KeyCode::Char('?') {
            view.help = !view.help;
            continue;
        }
        if view.help {
            if key.code == KeyCode::Enter {
                view.help = false;
            }
            continue;
        }
        if width < 90 || height < 30 {
            continue;
        }
        let action = match key.code {
            KeyCode::Up | KeyCode::Char('w' | 'W' | 'k' | 'K') => {
                Some(session.human_move(Direction::North))
            }
            KeyCode::Down | KeyCode::Char('s' | 'S' | 'j' | 'J') => {
                Some(session.human_move(Direction::South))
            }
            KeyCode::Left | KeyCode::Char('a' | 'A' | 'h' | 'H') => {
                Some(session.human_move(Direction::West))
            }
            KeyCode::Right | KeyCode::Char('d' | 'D' | 'l' | 'L') => {
                Some(session.human_move(Direction::East))
            }
            KeyCode::Char('e' | 'E') | KeyCode::Enter if key.kind == KeyEventKind::Press => Some(
                if matches!(
                    session.state.cell(session.state.position()),
                    Cell::Up | Cell::Down
                ) {
                    Action::Portal
                } else {
                    Action::Interact
                },
            ),
            KeyCode::Char('f' | 'F') if key.kind == KeyEventKind::Press => {
                session.finish()?;
                None
            }
            KeyCode::Char('g' | 'G') if key.kind == KeyEventKind::Press => {
                view.toggle_mode();
                None
            }
            KeyCode::Char('z' | 'Z') => {
                view.zoom(1);
                None
            }
            KeyCode::Char('x' | 'X') => {
                view.zoom(-1);
                None
            }
            KeyCode::Char(' ') if key.kind == KeyEventKind::Press => {
                view.paused = !view.paused;
                None
            }
            _ => None,
        };
        if let Some(action) = action {
            match session.apply(action)? {
                Ok(outcome) => {
                    if matches!(outcome, Event::Travelled { .. }) {
                        view.reset_camera();
                    }
                    view.on_expedition_event(outcome, session.state.position());
                }
                Err(error) => session.log(error),
            }
        }
    }
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn protocol_preserves_fog_and_secrets_and_restores_the_saved_history() {
        let mut suffix = [0; 8];
        getrandom::getrandom(&mut suffix).unwrap();
        let directory = std::env::temp_dir().join(format!("diggerchain-agent-{}", hex(&suffix)));
        let path = directory.join("run.json");
        let (save, record) = Save::open(path.clone(), false, None).unwrap();
        let session = Session::new(save, record).unwrap();
        let initial = session.observation();
        assert_eq!(
            initial["map"][3].as_str().unwrap().chars().nth(15),
            Some('?')
        );
        assert!(initial.get("secret").is_none());
        let mut output = Vec::new();
        agent(
            session,
            &b"look\neast\nunknown\nfinish\nwest\nquit\n"[..],
            &mut output,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(!text.contains('\x1b'));
        let values: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(values[0], values[1]);
        assert_eq!(values[2]["turn"], 1);
        assert_eq!(values[3]["ok"], false);
        assert_eq!(values[4]["finished"], true);
        assert_eq!(values[5]["ok"], false);
        let (save, record) = Save::open(path, false, None).unwrap();
        let restored = Session::new(save, record).unwrap();
        assert_eq!(restored.observation()["map"], values[4]["map"]);
        assert_eq!(restored.state.turns(), 1);
        drop(restored);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn decoration_matches_simulation_collision_and_both_views_render() {
        for seed in [0, u64::MAX] {
            let state = Expedition::new(seed);
            let game = Game::from_world(world(&state), state.position());
            for y in 0..vault::HEIGHT {
                for x in 0..vault::WIDTH {
                    let at = Point::new(x as i32, y as i32);
                    assert_eq!(game.world.blocked(at), !state.cell(at).walkable());
                }
            }
            for glyphs in [true, false] {
                render::View::new(glyphs)
                    .draw_expedition(&game, &state, false, 90, 30, &mut Vec::new())
                    .unwrap();
            }
        }
    }
}
