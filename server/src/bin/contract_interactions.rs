use anyhow::Ok;
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

    let value = U256::from(1);
    let amount = value * U256::from(10).pow(U256::from(decimals));

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

    pub async fn get_decimals(&self) -> u8 {
        let result: u8 = self
            .0
            .query("decimals", (), None, Options::default(), None)
            .await
            .unwrap();
        result
    }

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

    pub async fn transfer(&self, account: SecretKey, to: &String, value: U256) -> Result<()> {
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

        match tx.status {
            Some(status) => {
                if status == 0 {
                    return Err(Error::TransactionFailed);
                }
            }
            None => {
                return Err(Error::TransactionFailed);
            }
        }
        println!("Transfer completed! Transaction ID: {:?}", tx);

        Ok(())
    }
}
