//! Execution parameters for the CLI.

use std::path::PathBuf;

use crate::arg::*;
use color_eyre::eyre::eyre;
use color_eyre::{eyre::Context, Report, Result};
use hyperlane_core::{H160, H256, U256};
use relayer::settings::matching_list::MatchItem;
use relayer::settings::MatchingList;

/// Execution parameters for the CLI.
#[derive(Debug, PartialEq)]
pub struct Params {
    // Universal
    /// RPC URL of the chain to connect to.
    pub rpc_url: String,
    /// Whether to print verbose output.
    pub verbose: bool,
    /// Number of blocks to wait for transaction confirmation.
    pub confirmations: usize,
    /// Whether to print debug output.
    pub debug: bool,
    /// Private key of the sender.
    pub key: Option<H256>,

    // Actions
    /// Whether to dispatch a message.
    pub dispatch: bool,
    /// Whether to pay for a message.
    pub pay: bool,
    /// Whether to deploy a mock contract.
    pub deploy_mock: bool,

    // For dispatch
    /// Address of the mailbox contract.
    pub mailbox_address: Option<H160>,
    /// Destination domain of the message.
    pub dest_id: Option<u32>,
    /// Address of the recipient contract.
    pub recipient_address: Option<H160>,
    /// Payload of the message.
    pub payload: Option<Vec<u8>>,

    // For pay
    /// Address of the paymaster contract.
    pub paymaster_address: Option<H160>,
    /// Message ID of the message to pay for.
    pub msg_id: Option<H256>,
    /// Gas to pay for.
    pub gas: U256,

    // For query
    /// Matching list for query.
    pub matching_list: MatchingList,
    /// Start block for query.
    pub start_block: i32,
    /// End block for query.
    pub end_block: i32,
}

impl TryFrom<CliArgs> for Params {
    type Error = Report;

    fn try_from(args: CliArgs) -> Result<Self> {
        let matching_list = matching_list_from_args(&args)?;

        Ok(Self {
            rpc_url: args.url,
            verbose: args.verbose,
            confirmations: args.confirm,
            debug: args.debug,
            key: args.key,

            dispatch: args.dispatch,
            pay: args.pay,
            deploy_mock: args.deploy_mock,

            mailbox_address: args.mailbox,
            dest_id: args.dest,
            recipient_address: args.recipient,
            payload: if args.dispatch {
                Some(read_payload(&args.payload, &args.input)?)
            } else {
                None
            },

            paymaster_address: args.paymaster,
            msg_id: args.message_id,
            gas: args.gas.into(),

            matching_list,
            start_block: args.start_block,
            end_block: args.end_block,
        })
    }
}

fn read_payload(hex_str: &Option<String>, file: &Option<PathBuf>) -> Result<Vec<u8>> {
    if file.is_some() == hex_str.is_some() {
        return Err(eyre!("Specify exactly one of --payload and --file"));
    }

    let payload = if let Some(file) = file {
        std::fs::read(file)
            .with_context(|| format!("Failed to read payload from '{}'", file.to_string_lossy()))?
    } else if let Some(hex_str) = hex_str {
        let hex_str = hex_str.trim_start_matches("0x");
        hex::decode(hex_str)
            .with_context(|| format!("Invalid hex string for payload {}", hex_str))?
    } else {
        // Should not get here due to earlier check.
        return Err(eyre!("Specify exactly one of --payload and --file"));
    };
    Ok(payload)
}

fn matching_list_from_args(args: &CliArgs) -> Result<MatchingList> {
    let mut criteria: Vec<String> = args.query.clone();

    if let Some(file) = &args.query_file {
        let file_criteria = std::fs::read_to_string(file).with_context(|| {
            format!(
                "Failed to read query criteria from '{}'",
                file.to_string_lossy()
            )
        })?;

        // Add it to criteria Vec and let matching_list_from_criteria handle it.
        criteria.push(file_criteria);
    }

    matching_list_from_criteria(&criteria)
}

/// Create [`MatchingList`] from a vector criteria specification strings.
pub fn matching_list_from_criteria(criteria: &Vec<String>) -> Result<MatchingList> {
    let mut matching_list: Vec<MatchItem> = vec![];
    for criteria in criteria {
        let criteria = criteria.trim();
        if criteria.is_empty() {
            continue; // Do not add or try to parse empty match criteria
        }
        if criteria.starts_with('[') {
            // JSON array of matching criteria
            if let MatchingList(Some(list)) = serde_json::from_str(criteria)? {
                matching_list.extend(list);
            }
        } else if criteria.starts_with('{') {
            // JSON individual matching criteria
            matching_list.push(serde_json::from_str(criteria)?);
        } else {
            // CSV (semicolon separated CSV parts) individual matching criteria
            // Split by whitespace to allow multiple CSV criteria in a single string, particularly when read from a file.
            for csv in criteria.split_whitespace() {
                matching_list.push(MatchItem::from_csv(csv)?);
            }
        }
    }

    Ok(MatchingList::from_elements(matching_list))
}
