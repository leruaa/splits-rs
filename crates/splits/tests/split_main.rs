use std::{env, sync::Arc};

use alloy::{network::Ethereum, primitives::address, providers::ProviderBuilder};
use dotenv::dotenv;
use futures::StreamExt;
use splits::SplitMain;

#[tokio::test]
async fn test_get_create_splits_logs() {
    dotenv().ok();
    let eth_rpc = env::var("ETH_HTTP_RPC").unwrap();
    let provider = ProviderBuilder::<_, _, Ethereum>::new().on_http(eth_rpc.parse().unwrap());
    let split_main = SplitMain::new(address!("2ed6c4B5dA6378c7897AC67Ba9e43102Feb694EE"));

    let accounts = split_main
        .get_split_accounts(Arc::new(provider), 14206768, None)
        .await
        .unwrap();

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
