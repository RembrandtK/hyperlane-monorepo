use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
};

use crate::contracts::{
    DispatchFilter, DispatchIdFilter, GasPaymentFilter, ProcessFilter, ProcessIdFilter,
};
use color_eyre::Result;
use ethers::prelude::EthEvent;
use ethers::{abi::RawLog, types::Log};
use hyperlane_core::{HyperlaneMessage, RawHyperlaneMessage, H160, H256};

use super::MailboxLogType;

/// Dispatch and Process logs have different topic orders; this map is used to abstract that away.
pub struct LogItemMap {
    /// The type of log this map is for.
    pub event_type: MailboxLogType,

    /// The index of the sender topic.
    pub sender_topic_idx: usize,

    /// The index of the recipient topic.
    pub recipient_topic_idx: usize,

    /// The index of the domain topic.
    pub domain_topic_idx: usize,
}

impl LogItemMap {
    /// Create a new map for the given log type.
    pub fn new(log_type: MailboxLogType) -> Self {
        // TODO: Only need two of these, and just return the correct one (in Rc or Arc).
        Self {
            event_type: log_type,
            sender_topic_idx: match log_type {
                MailboxLogType::Dispatch => 1,
                MailboxLogType::Process => 2,
            },
            domain_topic_idx: match log_type {
                MailboxLogType::Dispatch => 2,
                MailboxLogType::Process => 1,
            },
            recipient_topic_idx: 3,
        }
    }
}

/// Provides a wrapper around a log item, abstracting away the differences between log types.
pub struct MailboxLogItem<'a> {
    /// The underlying log item.
    pub log: &'a Log,
    // pub map: Rc<LogItemMap>,
}

impl MailboxLogItem<'_> {
    // pub fn event_type(&self) -> MailboxLogType {
    //     self.map.event_type
    // }

    /// Provides readable names for the different log types.
    pub fn event_name(&self) -> String {
        let signature = self.event_signature();

        if signature == DispatchFilter::signature() {
            "Dispatch".to_string()
        } else if signature == DispatchIdFilter::signature() {
            "Dispatch ID".to_string()
        } else if signature == ProcessFilter::signature() {
            "Process".to_string()
        } else if signature == ProcessIdFilter::signature() {
            "Process ID".to_string()
        } else if signature == GasPaymentFilter::signature() {
            "Gas Payment".to_string()
        } else {
            signature.to_string()
        }
    }

    /// Attempt to extract the [`DispatchFilter`] from this log item.
    pub fn to_dispatch_event(&self) -> Result<Option<DispatchFilter>> {
        self.to_event()
    }

    /// Attempt to extract the [`DispatchIdFilter`] from this log item.
    pub fn to_dispatch_id_event(&self) -> Result<Option<DispatchIdFilter>> {
        self.to_event()
    }

    /// Attempt to extract the [`ProcessFilter`] from this log item.
    pub fn to_process_event(&self) -> Result<Option<ProcessFilter>> {
        self.to_event()
    }

    /// Attempt to extract the [`ProcessIdFilter`] from this log item.
    pub fn to_process_id_event(&self) -> Result<Option<ProcessIdFilter>> {
        self.to_event()
    }

    /// Attempt to extract the [`GasPaymentFilter`] from this log item.
    pub fn to_gas_pay_event(&self) -> Result<Option<GasPaymentFilter>> {
        self.to_event()
    }

    /// Get the signature of the event.
    pub fn event_signature(&self) -> H256 {
        self.log.topics[0]
    }

    /// Attempt to decode the event from the log item.
    pub fn to_event<E: EthEvent>(&self) -> Result<Option<E>> {
        Ok(if self.event_signature() == E::signature() {
            let raw_log = RawLog::from(self.log.clone());
            Some(E::decode_log(&raw_log)?)
        } else {
            None
        })
    }

    /// Attempt to extract the sender from the log item. Not all log items have a sender.
    pub fn sender(&self) -> Result<Option<H160>> {
        Ok(if let Some(dispatch) = self.to_dispatch_event()? {
            Some(dispatch.sender)
        } else {
            self.to_process_event()?
                .map(|process| H160::from_slice(&process.sender.as_slice()[12..]))
        })
    }

    /// Attempt to extract the recipient from the log item. Not all log items have a recipient.
    pub fn recipient(&self) -> Result<Option<H160>> {
        Ok(if let Some(dispatch) = self.to_dispatch_event()? {
            Some(H160::from_slice(&dispatch.recipient[12..]))
        } else {
            self.to_process_event()?.map(|process| process.recipient)
        })
    }

    /// Attempt to extract the destination domain from the log item.
    /// Not all log items have a destination domain.
    pub fn destination_domain(&self) -> Result<Option<u32>> {
        Ok(if let Some(dispatch) = self.to_dispatch_event()? {
            Some(dispatch.destination)
        } else {
            None
        })
    }

    /// Attempt to extract the origin domain from the log item.
    /// Not all log items have an origin domain.
    pub fn origin_domain(&self) -> Result<Option<u32>> {
        Ok(self.to_process_event()?.map(|process| process.origin))
    }

    /// Attempt to extract the [`HyperlaneMessage`] from the log item.
    /// Not applicable for all log items.
    pub fn hyperlane_message(&self) -> Result<Option<HyperlaneMessage>> {
        Ok(if let Some(dispatch) = self.to_dispatch_event()? {
            let raw_message: RawHyperlaneMessage = dispatch.message.to_vec();
            Some(raw_message.into())
        } else {
            None
        })
    }

    /// Attempt to extract the Hyperlane message id from the log item.
    /// Not applicable for all log items.
    pub fn message_id(&self) -> Result<Option<H256>> {
        Ok(if let Some(message) = self.hyperlane_message()? {
            Some(message.id())
        } else if let Some(dispatch) = self.to_dispatch_id_event()? {
            Some(H256::from_slice(&dispatch.message_id))
        } else {
            self.to_process_id_event()?
                .map(|process| H256::from_slice(&process.message_id))
        })
    }

    /// Block number of the log item.
    pub fn block_number(&self) -> Option<u64> {
        self.log.block_number.map(|index| index.as_u64())
    }

    /// Transaction hash of the log item.
    pub fn transaction_hash(&self) -> Option<H256> {
        self.log.transaction_hash
    }

    /// Log index of the log item.
    pub fn log_index(&self) -> Option<u64> {
        self.log.log_index.map(|index| index.as_u64())
    }

    /// Data of the log item.
    pub fn data(&self) -> &[u8] {
        &self.log.data
    }
}

