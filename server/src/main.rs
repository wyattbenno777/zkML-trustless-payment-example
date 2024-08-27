use std::fs::read_to_string;

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
    // utils::logging::init_logger,
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

#[tokio::main]
async fn main() {
    // // initialize tracing
    // tracing_subscriber::fmt::init();

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
    // as JSON into a `CreateUser` type
    Json(proof): Json<BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>>,
) -> (StatusCode, Json<VerifyResult>) {
    // insert your application logic here

    let public_params_str = read_to_string("public_params/public_params.json").unwrap();

    let public_params = serde_json::from_str::<BatchedPublicValues<E1, BS1<E1>, S1<E1>, S2<E1>>>(
        &public_params_str,
    )
    .unwrap();

    let result = proof.verify(public_params).unwrap();

    let result_json;
    if result {
        result_json = VerifyResult { success: true }
    } else {
        result_json = VerifyResult { success: false }
    }

    (StatusCode::CREATED, Json(result_json))
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct VerifyResult {
    success: bool,
}
