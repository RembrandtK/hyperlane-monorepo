use std::sync::Arc;

use crate::{
    contracts::Mailbox,
    core,
    query::{MailboxLogItem, MailboxLogs},
};
use color_eyre::{eyre::Context, Result};
use ethers::providers::Middleware;
use hyperlane_core::H160;
use relayer::settings::matching_list::MatchingList;

/// Query for messages sent to a Hyperlane mailbox contract that match the provided matching list.
pub async fn query<M: Middleware + 'static>(
    client: Arc<M>,
    chain_id: u32,
    mailbox_address: H160,
    start_block: i32,
    end_block: i32,
    matching_list: MatchingList,
    verbose: bool,
) -> Result<()> {
    let mailbox = Arc::new(Mailbox::new(mailbox_address, Arc::clone(&client)));

    let (start_block, end_block) =
        resolve_block_numbers(get_current_block(client).await?, start_block, end_block);

    println!("Querying logs from block {start_block} to {end_block}.");

    let logs = MailboxLogs::new(
        chain_id,
        mailbox.clone(),
        matching_list,
        start_block,
        end_block,
    )
    .await?;

    for log in &logs {
        print_log_item(log, verbose)?;
    }

    Ok(())
}

/// Resolve block parameters to actual block numbers based on the current block number.
fn resolve_block_numbers(current_block: u64, start_block: i32, end_block: i32) -> (u64, u64) {
    let end_block = std::cmp::min(
        current_block,
        resolve_negative_block_number(current_block, end_block),
    );
    let start_block = std::cmp::min(
        end_block,
        resolve_negative_block_number(current_block, start_block),
    );

    (start_block, end_block)
}

async fn get_current_block<M: Middleware + 'static>(client: Arc<M>) -> Result<u64> {
    Ok(client
        .get_block_number()
        .await
        .context("Failed to retrieve block number")?
        .as_u64())
}

fn print_log_item(log: crate::query::MailboxLogItem<'_>, verbose: bool) -> Result<()> {
    if verbose {
        println!("{:#?}", log.log);
    }

    print_log_item_first_line(&log)?;
    print_log_item_details(&log)
}

fn print_log_item_first_line(log: &MailboxLogItem<'_>) -> Result<()> {
    print!(
        "\n{} in block {}",
        log.event_name(),
        core::option_into_display_string(&log.block_number())
    );
    if let Some(domain) = log.destination_domain()? {
        core::print_hyperlane_domain_details(" to", domain);
    } else if let Some(domain) = log.origin_domain()? {
        core::print_hyperlane_domain_details(" from", domain);
    } else {
        println!(":");
    }

    Ok(())
}

fn print_log_item_details(log: &MailboxLogItem<'_>) -> Result<()> {
    println!(
        "  Tx hash  : {}",
        core::option_into_debug_string(&log.transaction_hash())
    );
    if let Some(sender) = log.sender()? {
        println!("  Sender   : {:?}", sender);
    }
    if let Some(recipient) = log.recipient()? {
        println!("  Recipient: {:?}", recipient);
    }
    if let Some(id) = log.message_id()? {
        println!("  ID       : {:?}", id);
    };

    Ok(())
}

// fn extract_hyperlane_msg_from_dispatch_log(log: Log) -> Result<HyperlaneMessage> {
//     let raw_log = RawLog::from(log);
//     let event = DispatchFilter::decode_log(&raw_log)?;

//     let raw_message: RawHyperlaneMessage = event.message.to_vec();
//     Ok(raw_message.into())
// }

fn resolve_negative_block_number(current_blocknumber: u64, relative_blocknumber: i32) -> u64 {
    if relative_blocknumber < 0 {
        let current_blocknumber = current_blocknumber as i64;
        std::cmp::max(0, current_blocknumber + 1 + relative_blocknumber as i64) as u64
    } else {
        relative_blocknumber as u64
    }
}

// Abandoned, for now, simpler approach of having a less restrictive filter on the chain
// and then further filtering the results in the client.
//
// This will not work well when wildcards are used in different positions for different filters.
//
// // We don't want to pull all logs from the mailbox contract, so we need to build a filter.
// // This filter might not be as restrictive as the MatchingList filter, but it will be a superset.
// // Build this filter by combining all .... but if there is a wildcard, we can't do that.
// let mut origins: HashSet<u32> = HashSet::new();
// let mut destinations: HashSet<u32> = HashSet::new();
// let mut senders: HashSet<H256> = HashSet::new();
// let mut recipients: HashSet<H256> = HashSet::new();
// if let Some(list) = &params.criteria.0 {
//     for item in list {
//         include_filter_items_in_set(&item.origin_domain, &mut origins);
//         include_filter_items_in_set(&item.sender_address, &mut senders);
//         include_filter_items_in_set(&item.destination_domain, &mut destinations);
//         include_filter_items_in_set(&item.recipient_address, &mut recipients);
//     }
// }
//
// fn include_filter_items_in_set<T: Copy + PartialEq + Eq + Hash>(item: &Filter<T>, set: &mut HashSet<T>) {
//     if let Filter::Enumerated(vec) = item {
//         for item in vec {
//             set.insert(*item);
//         }
//     }
// }

#[test]
fn test_resolve_block_numbers() {
    // Valid parameters
    assert_eq!((90, 100), resolve_block_numbers(100, -11, -1));
    assert_eq!((70, 80), resolve_block_numbers(100, 70, 80));
    assert_eq!((90, 95), resolve_block_numbers(100, -11, -6));
    assert_eq!((90, 95), resolve_block_numbers(100, 90, -6));
    assert_eq!((90, 95), resolve_block_numbers(100, -11, 95));

    // Best interpretation of parameters outside of block range
    assert_eq!((81, 81), resolve_block_numbers(100, -10, -20));
    assert_eq!((0, 0), resolve_block_numbers(100, -200, -150));
    assert_eq!((0, 100), resolve_block_numbers(100, -200, -1));
    assert_eq!((100, 100), resolve_block_numbers(100, 200, 300));
    assert_eq!((10, 10), resolve_block_numbers(100, 50, 10));
}
