// Copyright 2019-2026 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{Result, ensure};
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use num_traits::Zero;

use super::invariants::validate_streams_state;
use crate::v19::state::{
    DENOM, ExplicitDistribution, RecipientTable, Stream, StreamAccrual, StreamsState, WeightRecord,
};
use crate::v19::types::RegisterStreamParams;

/// Builds and validates the streams a network upgrade installs: stream 1 alone at constant
/// `DENOM`, or streams 1 and 2 with equal and opposite slopes, starting weights summing to
/// `DENOM` and one full-share recipient, all starting at `activation_epoch`.
///
/// Port of go-state-types `ValidateMigrationStreams`:
/// <https://github.com/filecoin-project/go-state-types/blob/6cb27cf2e8be76d9b20f0d58d6d580cd99e31ce6/builtin/v19/reward/stream_invariants.go#L717>
pub fn validate_migration_streams(
    params: &[RegisterStreamParams],
    activation_epoch: ChainEpoch,
) -> Result<(StreamsState, Vec<StreamAccrual>)> {
    ensure!(
        params.len() == 1 || params.len() == 2,
        "bootstrap requires one or two streams"
    );
    for param in params {
        ensure!(
            param.activation_epoch == activation_epoch,
            "stream {} activation epoch {} does not match upgrade epoch {activation_epoch}",
            param.id,
            param.activation_epoch
        );
        ensure!(
            param.weight.t_start == activation_epoch,
            "stream {} weight start {} does not match upgrade epoch {activation_epoch}",
            param.id,
            param.weight.t_start
        );
    }

    if let [consensus] = params {
        let neutral = WeightRecord {
            v_start: DENOM,
            slope: 0,
            t_start: activation_epoch,
            floor: DENOM,
            cap: DENOM,
        };
        ensure!(
            consensus.id == 1 && consensus.distribution.is_none() && consensus.weight == neutral,
            "single-stream bootstrap must be implicit stream 1 at constant DENOM"
        );
    } else if let [consensus, explicit] = params {
        ensure!(
            consensus.id == 1 && explicit.id == 2,
            "split bootstrap stream IDs must be 1 and 2"
        );
        let distribution = match (&consensus.distribution, &explicit.distribution) {
            (None, Some(distribution)) => distribution,
            _ => anyhow::bail!("split bootstrap distribution forms are invalid"),
        };
        ensure!(
            consensus.weight.v_start <= DENOM
                && explicit.weight.v_start == DENOM - consensus.weight.v_start,
            "bootstrap starting weights must sum to denominator"
        );
        ensure!(
            consensus.weight.slope < 0
                && explicit.weight.slope > 0
                && consensus.weight.slope == -explicit.weight.slope,
            "bootstrap weight slopes are invalid"
        );
        ensure!(
            matches!(distribution.shares.as_slice(), [share] if share.share == DENOM),
            "explicit bootstrap requires one full-share recipient"
        );
    }

    let mut streams = StreamsState::default();
    let mut accrued = Vec::new();
    for param in params {
        let distribution = param
            .distribution
            .as_ref()
            .map(|init| ExplicitDistribution {
                writer: init.writer,
                shares: init.shares.clone(),
                payable: RecipientTable::default(),
                claimed_period: RecipientTable::default(),
            });
        if distribution.is_some() {
            accrued.push(StreamAccrual {
                id: param.id,
                amount: TokenAmount::zero(),
            });
        }
        streams.streams.push(Stream {
            id: param.id,
            weight: param.weight.clone(),
            distribution,
        });
    }
    validate_streams_state(&streams, &accrued, activation_epoch)?;
    Ok((streams, accrued))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v19::state::RecipientShare;
    use crate::v19::types::DistributionInit;
    use fvm_shared4::address::Address;

    const PERCENT: u64 = DENOM / 100;

    fn weight(v_start: u64, slope: i64, floor: u64, cap: u64) -> WeightRecord {
        WeightRecord {
            v_start: v_start * PERCENT,
            slope,
            t_start: 100,
            floor: floor * PERCENT,
            cap: cap * PERCENT,
        }
    }

    fn split_bootstrap() -> Vec<RegisterStreamParams> {
        vec![
            RegisterStreamParams {
                id: 1,
                weight: weight(95, -1, 50, 95),
                distribution: None,
                activation_epoch: 100,
            },
            RegisterStreamParams {
                id: 2,
                weight: weight(5, 1, 5, 10),
                distribution: Some(DistributionInit {
                    writer: Address::new_id(101),
                    shares: vec![RecipientShare {
                        recipient: Address::new_id(102),
                        share: DENOM,
                    }],
                }),
                activation_epoch: 100,
            },
        ]
    }

    #[test]
    fn split_bootstrap_installs_both_streams_and_one_zero_accrual() {
        let (streams, accrued) = validate_migration_streams(&split_bootstrap(), 100).unwrap();

        assert_eq!(
            streams,
            StreamsState {
                streams: vec![
                    Stream {
                        id: 1,
                        weight: weight(95, -1, 50, 95),
                        distribution: None,
                    },
                    Stream {
                        id: 2,
                        weight: weight(5, 1, 5, 10),
                        distribution: Some(ExplicitDistribution {
                            writer: Address::new_id(101),
                            shares: vec![RecipientShare {
                                recipient: Address::new_id(102),
                                share: DENOM,
                            }],
                            payable: RecipientTable::default(),
                            claimed_period: RecipientTable::default(),
                        }),
                    },
                ],
                tombstones: vec![],
                pending_writes: vec![],
            }
        );
        assert_eq!(
            accrued,
            vec![StreamAccrual {
                id: 2,
                amount: TokenAmount::zero(),
            }]
        );
    }

    #[test]
    fn neutral_bootstrap_installs_the_consensus_stream_alone() {
        let neutral = vec![RegisterStreamParams {
            id: 1,
            weight: weight(100, 0, 100, 100),
            distribution: None,
            activation_epoch: 100,
        }];

        let (streams, accrued) = validate_migration_streams(&neutral, 100).unwrap();

        assert_eq!(
            streams.streams,
            vec![Stream {
                id: 1,
                weight: weight(100, 0, 100, 100),
                distribution: None,
            }]
        );
        assert!(accrued.is_empty());
    }

    #[test]
    fn accepts_alternative_bootstrap_weights() {
        let mut params = split_bootstrap();
        params[0].weight = weight(80, -1, 60, 80);
        params[1].weight = weight(20, 1, 10, 20);

        validate_migration_streams(&params, 100).unwrap();
    }

    #[test]
    fn rejects_malformed_bootstraps() {
        type Damage = fn(&mut Vec<RegisterStreamParams>);
        let cases: [(&str, Damage, &str); 11] = [
            (
                "three streams",
                |p| p.push(p[1].clone()),
                "bootstrap requires one or two streams",
            ),
            (
                "activation epoch mismatch",
                |p| p[0].activation_epoch += 1,
                "activation epoch 101 does not match upgrade epoch 100",
            ),
            (
                "weight start mismatch",
                |p| p[0].weight.t_start += 1,
                "weight start 101 does not match upgrade epoch 100",
            ),
            (
                "single stream that is not neutral",
                |p| p.truncate(1),
                "single-stream bootstrap must be implicit stream 1 at constant DENOM",
            ),
            (
                "service stream ID 3",
                |p| p[1].id = 3,
                "split bootstrap stream IDs must be 1 and 2",
            ),
            (
                "two implicit streams",
                |p| p[1].distribution = None,
                "split bootstrap distribution forms are invalid",
            ),
            (
                "starting weights under-sum",
                |p| p[1].weight.v_start -= 1,
                "bootstrap starting weights must sum to denominator",
            ),
            (
                "unequal slopes",
                |p| p[1].weight.slope += 1,
                "bootstrap weight slopes are invalid",
            ),
            (
                "partial share",
                |p| p[1].distribution.as_mut().unwrap().shares[0].share -= 1,
                "explicit bootstrap requires one full-share recipient",
            ),
            (
                "delegated writer",
                |p| {
                    p[1].distribution.as_mut().unwrap().writer =
                        Address::new_delegated(10, &[1]).unwrap()
                },
                "distribution writer f410",
            ),
            (
                "service cap above what the consensus floor leaves",
                |p| p[1].weight.cap = 60 * PERCENT,
                "stream weights exceed DENOM",
            ),
        ];

        for (case, damage, expected_error) in cases {
            let mut params = split_bootstrap();
            damage(&mut params);
            let error = validate_migration_streams(&params, 100)
                .err()
                .unwrap_or_else(|| panic!("{case}: accepted"));
            assert!(
                format!("{error:#}").contains(expected_error),
                "{case}: {error:#}"
            );
        }
    }
}
