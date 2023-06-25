use std::sync::Arc;

use crate::contracts::{DispatchIdFilter, Mailbox};
use crate::core;
use color_eyre::Result;
use ethers::contract::EthEvent;
use ethers::{providers::Middleware, types::Bytes};
use hyperlane_core::{H160, H256};

/// Dispatch a message to the Hyperlane mailbox contract.
pub async fn dispatch<M: Middleware + 'static>(
    client: Arc<M>,
    mailbox_address: H160,
    dest_id: u32,
    recipient_address: H160,
    message_body: Vec<u8>,
    verbose: bool,
) -> Result<Option<H256>> {
    let mailbox = Mailbox::new(mailbox_address, Arc::clone(&client));

    let recipient_address: H256 = recipient_address.into();
    let tx_receipt = mailbox
        .dispatch(dest_id, recipient_address.into(), Bytes::from(message_body))
        .send()
        .await?
        .confirmations(1)
        .await?;

    if verbose {
        println!("Transaction receipt: {:#?}", tx_receipt);
    };

    Ok(match tx_receipt {
        Some(receipt) => {
            println!(
                "Dispatch in block {}, tx hash: {:?}",
                core::option_into_display_string(&receipt.block_number),
                receipt.transaction_hash
            );

            let id = receipt
                .logs
                .iter()
                .find(|log| log.topics[0] == DispatchIdFilter::signature())
                .map(|log| log.topics[1]);

            if let Some(id) = id {
                println!("  Message ID: {:?}", id);
            }

            id
        }
        None => {
            println!("Transaction status unknown");
            None
        }
    })
}
