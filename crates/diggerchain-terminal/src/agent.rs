use std::io::{self, BufRead, Write};

use diggerchain_core::{Action, Game, Point, PropKind, Tile};
use serde_json::{Value, json};

pub fn run(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let mut game = Game::new();
    respond(&mut output, &observation(&game))?;
    for line in input.lines() {
        let line = line?;
        let command = line.trim().to_ascii_lowercase();
        let action = match command.as_str() {
            "north" => Some(Action::Move(0, -1)),
            "south" => Some(Action::Move(0, 1)),
            "west" => Some(Action::Move(-1, 0)),
            "east" => Some(Action::Move(1, 0)),
            "interact" => Some(Action::Interact),
            "wait" => Some(Action::Wait),
            "look" => None,
            "reset" => {
                game = Game::new();
                None
            }
            "quit" => {
                respond(&mut output, &json!({"ok": true, "type": "goodbye"}))?;
                break;
            }
            _ => {
                respond(
                    &mut output,
                    &json!({"ok": false, "error": "Unknown command. Use look, north, south, west, east, interact, wait, reset, or quit.", "turn": game.turns}),
                )?;
                continue;
            }
        };
        if let Some(action) = action {
            game.apply(action);
        }
        respond(&mut output, &observation(&game))?;
    }
    Ok(())
}

fn respond(output: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    output.flush()
}

fn glyph(tile: Tile) -> char {
    match tile {
        Tile::Void => ' ',
        Tile::Rock | Tile::Wall => '#',
        Tile::Grass => ',',
        Tile::Water => '~',
        Tile::DoorClosed => '+',
        Tile::DoorOpen => '/',
        Tile::Carpet => '=',
        _ => '.',
    }
}

fn observation(game: &Game) -> Value {
    let mut map = Vec::new();
    let mut visible_tiles = Vec::new();
    for y in 0..game.world.height {
        let mut row = String::new();
        for x in 0..game.world.width {
            let p = Point::new(x, y);
            row.push(if p == game.player {
                '@'
            } else if game.explored_at(p) {
                glyph(game.world.tile(p))
            } else {
                '?'
            });
            if game.visible_at(p) {
                visible_tiles.push(json!({"x": x, "y": y, "terrain": game.world.tile(p).name(), "blocked": game.world.blocked(p)}));
            }
        }
        map.push(row);
    }
    let objects: Vec<_> = game.world.props.iter()
        .filter(|p| p.present() && game.visible_at(p.at))
        .map(|p| json!({"x": p.at.x, "y": p.at.y, "name": p.kind.name(), "active": p.active, "interactive": p.kind.interactive(), "crystal": matches!(p.kind, PropKind::Crystal(_))}))
        .collect();
    let mut actions = vec!["look", "wait"];
    for (name, dx, dy) in [
        ("north", 0, -1),
        ("south", 0, 1),
        ("west", -1, 0),
        ("east", 1, 0),
    ] {
        let p = Point::new(game.player.x + dx, game.player.y + dy);
        if !game.world.blocked(p) || game.world.tile(p) == Tile::DoorClosed {
            actions.push(name);
        }
    }
    if game.interaction_target().is_some() {
        actions.push("interact");
    }
    json!({
        "ok": true,
        "type": "observation",
        "game": "DiggerChain",
        "turn": game.turns,
        "position": {"x": game.player.x, "y": game.player.y},
        "region": game.region().0,
        "objective": {"found": game.crystals.iter().map(|c| c.name()).collect::<Vec<_>>(), "required": 3, "completed": game.completed, "instruction": "Find three crystals and interact with the forest waystone."},
        "legal_actions": actions,
        "nearby_interaction": game.interaction_target().map(|i| game.world.props[i].kind.name()),
        "map": map,
        "map_legend": "@ player, ? undiscovered, # solid stone, , grass, ~ water, + closed door, / open door, = carpet, . floor; objects are listed separately",
        "visible_tiles": visible_tiles,
        "objects": objects,
        "messages": game.messages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_protocol_is_json_only_and_look_does_not_advance_time() {
        let mut output = Vec::new();
        run(&b"look\neast\nunknown\nreset\nquit\n"[..], &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(!text.contains('\x1b'));
        let responses: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(responses.len(), 6);
        assert_eq!(responses[0], responses[1]);
        assert_eq!(responses[2]["turn"], 1);
        assert_eq!(responses[3]["ok"], false);
        assert_eq!(responses[4], responses[0]);
        assert_eq!(responses[5]["type"], "goodbye");
    }

    #[test]
    fn observations_do_not_disclose_hidden_crystals() {
        let value = observation(&Game::new());
        assert!(
            value["objects"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| p["crystal"] == false)
        );
        assert_eq!(value["map"][5].as_str().unwrap().chars().nth(74), Some('?'));
    }
}
