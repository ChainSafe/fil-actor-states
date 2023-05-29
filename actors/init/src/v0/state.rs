// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use address::{Address, Protocol};
use encoding::tuple::*;
use encoding::Cbor;
use forest_cid::Cid;
use fvm_shared::{ActorID, HAMT_BIT_WIDTH};
use ipld_blockstore::BlockStore;
use ipld_hamt::{BytesKey, Error as HamtError, Hamt};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error as StdError;

/// Defines first available ID address after builtin actors
pub const FIRST_NON_SINGLETON_ADDR: ActorID = 100;

/// Map type to be used within actors. The underlying type is a hamt.
pub type Map<'bs, BS, V> = Hamt<'bs, BS, V, BytesKey>;

/// State is reponsible for creating
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub address_map: Cid,
    pub next_id: ActorID,
    pub network_name: String,
}

impl State {
    pub fn new(address_map: Cid, network_name: String) -> Self {
        Self {
            address_map,
            next_id: FIRST_NON_SINGLETON_ADDR,
            network_name,
        }
    }

    /// Allocates a new ID address and stores a mapping of the argument address to it.
    /// Returns the newly-allocated address.
    pub fn map_address_to_new_id<BS: BlockStore>(
        &mut self,
        store: &BS,
        addr: &Address,
    ) -> Result<Address, HamtError> {
        let id = self.next_id;
        self.next_id += 1;

        let mut map = make_map_with_root(&self.address_map, store)?;
        map.set(addr.to_bytes().into(), id)?;
        self.address_map = map.flush()?;

        Ok(Address::new_id(id))
    }

    /// ResolveAddress resolves an address to an ID-address, if possible.
    /// If the provided address is an ID address, it is returned as-is.
    /// This means that mapped ID-addresses (which should only appear as values, not keys) and
    /// singleton actor addresses (which are not in the map) pass through unchanged.
    ///
    /// Returns an ID-address and `true` if the address was already an ID-address or was resolved
    /// in the mapping.
    /// Returns an undefined address and `false` if the address was not an ID-address and not found
    /// in the mapping.
    /// Returns an error only if state was inconsistent.
    pub fn resolve_address<BS: BlockStore>(
        &self,
        store: &BS,
        addr: &Address,
    ) -> Result<Option<Address>, Box<dyn StdError>> {
        if addr.protocol() == Protocol::ID {
            return Ok(Some(*addr));
        }

        let map = make_map_with_root(&self.address_map, store)?;

        Ok(map.get(&addr.to_bytes())?.copied().map(Address::new_id))
    }
}

impl Cbor for State {}

/// Create a map with a root cid.
#[inline]
pub fn make_map_with_root<'bs, BS, V>(
    root: &Cid,
    store: &'bs BS,
) -> Result<Map<'bs, BS, V>, HamtError>
where
    BS: BlockStore,
    V: DeserializeOwned + Serialize,
{
    Map::<_, V>::load_with_bit_width(root, store, HAMT_BIT_WIDTH)
}
