use ark_bn254::Fr;
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use diggerchain_core::{Point, vault as world};

use crate::circuit::{any, encode, number};

#[derive(Clone)]
pub struct VaultCircuit {
    pub public: [Fr; 11],
    pub actions: Vec<u8>,
    pub max_steps: usize,
}

impl VaultCircuit {
    pub fn fixture(seed: u64, max_steps: usize) -> Result<Self, String> {
        if ![256, 512].contains(&max_steps) {
            return Err("The generated step bound must be 256 or 512.".into());
        }
        Self::from_actions(
            seed,
            world::demonstration(seed, max_steps, true)?,
            max_steps,
        )
    }

    fn from_actions(
        seed: u64,
        actions: Vec<world::Action>,
        max_steps: usize,
    ) -> Result<Self, String> {
        if actions.len() > max_steps || max_steps > world::MAX_ACTIONS {
            return Err("The action history exceeds the generated step bound.".into());
        }
        let mut game = world::Expedition::new(seed);
        for action in &actions {
            game.apply(*action).map_err(|error| error.to_string())?;
        }
        Ok(Self {
            public: [
                Fr::from(world::RULES_VERSION),
                Fr::from(seed),
                Fr::from(46630u64),
                Fr::from(0u64),
                Fr::from(1u64),
                Fr::from(game.score()),
                Fr::from(game.charge()),
                Fr::from(game.floor() as u64),
                Fr::from(game.position().x as u64),
                Fr::from(game.position().y as u64),
                Fr::from(game.turns() as u64),
            ],
            actions: actions.into_iter().map(encode).collect(),
            max_steps,
        })
    }

    pub fn forged_actions(&self) -> Vec<(&'static str, Self)> {
        let mut cases = Vec::new();
        if let Some(i) = self.actions.iter().position(|id| (5..=8).contains(id)) {
            let mut walking = self.clone();
            walking.actions[i] -= 4;
            cases.push(("walk_through_generated_rock", walking));
            if i + 1 < self.actions.len() {
                let mut repeated = self.clone();
                repeated.actions[i + 1] = self.actions[i];
                cases.push(("dig_opened_generated_rock", repeated));
            }
        }
        if let Some(i) = self.actions.iter().position(|id| *id == 9)
            && i + 1 < self.actions.len()
        {
            let mut repeated = self.clone();
            repeated.actions[i + 1] = 9;
            cases.push(("consume_generated_flask_twice", repeated));
        }
        let mut no_refills = self.clone();
        for action in &mut no_refills.actions {
            if *action == 9 {
                *action = 11;
            }
        }
        cases.push(("omit_all_required_refills", no_refills));
        let mut unknown = self.clone();
        unknown.actions[0] = 12;
        cases.push(("unknown_action", unknown));
        cases
    }
}

fn at(x: &FpVar<Fr>, y: &FpVar<Fr>, point: Point) -> Result<Boolean<Fr>, SynthesisError> {
    Ok(x.is_eq(&number(point.x as u64))? & y.is_eq(&number(point.y as u64))?)
}

fn coordinate(bits: &[Boolean<Fr>], center: i32) -> Result<FpVar<Fr>, SynthesisError> {
    let low = bits[0].select(&number((center - 1) as u64), &number((center - 2) as u64))?;
    let high = bits[0].select(&number((center + 2) as u64), &number((center + 1) as u64))?;
    bits[1].select(&high, &low)
}

fn room_ground(x: &FpVar<Fr>, y: &FpVar<Fr>) -> Result<Boolean<Fr>, SynthesisError> {
    let xs = (1..=17)
        .filter(|value| *value != 6 && *value != 12)
        .map(|value| x.is_eq(&number(value)))
        .collect::<Result<Vec<_>, _>>()?;
    let ys = (1..=11)
        .filter(|value| *value != 6)
        .map(|value| y.is_eq(&number(value)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(any(&xs) & any(&ys))
}

impl ConstraintSynthesizer<Fr> for VaultCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        if self.actions.len() > self.max_steps || self.max_steps > world::MAX_ACTIONS {
            return Err(SynthesisError::Unsatisfiable);
        }
        let public = self
            .public
            .iter()
            .map(|value| FpVar::new_input(cs.clone(), || Ok(*value)))
            .collect::<Result<Vec<_>, _>>()?;
        for input in &public[2..5] {
            let _ = input.to_bits_le_with_top_bits_zero(64)?;
        }
        self.enforce_gameplay(cs, &public)
    }
}

