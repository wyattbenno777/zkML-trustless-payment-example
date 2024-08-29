use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

use nova::provider::PallasEngine;
use std::path::PathBuf;

use zk_engine::{
    args::{WASMArgsBuilder, WASMCtx},
    nova::{
        provider::ipa_pc,
        spartan::{self, snark::RelaxedR1CSSNARK},
        traits::Dual,
    },
    run::batched::{public_values, BatchedZKEProof},
    traits::zkvm::ZKVM,
    utils::logging::init_logger,
};

// Curve cycle to use for proving
type E1 = PallasEngine;
// PCS used for final SNARK at the end of (N)IVC
type EE1<E> = ipa_pc::EvaluationEngine<E>;
// PCS on secondary curve
type EE2<E> = ipa_pc::EvaluationEngine<Dual<E>>;

// Spartan SNARKS used for compressing at then end of (N)IVC
type BS1<E> = spartan::batched::BatchedRelaxedR1CSSNARK<E, EE1<E>>;
type S1<E> = RelaxedR1CSSNARK<E, EE1<E>>;
type S2<E> = RelaxedR1CSSNARK<Dual<E>, EE2<E>>;

fn main() {
    // Configure the arguments needed for WASM execution
    //
    // Here we are configuring the path to the WASM file
    let args = WASMArgsBuilder::default()
        .file_path(PathBuf::from("wasm/fib.wat"))
        .invoke(Some(String::from("fib")))
        .func_args(vec![String::from("10")]) // This will generate 16,000 + opcodes
        .build();

    // Create a WASM execution context for proving.
    let mut wasm_ctx = WASMCtx::new_from_file(args).unwrap();

    // Retrieve the public values from the proving process
    let (_, public_values, _) =
        BatchedZKEProof::<E1, BS1<E1>, S1<E1>, S2<E1>>::prove_wasm(&mut wasm_ctx).unwrap();

    // Save the public values to a file
    let pp_string = serde_json::to_string(&public_values).unwrap();
    std::fs::create_dir("public_values");
    save_to_file("public_values/public_values.json", &pp_string);
}

fn save_to_file(filename: &str, data: &str) -> anyhow::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    println!("Data written to {}", filename);
    Ok(())
}
