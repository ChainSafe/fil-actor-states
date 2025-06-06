// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::v13::network::EPOCHS_IN_HOUR;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::{Error, RawBytes, strict_bytes, to_vec};
use fvm_shared4::MethodNum;
use fvm_shared4::address::Address;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::crypto::signature::Signature;
use fvm_shared4::econ::TokenAmount;

use super::Merge;

/// Maximum number of lanes in a channel
pub const MAX_LANE: u64 = i64::MAX as u64;

pub const SETTLE_DELAY: ChainEpoch = EPOCHS_IN_HOUR * 12;

// Maximum byte length of a secret that can be submitted with a payment channel update.
pub const MAX_SECRET_SIZE: usize = 256;

pub const LANE_STATES_AMT_BITWIDTH: u32 = 3;

/// Constructor parameters for payment channel actor
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub from: Address,
    pub to: Address,
}

/// A voucher is sent by `from` to `to` off-chain in order to enable
/// `to` to redeem payments on-chain in the future
#[derive(Debug, Clone, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct SignedVoucher {
    /// ChannelAddr is the address of the payment channel this signed voucher is valid for
    pub channel_addr: Address,
    /// Min epoch before which the voucher cannot be redeemed
    pub time_lock_min: ChainEpoch,
    /// Max epoch beyond which the voucher cannot be redeemed
    /// set to 0 means no timeout
    pub time_lock_max: ChainEpoch,
    /// (optional) Used by `to` to validate
    #[serde(with = "strict_bytes")]
    pub secret_pre_image: Vec<u8>,
    /// (optional) Specified by `from` to add a verification method to the voucher
    pub extra: Option<ModVerifyParams>,
    /// Specifies which lane the Voucher merges into (will be created if does not exist)
    pub lane: u64,
    /// Set by `from` to prevent redemption of stale vouchers on a lane
    pub nonce: u64,
    /// Amount voucher can be redeemed for
    pub amount: TokenAmount,
    /// (optional) Can extend channel min_settle_height if needed
    pub min_settle_height: ChainEpoch,

    /// (optional) Set of lanes to be merged into `lane`
    pub merges: Vec<Merge>,

    /// Sender's signature over the voucher (sign on none)
    pub signature: Option<Signature>,
}

impl SignedVoucher {
    pub fn signing_bytes(&self) -> Result<Vec<u8>, Error> {
        /// Helper struct to avoid cloning for serializing structure.
        #[derive(Serialize_tuple)]
        struct SignedVoucherSer<'a> {
            pub channel_addr: &'a Address,
            pub time_lock_min: ChainEpoch,
            pub time_lock_max: ChainEpoch,
            #[serde(with = "strict_bytes")]
            pub secret_pre_image: &'a [u8],
            pub extra: &'a Option<ModVerifyParams>,
            pub lane: u64,
            pub nonce: u64,
            pub amount: &'a TokenAmount,
            pub min_settle_height: ChainEpoch,
            pub merges: &'a [Merge],
            pub signature: (),
        }
        let osv = SignedVoucherSer {
            channel_addr: &self.channel_addr,
            time_lock_min: self.time_lock_min,
            time_lock_max: self.time_lock_max,
            secret_pre_image: &self.secret_pre_image,
            extra: &self.extra,
            lane: self.lane,
            nonce: self.nonce,
            amount: &self.amount,
            min_settle_height: self.min_settle_height,
            merges: &self.merges,
            signature: (),
        };
        // Cbor serialize struct
        to_vec(&osv)
    }
}

/// Modular Verification method
#[derive(Debug, Clone, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ModVerifyParams {
    pub actor: Address,
    pub method: MethodNum,
    pub data: RawBytes,
}

/// Payment Verification parameters
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PaymentVerifyParams {
    pub extra: RawBytes,
    #[serde(with = "strict_bytes")]
    pub proof: Vec<u8>,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct UpdateChannelStateParams {
    pub sv: SignedVoucher,
    #[serde(with = "strict_bytes")]
    pub secret: Vec<u8>,
    // * proof removed in v2
}

impl From<SignedVoucher> for UpdateChannelStateParams {
    fn from(sv: SignedVoucher) -> Self {
        UpdateChannelStateParams { secret: vec![], sv }
    }
}