impl VaultCircuit {
    pub(crate) fn enforce_gameplay(
        self,
        cs: ConstraintSystemRef<Fr>,
        public: &[FpVar<Fr>],
    ) -> Result<(), SynthesisError> {
        if public.len() != 11
            || self.actions.len() > self.max_steps
            || self.max_steps > world::MAX_ACTIONS
        {
            return Err(SynthesisError::Unsatisfiable);
        }
        public[0].enforce_equal(&number(world::RULES_VERSION))?;
        let (seed, _) = public[1].to_bits_le_with_top_bits_zero(64)?;

        let mut crystals = Vec::new();
        for floor in 0..world::FLOORS {
            for slot in 0..world::CRYSTAL_ROOMS.len() {
                let center = world::ROOMS[world::CRYSTAL_ROOMS[slot]];
                let shift = 27 + floor * 12 + slot * 4;
                let enabled = match (floor, slot) {
                    (0, _) | (_, 0) => Boolean::TRUE,
                    (1, slot) => seed[7 + slot].clone(),
                    (_, slot) => &seed[14 + slot * 2] & &seed[15 + slot * 2],
                };
                crystals.push((
                    floor,
                    coordinate(&seed[shift..shift + 2], center.x)?,
                    coordinate(&seed[shift + 2..shift + 4], center.y)?,
                    enabled,
                ));
            }
        }
        let mut opened = vec![Boolean::FALSE; world::FLOORS * world::GATES.len()];
        let mut consumed = vec![Boolean::FALSE; world::FLOORS * world::ROOMS.len()];
        let mut collected = vec![Boolean::FALSE; crystals.len()];
        let mut floor = number(0);
        let mut x = number(world::ENTRY.x as u64);
        let mut y = number(world::ENTRY.y as u64);
        let mut charge = number(world::INITIAL_CHARGE as u64);
        let mut score = number(0);
        let mut turns = number(0);

        for step in 0..self.max_steps {
            let action = self.actions.get(step).copied().unwrap_or(0);
            let selected = (0..12)
                .map(|id| Boolean::new_witness(cs.clone(), || Ok(action == id)))
                .collect::<Result<Vec<_>, _>>()?;
            selected
                .iter()
                .fold(number(0), |sum, bit| sum + FpVar::from(bit.clone()))
                .enforce_equal(&number(1))?;
            let moving = any(&selected[1..5]);
            let digging = any(&selected[5..9]);
            let interacting = &selected[9];
            let portal = &selected[10];
            let mx = &x + FpVar::from(selected[4].clone()) - FpVar::from(selected[3].clone());
            let my = &y + FpVar::from(selected[2].clone()) - FpVar::from(selected[1].clone());
            let dx = &x + FpVar::from(selected[8].clone()) - FpVar::from(selected[7].clone());
            let dy = &y + FpVar::from(selected[6].clone()) - FpVar::from(selected[5].clone());
            let levels = (0..world::FLOORS)
                .map(|level| floor.is_eq(&number(level as u64)))
                .collect::<Result<Vec<_>, _>>()?;
            let mut can_walk = room_ground(&mx, &my)?;
            let mut can_dig = Boolean::FALSE;
            let mut can_refill = Boolean::FALSE;
            let mut dig_cost = number(0);

            for (level, on_level) in levels.iter().enumerate() {
                for (gate, location) in world::GATES.iter().enumerate() {
                    let i = level * world::GATES.len() + gate;
                    let enabled = if gate < 5 {
                        Boolean::TRUE
                    } else {
                        seed[21 + level * 2 + gate - 5].clone()
                    };
                    let on_move = on_level & at(&mx, &my, *location)?;
                    can_walk |= on_move & &enabled & &opened[i];
                    let on_dig = on_level & at(&dx, &dy, *location)?;
                    let available = &enabled & !&opened[i];
                    can_dig |= &on_dig & &available;
                    let opening = &digging & on_dig & available;
                    let cost = number(2) + FpVar::from(seed[level * 7 + gate].clone());
                    dig_cost += FpVar::from(opening.clone()) * cost;
                    opened[i] |= opening;
                }
                for (room, location) in world::ROOMS.iter().enumerate() {
                    let i = level * world::ROOMS.len() + room;
                    let enabled = if room == 0 { !&seed[63] } else { Boolean::TRUE };
                    let on_flask = on_level & at(&x, &y, *location)?;
                    let available = enabled & !&consumed[i];
                    can_refill |= &on_flask & &available;
                    consumed[i] |= interacting & on_flask & available;
                }
            }
            for (i, (level, cx, cy, enabled)) in crystals.iter().enumerate() {
                let on_crystal = &levels[*level] & mx.is_eq(cx)? & my.is_eq(cy)?;
                let taking = &moving & on_crystal & enabled & !&collected[i];
                score += FpVar::from(taking.clone()) * Fr::from(world::crystal_value(*level));
                collected[i] |= taking;
            }

            let up = at(&x, &y, world::ENTRY)? & !&levels[0];
            let down = at(&x, &y, world::EXIT)? & !&levels[world::FLOORS - 1];
            (&moving & !can_walk).enforce_equal(&Boolean::FALSE)?;
            (&digging & !can_dig).enforce_equal(&Boolean::FALSE)?;
            (interacting & !can_refill).enforce_equal(&Boolean::FALSE)?;
            (portal & !(&up | &down)).enforce_equal(&Boolean::FALSE)?;
            (interacting & charge.is_eq(&number(world::CAPACITY as u64))?)
                .enforce_equal(&Boolean::FALSE)?;
            let low_charge = (0..=4)
                .map(|value| charge.is_eq(&number(value)))
                .collect::<Result<Vec<_>, _>>()?;
            let replenished = any(&low_charge).select(
                &(&charge + Fr::from(world::REFILL)),
                &number(world::CAPACITY as u64),
            )?;
            charge = interacting.select(&replenished, &(&charge - dig_cost))?;
            let (bits, _) = charge.to_bits_le_with_top_bits_zero(4)?;
            (&bits[3] & &bits[2] & (&bits[1] | &bits[0])).enforce_equal(&Boolean::FALSE)?;
            let next_level = &floor + FpVar::from(down.clone()) - FpVar::from(up);
            floor = portal.select(&next_level, &floor)?;
            x = portal.select(&number(world::ENTRY.x as u64), &mx)?;
            let portal_y = down.select(
                &number(world::ENTRY.y as u64),
                &number(world::EXIT.y as u64),
            )?;
            y = portal.select(&portal_y, &my)?;
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

    fn satisfied(circuit: VaultCircuit) -> bool {
        let cs = ConstraintSystem::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap(),
            Err(SynthesisError::Unsatisfiable) => false,
            Err(error) => panic!("Unexpected synthesis error: {error}"),
        }
    }

    fn prefix(seed: u64) -> VaultCircuit {
        let actions = world::demonstration(seed, world::MAX_ACTIONS, false).unwrap();
        VaultCircuit::from_actions(seed, actions.into_iter().take(48).collect(), 64).unwrap()
    }

    #[test]
    fn generated_transitions_match_native_results_for_different_seeds() {
        for seed in [0, 42, u64::MAX] {
            assert!(satisfied(prefix(seed)), "seed {seed}");
        }
    }

    #[test]
    fn generated_setup_and_proving_use_identical_constraint_matrices() {
        let setup = ConstraintSystem::new_ref();
        setup.set_mode(SynthesisMode::Setup);
        prefix(0).generate_constraints(setup.clone()).unwrap();
        let proving = ConstraintSystem::new_ref();
        prefix(u64::MAX)
            .generate_constraints(proving.clone())
            .unwrap();
        assert!(proving.is_satisfied().unwrap());
        setup.finalize();
        proving.finalize();
        assert!(setup.to_matrices().unwrap() == proving.to_matrices().unwrap());
    }

    #[test]
    fn generated_results_cannot_be_fabricated() {
        let valid = prefix(0);
        for field in 5..11 {
            let mut forged = valid.clone();
            forged.public[field] += Fr::from(1u64);
            assert!(!satisfied(forged), "public field {field}");
        }
    }

    #[test]
    fn generated_rocks_and_flasks_cannot_be_used_illegally() {
        let valid = prefix(0);
        for (name, forged) in valid.forged_actions() {
            assert!(!satisfied(forged), "{name}");
        }
    }

    #[test]
    fn generated_seed_and_context_have_canonical_widths() {
        for field in 1..5 {
            let mut forged = prefix(0);
            let mut overflow = Fr::from(1u64);
            for _ in 0..64 {
                overflow += overflow;
            }
            forged.public[field] = overflow;
            assert!(!satisfied(forged));
        }
    }
}
