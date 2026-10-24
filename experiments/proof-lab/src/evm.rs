use crate::{circuit::INPUT_NAMES, write_new};
use ark_bn254::{Bn254, Fr, G1Affine, G2Affine};
use ark_ff::PrimeField;
use ark_groth16::{Proof, VerifyingKey};
use serde_json::json;
use std::{error::Error, fs, path::Path};

fn g1(point: &G1Affine) -> [String; 2] {
    if point.infinity {
        ["0".into(), "0".into()]
    } else {
        [
            point.x.into_bigint().to_string(),
            point.y.into_bigint().to_string(),
        ]
    }
}

fn g2(point: &G2Affine) -> [String; 4] {
    if point.infinity {
        std::array::from_fn(|_| "0".into())
    } else {
        // EIP-197 encodes each Fq2 value with the imaginary coefficient first.
        [
            point.x.c1.into_bigint().to_string(),
            point.x.c0.into_bigint().to_string(),
            point.y.c1.into_bigint().to_string(),
            point.y.c0.into_bigint().to_string(),
        ]
    }
}

fn literals(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("uint256({value})"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn export(
    directory: &Path,
    key: &VerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    inputs: &[Fr; 11],
) -> Result<(), Box<dyn Error>> {
    export_named(
        directory,
        key,
        proof,
        inputs,
        &INPUT_NAMES,
        "Experiment-only verifier. Its local setup is unsuitable for funded rewards.",
    )
}

pub fn export_verifier(
    directory: &Path,
    key: &VerifyingKey<Bn254>,
    count: usize,
    note: &str,
) -> Result<(), Box<dyn Error>> {
    if key.gamma_abc_g1.len() != count + 1 {
        return Err("Unexpected Groth16 public input count.".into());
    }
    let ic: Vec<_> = key.gamma_abc_g1.iter().flat_map(g1).collect();
    let alpha = g1(&key.alpha_g1);
    let beta = g2(&key.beta_g2);
    let gamma = g2(&key.gamma_g2);
    let delta = g2(&key.delta_g2);
    let verifier = include_str!("verifier.sol.in")
        .replace(
            "Experiment-only verifier. Its local setup is unsuitable for funded rewards.",
            note,
        )
        .replace("uint256[11]", &format!("uint256[{count}]"))
        .replace("i < 11", &format!("i < {count}"))
        .replace(
            "uint256[24] memory ic",
            &format!("uint256[{}] memory ic", (count + 1) * 2),
        )
        .replace("{{IC}}", &literals(&ic))
        .replace("{{ALPHA}}", &literals(&alpha))
        .replace("{{BETA}}", &literals(&beta))
        .replace("{{GAMMA}}", &literals(&gamma))
        .replace("{{DELTA}}", &literals(&delta));
    fs::create_dir_all(directory.join("src"))?;
    write_new(&directory.join("src/Verifier.sol"), verifier.as_bytes())?;
    write_new(
        &directory.join("foundry.toml"),
        b"[profile.default]\nsrc = \"src\"\ntest = \"test\"\nsolc_version = \"0.8.28\"\nevm_version = \"paris\"\noptimizer = true\noptimizer_runs = 200\nvia_ir = true\n",
    )?;
    Ok(())
}

pub fn export_named(
    directory: &Path,
    key: &VerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    inputs: &[Fr],
    names: &[&str],
    note: &str,
) -> Result<(), Box<dyn Error>> {
    if inputs.len() != names.len() {
        return Err("Unexpected public input names.".into());
    }
    export_verifier(directory, key, names.len(), note)?;
    let proof_words: Vec<_> = g1(&proof.a)
        .into_iter()
        .chain(g2(&proof.b))
        .chain(g1(&proof.c))
        .collect();
    let public_words: Vec<_> = inputs
        .iter()
        .map(|input| input.into_bigint().to_string())
        .collect();
    let tests = include_str!("verifier-test.sol.in")
        .replace("uint256[11]", &format!("uint256[{}]", names.len()))
        .replace("i < 11", &format!("i < {}", names.len()))
        .replace("{{PROOF}}", &literals(&proof_words))
        .replace("{{INPUTS}}", &literals(&public_words));
    fs::create_dir_all(directory.join("src"))?;
    fs::create_dir_all(directory.join("test"))?;
    write_new(&directory.join("test/Verifier.t.sol"), tests.as_bytes())?;
    let payload = json!({
        "function": format!("verify(uint256[8],uint256[{}])", names.len()),
        "proof": proof_words,
        "public_input_names": names,
        "public_inputs": public_words,
        "verifier_key": "hardcoded_in_src/Verifier.sol",
        "verified_on_evm": false
    });
    write_new(
        &directory.join("evm.json"),
        serde_json::to_string_pretty(&payload)?.as_bytes(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infinity_uses_the_evm_zero_encoding() {
        assert_eq!(g1(&G1Affine::default()), ["0", "0"]);
        assert_eq!(g2(&G2Affine::default()), ["0", "0", "0", "0"]);
    }
}
