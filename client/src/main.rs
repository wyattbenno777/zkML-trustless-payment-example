use anyhow::Result;
use serde::{Deserialize, Serialize};

use nova::provider::PallasEngine;
use std::path::PathBuf;
// Backend imports for ZK
use zk_engine::{
    args::{WASMArgsBuilder, WASMCtx},
    nova::{
        provider::ipa_pc,
        spartan::{self, snark::RelaxedR1CSSNARK},
        traits::Dual,
    },
    run::batched::BatchedZKEProof,
    traits::zkvm::ZKVM,
    utils::logging::init_logger,
    TraceSliceValues,
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

const RECIPIENT_ADDRESS: &str = "0x73987bF167b5cC201cBa676F64d43A063C62018b";

// The input struct sent to the server
#[derive(Serialize)]
struct Body {
    proof: BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>,
    recipient_address: String,
}

// the output of verification process, received from the server
#[derive(Deserialize, Debug)]
struct VerifyResult {
    failure_reason: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    // Create a reqwest client to send requests to the server
    let client = reqwest::Client::new();

    // get information from the server
    let url = "http://127.0.0.1:3000/";
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    println!("{}", body);
    println!("------------------------------ ");

    // Configure the arguments needed for WASM execution
    //
    // Here we are configuring the path to the WASM file
    let args = WASMArgsBuilder::default()
        .file_path(PathBuf::from("wasm/gradient_boosting.wasm"))
        .invoke(Some(String::from("_start")))
        .trace_slice_values(TraceSliceValues::new(0, 100000))
        .build();

    // Create a WASM execution context for proving.
    let mut wasm_ctx = WASMCtx::new_from_file(&args).unwrap();

    println!("Building proof");
    let pp = BatchedZKEProof::setup(&mut wasm_ctx)?;
    let mut wasm_ctx = WASMCtx::new_from_file(&args).unwrap();
    let (proof, _, _) = BatchedZKEProof::prove_wasm(&mut wasm_ctx, &pp).unwrap();

    // Send the proof to the server
    let url = "http://127.0.0.1:3000/post";

    // Create a Body struct containing the proof, to send to the server
    let body = Body {
        proof,
        recipient_address: RECIPIENT_ADDRESS.to_string(),
    };

    println!("Sending proof to server");
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await?;

    // Parse the response into a VerifyResult struct,
    // Similar to the one defined and sent by the server
    let body = res.json::<VerifyResult>().await?;
    match body.failure_reason {
        Some(reason) => println!(
            "Error when verifying proof and processing payment: {}",
            reason
        ),
        None => println!("Proof successfully verified and payment processed"),
    }

    Ok(())
}
