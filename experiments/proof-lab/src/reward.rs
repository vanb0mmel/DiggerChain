use ark_bn254::Fr;
use ark_crypto_primitives::crh::sha256::constraints::Sha256Gadget;
use ark_ff::{BigInteger, PrimeField};
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};
use diggerchain_core::{
    record::{Record, hex},
    vault as world,
};
use sha2::{Digest, Sha256};

use crate::{circuit::number, vault::VaultCircuit};

pub const INPUT_NAMES: [&str; 14] = [
    "rules_version",
    "seed",
    "chain_id",
    "epoch",
    "expedition_id_high",
    "score",
    "charge",
    "floor",
    "x",
    "y",
    "turns",
    "expedition_id_low",
    "recipient",
    "settlement_contract",
];

pub fn matrix_fingerprint(matrices: &ConstraintMatrices<Fr>) -> String {
    let mut digest = Sha256::new();
    digest.update(b"DiggerChain/reward-v1/BN254/R1CS-matrices");
    for count in [
        matrices.num_instance_variables,
        matrices.num_witness_variables,
        matrices.num_constraints,
    ] {
        digest.update((count as u64).to_be_bytes());
    }
    for matrix in [&matrices.a, &matrices.b, &matrices.c] {
        digest.update((matrix.len() as u64).to_be_bytes());
        for row in matrix {
            digest.update((row.len() as u64).to_be_bytes());
            for (coefficient, column) in row {
                digest.update(coefficient.into_bigint().to_bytes_le());
                digest.update((*column as u64).to_be_bytes());
            }
        }
    }
    hex(&digest.finalize())
}

#[derive(Clone)]
pub struct RewardCircuit {
    pub public: [Fr; 14],
    pub secret: [u8; 32],
    pub actions: Vec<u8>,
    pub max_steps: usize,
}

impl RewardCircuit {
    pub fn from_record(
        record: &Record,
        chain: u64,
        epoch: u64,
        recipient: [u8; 20],
        contract: [u8; 20],
    ) -> Result<Self, String> {
        if !record.finished {
            return Err("Seal the expedition before proving it.".into());
        }
        let secret = record
            .secret
            .ok_or("Practice expeditions cannot produce reward proofs.")?;
        let game = record.replay()?;
        let id = record.id().expect("a private expedition has an identity");
        Ok(Self {
            public: [
                Fr::from(world::RULES_VERSION),
                Fr::from(record.seed),
                Fr::from(chain),
                Fr::from(epoch),
                Fr::from_be_bytes_mod_order(&id[..16]),
                Fr::from(game.score()),
                Fr::from(game.charge()),
                Fr::from(game.floor() as u64),
                Fr::from(game.position().x as u64),
                Fr::from(game.position().y as u64),
                Fr::from(game.turns() as u64),
                Fr::from_be_bytes_mod_order(&id[16..]),
                Fr::from_be_bytes_mod_order(&recipient),
                Fr::from_be_bytes_mod_order(&contract),
            ],
            secret,
            actions: record.actions.clone(),
            max_steps: world::MAX_ACTIONS,
        })
    }

    pub fn setup_fixture() -> Self {
        let mut record = Record::new([0; 32]);
        record.finished = true;
        Self::from_record(&record, 46630, 0, [1; 20], [2; 20]).expect("canonical setup fixture")
    }
}

