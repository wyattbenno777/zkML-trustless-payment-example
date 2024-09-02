use anyhow::Ok;
use secp256k1::SecretKey;
use web3::signing::SecretKeyRef;
use web3::transports::Http;
use web3::{
    contract::{Contract, Options},
    types::{Address, U256},
};

use std::fmt;
use std::str::FromStr;
use web3::types::TransactionReceipt;

#[derive(Debug, Clone)]
pub struct TransactionFailed;

impl fmt::Display for TransactionFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction failed")
    }
}

// newtype to better manage contract function handling.
pub struct UsdcContract(Contract<Http>);

impl UsdcContract {
    pub async fn new(
        web3: &web3::Web3<web3::transports::Http>,
        address: String,
        abi: &[u8],
    ) -> Self {
        let address = Address::from_str(&address).unwrap();
        let contract = Contract::from_json(web3.eth(), address, abi).unwrap();
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

    pub async fn transfer(
        &self,
        account: SecretKey,
        to: &String,
        value: u32,
    ) -> Result<TransactionReceipt, TransactionFailed> {
        // Signed call to create the transaction

        let decimals = self.get_decimals().await;
        let amount = U256::from(value) * U256::from(10).pow(U256::from(decimals));

        let tx = self
            .0
            .signed_call_with_confirmations(
                "transfer",
                (Address::from_str(&to).unwrap(), amount),
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
                if status == 0.into() {
                    return Err(TransactionFailed);
                }
            }
            None => {
                return Err(TransactionFailed);
            }
        }

        Ok(tx).map_err(|_| TransactionFailed)
    }
}
