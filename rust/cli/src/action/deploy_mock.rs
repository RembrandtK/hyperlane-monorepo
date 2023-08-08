use std::sync::Arc;

use crate::mock;
use color_eyre::Result;
use ethers::providers::Middleware;
use hyperlane_core::H160;

/// Deploy a set of mock contracts to the provided chain.
pub async fn deploy_mock_contracts<M: Middleware + 'static>(
    client: Arc<M>,
    origin_id: u32,
    dest_id: u32,
) -> Result<()> {
    println!("Deploying mock contracts:");

    let environment = mock::environment::create_mock_environment_contract(
        Arc::clone(&client),
        origin_id,
        dest_id,
    )
    .await?;

    let recipient = mock::environment::create_test_recipient_contract(client.clone()).await?;
    let recipient_address = recipient.address();

    let mailbox_address: H160 = environment.mailboxes(origin_id).await?;
    let destination_mbox_addr = environment.mailboxes(dest_id).await?;

    println!("  Origin mailbox address: {:?}", mailbox_address);
    println!("  Destination mailbox address: {:?}", destination_mbox_addr);
    println!("  Recipient address: {:?}", recipient_address);

    Ok(())
}
