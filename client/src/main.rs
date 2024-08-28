use reqwest::Result;
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
#[tokio::main]
async fn main() -> Result<()> {
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

    println!("Building proof");
    let (proof, _, _) =
        BatchedZKEProof::<E1, BS1<E1>, S1<E1>, S2<E1>>::prove_wasm(&mut wasm_ctx).unwrap();

    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:3000/post";

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

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body = res.json::<VerifyResult>().await?;
    println!("Body:\n{:?}", body);
    if body.success {
        println!("Successfully verified proof!");
    } else {
        println!("Error verifying proof");
    }

    Ok(())
}

#[derive(Serialize)]
struct Body {
    proof: BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>,
    recipient_address: String,
}

// the output of verification process
#[derive(Deserialize, Debug)]
struct VerifyResult {
    success: bool,
}