fn packed_be(bytes: &[UInt8<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
    let mut value = number(0);
    for byte in bytes {
        value = value * Fr::from(256u64) + Boolean::le_bits_to_fp(&byte.to_bits_le()?)?;
    }
    Ok(value)
}

impl ConstraintSynthesizer<Fr> for RewardCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let public = self
            .public
            .iter()
            .map(|value| FpVar::new_input(cs.clone(), || Ok(*value)))
            .collect::<Result<Vec<_>, _>>()?;
        for (index, bits) in [(2, 64), (3, 64), (4, 128), (11, 128), (12, 160), (13, 160)] {
            let _ = public[index].to_bits_le_with_top_bits_zero(bits)?;
        }
        let secret = UInt8::new_witness_vec(cs.clone(), &self.secret)?;
        let digest = Sha256Gadget::digest(&secret)?.0;
        packed_be(&digest[..16])?.enforce_equal(&public[4])?;
        packed_be(&digest[16..])?.enforce_equal(&public[11])?;
        packed_be(&digest[..8])?.enforce_equal(&public[1])?;
        VaultCircuit {
            public: self.public[..11]
                .try_into()
                .expect("eleven gameplay inputs"),
            actions: self.actions,
            max_steps: self.max_steps,
        }
        .enforce_gameplay(cs, &public[..11])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::{ConstraintSystem, SynthesisMode};

    fn sample(secret: u8) -> RewardCircuit {
        let mut record = Record::new([secret; 32]);
        record.actions = world::demonstration(record.seed, world::MAX_ACTIONS, false)
            .unwrap()
            .into_iter()
            .take(32)
            .map(world::Action::code)
            .collect();
        record.finished = true;
        let mut circuit = RewardCircuit::from_record(&record, 46630, 9, [1; 20], [2; 20]).unwrap();
        circuit.max_steps = 64;
        circuit
    }

    fn satisfied(circuit: RewardCircuit) -> bool {
        let cs = ConstraintSystem::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap(),
            Err(SynthesisError::Unsatisfiable) => false,
            Err(error) => panic!("Unexpected synthesis error: {error}"),
        }
    }

    #[test]
    fn authenticated_history_binds_sha256_identity_and_seed() {
        let valid = sample(7);
        assert!(satisfied(valid.clone()));
        let mut forged = valid.clone();
        forged.secret[0] ^= 1;
        assert!(!satisfied(forged));
        for index in [1, 4, 5, 11] {
            let mut forged = valid.clone();
            forged.public[index] += Fr::from(1u64);
            assert!(!satisfied(forged));
        }
    }

    #[test]
    fn release_context_has_canonical_widths_and_practice_runs_are_rejected() {
        for (index, width) in [(2, 64), (3, 64), (4, 128), (11, 128), (12, 160), (13, 160)] {
            let mut forged = sample(7);
            let mut overflow = Fr::from(1u64);
            for _ in 0..width {
                overflow += overflow;
            }
            forged.public[index] = overflow;
            assert!(!satisfied(forged));
        }
        let mut practice = Record::practice(0);
        practice.finished = true;
        assert!(RewardCircuit::from_record(&practice, 46630, 0, [1; 20], [2; 20]).is_err());
    }

    #[test]
    fn authenticated_setup_and_proving_have_identical_matrices() {
        let setup = ConstraintSystem::new_ref();
        setup.set_mode(SynthesisMode::Setup);
        let mut fixture = RewardCircuit::setup_fixture();
        fixture.max_steps = 64;
        fixture.generate_constraints(setup.clone()).unwrap();
        let proving = ConstraintSystem::new_ref();
        sample(7).generate_constraints(proving.clone()).unwrap();
        assert!(proving.is_satisfied().unwrap());
        setup.finalize();
        proving.finalize();
        let setup_matrices = setup.to_matrices().unwrap();
        let proving_matrices = proving.to_matrices().unwrap();
        assert!(setup_matrices == proving_matrices);
        assert_eq!(
            matrix_fingerprint(&setup_matrices),
            matrix_fingerprint(&proving_matrices)
        );
        let shorter = ConstraintSystem::new_ref();
        let mut circuit = sample(7);
        circuit.max_steps = 32;
        circuit.generate_constraints(shorter.clone()).unwrap();
        shorter.finalize();
        assert_ne!(
            matrix_fingerprint(&setup_matrices),
            matrix_fingerprint(&shorter.to_matrices().unwrap())
        );
    }
}
