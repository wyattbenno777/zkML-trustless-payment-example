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

const SEND_AMOUNT: u32 = 1;
// the output containing verification result
#[derive(Serialize)]
struct VerifyResult {
    failure_reason: Option<String>,
}

#[derive(Deserialize)]
struct Body {
    proof: BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>,
    recipient_address: String,
}

#[tokio::main]
async fn main() {
    init_logger();
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
    // as JSON into a `Body` type, similar to the type declared and sent in client code
    Json(body): Json<Body>,
) -> (StatusCode, Json<VerifyResult>) {
    println!("Proof received, verifying");

    let proof = body.proof;
    let recipient_address = body.recipient_address;

    // Retrieve public params from file
    let public_values = get_public_values();

    // Verify the proof
    let is_proof_valid = proof.verify(public_values).unwrap();

    if !is_proof_valid {
        println!("Error when verifying proof");
        let result_json = VerifyResult {
            failure_reason: Some("Proof verification failed".to_string()),
        };
        return (StatusCode::BAD_REQUEST, Json(result_json));
    }

    println!("Proof successfully verified");
    println!("Sending USDC");
    let output = send_money(&recipient_address, SEND_AMOUNT);

    if !output.status.success() {
        println!("Error when sending USDC: {:?}", output);
        let result_json = VerifyResult {
            failure_reason: Some(
                "Could not send USDC: ".to_string() + &String::from_utf8(output.stderr).unwrap(),
            ),
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(result_json));
    }

    println!("USDC sent successfully");
    let result_json = VerifyResult {
        failure_reason: None,
    };

    (StatusCode::CREATED, Json(result_json))
}

/**
 * Gets and returns the public values object from the file generated during server setup
 */
fn get_public_values() -> BatchedPublicValues<E1, BS1<E1>, S1<E1>, S2<E1>> {
    let public_values_str = read_to_string("public_values/public_values.json").unwrap();

    match serde_json::from_str::<BatchedPublicValues<E1, BS1<E1>, S1<E1>, S2<E1>>>(
        &public_values_str,
    ) {
        Ok(public_values) => public_values,
        Err(e) => {
            panic!("Error when deserializing public values: {}", e);
        }
    }
}

/**
 * Runs a bash command to start a js script to send USDC to the recipient
 */
fn send_money(recipient_address: &String, amount: u32) -> std::process::Output {
    Command::new("node")
        .arg("send_usdc/send_usdc.js")
        .arg(recipient_address)
        .arg(amount.to_string())
        .output()
        .expect("Failed to send USDC")
}
