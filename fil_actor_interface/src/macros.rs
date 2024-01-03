// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

/// `parse_pending_transactions` is a macro for parsing pending transactions and populating a vector with transaction data.
/// It has three different patterns to match based on the provided arguments.
///
/// # Arguments
/// * `$res:ident` - A mutable reference to a vector where parsed transactions will be pushed.
/// * `$txns:expr` - An expression that yields a collection of transactions to be parsed.
/// * `$from_address:expr` - (Optional, based on pattern) A function for transforming address between different versions.
/// * `$from_token:expr` - (Optional, based on pattern) A function for transforming token between different versions.
/// * `true/false` - (Optional, based on pattern) A boolean flag to determine the parsing strategy for transaction id.
///
/// # Usage
/// This macro supports three different invocation patterns:
///
/// 1. When `:decode` is passed as the last argument, it expects `$from_address` and `$from_token` to transform between different versions.
///    The transaction ID is extracted and decoded using `integer_encoding::VarInt::decode_var`.
///
/// 2. When `:no_decode` is passed as the last argument, it also expects `$from_address` and `$from_token` to transform between different versions,
///    but uses the transaction id directly as provided in `$txns`.
///
/// 3. When only `$res` and `$txns` are provided, it performs a basic parsing without transforming the 'to' address and 'value' fields.
///    It also decodes the transaction ID using `integer_encoding::VarInt::decode_var`.
#[macro_export]
macro_rules! parse_pending_transactions {
    ($res:ident, $txns:expr, $from_address:expr, $from_token:expr, :decode) => {
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
    ($res:ident, $txns:expr, $from_address:expr, $from_token:expr, :no_decode) => {
        $txns.for_each(|tx_id, txn| {
            $res.push(Transaction {
                id: tx_id.0,
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
