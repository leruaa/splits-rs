use std::{convert, pin::Pin, sync::Arc};

use alloy::{
    network::{Ethereum, Network, TransactionBuilder},
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::eth::{BlockNumberOrTag, Filter},
    sol,
    sol_types::{SolCall, SolEvent},
    transports::{Transport, TransportErrorKind, TransportResult},
};
use futures::{stream::unfold, Stream, StreamExt};

use crate::types::Account;

sol!(
    #[sol(rpc)]
    SplitMainContract,
    "abi/SplitMain.json"
);

pub struct SplitMain {
    address: Address,
}

impl SplitMain {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    pub async fn get_erc20_balance<P, N, T>(
        &self,
        provider: &P,
        account: Address,
        token: Address,
    ) -> TransportResult<U256>
    where
        P: Provider<T, N>,
        N: Network,
        T: Transport + Clone,
    {
        let call = SplitMainContract::getERC20BalanceCall::new((account, token));

        let tx = N::TransactionRequest::default()
            .with_to(self.address)
            .with_input(call.abi_encode());

        let response = provider.call(&tx).await?;

        let response = SplitMainContract::getERC20BalanceCall::abi_decode_returns(&response, true)
            .map_err(TransportErrorKind::custom)?;

        Ok(response._0)
    }

    pub async fn get_eth_balance<P, N, T>(
        &self,
        provider: &P,
        account: Address,
    ) -> TransportResult<U256>
    where
        P: Provider<T, N>,
        N: Network,
        T: Transport + Clone,
    {
        let call = SplitMainContract::getETHBalanceCall::new((account,));

        let tx = N::TransactionRequest::default()
            .with_to(self.address)
            .with_input(call.abi_encode());

        let response = provider.call(&tx).await?;

        let response = SplitMainContract::getETHBalanceCall::abi_decode_returns(&response, true)
            .map_err(TransportErrorKind::custom)?;

        Ok(response._0)
    }

    pub async fn get_split_accounts<P, T, B>(
        &self,
        provider: Arc<P>,
        from_block: B,
        to_block: Option<B>,
    ) -> TransportResult<Pin<Box<dyn Stream<Item = TransportResult<Account>>>>>
    where
        P: Provider<T, Ethereum> + 'static,
        T: Transport + Clone,
        B: Into<BlockNumberOrTag>,
    {
        let filter = Filter::new()
            .from_block(from_block)
            .to_block(to_block.map(|b| b.into()).unwrap_or_default())
            .address(self.address)
            .event(SplitMainContract::CreateSplit::SIGNATURE);

        let fill_event_logs = provider.get_logs(&filter).await?;

        let logs = fill_event_logs.into_iter().filter(|l| !l.removed);

        let stream = unfold((logs, provider), |(mut logs, provider)| async {
            match logs
                .next()
                .and_then(|l| l.transaction_hash.map(|hash| (l, hash)))
            {
                Some((log, tx_hash)) => {
                    let account = provider
                        .get_transaction_by_hash(tx_hash)
                        .await
                        .map(|tx| tx.ok_or(TransportErrorKind::custom_str("Transaction not found")))
                        .and_then(convert::identity)
                        .and_then(|tx| {
                            SplitMainContract::createSplitCall::abi_decode(&tx.input, true)
                                .map_err(TransportErrorKind::custom)
                        })
                        .and_then(|call| {
                            SplitMainContract::CreateSplit::decode_log_data(log.data(), true)
                                .map(|event| {
                                    Account::new(
                                        event.split,
                                        call.accounts,
                                        call.percentAllocations,
                                        call.distributorFee,
                                    )
                                })
                                .map_err(TransportErrorKind::custom)
                        });

                    Some((account, (logs, provider)))
                }
                None => None,
            }
        });

        Ok(stream.boxed())
    }

    pub fn distribute_erc20<N>(
        &self,
        account: Account,
        token: Address,
        distributor_address: Address,
    ) -> N::TransactionRequest
    where
        N: Network,
    {
        let call = SplitMainContract::distributeERC20Call::new((
            account.address,
            token,
            account.accounts,
            account.percents_allocation,
            account.distributor_fee,
            distributor_address,
        ));

        N::TransactionRequest::default()
            .with_to(self.address)
            .with_input(call.abi_encode())
    }

    pub fn withdraw<N>(&self, account: Address, eth: U256, tokens: Address) -> N::TransactionRequest
    where
        N: Network,
    {
        let call = SplitMainContract::withdrawCall::new((account, eth, vec![tokens]));

        N::TransactionRequest::default()
            .with_from(self.address)
            .with_input(call.abi_encode())
    }
}
