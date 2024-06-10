use std::env;

use alloy::{network::Ethereum, providers::ProviderBuilder};
use dotenv::dotenv;
use futures::StreamExt;
use splits::get_split_accounts;

#[tokio::test]
async fn test_get_create_splits_logs() {
    dotenv().ok();
    let eth_rpc = env::var("ETH_HTTP_RPC").unwrap();
    let provider = ProviderBuilder::<_, _, Ethereum>::new().on_http(eth_rpc.parse().unwrap());

    let accounts = get_split_accounts(provider, 14206768, None).await.unwrap();

    accounts
        .for_each(|acc| async {
            match acc {
                Ok(acc) => {
                    if acc.distributor_fee > 0 {
                        println!("{}: {}", acc.address, acc.distributor_fee)
                    }
                }
                Err(err) => println!("{err}"),
            }
        })
        .await;
}
