// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::v13::{ActorError, Config, DEFAULT_HAMT_CONFIG, Map2};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::BigInt;
use fvm_shared4::bigint::Integer;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use indexmap::IndexMap;
use num_traits::Zero;

use super::TxnID;
use super::types::Transaction;

pub type PendingTxnMap<BS> = Map2<BS, TxnID, Transaction>;
pub const PENDING_TXN_CONFIG: Config = DEFAULT_HAMT_CONFIG;

/// Multisig actor state
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    pub signers: Vec<Address>,
    pub num_approvals_threshold: u64,
    pub next_tx_id: TxnID,

    // Linear unlock
    pub initial_balance: TokenAmount,
    pub start_epoch: ChainEpoch,
    pub unlock_duration: ChainEpoch,

    pub pending_txs: Cid,
}

impl State {
    /// Checks if `address` is in the list of signers
    pub fn is_signer(&self, address: &Address) -> bool {
        self.signers.contains(address)
    }

    /// Set locked amount in multisig state.
    pub fn set_locked(
        &mut self,
        start_epoch: ChainEpoch,
        unlock_duration: ChainEpoch,
        locked_amount: TokenAmount,
    ) {
        self.start_epoch = start_epoch;
        self.unlock_duration = unlock_duration;
        self.initial_balance = locked_amount;
    }

    /// Returns amount locked in multisig contract
    pub fn amount_locked(&self, elapsed_epoch: ChainEpoch) -> TokenAmount {
        if elapsed_epoch >= self.unlock_duration {
            return TokenAmount::zero();
        }
        if elapsed_epoch <= 0 {
            return self.initial_balance.clone();
        }

        let remaining_lock_duration = self.unlock_duration - elapsed_epoch;

        // locked = ceil(InitialBalance * remainingLockDuration / UnlockDuration)
        let numerator: TokenAmount = &self.initial_balance * remaining_lock_duration;
        let denominator = BigInt::from(self.unlock_duration);

        TokenAmount::from_atto(numerator.atto().div_ceil(&denominator))
    }

    /// Iterates all pending transactions and removes an address from each list of approvals,
    /// if present.  If an approval list becomes empty, the pending transaction is deleted.
    pub fn purge_approvals<BS: Blockstore>(
        &mut self,
        store: &BS,
        addr: &Address,
    ) -> Result<(), ActorError> {
        let mut txns =
            PendingTxnMap::load(store, &self.pending_txs, PENDING_TXN_CONFIG, "pending txns")?;

        // Identify transactions that need updating
        let mut txn_ids_to_purge = IndexMap::new();
        txns.for_each(|tx_id, txn: &Transaction| {
            for approver in txn.approved.iter() {
                if approver == addr {
                    txn_ids_to_purge.insert(tx_id, txn.clone());
                }
            }
            Ok(())
        })?;

        // Update or remove those transactions.
        for (tx_id, mut txn) in txn_ids_to_purge {
            txn.approved.retain(|approver| approver != addr);

            if !txn.approved.is_empty() {
                txns.set(&tx_id, txn)?;
            } else {
                txns.delete(&tx_id)?;
            }
        }

        self.pending_txs = txns.flush()?;
        Ok(())
    }
}
