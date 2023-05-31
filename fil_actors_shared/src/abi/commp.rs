// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_shared::{
    piece::{
        zero_piece_commitment as zero_piece_commitment_v2, PaddedPieceSize as PaddedPieceSizeV2,
        PieceInfo as PieceInfoV2,
    },
    sector::RegisteredSealProof as RegisteredSealProofV2,
};

/// Computes an unsealed sector CID (`CommD`) from its constituent piece CIDs (`CommPs`) and sizes.
pub fn compute_unsealed_sector_cid_v2(
    proof_type: RegisteredSealProofV2,
    pieces: &[PieceInfoV2],
) -> anyhow::Result<Cid> {
    let ssize = proof_type.sector_size().map_err(anyhow::Error::msg)? as u64;

    let mut all_pieces = Vec::<filecoin_proofs_api::PieceInfo>::with_capacity(pieces.len());

    let pssize = PaddedPieceSizeV2(ssize);
    if pieces.is_empty() {
        all_pieces.push(filecoin_proofs_api::PieceInfo {
            size: pssize.unpadded().into(),
            commitment: zero_piece_commitment_v2(pssize),
        })
    } else {
        // pad remaining space with 0 piece commitments
        let mut sum = PaddedPieceSizeV2(0);
        let pad_to = |pads: Vec<PaddedPieceSizeV2>,
                      all_pieces: &mut Vec<filecoin_proofs_api::PieceInfo>,
                      sum: &mut PaddedPieceSizeV2| {
            for p in pads {
                all_pieces.push(filecoin_proofs_api::PieceInfo {
                    size: p.unpadded().into(),
                    commitment: zero_piece_commitment_v2(p),
                });

                sum.0 += p.0;
            }
        };
        for p in pieces {
            let (ps, _) = get_required_padding_v2(sum, p.size);
            pad_to(ps, &mut all_pieces, &mut sum);
            all_pieces
                .push(filecoin_proofs_api::PieceInfo::try_from(p).map_err(anyhow::Error::msg)?);
            sum.0 += p.size.0;
        }

        let (ps, _) = get_required_padding_v2(sum, pssize);
        pad_to(ps, &mut all_pieces, &mut sum);
    }

    let comm_d = filecoin_proofs_api::seal::compute_comm_d(
        proof_type.try_into().map_err(anyhow::Error::msg)?,
        &all_pieces,
    )
    .map_err(anyhow::Error::msg)?;

    fvm_shared::commcid::data_commitment_v1_to_cid(&comm_d).map_err(anyhow::Error::msg)
}

fn get_required_padding_v2(
    old_length: PaddedPieceSizeV2,
    new_piece_length: PaddedPieceSizeV2,
) -> (Vec<PaddedPieceSizeV2>, PaddedPieceSizeV2) {
    let mut sum = 0;

    let mut to_fill = 0u64.wrapping_sub(old_length.0) % new_piece_length.0;
    let n = to_fill.count_ones();
    let mut pad_pieces = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let next = to_fill.trailing_zeros();
        let p_size = 1 << next;
        to_fill ^= p_size;

        let padded = PaddedPieceSizeV2(p_size);
        pad_pieces.push(padded);
        sum += padded.0;
    }

    (pad_pieces, PaddedPieceSizeV2(sum))
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;
    use anyhow::*;
    use fil_actors_test_utils::go_compat::{ensure_go_mod_prepared, go_compat_tests_dir};
    use fvm_shared::commcid::{FIL_COMMITMENT_UNSEALED, SHA2_256_TRUNC254_PADDED};
    use multihash::{Code::Sha2_256, Multihash, MultihashDigest};
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    #[derive(Debug, Clone)]
    struct RegisteredSealProofWrapper(RegisteredSealProofV2);

    impl Arbitrary for RegisteredSealProofWrapper {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            use RegisteredSealProofV2::*;
            const OPTIONS: [RegisteredSealProofV2; 9] = [
                StackedDRG512MiBV1,
                StackedDRG8MiBV1,
                StackedDRG32GiBV1,
                StackedDRG64GiBV1,
                StackedDRG2KiBV1P1,
                StackedDRG512MiBV1P1,
                StackedDRG8MiBV1P1,
                StackedDRG32GiBV1P1,
                StackedDRG64GiBV1P1,
            ];

            Self(g.choose(OPTIONS.as_slice()).unwrap().clone())
        }
    }

    #[derive(Debug, Clone)]
    struct Pieces(Vec<PieceInfoV2>);

    impl Arbitrary for Pieces {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let cap = *g.choose((1..11).collect::<Vec<_>>().as_slice()).unwrap();
            let mut pieces = Vec::with_capacity(cap);
            for _ in 0..cap {
                let cid = {
                    let hash = Sha2_256.digest(String::arbitrary(g).as_bytes());
                    let mh = Multihash::wrap(SHA2_256_TRUNC254_PADDED, hash.digest()).unwrap();
                    Cid::new_v1(FIL_COMMITMENT_UNSEALED, mh)
                };
                let size = PaddedPieceSizeV2(
                    2_u64.pow(*g.choose((7..9).collect::<Vec<_>>().as_slice()).unwrap()),
                );
                pieces.push(PieceInfoV2 { size, cid });
            }

            Self(pieces)
        }
    }

    #[quickcheck]
    fn test_compute_unsealed_sector_cid_v2(
        proof: RegisteredSealProofWrapper,
        pieces: Pieces,
    ) -> Result<()> {
        ensure_go_mod_prepared();

        let proof = proof.0;
        let pieces = pieces.0;

        let pieces_hex_list: Vec<_> = pieces
            .iter()
            .map(|p| {
                let bytes = fvm_ipld_encoding::to_vec(p).unwrap();
                let bytes_hex = hex::encode(bytes);
                bytes_hex
            })
            .collect();
        let pieces_hex = pieces_hex_list.join(",");
        let proof_num: i64 = proof.into();

        let unsealed_sector_cid: cid::CidGeneric<64> =
            compute_unsealed_sector_cid_v2(proof, &pieces)?;
        println!("{unsealed_sector_cid}");

        let app = Command::new("go")
            .args([
                "run",
                "abi/commp/test_compute_unsealed_sector_cid.go",
                "--proof",
                proof_num.to_string().as_str(),
                "--pieces",
                pieces_hex.as_str(),
            ])
            .current_dir(go_compat_tests_dir()?)
            .output()?;

        if !app.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&app.stderr));
            anyhow::bail!("Fail to run go test");
        }

        let unsealed_sector_cid_from_go = String::from_utf8_lossy(&app.stdout);

        assert_eq!(unsealed_sector_cid.to_string(), unsealed_sector_cid_from_go);

        Ok(())
    }
}
