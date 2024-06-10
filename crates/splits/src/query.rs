use std::{convert, pin::Pin};

use alloy::{
    eips::BlockNumberOrTag,
    network::Ethereum,
    providers::Provider,
    rpc::types::eth::Filter,
    sol_types::{SolCall, SolEvent},
    transports::{Transport, TransportErrorKind, TransportResult},
};
use futures::{stream::unfold, Stream, StreamExt};

use crate::{types::Account, Config, SplitMainContract};

pub async fn get_split_accounts<P, T, B>(
    provider: P,
    from_block: B,
    to_block: Option<B>,
) -> TransportResult<Pin<Box<dyn Stream<Item = TransportResult<Account>>>>>
where
    P: Provider<T, Ethereum> + 'static,
    T: Transport + Clone,
    B: Into<BlockNumberOrTag>,
{
    let chain_id = provider.get_chain_id().await?;
    let config = Config::new(chain_id);
    let filter = Filter::new()
        .from_block(from_block)
        .to_block(to_block.map(|b| b.into()).unwrap_or_default())
        .address(config.main_address)
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
