use core::num;

use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;
// use futures::future::join_all;
use log::{error, info};
// use once_cell::sync::Lazy;

use circle_api::api::CircleClient;
use circle_api::models::blockchain::Blockchain;
use circle_api::models::wallet_balance::WalletBalanceQueryParams;
use circle_api::models::wallet_list::WalletListQueryParams;
use circle_api::models::wallet_nfts::WalletNftsQueryParams;
use circle_api::models::wallet_set::{
    CreateWalletSetRequest, CreateWalletSetResponse, WalletSetObjectResponse, WalletSetsQueryParams,
};

pub fn get_env(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {} env variable", env))
}

pub struct Config {
    pub circle_client: CircleClient,
}

// pub static CONFIG: Lazy<Config> = Lazy::new(|| {
//     dotenv().expect("Failed to read .env file");

//     Config {
//         circle_api_key: get_env("CIRCLE_API_KEY"),
//         circle_entity_secret: get_env("CIRCLE_ENTITY_SECRET"),
//     }
// });

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // match run().await {
    //     Ok(_) => {}
    //     Err(err) => {
    //         error!("Error: {:?}", err);
    //         Err(err)?
    //     }
    // }

    dotenv().expect("Failed to read .env file");

    let circle_api_key = get_env("CIRCLE_API_KEY");
    let circle_entity_secret = get_env("CIRCLE_ENTITY_SECRET");

    let config = Config::new(circle_api_key, circle_entity_secret).await?;

    let wallet_info = config
        .create_wallet_set("test_wallet_set".to_string())
        .await?;

    // let wallet_id = wallet_info.id;
    // config.get_wallet_set(wallet_id).await?;
    // config
    //     .update_wallet_set(wallet_id, "new_name".to_string())
    //     .await?;

    config.list_wallet_sets().await?;

    Ok(())
}

impl Config {
    pub async fn new(
        circle_api_key: String,
        circle_entity_secret: String,
    ) -> Result<Self, anyhow::Error> {
        let result = Self {
            circle_client: CircleClient::new(circle_api_key, circle_entity_secret).await?,
        };
        Ok(result)
    }

    pub async fn create_wallet_set(
        &self,
        wallet_set_name: String,
    ) -> Result<(WalletSetObjectResponse), anyhow::Error> {
        let idempotency_key = uuid::Uuid::new_v4();
        let wallet_set_response = self
            .circle_client
            .create_wallet_set(idempotency_key, wallet_set_name)
            .await?
            .wallet_set;
        info!("Wallet set response: {:?}", wallet_set_response);

        Ok(wallet_set_response)
    }

    pub async fn update_wallet_set(
        &self,
        wallet_set_id: uuid::Uuid,
        wallet_set_name: String,
    ) -> Result<WalletSetObjectResponse, anyhow::Error> {
        let update_wallet_set_response = self
            .circle_client
            .update_wallet_set(wallet_set_id, wallet_set_name)
            .await?
            .wallet_set;
        info!(
            "Updated wallet set response: {:?}",
            update_wallet_set_response
        );

        Ok(update_wallet_set_response)
    }

    pub async fn get_wallet_set(&self, wallet_set_id: uuid::Uuid) -> Result<(), anyhow::Error> {
        let wallet_set_response = self.circle_client.get_wallet_set(wallet_set_id).await?;
        info!("Wallet set response: {:?}", wallet_set_response);

        Ok(())
    }

    pub async fn list_wallet_sets(&self) -> Result<(), anyhow::Error> {
        let wallet_sets_response = self
            .circle_client
            .list_wallet_sets(WalletSetsQueryParams::default())
            .await?;
        for wallet_set in &wallet_sets_response.wallet_sets {
            info!("Wallet set: {:?}", wallet_set);
        }

        Ok(())
    }

    pub async fn create_wallet(
        &self,
        wallet_set_id: uuid::Uuid,
        num_wallets: u32,
    ) -> Result<(), anyhow::Error> {
        let idempotency_key = uuid::Uuid::new_v4();
        let create_wallet_response = self
            .circle_client
            .create_wallet(
                idempotency_key,
                wallet_set_id,
                vec![Blockchain::MaticMumbai],
                num_wallets,
            )
            .await?;
        for (i, wallet) in create_wallet_response.wallets.iter().enumerate() {
            info!("Wallet #{}: {:?}", i, wallet);
        }

        Ok(())
    }
}

