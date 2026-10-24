use std::io::{self, BufRead, Write};

use diggerchain_core::expedition::{
    Action, CAPACITY, Direction, Event, Expedition, SIZE, demonstration,
};
use serde_json::{Value, json};

const HELP: &str = "DiggerChain feasibility fixture (lab-v0)

Usage: diggerchain-lab [play | demo | trace] [SEED]

play: line-based JSON commands, with one observation per command.
demo: print an inspectable two-floor run and its final state.
trace: export the demo and all resulting states as one JSON document.

Commands: look, north, south, west, east, dig-north, dig-south,
dig-west, dig-east, interact, portal, wait, quit.

Legend: @ player, # wall, . ground, r soft rock, R hard rock,
C crystal, F ember flask, O portal. This lab shows both full maps.
The authored game keeps its normal fog behavior.

This fixture does not create cryptographic proofs or token claims.
";

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
        Err(error) => {
            eprintln!("diggerchain-lab: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "play".into());
    if matches!(mode.as_str(), "--help" | "-h") {
        print!("{HELP}");
        return Ok(());
    }
    if !matches!(mode.as_str(), "play" | "demo" | "trace") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, HELP));
    }
    let seed = args
        .next()
        .map(|text| text.parse::<u32>())
        .transpose()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "SEED must be an unsigned 32-bit integer",
            )
        })?
        .unwrap_or(0);
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Too many arguments",
        ));
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match mode.as_str() {
        "play" => play(io::stdin().lock(), &mut output, seed),
        "demo" => demo(&mut output, seed),
        "trace" => write_json(&mut output, &trace(seed)),
        _ => unreachable!(),
    }
}

fn parse_action(command: &str) -> Option<Action> {
    Some(match command {
        "north" => Action::Move(Direction::North),
        "south" => Action::Move(Direction::South),
        "west" => Action::Move(Direction::West),
        "east" => Action::Move(Direction::East),
        "dig-north" => Action::Dig(Direction::North),
        "dig-south" => Action::Dig(Direction::South),
        "dig-west" => Action::Dig(Direction::West),
        "dig-east" => Action::Dig(Direction::East),
        "interact" => Action::Interact,
        "portal" => Action::Portal,
        "wait" => Action::Wait,
        _ => return None,
    })
}

fn action_name(action: Action) -> &'static str {
    match action {
        Action::Move(Direction::North) => "north",
        Action::Move(Direction::South) => "south",
        Action::Move(Direction::West) => "west",
        Action::Move(Direction::East) => "east",
        Action::Dig(Direction::North) => "dig-north",
        Action::Dig(Direction::South) => "dig-south",
        Action::Dig(Direction::West) => "dig-west",
        Action::Dig(Direction::East) => "dig-east",
        Action::Interact => "interact",
        Action::Portal => "portal",
        Action::Wait => "wait",
    }
}

fn observation(expedition: &Expedition) -> Value {
    let floors: Vec<Vec<String>> = expedition
        .floors()
        .iter()
        .enumerate()
        .map(|(floor, cells)| {
            cells
                .as_chunks::<SIZE>()
                .0
                .iter()
                .enumerate()
                .map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(x, cell)| {
                            if floor == expedition.floor()
                                && expedition.position().x == x as i32
                                && expedition.position().y == y as i32
                            {
                                '@'
                            } else {
                                cell.glyph()
                            }
                        })
                        .collect()
                })
                .collect()
        })
        .collect();
    json!({
        "fixture": "lab-v0",
        "cryptographic_proof": false,
        "seed": expedition.seed(),
        "turn": expedition.turns(),
        "floor": expedition.floor(),
        "position": {"x": expedition.position().x, "y": expedition.position().y},
        "standing_on": expedition.floors()[expedition.floor()][expedition.position().y as usize * SIZE + expedition.position().x as usize].glyph().to_string(),
        "charge": expedition.charge(),
        "capacity": CAPACITY,
        "score": expedition.score(),
        "floors": floors,
    })
}

