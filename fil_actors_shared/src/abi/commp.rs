// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_shared::{
    piece::PieceInfo as PieceInfoV2, sector::RegisteredSealProof as RegisteredSealProofV2,
};
use fvm_shared4::commcid::data_commitment_v1_to_cid;

/// Computes an unsealed sector CID (`CommD`) from its constituent piece CIDs (`CommPs`) and sizes.
///
/// Ported from <https://github.com/filecoin-project/go-commp-utils/blob/62059082a8378046f27a01d62777ae539a2d1feb/nonffi/commd.go#L20>
pub fn compute_unsealed_sector_cid_v2(
    proof_type: RegisteredSealProofV2,
    pieces: &[PieceInfoV2],
) -> anyhow::Result<Cid> {
    if pieces.is_empty() {
        anyhow::bail!("no pieces provided");
    }

    let mut all_pieces = Vec::with_capacity(pieces.len());
    for p in pieces {
        all_pieces.push(filecoin_proofs_api::PieceInfo::try_from(p).map_err(anyhow::Error::msg)?);
    }

    let comm_d = filecoin_proofs_api::seal::compute_comm_d(
        proof_type.try_into().map_err(anyhow::Error::msg)?,
        &all_pieces,
    )
    .map_err(anyhow::Error::msg)?;

    data_commitment_v1_to_cid(&comm_d).map_err(anyhow::Error::msg)
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::str::FromStr;

    use super::*;
    use anyhow::*;
    use cid::multihash::Multihash;
    use fil_actors_test_utils::go_compat::{ensure_go_mod_prepared, go_compat_tests_dir};
    use fvm_shared::commcid::{FIL_COMMITMENT_UNSEALED, SHA2_256_TRUNC254_PADDED};
    use fvm_shared::piece::{
        zero_piece_commitment as zero_piece_commitment_v2, PaddedPieceSize as PaddedPieceSizeV2,
    };
    use multihash_codetable::{Code, MultihashDigest};
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

            Self(*g.choose(OPTIONS.as_slice()).unwrap())
        }
    }

    #[derive(Debug, Clone)]
    struct Pieces(Vec<PieceInfoV2>);

    impl Arbitrary for Pieces {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let cap = *g.choose((1..4).collect::<Vec<_>>().as_slice()).unwrap();
            let mut pieces = Vec::with_capacity(cap);
            for _ in 0..cap {
                let cid = {
                    let hash = Code::Sha2_256.digest(String::arbitrary(g).as_bytes());
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
    fn compute_unsealed_sector_cid_v2_go_parity_test(
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
                hex::encode(bytes)
            })
            .collect();
        let pieces_hex = pieces_hex_list.join(",");
        let proof_num: i64 = proof.into();

        let unsealed_sector_cid = compute_unsealed_sector_cid_v2(proof, &pieces)?;
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

    /// Ported from <https://github.com/filecoin-project/go-commp-utils/blob/62059082a8378046f27a01d62777ae539a2d1feb/nonffi/commd_test.go#L11>
    #[test]
    fn compute_unsealed_sector_cid_v2_test() -> Result<()> {
        /*
            Testing live sector data with the help of a fellow SP

            ~$ lotus-miner sectors status --log 139074
                SectorID:	139074
                Status:		Proving
                CIDcommD:	baga6ea4seaqiw3gbmstmexb7sqwkc5r23o3i7zcyx5kr76pfobpykes3af62kca
                ...
                Precommit:	bafy2bzacec3dyxgqfbjekvnbin6uhcel7adis576346bi3tahp64bhijeiymy
                Commit:		bafy2bzacecafq4ksrjzlhjagxkrrpycmfpjo5ch62s3tbq7gr5rop75fuqhwk
                Deals:		[3755444 0 0 3755443 3755442 3755608 3755679 3755680 0 3755754 3755803 3755883 0 3755882 0 0 0]
        */

        let pieces = vec![
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqknzm22isnhsxt2s4dnw45kfywmhenngqq3nc7jvecakoca6ksyhy",
                )?,
                size: PaddedPieceSizeV2(256 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqnq6o5wuewdpviyoafno4rdpqnokz6ghvg2iyeyfbqxgcwdlj2egi",
                )?,
                size: PaddedPieceSizeV2(1024 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqpixk4ifbkzato3huzycj6ty6gllqwanhdpsvxikawyl5bg2h44mq",
                )?,
                size: PaddedPieceSizeV2(512 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqaxwe5dy6nt3ko5tngtmzvpqxqikw5mdwfjqgaxfwtzenc6bgzajq",
                )?,
                size: PaddedPieceSizeV2(512 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqpy33nbesa4d6ot2ygeuy43y4t7amc4izt52mlotqenwcmn2kyaai",
                )?,
                size: PaddedPieceSizeV2(1024 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqphvv4x2s2v7ykgc3ugs2kkltbdeg7icxstklkrgqvv72m2v3i2aa",
                )?,
                size: PaddedPieceSizeV2(256 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqf5u55znk6jwhdsrhe37emzhmehiyvjxpsww274f6fiy3h4yctady",
                )?,
                size: PaddedPieceSizeV2(512 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqa3qbabsbmvk5er6rhsjzt74beplzgulthamm22jue4zgqcuszofi",
                )?,
                size: PaddedPieceSizeV2(1024 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqiekvf623muj6jpxg6vsqaikyw3r4ob5u7363z7zcaixqvfqsc2ji",
                )?,
                size: PaddedPieceSizeV2(256 << 20),
            },
            PieceInfoV2 {
                cid: Cid::from_str(
                    "baga6ea4seaqhsewv65z2d4m5o4vo65vl5o6z4bcegdvgnusvlt7rao44gro36pi",
                )?,
                size: PaddedPieceSizeV2(512 << 20),
            },
            // GenerateUnsealedCID does not "fill a sector", do it here to match the SP provided sector commD
            PieceInfoV2 {
                cid: data_commitment_v1_to_cid(&zero_piece_commitment_v2(PaddedPieceSizeV2(
                    8 << 30,
                )))
                .map_err(Error::msg)?,
                size: PaddedPieceSizeV2(8 << 30),
            },
            PieceInfoV2 {
                cid: data_commitment_v1_to_cid(&zero_piece_commitment_v2(PaddedPieceSizeV2(
                    16 << 30,
                )))
                .map_err(Error::msg)?,
                size: PaddedPieceSizeV2(16 << 30),
            },
        ];

        let commd =
            compute_unsealed_sector_cid_v2(RegisteredSealProofV2::StackedDRG32GiBV1P1, &pieces)?;

        assert_eq!(
            commd.to_string(),
            "baga6ea4seaqiw3gbmstmexb7sqwkc5r23o3i7zcyx5kr76pfobpykes3af62kca"
        );

        Ok(())
    }
}
