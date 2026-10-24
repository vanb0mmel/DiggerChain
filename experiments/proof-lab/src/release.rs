use std::{
    error::Error,
    fs,
    io::{Cursor, Read},
    path::Path,
    time::Instant,
};

use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, ProvingKey, prepare_verifying_key};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, SynthesisMode};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use diggerchain_core::{
    record::{Record, hex},
    vault::MAX_ACTIONS,
};
use diggerchain_proof_lab::{
    evm,
    reward::{INPUT_NAMES, RewardCircuit, matrix_fingerprint},
    write_new,
};
use rand::rngs::OsRng;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SETUP_NOTE: &str =
    "DiggerChain fixed-parameter verifier. Security assumes an honest single setup operator.";
const HELP: &str = "DiggerChain reward proof tool

  diggerchain-proof setup OUTPUT_DIRECTORY
  diggerchain-proof prove SAVE PARAMETERS_DIRECTORY CHAIN_ID EPOCH RECIPIENT CONTRACT OUTPUT_DIRECTORY

Setup runs once on the release operator's machine, after freezing the circuit.
Proving uses that fixed public key. It never creates replacement parameters.
SAVE is private. Output proof files contain public results, never its secret.
Only Robinhood mainnet 4663 and testnet 46630 are accepted.
";

fn main() {
    if let Err(error) = run() {
        eprintln!("DiggerChain proof: {error}");
        std::process::exit(1);
    }
}

fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn source_fingerprint() -> String {
    let mut digest = Sha256::new();
    for source in [
        include_str!("reward.rs"),
        include_str!("vault.rs"),
        include_str!("circuit.rs"),
        include_str!("../../../crates/diggerchain-core/src/vault.rs"),
        include_str!("../../../crates/diggerchain-core/src/record.rs"),
        include_str!("../../../crates/diggerchain-core/src/expedition.rs"),
        include_str!("../Cargo.lock"),
    ] {
        digest.update((source.len() as u64).to_be_bytes());
        digest.update(source);
    }
    hex(&digest.finalize())
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err(format!("File exceeds its size limit: {}", path.display()).into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("File grew beyond its size limit.".into());
    }
    Ok(bytes)
}

fn directory(path: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(path)
        .map_err(|error| format!("Use a new output directory {}: {error}", path.display()))?;
    Ok(())
}

