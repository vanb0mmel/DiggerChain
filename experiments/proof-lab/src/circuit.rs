use ark_bn254::Fr;
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use diggerchain_core::expedition::{Action, Cell, Direction, Expedition, SIZE, demonstration};

pub const INPUT_NAMES: [&str; 11] = [
    "rules_version",
    "seed",
    "settlement_domain",
    "fixture_epoch",
    "fixture_expedition_id",
    "score",
    "charge",
    "floor",
    "x",
    "y",
    "turns",
];

#[derive(Clone)]
pub struct RunCircuit {
    pub public: [Fr; 11],
    pub actions: Vec<u8>,
    pub max_steps: usize,
}

impl RunCircuit {
    pub fn fixture(seed: u32, max_steps: usize, round_trips: usize) -> Result<Self, String> {
        if ![64, 128, 256].contains(&max_steps) {
            return Err("The step bound must be 64, 128, or 256.".into());
        }
        if round_trips > (max_steps - 36) / 18 {
            return Err("The requested round trips do not fit the step bound.".into());
        }
        let mut expedition = Expedition::new(seed);
        let mut actions = Vec::new();
        for action in demonstration(seed) {
            if expedition.apply(action).is_ok() {
                actions.push(encode(action));
            }
        }
        for _ in 0..round_trips {
            let trip = [
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Portal,
                Action::Move(Direction::North),
                Action::Move(Direction::North),
                Action::Move(Direction::North),
                Action::Move(Direction::North),
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Move(Direction::South),
                Action::Portal,
                Action::Move(Direction::North),
                Action::Move(Direction::North),
                Action::Move(Direction::North),
                Action::Move(Direction::North),
            ];
            for action in trip {
                expedition.apply(action).map_err(|e| e.to_string())?;
                actions.push(encode(action));
            }
        }
        Ok(Self {
            public: [
                Fr::from(1u64),
                Fr::from(seed),
                Fr::from(46630u64),
                Fr::from(0u64),
                Fr::from(1u64),
                Fr::from(expedition.score()),
                Fr::from(expedition.charge()),
                Fr::from(expedition.floor() as u64),
                Fr::from(expedition.position().x as u64),
                Fr::from(expedition.position().y as u64),
                Fr::from(expedition.turns()),
            ],
            actions,
            max_steps,
        })
    }
}

pub(crate) fn encode(action: Action) -> u8 {
    match action {
        Action::Move(Direction::North) => 1,
        Action::Move(Direction::South) => 2,
        Action::Move(Direction::West) => 3,
        Action::Move(Direction::East) => 4,
        Action::Dig(Direction::North) => 5,
        Action::Dig(Direction::South) => 6,
        Action::Dig(Direction::West) => 7,
        Action::Dig(Direction::East) => 8,
        Action::Interact => 9,
        Action::Portal => 10,
        Action::Wait => 11,
    }
}

pub(crate) fn number(value: u64) -> FpVar<Fr> {
    FpVar::Constant(Fr::from(value))
}

pub(crate) fn any(bits: &[Boolean<Fr>]) -> Boolean<Fr> {
    // arkworks 0.5's multi-input OR panics when every input is constant.
    bits.iter().fold(Boolean::FALSE, |result, bit| result | bit)
}

fn at(
    floor: &FpVar<Fr>,
    x: &FpVar<Fr>,
    y: &FpVar<Fr>,
    location: (usize, usize, usize),
) -> Result<Boolean<Fr>, SynthesisError> {
    Ok(floor.is_eq(&number(location.0 as u64))?
        & x.is_eq(&number(location.1 as u64))?
        & y.is_eq(&number(location.2 as u64))?)
}

