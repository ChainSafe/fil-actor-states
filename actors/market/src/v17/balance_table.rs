// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::actor_error_v17;
use fil_actors_shared::v17::{ActorContext, ActorError, Config, DEFAULT_HAMT_CONFIG, Map2};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use num_traits::Zero;

/// Balance table which handles getting and updating token balances specifically
pub struct BalanceTable<BS: Blockstore>(pub Map2<BS, Address, TokenAmount>);

const CONF: Config = Config {
    bit_width: 6,
    ..DEFAULT_HAMT_CONFIG
};

impl<BS> BalanceTable<BS>
where
    BS: Blockstore,
{
    /// Initializes a new empty balance table
    pub fn new(bs: BS, name: &'static str) -> Self {
        Self(Map2::empty(bs, CONF, name))
    }

    /// Initializes a balance table from a root Cid
    pub fn from_root(bs: BS, cid: &Cid, name: &'static str) -> Result<Self, ActorError> {
        Ok(Self(Map2::load(bs, cid, CONF, name)?))
    }

    /// Retrieve root from balance table
    pub fn root(&mut self) -> Result<Cid, ActorError> {
        self.0.flush()
    }

    /// Gets token amount for given address in balance table
    pub fn get(&self, key: &Address) -> Result<TokenAmount, ActorError> {
        if let Some(v) = self.0.get(key)? {
            Ok(v.clone())
        } else {
            Ok(TokenAmount::zero())
        }
    }

    /// Adds token amount to previously initialized account.
    pub fn add(&mut self, key: &Address, value: &TokenAmount) -> Result<(), ActorError> {
        let prev = self.get(key)?;
        let sum = &prev + value;
        if sum.is_negative() {
            Err(actor_error_v17!(
                illegal_argument,
                "negative balance for {} adding {} to {}",
                key,
                value,
                prev
            ))
        } else if sum.is_zero() && !prev.is_zero() {
            self.0.delete(key).context("adding balance")?;
            Ok(())
        } else {
            self.0.set(key, sum).context("adding balance")?;
            Ok(())
        }
    }

    /// Subtracts up to the specified amount from a balance, without reducing the balance
    /// below some minimum.
    /// Returns the amount subtracted (always positive or zero).
    pub fn subtract_with_minimum(
        &mut self,
        key: &Address,
        req: &TokenAmount,
        floor: &TokenAmount,
    ) -> Result<TokenAmount, ActorError> {
        let prev = self.get(key)?;
        let available = std::cmp::max(TokenAmount::zero(), prev - floor);
        let sub: TokenAmount = std::cmp::min(&available, req).clone();

        if sub.is_positive() {
            self.add(key, &-sub.clone())
                .context("subtracting balance")?;
        }

        Ok(sub)
    }

    /// Subtracts value from a balance, and errors if full amount was not substracted.
    pub fn must_subtract(&mut self, key: &Address, req: &TokenAmount) -> Result<(), ActorError> {
        let prev = self.get(key)?;

        if req > &prev {
            Err(actor_error_v17!(
                illegal_argument,
                "negative balance for {} subtracting {} from {}",
                key,
                req,
                prev
            ))
        } else {
            self.add(key, &-req)
        }
    }

    /// Returns total balance held by this balance table
    #[allow(dead_code)]
    pub fn total(&self) -> Result<TokenAmount, ActorError> {
        let mut total = TokenAmount::zero();
        self.0.for_each(|_, v: &TokenAmount| {
            total += v;
            Ok(())
        })?;

        Ok(total)
    }
}
