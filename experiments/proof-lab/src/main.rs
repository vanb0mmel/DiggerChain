use diggerchain_proof_lab::{circuit, evm, vault, write_new};

use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, prepare_verifying_key};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, SynthesisError};
use ark_serialize::{CanonicalSerialize, Compress};
use circuit::{INPUT_NAMES, RunCircuit};
use rand::rngs::OsRng;
use serde_json::json;
use std::{error::Error, fs, path::Path, time::Instant};

#[derive(Clone)]
enum GameplayCircuit {
    Authored(RunCircuit),
    Generated(vault::VaultCircuit),
}

impl GameplayCircuit {
    fn public(&self) -> [Fr; 11] {
        match self {
            Self::Authored(run) => run.public,
            Self::Generated(run) => run.public,
        }
    }

    fn action_count(&self) -> usize {
        match self {
            Self::Authored(run) => run.actions.len(),
            Self::Generated(run) => run.actions.len(),
        }
    }
}

impl ConstraintSynthesizer<Fr> for GameplayCircuit {
    fn generate_constraints(
        self,
        cs: ark_relations::r1cs::ConstraintSystemRef<Fr>,
    ) -> Result<(), SynthesisError> {
        match self {
            Self::Authored(run) => run.generate_constraints(cs),
            Self::Generated(run) => run.generate_constraints(cs),
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Proof lab failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "Usage: diggerchain-proof-lab SEED MAX_STEPS ROUND_TRIPS OUTPUT_DIRECTORY\n       diggerchain-proof-lab generated SEED MAX_STEPS OUTPUT_DIRECTORY".into(),
        );
    }
    let generated = args[0] == "generated";
    let seed: u64 = args[usize::from(generated)].parse()?;
    let max_steps: usize = args[1 + usize::from(generated)].parse()?;
    let trips: usize = if generated { 0 } else { args[2].parse()? };
    let directory = Path::new(&args[3]);
    fs::create_dir_all(directory)?;
    let started = Instant::now();
    let circuit = if generated {
        GameplayCircuit::Generated(vault::VaultCircuit::fixture(seed, max_steps)?)
    } else {
        GameplayCircuit::Authored(RunCircuit::fixture(seed.try_into()?, max_steps, trips)?)
    };
    let public = circuit.public();
    eprintln!("Checking the canonical gameplay route...");
    let cs = ConstraintSystem::new_ref();
    circuit
        .clone()
        .generate_constraints(cs.clone())
        .map_err(|error| format!("Canonical route synthesis failed: {error}"))?;
    if !cs.is_satisfied()? {
        return Err("The native route does not satisfy the circuit.".into());
    }
    let constraints = cs.num_constraints();
    let witness_variables = cs.num_witness_variables();
    drop(cs);
    eprintln!("Canonical route accepted; checking forged witnesses...");
    let rejected_witness_cases = match &circuit {
        GameplayCircuit::Authored(run) => check_rejections(run)?,
        GameplayCircuit::Generated(run) => check_generated_rejections(run)?,
    };
    eprintln!(
        "Rejected all {} forged witnesses.",
        rejected_witness_cases.len()
    );
    let constraint_check_ms = started.elapsed().as_millis();

    eprintln!("Generating a local experiment-only Groth16 setup...");
    let mut rng = OsRng;
    let setup_started = Instant::now();
    let parameters =
        Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit.clone(), &mut rng)?;
    let setup_ms = setup_started.elapsed().as_millis();
    eprintln!("Proving the legal run...");
    let prove_started = Instant::now();
    let proof = Groth16::<Bn254>::create_random_proof_with_reduction(
        circuit.clone(),
        &parameters,
        &mut rng,
    )?;
    let prove_ms = prove_started.elapsed().as_millis();
    let verify_started = Instant::now();
    let prepared = prepare_verifying_key(&parameters.vk);
    if !Groth16::<Bn254>::verify_proof(&prepared, &proof, &public)? {
        return Err("The generated proof failed native verification.".into());
    }
    let verify_ms = verify_started.elapsed().as_millis();
    for (i, name) in INPUT_NAMES.iter().enumerate() {
        let mut altered = public;
        altered[i] += Fr::from(1u64);
        if Groth16::<Bn254>::verify_proof(&prepared, &proof, &altered)? {
            return Err(format!("An altered public input was accepted: {name}").into());
        }
    }
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    let mut vk_bytes = Vec::new();
    parameters.vk.serialize_compressed(&mut vk_bytes)?;
    write_new(&directory.join("proof.bin"), &proof_bytes)?;
    write_new(&directory.join("verifying-key.bin"), &vk_bytes)?;
    evm::export(directory, &parameters.vk, &proof, &public)?;
    let public_inputs: Vec<_> = public
        .iter()
        .map(|value| value.into_bigint().to_string())
        .collect();
    let peak_rss_kib = fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| parse_peak_rss_kib(&status));
    let report = json!({
        "experiment": if generated { "vault-v1-feasibility-groth16-bn254" } else { "lab-v0-groth16-bn254" },
        "cryptographic_proof": true,
        "production_ready": false,
        "native_verification": "passed",
        "evm_verification": "not_run",
        "robinhood_testnet_verification": "not_run",
        "setup": "fresh_local_experiment_only_not_a_production_ceremony",
        "seed": seed,
        "max_steps": max_steps,
        "round_trips": trips,
        "successful_actions": circuit.action_count(),
        "rejected_demo_actions_omitted": if generated { 0 } else { 3 },
        "world": if generated { Some(json!({
            "generator": "seeded-castle-v1",
            "width": diggerchain_core::vault::WIDTH,
            "height": diggerchain_core::vault::HEIGHT,
            "floors": diggerchain_core::vault::FLOORS,
            "seed_bits": 64,
            "generation_enforced_by_circuit": true
        })) } else { None },
        "constraints_before_prover_optimization": constraints,
        "witness_variables_before_prover_optimization": witness_variables,
        "rejected_witness_cases": rejected_witness_cases,
        "altered_public_inputs_rejected": INPUT_NAMES.len(),
        "timings_ms": {
            "constraint_checks": constraint_check_ms,
            "setup": setup_ms,
            "prove": prove_ms,
            "native_verify_including_key_preparation": verify_ms,
            "total_before_report": started.elapsed().as_millis()
        },
        "memory": {
            "peak_rss_kib_before_report": peak_rss_kib,
            "source": "/proc/self/status:VmHWM"
        },
        "sizes_bytes": {
            "compressed_proof": proof_bytes.len(),
            "compressed_verifying_key": vk_bytes.len(),
            "compressed_proving_key_not_saved": parameters.serialized_size(Compress::Yes),
            "evm_proof": 8 * 32,
            "evm_static_call_data_including_selector": (8 + INPUT_NAMES.len()) * 32 + 4
        },
        "public_input_names": INPUT_NAMES,
        "public_inputs_decimal": public_inputs
    });
    let report = serde_json::to_string_pretty(&report)?;
    write_new(&directory.join("report.json"), report.as_bytes())?;
    println!("{report}");
    Ok(())
}