fn serialize(value: &impl CanonicalSerialize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    value.serialize_compressed(&mut bytes)?;
    Ok(bytes)
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn Error>> {
    write_new(path, &serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn setup(output: &Path) -> Result<(), Box<dyn Error>> {
    directory(output)?;
    eprintln!(
        "Generating fixed parameters. Trust assumption: one honest setup operator and machine."
    );
    let started = Instant::now();
    let cs = ConstraintSystem::new_ref();
    cs.set_mode(SynthesisMode::Setup);
    RewardCircuit::setup_fixture().generate_constraints(cs.clone())?;
    cs.finalize();
    let circuit_fingerprint = matrix_fingerprint(
        &cs.to_matrices()
            .ok_or("Missing setup constraint matrices.")?,
    );
    drop(cs);
    let key = Groth16::<Bn254>::generate_random_parameters_with_reduction(
        RewardCircuit::setup_fixture(),
        &mut OsRng,
    )?;
    let proving = serialize(&key)?;
    let verifying = serialize(&key.vk)?;
    write_new(&output.join("proving-key.bin"), &proving)?;
    write_new(&output.join("verifying-key.bin"), &verifying)?;
    evm::export_verifier(output, &key.vk, INPUT_NAMES.len(), SETUP_NOTE)?;
    let manifest = json!({
        "format_version": 1, "scheme": "groth16-bn254-reward-v1", "max_actions": MAX_ACTIONS,
        "rules_version": diggerchain_core::vault::RULES_VERSION, "public_input_names": INPUT_NAMES,
        "circuit_sha256": circuit_fingerprint, "source_sha256": source_fingerprint(), "proving_key_sha256": hash(&proving),
        "verifying_key_sha256": hash(&verifying), "setup_assumption": "single_operator_trusted_setup",
        "verifier_source_sha256": hash(&fs::read(output.join("src/Verifier.sol"))?),
        "status": "candidate_parameters_require_user_validation", "setup_ms": started.elapsed().as_millis(),
        "proving_key_bytes": proving.len(), "verifying_key_bytes": verifying.len(),
    });
    write_json(&output.join("manifest.json"), &manifest)?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}

fn load_key(
    path: &Path,
    circuit_fingerprint: &str,
) -> Result<(ProvingKey<Bn254>, Value), Box<dyn Error>> {
    let manifest: Value =
        serde_json::from_slice(&read_limited(&path.join("manifest.json"), 65536)?)?;
    if manifest["format_version"] != 1
        || manifest["scheme"] != "groth16-bn254-reward-v1"
        || manifest["rules_version"] != diggerchain_core::vault::RULES_VERSION
        || manifest["max_actions"] != MAX_ACTIONS
        || manifest["circuit_sha256"] != circuit_fingerprint
        || manifest["public_input_names"] != json!(INPUT_NAMES)
    {
        return Err("Parameter manifest is incompatible with this proof tool; no replacement setup was generated.".into());
    }
    let bytes = read_limited(&path.join("proving-key.bin"), 256 * 1024 * 1024)?;
    if manifest["proving_key_sha256"] != hash(&bytes) {
        return Err("Proving-key fingerprint mismatch.".into());
    }
    let mut reader = Cursor::new(&bytes);
    let key = ProvingKey::<Bn254>::deserialize_compressed(&mut reader)?;
    if reader.position() as usize != bytes.len()
        || key.vk.gamma_abc_g1.len() != INPUT_NAMES.len() + 1
    {
        return Err("Malformed proving key or unexpected public input count.".into());
    }
    let verifying = read_limited(&path.join("verifying-key.bin"), 65536)?;
    if verifying != serialize(&key.vk)? || manifest["verifying_key_sha256"] != hash(&verifying) {
        return Err("Verifying-key fingerprint mismatch.".into());
    }
    Ok((key, manifest))
}

fn address(text: &str) -> Result<[u8; 20], Box<dyn Error>> {
    let text = text.strip_prefix("0x").unwrap_or(text);
    if text.len() != 40 || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Address must contain exactly 20 hexadecimal bytes.".into());
    }
    let mut bytes = [0; 20];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    if bytes == [0; 20] {
        return Err("The zero address is not a recipient or settlement contract.".into());
    }
    Ok(bytes)
}

fn prove(args: &[String]) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let record: Record = serde_json::from_slice(&read_limited(Path::new(&args[0]), 65536)?)?;
    let chain: u64 = args[2].parse()?;
    if ![4663, 46630].contains(&chain) {
        return Err("Unsupported chain; use Robinhood 4663 or testnet 46630.".into());
    }
    let circuit = RewardCircuit::from_record(
        &record,
        chain,
        args[3].parse()?,
        address(&args[4])?,
        address(&args[5])?,
    )?;
    if circuit.public[5] == Fr::from(0u64) {
        return Err("This expedition has no crystals to submit.".into());
    }
    let check_started = Instant::now();
    eprintln!("Checking the saved expedition and private ownership witness...");
    let cs = ConstraintSystem::new_ref();
    circuit.clone().generate_constraints(cs.clone())?;
    if !cs.is_satisfied()? {
        return Err("Saved expedition failed the reward constraints.".into());
    }
    let constraints = cs.num_constraints();
    cs.finalize();
    let circuit_fingerprint = matrix_fingerprint(
        &cs.to_matrices()
            .ok_or("Missing proving constraint matrices.")?,
    );
    drop(cs);
    let checked_ms = check_started.elapsed().as_millis();
    let key_started = Instant::now();
    let (parameters, manifest) = load_key(Path::new(&args[1]), &circuit_fingerprint)?;
    let key_load_ms = key_started.elapsed().as_millis();
    let output = Path::new(&args[6]);
    directory(output)?;
    eprintln!("Proving with fixed release parameters; no setup is running...");
    let proof_started = Instant::now();
    let proof = Groth16::<Bn254>::create_random_proof_with_reduction(
        circuit.clone(),
        &parameters,
        &mut OsRng,
    )?;
    let prove_ms = proof_started.elapsed().as_millis();
    let prepared = prepare_verifying_key(&parameters.vk);
    if !Groth16::<Bn254>::verify_proof(&prepared, &proof, &circuit.public)? {
        return Err("Proof failed native verification.".into());
    }
    for i in 0..INPUT_NAMES.len() {
        let mut forged = circuit.public;
        forged[i] += Fr::from(1u64);
        if Groth16::<Bn254>::verify_proof(&prepared, &proof, &forged)? {
            return Err(format!("Altered public input accepted: {}", INPUT_NAMES[i]).into());
        }
    }
    write_new(&output.join("proof.bin"), &serialize(&proof)?)?;
    evm::export_named(
        output,
        &parameters.vk,
        &proof,
        &circuit.public,
        &INPUT_NAMES,
        SETUP_NOTE,
    )?;
    write_json(&output.join("parameters.json"), &manifest)?;
    let report = json!({
        "format_version": 1, "scheme": "groth16-bn254-reward-v1", "native_verification": "passed",
        "public_input_names": INPUT_NAMES,
        "public_inputs_decimal": circuit.public.iter().map(|v| v.into_bigint().to_string()).collect::<Vec<_>>(),
        "expedition_id": record.id().map(|id| hex(&id)), "successful_actions": record.actions.len(),
        "constraints": constraints, "parameter_fingerprint": manifest["verifying_key_sha256"], "circuit_sha256": circuit_fingerprint,
        "proof_tool_source_sha256": source_fingerprint(),
        "timings_ms": {"key_load": key_load_ms, "constraint_check": checked_ms, "prove": prove_ms, "total": started.elapsed().as_millis()},
        "peak_rss_kib": fs::read_to_string("/proc/self/status").ok().and_then(|status| {
            status.lines().find_map(|line| line.strip_prefix("VmHWM:")?.split_whitespace().next()?.parse::<u64>().ok())
        }),
        "altered_public_inputs_rejected": INPUT_NAMES.len(), "secret_exported": false,
    });
    write_json(&output.join("report.json"), &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|command| matches!(command.as_str(), "setup" | "prove"))
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build_global()?;
    }
    match args.first().map(String::as_str) {
        Some("setup") if args.len() == 2 => setup(Path::new(&args[1])),
        Some("prove") if args.len() == 8 => prove(&args[1..]),
        Some("--help" | "-h") => {
            print!("{HELP}");
            Ok(())
        }
        _ => Err(HELP.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn address_parser_rejects_zero_overflow_and_non_ascii() {
        assert!(address("0x87660E6337eFb749a3a6DE321F9D38ad587ed8FC").is_ok());
        for text in [
            "",
            "0x0",
            "0000000000000000000000000000000000000000",
            "gggggggggggggggggggggggggggggggggggggggg",
            "é000000000000000000000000000000000000000",
        ] {
            assert!(address(text).is_err());
        }
        assert_eq!(source_fingerprint().len(), 64);
    }
}
