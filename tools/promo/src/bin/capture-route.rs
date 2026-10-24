use diggerchain_core::vault::{Action, Direction, Expedition, demonstration};
use serde_json::json;

fn main() {
    let mut world = Expedition::new(0);
    let mut inputs = Vec::new();
    for action in demonstration(0, 512, false).expect("demonstration route") {
        let key = match action {
            Action::Move(direction) | Action::Dig(direction) => match direction {
                Direction::North => "w",
                Direction::South => "s",
                Direction::West => "a",
                Direction::East => "d",
            },
            Action::Interact | Action::Portal => "e",
            Action::Wait => ".",
        };
        world.apply(action).expect("legal recording route");
        inputs.push(json!({"key":key, "floor":world.floor(), "score":world.score(), "action":action.code()}));
    }
    println!("{}", json!({"seed":0,"inputs":inputs}));
}