fn step(expedition: &mut Expedition, action: Action) -> Value {
    let outcome = match expedition.apply(action) {
        Ok(event) => json!({"ok": true, "event": format!("{event:?}")}),
        Err(error) => json!({"ok": false, "error": error.to_string()}),
    };
    json!({"command": action_name(action), "outcome": outcome, "state": observation(expedition)})
}

fn play(input: impl BufRead, output: &mut impl Write, seed: u32) -> io::Result<()> {
    let mut expedition = Expedition::new(seed);
    write_json(
        output,
        &json!({"ok": true, "state": observation(&expedition)}),
    )?;
    for line in input.lines() {
        let command = line?.trim().to_ascii_lowercase();
        match command.as_str() {
            "quit" => {
                write_json(output, &json!({"ok": true, "type": "goodbye"}))?;
                break;
            }
            "look" => write_json(
                output,
                &json!({"ok": true, "state": observation(&expedition)}),
            )?,
            _ => match parse_action(&command) {
                Some(action) => write_json(output, &step(&mut expedition, action))?,
                None => write_json(
                    output,
                    &json!({"ok": false, "error": "Unknown command. Run --help for the command list.", "state": observation(&expedition)}),
                )?,
            },
        }
    }
    Ok(())
}

fn demo(output: &mut impl Write, seed: u32) -> io::Result<()> {
    let mut expedition = Expedition::new(seed);
    writeln!(
        output,
        "DiggerChain lab-v0: native gameplay fixture; no cryptographic proof."
    )?;
    writeln!(
        output,
        "Seed: {seed}. Legend: # wall, r soft rock, R hard rock, C crystal, F flask, O portal."
    )?;
    for action in demonstration(seed) {
        let result = expedition.apply(action);
        let outcome = match result {
            Ok(Event::Collected { value }) => format!("Collected crystal: +{value}"),
            Ok(event) => format!("{event:?}"),
            Err(error) => format!("Rejected: {error}"),
        };
        writeln!(
            output,
            "{:02} {:10} floor={} charge={:02} score={} {outcome}",
            expedition.turns(),
            action_name(action),
            expedition.floor(),
            expedition.charge(),
            expedition.score()
        )?;
    }
    serde_json::to_writer_pretty(&mut *output, &observation(&expedition))?;
    writeln!(output)
}

fn trace(seed: u32) -> Value {
    let mut expedition = Expedition::new(seed);
    let initial = observation(&expedition);
    let steps: Vec<_> = demonstration(seed)
        .into_iter()
        .map(|action| step(&mut expedition, action))
        .collect();
    json!({"fixture": "lab-v0", "cryptographic_proof": false, "initial": initial, "steps": steps, "final": observation(&expedition)})
}

fn write_json(output: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_protocol_preserves_state_on_unknown_commands_and_observation() {
        let mut output = Vec::new();
        play(
            &b"look\nunknown\ndig-east\neast\nquit\n"[..],
            &mut output,
            0,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        let responses: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(responses.len(), 6);
        assert_eq!(responses[0]["state"], responses[1]["state"]);
        assert_eq!(responses[0]["state"], responses[2]["state"]);
        assert_eq!(responses[2]["ok"], false);
        assert_eq!(responses[3]["state"]["charge"], 2);
        assert_eq!(responses[4]["state"]["position"]["x"], 2);
        assert_eq!(responses[5]["type"], "goodbye");
    }

    #[test]
    fn trace_explains_the_complete_demo_without_claiming_to_be_a_proof() {
        let result = trace(1);
        assert_eq!(result["cryptographic_proof"], false);
        assert_eq!(result["final"]["score"], 5);
        assert_eq!(result["final"]["charge"], 7);
        assert_eq!(result["final"]["turn"], 36);
        let steps = result["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 39);
        assert_eq!(
            steps
                .iter()
                .filter(|step| step["outcome"]["ok"] == false)
                .count(),
            3
        );
    }
}
