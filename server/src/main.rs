use std::fs::read_to_string;
use std::process::Command;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use nova::provider::PallasEngine;

use zk_engine::{
    args::{WASMArgsBuilder, WASMCtx},
    nova::{
        provider::ipa_pc,
        spartan::{self, snark::RelaxedR1CSSNARK},
        traits::Dual,
    },
    run::batched::{public_values::BatchedPublicValues, BatchedZKEProof},
    traits::zkvm::ZKVM,
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

// the output containing verification result
#[derive(Serialize)]
struct VerifyResult {
    success: bool,
}

#[derive(Deserialize)]
struct Body {
    proof: BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>,
    recipient_address: String,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/post", post(test_post));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn test_post(
    // this argument tells axum to parse the request body
    // as JSON into a `BatchedZKEProof` type
    Json(body): Json<Body>,
) -> (StatusCode, Json<VerifyResult>) {
    println!("Proof received, verifying");

    let proof = body.proof;

    // Retrieve public params from file
    let public_values = get_public_values();

    // Verify the proof
    let result = proof.verify(public_values).unwrap();

    let result_json;
    if result {
        println!("Proof successfully verified");
        result_json = VerifyResult { success: true };
        println!("Sending USDC");
        Command::new("node")
            .arg("send_usdc/send_usdc.js")
            .arg(&body.recipient_address)
            .spawn()
            .expect("Failed to send USDC");
    } else {
        println!("Error when verifying proof");
        result_json = VerifyResult { success: false };
    }

    (StatusCode::CREATED, Json(result_json))
}

fn get_public_values() -> BatchedPublicValues<E1, BS1<E1>, S1<E1>, S2<E1>> {
    let public_values_str = read_to_string("public_params/public_params.json").unwrap();

    match serde_json::from_str::<BatchedPublicValues<E1, BS1<E1>, S1<E1>, S2<E1>>>(
        &public_values_str,
    ) {
        Ok(public_values) => public_values,
        Err(e) => {
            panic!("Error when deserializing public params: {}", e);
        }
    }
}
