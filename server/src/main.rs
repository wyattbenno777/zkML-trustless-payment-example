mod contract_interactions;

use contract_interactions::UsdcContract;
use secp256k1::SecretKey;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, str::FromStr};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use nova::provider::PallasEngine;
use zk_engine::{
    nova::{
        provider::ipa_pc,
        spartan::{self, snark::RelaxedR1CSSNARK},
        traits::Dual,
    },
    run::batched::{public_values::BatchedPublicValues, BatchedZKEProof, BatchedZKEPublicParams},
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

// Quantity to send for recieving valid proof
const SEND_AMOUNT: u32 = 100;
const USDC_ABI: &[u8] = include_bytes!("../usdc_abi/usdc_abi.json");

// the output containing verification result
#[derive(Serialize)]
struct VerifyResult {
    failure_reason: Option<String>,
}

// The struct recieved from the client
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
        .route("/post", post(recieve_proof));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> String {
    let contract = init_contract(USDC_ABI).await;
    let decimals = contract.get_decimals().await;
    let balance = get_balance(&contract, &std::env::var("SENDER_ADDRESS").unwrap()).await;
    let balance = balance as f64 / 10f64.powf(decimals as f64);
    "The reward for sending a valid proof is ".to_owned()
        + &SEND_AMOUNT.to_string()
        + " USDC.
Current balance is "
        + &balance.to_string()
        + " USDC"
}

async fn recieve_proof(
    // this argument tells axum to parse the request body
    // as JSON into a `Body` type, similar to the type declared and sent by the client
    Json(body): Json<Body>,
) -> (StatusCode, Json<VerifyResult>) {
    println!("Proof received, verifying");

    let proof = body.proof;
    let recipient_address = body.recipient_address;

    // Retrieve public params from files
    let public_values = get_public_values();
    let pp = get_pp();

    // Verify the proof
    let is_proof_valid = proof.verify(public_values, &pp).unwrap();

    if !is_proof_valid {
        println!("Error when verifying proof");
        let result_json = VerifyResult {
            failure_reason: Some("Proof verification failed".to_string()),
        };
        return (StatusCode::BAD_REQUEST, Json(result_json));
    }

    println!("Proof successfully verified");
    println!("Sending USDC");

    // Send USDC to recipient
    let contract = init_contract(USDC_ABI).await;
    let account = init_account(&std::env::var("SENDER_PRIVATE_KEY").unwrap());
    let tx = contract
        .transfer(account, &recipient_address, SEND_AMOUNT)
        .await;

    return match tx {
        Ok(_) => {
            println!("USDC sent successfully");
            let result_json = VerifyResult {
                failure_reason: None,
            };
            (StatusCode::CREATED, Json(result_json))
        }
        Err(e) => {
            println!("Error when sending USDC: {:?}", e);
            let result_json = VerifyResult {
                failure_reason: Some("Could not send USDC".to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(result_json))
        }
    };
}

/**
 * Gets and returns the public values object from the file generated during server setup
 */
fn get_public_values() -> BatchedPublicValues<E1> {
    let public_values_str = read_to_string("public_values/public_values.json").unwrap();

    match serde_json::from_str::<BatchedPublicValues<E1>>(&public_values_str) {
        Ok(public_values) => public_values,
        Err(e) => {
            panic!("Error when deserializing public values: {}", e);
        }
    }
}

fn get_pp() -> BatchedZKEPublicParams<E1, BS1<E1>, S1<E1>, S2<E1>> {
    let pp_str = read_to_string("public_values/pp.json").unwrap();

    match serde_json::from_str::<BatchedZKEPublicParams<E1, BS1<E1>, S1<E1>, S2<E1>>>(&pp_str) {
        Ok(pp) => pp,
        Err(e) => {
            panic!("Error when deserializing public params: {}", e);
        }
    }
}

async fn init_contract(abi: &[u8]) -> UsdcContract {
    dotenv().ok();
    let web3 = web3::Web3::new(
        web3::transports::Http::new(&std::env::var("SEPOLIA_RPC_ENDPOINT").unwrap()).unwrap(),
    );

    let contract = UsdcContract::new(
        &web3,
        std::env::var("SEPOLIA_USDC_CONTRACT").expect("SEPOLIA_USDC_CONTRACT must be set"),
        abi,
    )
    .await;

    contract
}

fn init_account(private_key: &String) -> SecretKey {
    dotenv().ok();
    SecretKey::from_str(private_key).unwrap()
}

async fn get_balance(contract: &UsdcContract, account: &String) -> u64 {
    let balance = contract.balance_of(account).await;
    balance.as_u64()
}
