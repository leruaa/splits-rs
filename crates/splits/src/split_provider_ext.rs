use alloy::{
    network::{Network, TransactionBuilder},
    primitives::{Address, U256},
    providers::Provider,
    sol,
    sol_types::SolCall,
    transports::{Transport, TransportErrorKind, TransportResult},
};
use async_trait::async_trait;

use crate::{types::Account, Config};

sol!(
    #[sol(rpc)]
    SplitMainContract,
    "abi/SplitMain.json"
);

#[async_trait]
pub trait SplitProviderExt<T, N>
where
    N: Network,
{
    async fn get_erc20_balance(&self, account: Address, token: Address) -> TransportResult<U256>;

    async fn get_eth_balance(&self, account: Address) -> TransportResult<U256>;

    async fn distribute_erc20(
        &self,
        account: Account,
        token: Address,
        distributor_address: Address,
    ) -> TransportResult<N::TransactionRequest>;

    async fn withdraw(
        &self,
        account: Address,
        eth: U256,
        tokens: Vec<Address>,
    ) -> TransportResult<N::TransactionRequest>;
}

#[async_trait]
impl<P, T, N> SplitProviderExt<T, N> for P
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    async fn get_erc20_balance(&self, account: Address, token: Address) -> TransportResult<U256> {
        let chain_id = self.get_chain_id().await?;
        let config = Config::new(chain_id);
        let call = SplitMainContract::getERC20BalanceCall::new((account, token));

        let tx = N::TransactionRequest::default()
            .with_to(config.main_address)
            .with_input(call.abi_encode());

        let response = self.call(&tx).await?;

        let response = SplitMainContract::getERC20BalanceCall::abi_decode_returns(&response, true)
            .map_err(TransportErrorKind::custom)?;

        Ok(response._0)
    }

    async fn get_eth_balance(&self, account: Address) -> TransportResult<U256>
    where
        P: Provider<T, N>,
        N: Network,
        T: Transport + Clone,
    {
        let chain_id = self.get_chain_id().await?;
        let config = Config::new(chain_id);
        let call = SplitMainContract::getETHBalanceCall::new((account,));

        let tx = N::TransactionRequest::default()
            .with_to(config.main_address)
            .with_input(call.abi_encode());

        let response = self.call(&tx).await?;

        let response = SplitMainContract::getETHBalanceCall::abi_decode_returns(&response, true)
            .map_err(TransportErrorKind::custom)?;

        Ok(response._0)
    }

    async fn distribute_erc20(
        &self,
        account: Account,
        token: Address,
        distributor_address: Address,
    ) -> TransportResult<N::TransactionRequest> {
        let chain_id = self.get_chain_id().await?;
        let config = Config::new(chain_id);
        let call = SplitMainContract::distributeERC20Call::new((
            account.address,
            token,
            account.accounts,
            account.percents_allocation,
            account.distributor_fee,
            distributor_address,
        ));

        let tx = N::TransactionRequest::default()
            .with_to(config.main_address)
            .with_input(call.abi_encode());

        Ok(tx)
    }

    async fn withdraw(
        &self,
        account: Address,
        eth: U256,
        tokens: Vec<Address>,
    ) -> TransportResult<N::TransactionRequest>
    where
        N: Network,
    {
        let chain_id = self.get_chain_id().await?;
        let config = Config::new(chain_id);
        let call = SplitMainContract::withdrawCall::new((account, eth, tokens));

        let tx = N::TransactionRequest::default()
            .with_to(config.main_address)
            .with_input(call.abi_encode());

        Ok(tx)
    }
}
