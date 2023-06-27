//! # Hyperlane CLI
//!
//! The `hl` command-line application allows you to send Hyperlane messages via a Hyperlane mailbox.
//! This application provides the ability to test chain connections, dispatch messages, pay for gas,
//! query messages, and fetch help details.
//!
//! # Usage
//! ```
//! hl <URL> [OPTIONS]
//! ```
//!
//! # Arguments
//! - `<URL>`: The RPC URL for the chain to call.
//!
//! # Actions
//! - `--dispatch`: Dispatches a message to the destination chain via the Hyperlane mailbox contract.
//! - `--pay`: Pays for the gas of delivery on the destination chain via the Hyperlane gas paymaster contract.
//! - `--query`: Queries for Hyperlane messages that were sent from the origin chain.
//! - `--help`: Prints help message.
//!
//! Run `hl --help` for more information.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::Arc;

use clap::Parser;
use cli::{action, arg::CliArgs, core, param::Params};
use color_eyre::{eyre::eyre, Result};
use ethers::providers::{Http, Provider};
use hyperlane_base::setup_error_handling;

/// # High level execution flow
///
/// Command line arguments are parsed using the clap crate into [CliArgs](cli::arg::CliArgs).
///
/// The CommandArgs struct is then converted into a [Params](cli::param::Params) struct.
/// * Syntactic correctness of the arguments is checked without connecting to the chain.
///
/// Parameters are then passed to execution functions as required.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_error_handling()?;

    let params = get_params_from_args()?;
    if params.debug {
        println!("{params:#?}");
        return Ok(());
    }

    check_params(&params)?;

    run(&params).await?;

    Ok(())
}

fn get_params_from_args() -> Result<Params, color_eyre::Report> {
    let params: Params = CliArgs::parse().try_into()?;
    Ok(params)
}

async fn run(params: &Params) -> Result<(), color_eyre::Report> {
    let (provider, chain_id) = core::get_provider(params.rpc_url.clone()).await?;
    if let Some(dest_id) = params.dest_id {
        core::print_hyperlane_domain_details("Destination", dest_id);
    };

    if params.dispatch || params.pay || params.deploy_mock {
        perform_transactions(params, chain_id, Arc::clone(&provider)).await?;
    };

    query_logs(params, provider, chain_id).await?;

    Ok(())
}

async fn query_logs(params: &Params, provider: Arc<Provider<Http>>, chain_id: u32) -> Result<()> {
    if let Some(_match_list) = &params.matching_list.0 {
        action::query(
            Arc::clone(&provider),
            chain_id,
            params
                .mailbox_address
                .ok_or_else(|| eyre!("Missing mailbox address for query"))?,
            params.start_block,
            params.end_block,
            params.matching_list.clone(),
            params.verbose,
        )
        .await?;
    };

    Ok(())
}

/// Performs transactions for action options (deploy_mock, dispatch, pay).
async fn perform_transactions(
    params: &Params,
    chain_id: u32,
    provider: Arc<Provider<Http>>,
) -> Result<()> {
    let sender_wallet = core::get_wallet(
        params.key.ok_or_else(|| eyre!("No signing key provided"))?,
        chain_id,
    )?;
    let client = core::get_client(Arc::clone(&provider), sender_wallet.clone());

    let dest_id = params
        .dest_id
        .ok_or_else(|| eyre!("Missing destination ID"))?;

    if params.deploy_mock {
        action::deploy_mock_contracts(Arc::clone(&client), chain_id, dest_id).await?;
    };

    let message_id = if params.dispatch {
        action::dispatch(
            Arc::clone(&client),
            params
                .mailbox_address
                .ok_or_else(|| eyre!("Missing mailbox address for dispatch"))?,
            dest_id,
            params
                .recipient_address
                .ok_or_else(|| eyre!("Missing recipient address for dispatch"))?,
            params
                .payload
                .as_ref()
                .ok_or_else(|| eyre!("Missing payload for dispatch"))?
                .clone(),
            params.confirmations,
            params.verbose,
        )
        .await?
    } else {
        params.msg_id
    };

    if params.pay {
        action::pay(
            sender_wallet,
            client,
            params
                .paymaster_address
                .ok_or_else(|| eyre!("Missing paymaster address for pay"))?,
            message_id.ok_or_else(|| eyre!("Missing message ID for pay"))?,
            dest_id,
            params.gas,
            params.confirmations,
            params.verbose,
        )
        .await?;
    };

    Ok(())
}

fn check_params(params: &Params) -> Result<()> {
    if params.dispatch || params.pay || params.deploy_mock {
        if params.key.is_none() {
            return Err(eyre!("No signing key provided"));
        }

        if params.dispatch {
            if params.recipient_address.is_none() {
                return Err(eyre!("Missing recipient address for dispatch"));
            }
            if params.payload.is_none() {
                return Err(eyre!("Missing payload for dispatch"));
            }
            if params.msg_id.is_some() {
                return Err(eyre!("Message ID is not allowed with dispatch"));
            }
        }

        if params.pay {
            if params.paymaster_address.is_none() {
                return Err(eyre!("Missing paymaster address for pay"));
            }
            if !params.dispatch && params.msg_id.is_none() {
                return Err(eyre!("Missing message ID for pay"));
            }
        }
    }

    Ok(())
}
