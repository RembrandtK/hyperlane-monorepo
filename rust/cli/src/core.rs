//! Core utilities for CLI implementation.

use color_eyre::{eyre::Context, Result};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::Wallet;
use ethers::{
    prelude::SignerMiddleware,
    signers::{LocalWallet, Signer},
};
use hyperlane_core::H256;
use hyperlane_core::{
    HyperlaneDomain, HyperlaneDomainProtocol, HyperlaneDomainType, KnownHyperlaneDomain,
};
use std::sync::Arc;

/// Get the Hyperlane domain for a given chain ID.
pub fn get_hyperlane_domain(chain_id: u32) -> HyperlaneDomain {
    match KnownHyperlaneDomain::try_from(chain_id) {
        Ok(domain) => HyperlaneDomain::Known(domain),
        Err(_) => HyperlaneDomain::Unknown {
            domain_id: chain_id,
            domain_name: "Unknown".to_string(),
            domain_type: HyperlaneDomainType::Unknown,
            domain_protocol: HyperlaneDomainProtocol::Ethereum,
        },
    }
}

/// Get the chain client.
pub fn get_client<S: Signer>(
    provider: Arc<Provider<Http>>,
    sender_wallet: S,
) -> Arc<SignerMiddleware<Arc<Provider<Http>>, S>> {
    Arc::new(SignerMiddleware::new(provider, sender_wallet))
}

/// Get a wallet from a private key for signing transactions.
pub fn get_wallet(key: H256, chain_id: u32) -> Result<Wallet<SigningKey>> {
    let sender_wallet = LocalWallet::from_bytes(key.as_bytes())
        .context("Failed to create wallet from private key")?
        .with_chain_id(chain_id);
    Ok(sender_wallet)
}

/// Print the Hyperlane domain for a given chain ID in a human-readable format.
pub fn print_hyperlane_domain(chain_id: u32) {
    let domain = get_hyperlane_domain(chain_id);
    println!(
        "\nHyperlane domain: {:?}, {:?} ({}: {})",
        domain.domain_type(),
        domain.domain_protocol(),
        domain.id(),
        domain.name()
    );
}

/// Get the chain provider given the RPC URL, printing summary information.
pub async fn get_provider(
    rpc_url: String,
) -> Result<(Arc<Provider<Http>>, u32), color_eyre::Report> {
    println!("Connecting to: {rpc_url}");
    let provider = Arc::new(
        Provider::<Http>::try_from(rpc_url.clone())
            .with_context(|| format!("Failed to create provider for {rpc_url}"))?,
    );

    let chain_id = provider
        .get_chainid()
        .await
        .with_context(|| format!("Failed to retrieve chain id for {rpc_url}"))?
        .as_u32();

    print_hyperlane_domain_details("Origin", chain_id);

    Ok((provider, chain_id))
}

/// Print the Hyperlane domain for a given chain ID in a human-readable format.
pub fn print_hyperlane_domain_details(context: &str, chain_id: u32) {
    let domain = get_hyperlane_domain(chain_id);
    let domain_name = match &domain {
        HyperlaneDomain::Known(domain) => format!("{domain:?}"),
        HyperlaneDomain::Unknown { domain_name, .. } => domain_name.to_owned(),
    };

    println!(
        "{}: {} {} {:?} {:?}",
        context,
        chain_id,
        domain_name,
        domain.domain_protocol(),
        domain.domain_type()
    );
}

/// Format an Option\<T\> into either display format of T or "None".
pub fn option_into_display_string<T: std::fmt::Display>(opt: &Option<T>) -> String {
    match opt {
        Some(value) => format!("{}", value),
        None => String::from("None"),
    }
}

/// Format an Option\<T\> into debug format of T or "None".
pub fn option_into_debug_string<T: std::fmt::Debug>(opt: &Option<T>) -> String {
    match opt {
        Some(value) => format!("{:?}", value),
        None => String::from("None"),
    }
}
