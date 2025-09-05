// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::v8::DealWeight;
use fvm_ipld_encoding::BytesSer;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::crypto::signature::Signature;
use fvm_shared::econ::TokenAmount;
use fvm_shared::piece::PaddedPieceSize;
use ipld_core::ipld::Ipld;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, PartialEq)]
pub enum Label {
    String(String),
    Bytes(Vec<u8>),
}

/// Serialize the Label like an untagged enumerable.
impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Label::String(v) => v.serialize(serializer),
            Label::Bytes(v) => BytesSer(v).serialize(serializer),
        }
    }
}

impl TryFrom<Ipld> for Label {
    type Error = String;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => Ok(Label::String(s)),
            Ipld::Bytes(b) => Ok(Label::Bytes(b)),
            other => Err(format!(
                "Expected `Ipld::String` or `Ipld::Bytes`, got {:#?}",
                other
            )),
        }
    }
}

/// Deserialize the Label like an untagged enumerable.
impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ipld::deserialize(deserializer).and_then(|ipld| ipld.try_into().map_err(de::Error::custom))
    }
}

impl Label {
    pub fn len(&self) -> usize {
        match self {
            Label::String(s) => s.len(),
            Label::Bytes(b) => b.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Label::String(s) => s.is_empty(),
            Label::Bytes(b) => b.is_empty(),
        }
    }
}

/// Note: Deal Collateral is only released and returned to clients and miners
/// when the storage deal stops counting towards power. In the current iteration,
/// it will be released when the sector containing the storage deals expires,
/// even though some storage deals can expire earlier than the sector does.
/// Collaterals are denominated in `PerEpoch` to incur a cost for self dealing or
/// minimal deals that last for a long time.
/// Note: `ClientCollateralPerEpoch` may not be needed and removed pending future confirmation.
/// There will be a Minimum value for both client and provider deal collateral.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct DealProposal {
    pub piece_cid: Cid,
    pub piece_size: PaddedPieceSize,
    pub verified_deal: bool,
    pub client: Address,
    pub provider: Address,

    /// Arbitrary client chosen label to apply to the deal
    // ! This is the field that requires unsafe unchecked utf8 deserialization
    pub label: Label,

    // Nominal start epoch. Deal payment is linear between StartEpoch and EndEpoch,
    // with total amount StoragePricePerEpoch * (EndEpoch - StartEpoch).
    // Storage deal must appear in a sealed (proven) sector no later than StartEpoch,
    // otherwise it is invalid.
    pub start_epoch: ChainEpoch,
    pub end_epoch: ChainEpoch,
    pub storage_price_per_epoch: TokenAmount,

    pub provider_collateral: TokenAmount,
    pub client_collateral: TokenAmount,
}

impl DealProposal {
    pub fn duration(&self) -> ChainEpoch {
        self.end_epoch - self.start_epoch
    }
    /// Computes weight for a deal proposal, which is a function of its size and duration.
    pub fn weight(&self) -> DealWeight {
        DealWeight::from(self.duration()) * self.piece_size.0
    }
    pub fn total_storage_fee(&self) -> TokenAmount {
        self.storage_price_per_epoch.clone() * self.duration() as u64
    }
    pub fn client_balance_requirement(&self) -> TokenAmount {
        &self.client_collateral + self.total_storage_fee()
    }
    pub fn provider_balance_requirement(&self) -> &TokenAmount {
        &self.provider_collateral
    }
    pub fn cid(&self) -> anyhow::Result<Cid> {
        let bytes = fvm_ipld_encoding::to_vec(self)?;
        Ok(Cid::new_v1(
            fvm_ipld_encoding::DAG_CBOR,
            Code::Blake2b256.digest(&bytes),
        ))
    }
}

/// `ClientDealProposal` is a `DealProposal` signed by a client
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ClientDealProposal {
    pub proposal: DealProposal,
    pub client_signature: Signature,
}

#[derive(Clone, Debug, PartialEq, Copy, Serialize_tuple, Deserialize_tuple)]
pub struct DealState {
    // -1 if not yet included in proven sector
    pub sector_start_epoch: ChainEpoch,
    // -1 if deal state never updated
    pub last_updated_epoch: ChainEpoch,
    // -1 if deal never slashed
    pub slash_epoch: ChainEpoch,
}

#[cfg(feature = "arb")]
impl quickcheck::Arbitrary for DealProposal {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use fvm_ipld_encoding::DAG_CBOR;
        use multihash_codetable::Code::Blake2b256;

        Self {
            piece_cid: Cid::new_v1(DAG_CBOR, Blake2b256.digest(String::arbitrary(g).as_bytes())),
            verified_deal: bool::arbitrary(g),
            piece_size: PaddedPieceSize(u64::arbitrary(g)),
            // address ids greater than u63 upper bound are not supported on go side
            client: Address::new_id(u32::arbitrary(g) as _),
            provider: Address::new_id(u32::arbitrary(g) as _),
            label: Label::String(String::arbitrary(g)),
            start_epoch: i64::arbitrary(g),
            end_epoch: i64::arbitrary(g),
            storage_price_per_epoch: TokenAmount::from_atto(u64::arbitrary(g)),
            provider_collateral: TokenAmount::from_atto(u64::arbitrary(g)),
            client_collateral: TokenAmount::from_atto(u64::arbitrary(g)),
        }
    }
}

#[cfg(all(test, feature = "arb"))]
mod tests {
    use std::process::Command;

    use anyhow::*;
    use fil_actors_test_utils::go_compat::{ensure_go_mod_prepared, go_compat_tests_dir};
    use pretty_assertions::assert_eq;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_deal_proposal_cid(proposal: DealProposal) -> Result<()> {
        ensure_go_mod_prepared();

        let bytes = fvm_ipld_encoding::to_vec(&proposal)?;

        let app = Command::new("go")
            .args([
                "run",
                "actors/market/v8/test_deal_proposal_cid.go",
                "--data",
                hex::encode(bytes).as_str(),
            ])
            .current_dir(go_compat_tests_dir()?)
            .output()?;

        if !app.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&app.stderr));
            anyhow::bail!("Fail to run go test");
        }

        let cid_from_go = String::from_utf8_lossy(&app.stdout);

        assert_eq!(proposal.cid()?.to_string(), cid_from_go);

        Ok(())
    }
}
