use std::env;

use alloy::{
    network::EthereumSigner,
    node_bindings::Anvil,
    primitives::{address, U256},
    providers::{ext::AnvilApi, Provider, ProviderBuilder},
    signers::wallet::LocalWallet,
};
use dotenv::dotenv;
use splits::{Account, SplitProviderExt};

#[tokio::test]
async fn test_withdraw() {
    dotenv().ok();

    let eth_rpc = env::var("ETH_HTTP_RPC").unwrap();

    let anvil = Anvil::new()
        .fork(eth_rpc)
        .fork_block_number(19962298)
        .spawn();

    let signer = EthereumSigner::new(LocalWallet::from(anvil.keys().first().unwrap().clone()));
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(signer)
        .on_http(anvil.endpoint_url());

    provider
        .anvil_impersonate_account(address!("db3Ec7B16Fd60fB4fDB58A438Bd8AF57d8d3a91c"))
        .await
        .unwrap();

    // https://etherscan.io/tx/0x14ada175a8852b0c2c77c2c876fec0cde184617e9aa95475b3a2d9286d73eac4
    let distribute_request = provider
        .distribute_erc20(
            Account::new(
                address!("aD30f7EEBD9Bd5150a256F47DA41d4403033CdF0"),
                vec![
                    address!("8a14D4a671fBe267844B08D9748eD946348aEbFD"),
                    address!("bbcec987E4C189FCbAB0a2534c77b3ba89229F11"),
                ],
                vec![140000, 860000],
                9998,
            ),
            address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            address!("db3Ec7B16Fd60fB4fDB58A438Bd8AF57d8d3a91c"),
        )
        .await
        .unwrap();

    let hash = provider
        .send_transaction(distribute_request)
        .await
        .unwrap()
        .watch()
        .await
        .unwrap();

    println!("{hash}");

    let withdraw_request = provider
        .withdraw(
            address!("db3Ec7B16Fd60fB4fDB58A438Bd8AF57d8d3a91c"),
            U256::ZERO,
            vec![address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")],
        )
        .await
        .unwrap();

    let hash = provider
        .send_transaction(withdraw_request)
        .await
        .unwrap()
        .watch()
        .await
        .unwrap();

    println!("{hash}");
}