/// Log items are tested for equality based on transaction hash; seperate items with the same transaction hash are considered equal.
impl PartialEq for MailboxLogItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Transaction hash uniquely identifies a transaction.
        self.transaction_hash() == other.transaction_hash()
    }
}

/// Partially order as transactions by block number, then by log index, and they by transaction hash.
///
/// This sorts transactions according to their order in the blockchain.
///
/// Order transactions with no block number after those with a block number.
impl PartialOrd for MailboxLogItem<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Sort by block number, with None (no block) last.
        let mut cmp = partial_cmp_some_lt_none(&self.block_number(), &other.block_number());

        if cmp == Some(Ordering::Equal) {
            // If None neither have a block so do not to compare on log index.
            cmp = partial_cmp_some_lt_none(&self.log_index(), &other.log_index());
        }

        if cmp == Some(Ordering::Equal) || cmp.is_none() {
            // Do not expect transaction hash to ever be None, but still a safe and correct comparison.
            partial_cmp_some_lt_none(&self.transaction_hash(), &other.transaction_hash())
        } else {
            cmp
        }
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}

/// Treat any `Some(_)` as coming before, so being less than, `None`.
///
/// This is used to sort transactions with no block number after transactions with a block number.
fn partial_cmp_some_lt_none<T: Ord>(a: &Option<T>, b: &Option<T>) -> Option<Ordering> {
    match a {
        Some(a) => match b {
            Some(b) => Some(a.cmp(b)),
            None => Some(Ordering::Less),
        },
        None => b.as_ref().map(|_| Ordering::Greater),
    }
}

impl Debug for MailboxLogItem<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailboxLogItem")
            .field("sender", &self.sender())
            .field("recipient", &self.recipient())
            .field("destination_domain", &self.destination_domain())
            .field("block_number", &self.block_number())
            .field("transaction_hash", &self.transaction_hash())
            .field("log_index", &self.log_index())
            .field("data", &hex::encode(self.data()))
            .finish()
    }
}

#[test]
fn test_some_lt_none() {
    for (a, cmp, b) in [
        (Some(1), Some(Ordering::Less), Some(2)),
        (Some(2), Some(Ordering::Greater), Some(1)),
        (Some(1), Some(Ordering::Equal), Some(1)),
        (Some(1), Some(Ordering::Less), None),
        (None, Some(Ordering::Greater), Some(1)),
        (None, None, None),
    ] {
        assert_eq!(partial_cmp_some_lt_none(&a, &b), cmp);
    }
}
