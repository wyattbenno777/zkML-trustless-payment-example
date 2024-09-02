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

fn main() -> anyhow::Result<()> {
    // Configure the arguments needed for WASM execution
    //
    // Here we are configuring the path to the WASM file
    let args = WASMArgsBuilder::default()
        .file_path(PathBuf::from("wasm/gradient_boosting.wasm"))
        .invoke(Some(String::from("_start")))
        .trace_slice_values(TraceSliceValues::new(0, 100000))
        .build();

    // Create a WASM execution context for proving.
    let mut wasm_ctx = WASMCtx::new_from_file(&args)?;

    let pp = BatchedZKEProof::setup(&mut wasm_ctx)?;

    let mut wasm_ctx = WASMCtx::new_from_file(&args)?;
    // Retrieve the public values from the proving process
    let (_, public_values, _) =
        BatchedZKEProof::<E1, BS1<E1>, S1<E1>, S2<E1>>::prove_wasm(&mut wasm_ctx, &pp)?;

    // Save the public values and params to files
    std::fs::create_dir("public_values");
    let public_values_str = serde_json::to_string(&public_values)?;
    let pp_string = serde_json::to_string(&pp)?;
    save_to_file("public_values/public_values.json", &public_values_str)?;
    save_to_file("public_values/pp.json", &pp_string)?;

    Ok(())
}

fn save_to_file(filename: &str, data: &str) -> anyhow::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    println!("Data written to {}", filename);
    Ok(())
}
