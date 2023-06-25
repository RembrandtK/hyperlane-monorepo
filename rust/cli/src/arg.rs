//! Command line argument parsing for Hyperlane CLI.

use hyperlane_core::{H160, H256};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]

/// CLI for Hyperlane message dispatch, delivery payment, and querying message logs.
pub struct CliArgs {
    /// RCP URL for chain to call
    #[clap()]
    pub url: String,

    /// Private key (optional, if needed to sign), as H256 hex string (64 characters)
    #[clap(long, short)]
    pub key: Option<H256>,

    /// Perform message dispatch. Requires mailbox, key, and payload
    #[clap(long, default_value = "false", default_missing_value = "true")]
    pub dispatch: bool,

    /// Perform gas payment. Requires paymaster, key, and either a message id or being run with dispatch
    #[clap(long, default_value = "false", default_missing_value = "true")]
    pub pay: bool,

    // TODO: Get JSON format working
    // / Query for messages, with criteria in either JSON or CSV format.
    ///
    /// CSV format (each item is a CSV list):
    ///     originDomain:senderAddress:destinationDomain:recipientAddress
    ///
    /// CSV example: 11155111,80001:0x05047e42F75eaFf3f6C7a347930F778FB41C5dD0:80001:0x36FdA966CfffF8a9Cdc814f546db0e6378bFef35
    // /
    // / JSON equivalent (outer list is optional if only one item):
    // / [{"origin_domain":[11155111,80001],"sender_address":"0x5047e42f75eaff3f6c7a347930f778fb41c5dd0","destination_domain":80001,"recipient_address":"0x36fda966cffff8a9cdc814f546db0e6378bfef35"}]
    #[arg(long)]
    pub query: Vec<String>,

    /// Deploy mock contracts for testing, typically on a local chain. Requires key and dest
    /// The mock contracts will not work with query or pay actions
    #[clap(
        long,
        default_value = "false",
        default_missing_value = "true",
        hide = true
    )]
    pub deploy_mock: bool,

    /// Mailbox contract address as H160 hex string (40 characters)
    #[clap(long)]
    pub mailbox: Option<H160>,

    /// Paymaster contract address as H160 hex string (40 characters)
    #[clap(long)]
    pub paymaster: Option<H160>,

    /// Destination chain identifier. Required for dispatch
    #[arg(short, long)]
    pub dest: Option<u32>,

    /// Recipient contract address as H160 hex string (40 characters)
    #[arg(short, long)]
    pub recipient: Option<H160>,

    /// Hex encoded message payload to send
    #[arg(short, long, conflicts_with = "input")]
    pub payload: Option<String>,

    /// Input file for message payload (bytes) to send.
    ///
    /// (Alternative to --payload, specify one or the other.)
    #[arg(short, long, conflicts_with = "payload")]
    pub input: Option<PathBuf>,

    // TODO: Add confirmation blocks option
    // /// Number of confirmation blocks to wait for
    // #[arg(long, default_value = "1")]
    //
    // pub confirmations: u32,
    /// Id of message to pay for
    #[arg(short, long, conflicts_with = "dispatch")]
    pub message_id: Option<H256>,

    /// Gas to pay on destination chain
    ///
    /// Will be converted according to gas price and exchange rate
    #[arg(long, default_value = "100000")]
    pub gas: u32,

    // /// Match criteria file in JSON format.
    // #[arg(short, long)]
    // pub file: Option<PathBuf>,

    // /// Maximum number of messages to return.
    // /// If negative, will return last N matching messages.
    // #[arg(short, long, default_value = "-10")]
    // pub max: i32,
    //
    /// Start block number to search from.
    ///
    /// If not specified, will search last 100 blocks.
    ///
    /// If negative (-n), will search from latest block + 1 - n.
    #[arg(short, long, default_value = "-1000")]
    pub start_block: i32,

    /// End block number to search to.
    ///
    /// If not specified, will search until latest block.
    ///
    /// If negative (-n), will search to latest block + 1 - n.
    #[arg(short, long, default_value = "-1")]
    pub end_block: i32,

    /// Do not run; print extracted parameters and exit.
    #[arg(long, default_value = "false", default_missing_value = "true")]
    pub debug: bool,

    /// Show verbose output (including transaction logs)
    #[clap(short, long)]
    pub verbose: bool,
}