async fn run() -> Result<(), anyhow::Error> {
    info!("Starting payments-service");
    dotenv().expect("Failed to read .env file");

    let circle_api_key = get_env("CIRCLE_API_KEY");
    let circle_entity_secret = get_env("CIRCLE_ENTITY_SECRET");

    let circle_client = CircleClient::new(circle_api_key, circle_entity_secret).await?;

    let wallet_set_name = "test_wallet_set";
    let idempotency_key = uuid::Uuid::new_v4();
    let wallet_set_response = circle_client
        .create_wallet_set(idempotency_key, wallet_set_name.to_string())
        .await?
        .wallet_set;
    info!("Wallet set response: {:?}", wallet_set_response);

    let wallet_set_name = "test_updated_wallet_set";
    let update_wallet_set_response = circle_client
        .update_wallet_set(wallet_set_response.id, wallet_set_name.to_string())
        .await?
        .wallet_set;
    info!(
        "Updated wallet set response: {:?}",
        update_wallet_set_response
    );

    let get_wallet_set_response = circle_client.get_wallet_set(wallet_set_response.id).await?;
    info!("Get wallet set response: {:?}", get_wallet_set_response);

    let wallet_sets_response = circle_client
        .list_wallet_sets(WalletSetsQueryParams::default())
        .await?;
    for wallet_set in &wallet_sets_response.wallet_sets {
        info!("Wallet set: {:?}", wallet_set);
    }

    let wallet_set = &wallet_sets_response.wallet_sets[0];

    let idempotency_key = uuid::Uuid::new_v4();
    let create_wallet_response = circle_client
        .create_wallet(
            idempotency_key,
            wallet_set.id,
            vec![Blockchain::MaticMumbai],
            2,
        )
        .await?;
    for (i, wallet) in create_wallet_response.wallets.iter().enumerate() {
        info!("Wallet #{}: {:?}", i, wallet);
    }

    let list_wallet_response = circle_client
        .list_wallets(WalletListQueryParams::default())
        .await?;
    for (i, wallet) in list_wallet_response.wallets.iter().enumerate() {
        info!("Wallet #{}: {:?}", i, wallet);
    }

    let wallet = &list_wallet_response.wallets[0];
    let get_wallet_response = circle_client.get_wallet(wallet.id).await?;
    info!("Get wallet response: {:?}", get_wallet_response);

    // get non-existent wallet
    let get_wallet_response = circle_client.get_wallet(uuid::Uuid::new_v4()).await;
    info!(
        "Get non-existent wallet response: {:?}",
        get_wallet_response
    );

    // Update wallet
    let wallet = &list_wallet_response.wallets[0];
    let update_wallet_response = circle_client
        .update_wallet(
            wallet.id,
            circle_api::models::wallet_update::WalletUpdateRequest {
                name: "test_wallet".to_string(),
                ref_id: "test_ref_id".to_string(),
            },
        )
        .await?;
    info!("Update wallet response: {:?}", update_wallet_response);

    // let balance_futures = create_wallet_response
    //     .wallets
    //     .iter()
    //     .map(|w| {
    //         circle_client
    //             .get_wallet_balance(w.id, WalletBalanceQueryParams::default().include_all(true))
    //     })
    //     .collect::<Vec<_>>();

    // let balances = join_all(balance_futures)
    //     .await
    //     .into_iter()
    //     .map(|r| r.map_err(|e| e.into()))
    //     .collect::<Result<Vec<_>>>()?;
    // for balance in balances {
    //     info!("Balance: {:?}", balance);
    // }

    // let nfts_futures = create_wallet_response
    //     .wallets
    //     .iter()
    //     .map(|w| circle_client.get_wallet_nfts(w.id, WalletNftsQueryParams::default()))
    //     .collect::<Vec<_>>();

    // let nfts = join_all(nfts_futures)
    //     .await
    //     .into_iter()
    //     .map(|r| r.map_err(|e| e.into()))
    //     .collect::<Result<Vec<_>>>()?;
    // for nft in nfts {
    //     info!("NFT: {:?}", nft);
    // }

    Ok(())
}