fn check_rejections(valid: &RunCircuit) -> Result<Vec<String>, Box<dyn Error>> {
    let mut checked = Vec::new();
    for (field, input_name) in INPUT_NAMES.iter().enumerate().skip(5) {
        let name = format!("fabricated_{input_name}");
        let mut forged = valid.clone();
        forged.public[field] += Fr::from(1u64);
        require_rejection(forged)
            .map_err(|error| format!("Rejection case {name} failed: {error}"))?;
        checked.push(name);
    }
    for (name, index, action) in [
        ("walk_through_rock", 0, 4),
        ("dig_opened_rock", 1, 8),
        ("skip_required_refill", 6, 11),
        ("consume_flask_twice", 25, 9),
        ("forget_opened_rock_after_backtracking", 28, 5),
        ("unknown_action", 0, 12),
    ] {
        let mut forged = valid.clone();
        let physical_action = if seed_is_mirrored(valid) {
            match action {
                3 => 4,
                4 => 3,
                7 => 8,
                8 => 7,
                other => other,
            }
        } else {
            action
        };
        forged.actions[index] = physical_action;
        require_rejection(forged)
            .map_err(|error| format!("Rejection case {name} failed: {error}"))?;
        checked.push(name.into());
    }
    Ok(checked)
}

fn seed_is_mirrored(circuit: &RunCircuit) -> bool {
    use ark_ff::BigInteger;
    circuit.public[1].into_bigint().get_bit(0)
}

fn check_generated_rejections(valid: &vault::VaultCircuit) -> Result<Vec<String>, Box<dyn Error>> {
    let mut checked = Vec::new();
    for (field, input_name) in INPUT_NAMES.iter().enumerate().skip(5) {
        let name = format!("fabricated_{input_name}");
        let mut forged = valid.clone();
        forged.public[field] += Fr::from(1u64);
        require_rejection(forged)
            .map_err(|error| format!("Rejection case {name} failed: {error}"))?;
        checked.push(name);
    }
    for (name, forged) in valid.forged_actions() {
        require_rejection(forged)
            .map_err(|error| format!("Rejection case {name} failed: {error}"))?;
        checked.push(name.into());
    }
    Ok(checked)
}

fn require_rejection(circuit: impl ConstraintSynthesizer<Fr>) -> Result<(), Box<dyn Error>> {
    let cs = ConstraintSystem::new_ref();
    match circuit.generate_constraints(cs.clone()) {
        Err(SynthesisError::Unsatisfiable) => return Ok(()),
        Err(error) => return Err(error.into()),
        Ok(()) => {}
    }
    if cs.is_satisfied()? {
        return Err("A forged gameplay witness satisfied the circuit.".into());
    }
    Ok(())
}

fn parse_peak_rss_kib(status: &str) -> Option<u64> {
    let value = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))?;
    let mut fields = value.split_whitespace();
    let peak = fields.next()?.parse().ok()?;
    (fields.next()? == "kB").then_some(peak)
}

#[cfg(test)]
mod tests {
    use super::parse_peak_rss_kib;

    #[test]
    fn peak_memory_uses_the_high_water_mark_and_preserves_missing_data() {
        assert_eq!(
            parse_peak_rss_kib("Name:\tproof-lab\nVmHWM:\t 123456 kB\nVmRSS:\t42 kB\n"),
            Some(123456)
        );
        assert_eq!(parse_peak_rss_kib("VmRSS: 42 kB\n"), None);
        assert_eq!(parse_peak_rss_kib("VmHWM: unknown kB\n"), None);
        assert_eq!(parse_peak_rss_kib("VmHWM: 123 MB\n"), None);
    }
}
