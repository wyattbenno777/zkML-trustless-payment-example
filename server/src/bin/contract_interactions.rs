use secp256k1::SecretKey;
use web3::api::Eth;
use web3::signing::SecretKeyRef;
use web3::transports::Http;
use web3::{
    contract::{Contract, Options},
    types::{Address, U256},
};

use alloy::primitives::utils::parse_units;
use dotenv::dotenv;
use std::fmt;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Create a new web3 instance
    let web3 = web3::Web3::new(
        web3::transports::Http::new(&std::env::var("SEPOLIA_RPC_ENDPOINT").unwrap()).unwrap(),
    );

    let contract = UsdcContract::new(
        &web3,
        std::env::var("SEPOLIA_USDC_CONTRACT").expect("SEPOLIA_USDC_CONTRACT must be set"),
    )
    .await;

    let decimals = contract.get_decimals().await;
    println!("Decimals: {}", decimals);

    let balance = contract
        .balance_of(&std::env::var("SENDER_ADDRESS").expect("SENDER_ADDRESS must be set"))
        .await;
    println!("Balance: {}", balance);

    let value = 1;
    let amount = parse_units(&value.to_string(), decimals).unwrap();
    let amount = U256::from_dec_str(&amount.to_string()).unwrap();

    let account = SecretKey::from_str(&std::env::var("SENDER_PRIVATE_KEY").unwrap()).unwrap();
    let tx = contract
        .transfer(
            account,
            &"0x73987bF167b5cC201cBa676F64d43A063C62018b".to_string(),
            amount,
        )
        .await;
}

// newtype to better manage contract function handling.
pub struct UsdcContract(Contract<Http>);

impl UsdcContract {
    pub async fn new(web3: &web3::Web3<web3::transports::Http>, address: String) -> Self {
        let address = Address::from_str(&address).unwrap();
        let contract = Contract::from_json(
            web3.eth(),
            address,
            include_bytes!("../../usdc_abi/usdc_abi.json"),
        )
        .unwrap();
        UsdcContract(contract)
    }
    // Query the totalMatches state variable
    pub async fn get_decimals(&self) -> u8 {
        let result: u8 = self
            .0
            .query("decimals", (), None, Options::default(), None)
            .await
            .unwrap();
        result
    }
    // Query the lifetimeValue state variable
    pub async fn balance_of(&self, address: &String) -> U256 {
        let result: U256 = self
            .0
            .query(
                "balanceOf",
                Address::from_str(address).unwrap(),
                None,
                Options::default(),
                None,
            )
            .await
            .unwrap();
        result
    }

    // Create a match
    pub async fn transfer(&self, account: SecretKey, to: &String, value: U256) {
        // Signed call to create the transaction
        let tx = self
            .0
            .signed_call_with_confirmations(
                "transfer",
                (Address::from_str(&to).unwrap(), value),
                Options {
                    gas: Some(5_000_000.into()),
                    value: None,
                    ..Default::default()
                },
                1,
                SecretKeyRef::new(&account),
            )
            .await
            .unwrap();

        println!("Transfer completed! Transaction ID: {:?}", tx);
    }
    // Join a match
}
pub struct Match {
    match_id: u128,
    player1: Address,
    player2: Address,
    player1_bet: u128,
    player2_bet: u128,
    winner: u128,
    match_complete: bool,
}
// Handle the display of the match values
impl fmt::Display for Match {
    //Display formatter for the given banner entry
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Bet values are being output in Wei by default. Must conver them later
        write!(
            f,
            "
Match ID: {}{}
Player 1: {}: {} Wei 
Player 2: {}: {} Wei
Completed: {}
Winner: Player {}",
            "",
            self.match_id,
            self.player1,
            self.player1_bet,
            self.player2,
            self.player2_bet,
            self.match_complete,
            self.winner
        )
    }
}
