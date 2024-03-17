use std::{pin::Pin, sync::Arc};

use alloy::{
    network::{Ethereum, Network, TransactionBuilder},
    providers::Provider,
    rpc::types::eth::{BlockNumberOrTag, Filter},
    transports::{Transport, TransportErrorKind, TransportResult},
};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall, SolEvent};
use futures::{stream::unfold, Stream, StreamExt};

use crate::types::Account;

sol!(SplitMainContract, "abi/SplitMain.json");

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
        P: Provider<N, T>,
        N: Network,
        T: Transport + Clone,
    {
        let call = SplitMainContract::getERC20BalanceCall::new((account, token));

        let tx = N::TransactionRequest::default()
            .with_to(self.address.into())
            .with_input(call.abi_encode().into());

        let response = provider.call(&tx, None).await?;

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
        P: Provider<N, T>,
        N: Network,
        T: Transport + Clone,
    {
        let call = SplitMainContract::getETHBalanceCall::new((account,));

        let tx = N::TransactionRequest::default()
            .with_to(self.address.into())
            .with_input(call.abi_encode().into());

        let response = provider.call(&tx, None).await?;

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
        P: Provider<Ethereum, T> + 'static,
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
                        .map(|tx| {
                            SplitMainContract::createSplitCall::abi_decode(&tx.input, true).ok()
                        })
                        .and_then(|call| {
                            SplitMainContract::CreateSplit::decode_raw_log(
                                log.topics, &log.data, true,
                            )
                            .map(|event| Account::new(event.split, call.map(|c| c.distributorFee)))
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
        split: Address,
        token: Address,
        distributor_fee: u32,
        distributor_address: Address,
    ) -> N::TransactionRequest
    where
        N: Network,
    {
        let call = SplitMainContract::distributeERC20Call::new((
            split,
            token,
            vec![],
            vec![],
            distributor_fee,
            distributor_address,
        ));

        N::TransactionRequest::default()
            .with_from(self.address)
            .with_input(call.abi_encode().into())
    }

    pub fn withdraw<N>(&self, account: Address, eth: U256, tokens: Address) -> N::TransactionRequest
    where
        N: Network,
    {
        let call = SplitMainContract::withdrawCall::new((account, eth, tokens));

        N::TransactionRequest::default()
            .with_from(self.address)
            .with_input(call.abi_encode().into())
    }
}