impl ConstraintSynthesizer<Fr> for RunCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        if self.actions.len() > self.max_steps {
            return Err(SynthesisError::Unsatisfiable);
        }
        let public = self
            .public
            .iter()
            .map(|value| FpVar::new_input(cs.clone(), || Ok(*value)))
            .collect::<Result<Vec<_>, _>>()?;
        public[0].enforce_equal(&number(1))?;
        let (seed_bits, _) = public[1].to_bits_le_with_top_bits_zero(32)?;
        for input in &public[2..5] {
            let _ = input.to_bits_le_with_top_bits_zero(64)?;
        }
        let mirrored = &seed_bits[0];
        let base = Expedition::new(0);
        let tiles: Vec<_> = base
            .floors()
            .iter()
            .enumerate()
            .flat_map(|(floor, cells)| {
                cells.iter().enumerate().filter_map(move |(i, cell)| {
                    (*cell != Cell::Wall).then_some(((floor, i % SIZE, i / SIZE), *cell))
                })
            })
            .collect();
        let mut changed = vec![Boolean::FALSE; tiles.len()];
        let mut floor = number(0);
        let mut x = mirrored.select(&number(5), &number(1))?;
        let mut y = number(1);
        let mut charge = number(4);
        let mut score = number(0);
        let mut turns = number(0);

        for step in 0..self.max_steps {
            let action = self.actions.get(step).copied().unwrap_or(0);
            let selected = (0..12)
                .map(|id| Boolean::new_witness(cs.clone(), || Ok(action == id)))
                .collect::<Result<Vec<_>, _>>()?;
            let flags = selected
                .iter()
                .fold(number(0), |sum, bit| sum + FpVar::from(bit.clone()));
            flags.enforce_equal(&number(1))?;
            let moving = any(&selected[1..5]);
            let digging = any(&selected[5..9]);
            let interacting = &selected[9];
            let portal = &selected[10];
            let move_x = &x + FpVar::from(selected[4].clone()) - FpVar::from(selected[3].clone());
            let move_y = &y + FpVar::from(selected[2].clone()) - FpVar::from(selected[1].clone());
            let dig_x = &x + FpVar::from(selected[8].clone()) - FpVar::from(selected[7].clone());
            let dig_y = &y + FpVar::from(selected[6].clone()) - FpVar::from(selected[5].clone());
            let logical_move_x = mirrored.select(&(number(6) - &move_x), &move_x)?;
            let logical_dig_x = mirrored.select(&(number(6) - &dig_x), &dig_x)?;
            let logical_x = mirrored.select(&(number(6) - &x), &x)?;
            let mut can_walk = Boolean::FALSE;
            let mut can_dig = Boolean::FALSE;
            let mut can_refill = Boolean::FALSE;
            let mut can_portal = Boolean::FALSE;
            let mut dig_cost = number(0);

            for (i, (location, cell)) in tiles.iter().enumerate() {
                let on_move = at(&floor, &logical_move_x, &move_y, *location)?;
                match cell {
                    Cell::SoftRock | Cell::HardRock => {
                        can_walk |= &on_move & &changed[i];
                        let on_dig = at(&floor, &logical_dig_x, &dig_y, *location)?;
                        can_dig |= &on_dig & !&changed[i];
                        let opened = &digging & on_dig;
                        let cost = if *cell == Cell::HardRock { 3 } else { 2 };
                        dig_cost += FpVar::from(opened.clone()) * Fr::from(cost as u64);
                        changed[i] |= opened;
                    }
                    Cell::Crystal => {
                        can_walk |= &on_move;
                        let collected = &moving & on_move & !&changed[i];
                        let value = if location.0 == 0 { 1 } else { 4 };
                        score += FpVar::from(collected.clone()) * Fr::from(value as u64);
                        changed[i] |= collected;
                    }
                    Cell::Flask => {
                        can_walk |= &on_move;
                        let on_flask = at(&floor, &logical_x, &y, *location)?;
                        can_refill |= &on_flask & !&changed[i];
                        changed[i] |= interacting & on_flask;
                    }
                    Cell::Portal => {
                        can_walk |= &on_move;
                        can_portal |= at(&floor, &logical_x, &y, *location)?;
                    }
                    Cell::Ground => can_walk |= &on_move,
                    Cell::Wall => unreachable!("walls are excluded from the fixture lookup"),
                }
            }

            // Boolean conditional equality in arkworks 0.5 rejects unequal constants even when disabled.
            (&moving & !can_walk).enforce_equal(&Boolean::FALSE)?;
            (&digging & !can_dig).enforce_equal(&Boolean::FALSE)?;
            (interacting & !can_refill).enforce_equal(&Boolean::FALSE)?;
            (portal & !can_portal).enforce_equal(&Boolean::FALSE)?;
            (interacting & charge.is_eq(&number(12))?).enforce_equal(&Boolean::FALSE)?;
            let low_charge = (0..=4)
                .map(|value| charge.is_eq(&number(value)))
                .collect::<Result<Vec<_>, _>>()?;
            let replenished = any(&low_charge).select(&(&charge + Fr::from(8u64)), &number(12))?;
            charge = interacting.select(&replenished, &(&charge - dig_cost))?;
            let (bits, _) = charge.to_bits_le_with_top_bits_zero(4)?;
            (&bits[3] & &bits[2] & (&bits[1] | &bits[0])).enforce_equal(&Boolean::FALSE)?;
            x = portal.select(&number(3), &move_x)?;
            y = portal.select(&number(5), &move_y)?;
            floor = portal.select(&(number(1) - &floor), &floor)?;
            turns += FpVar::from(!&selected[0]);
        }

        for (computed, claimed) in [score, charge, floor, x, y, turns].iter().zip(&public[5..]) {
            computed.enforce_equal(claimed)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::{ConstraintSystem, SynthesisMode};

    fn satisfied(circuit: RunCircuit) -> bool {
        let cs = ConstraintSystem::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap(),
            Err(SynthesisError::Unsatisfiable) => false,
            Err(error) => panic!("Unexpected circuit synthesis error: {error}"),
        }
    }

    #[test]
    fn replenishment_masks_accept_constant_inputs() {
        let mut mask: [Boolean<Fr>; 5] = std::array::from_fn(|_| Boolean::FALSE);
        assert_eq!(any(&mask), Boolean::FALSE);
        mask[4] = Boolean::TRUE;
        assert_eq!(any(&mask), Boolean::TRUE);
    }

    #[test]
    fn setup_and_mirrored_witness_generate_the_same_constraint_matrices() {
        let setup = ConstraintSystem::new_ref();
        setup.set_mode(SynthesisMode::Setup);
        RunCircuit::fixture(0, 64, 0)
            .unwrap()
            .generate_constraints(setup.clone())
            .unwrap();
        let proving = ConstraintSystem::new_ref();
        RunCircuit::fixture(1, 64, 0)
            .unwrap()
            .generate_constraints(proving.clone())
            .unwrap();
        assert!(proving.is_satisfied().unwrap());
        setup.finalize();
        proving.finalize();
        assert!(
            setup.to_matrices().unwrap() == proving.to_matrices().unwrap(),
            "Setup and proving must generate identical constraint matrices"
        );
    }

    #[test]
    fn absent_flasks_and_portals_only_reject_the_actions_that_require_them() {
        let mut start = RunCircuit::fixture(0, 64, 0).unwrap();
        start.actions.clear();
        start.public[5..].copy_from_slice(&[
            Fr::from(0u64),
            Fr::from(4u64),
            Fr::from(0u64),
            Fr::from(1u64),
            Fr::from(1u64),
            Fr::from(0u64),
        ]);
        assert!(satisfied(start.clone()));
        start.public[10] = Fr::from(1u64);
        start.actions = vec![11];
        assert!(satisfied(start.clone()));
        for action in [9, 10] {
            start.actions = vec![action];
            assert!(!satisfied(start.clone()));
        }
    }

    #[test]
    fn mirrored_routes_and_longer_backtracking_match_native_results() {
        for (seed, steps, trips) in [(0, 64, 0), (1, 64, 0), (42, 128, 5)] {
            assert!(satisfied(RunCircuit::fixture(seed, steps, trips).unwrap()));
        }
    }

    #[test]
    fn fabricated_public_results_are_rejected() {
        let valid = RunCircuit::fixture(0, 64, 0).unwrap();
        for field in 5..11 {
            let mut forged = valid.clone();
            forged.public[field] += Fr::from(1u64);
            assert!(!satisfied(forged), "public field {field}");
        }
    }

    #[test]
    fn illegal_actions_cannot_be_hidden_by_a_valid_final_result() {
        let valid = RunCircuit::fixture(0, 64, 0).unwrap();
        for (index, action) in [(0, 4), (1, 8), (6, 11), (25, 9), (28, 5), (0, 12)] {
            let mut forged = valid.clone();
            forged.actions[index] = action;
            assert!(!satisfied(forged), "action {action} at step {index}");
        }
    }

    #[test]
    fn range_checks_bind_seed_and_context_to_the_declared_widths() {
        for (field, width) in [(1, 32), (2, 64), (3, 64), (4, 64)] {
            let mut forged = RunCircuit::fixture(0, 64, 0).unwrap();
            let mut overflow = Fr::from(1u64);
            for _ in 0..width {
                overflow += overflow;
            }
            forged.public[field] = overflow;
            assert!(!satisfied(forged));
        }
    }
}
