// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#[macro_export]
macro_rules! parse_pending_transactions {
    ($res:ident, $txns:expr, $from_address:expr, $from_token:expr) => {
        $txns.for_each(|tx_key, txn| {
            match integer_encoding::VarInt::decode_var(&tx_key) {
                Some((tx_id, _)) => {
                    $res.push(Transaction {
                        id: tx_id,
                        to: $from_address(txn.to),
                        value: $from_token(txn.value.clone()),
                        method: txn.method,
                        params: txn.params.clone(),
                        approved: txn
                            .approved
                            .iter()
                            .map(|&addr| $from_address(addr))
                            .collect(),
                    });
                }
                None => anyhow::bail!("Error decoding varint"),
            }
            Ok(())
        })?;
    };
    ($res:ident, $txns:expr) => {
        $txns.for_each(|tx_key, txn| {
            match integer_encoding::VarInt::decode_var(&tx_key) {
                Some((tx_id, _)) => {
                    $res.push(Transaction {
                        id: tx_id,
                        to: txn.to,
                        value: txn.value.clone(),
                        method: txn.method,
                        params: txn.params.clone(),
                        approved: txn.approved.clone(),
                    });
                }
                None => anyhow::bail!("Error decoding varint"),
            }
            Ok(())
        })?;
    };
}
