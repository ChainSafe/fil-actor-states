use std::ops::Range;

use fil_actors_evm_shared::uints::byteorder::{ByteOrder, LE};
use fil_actors_evm_shared::uints::U256;

use fil_actors_runtime_v11::runtime::Runtime;
use fvm_shared::crypto::hash::SupportedHashes;
use fvm_shared::crypto::signature::{SECP_SIG_LEN, SECP_SIG_MESSAGE_HASH_SIZE};
use num_traits::{One, Zero};
use substrate_bn::{pairing_batch, AffineG1, AffineG2, Fq, Fq2, Fr, Group, Gt, G1, G2};

use crate::{
    interpreter::{precompiles::PrecompileError, System},
    EVM_WORD_SIZE,
};

use super::{PrecompileContext, PrecompileResult};
use crate::reader::ValueReader;

const SECP256K1_N: U256 = U256::from_u128_words(
    0xfffffffffffffffffffffffffffffffe,
    0xbaaedce6af48a03bbfd25e8cd0364141,
);

const SECP256K1_RANGE: Range<U256> = U256::ONE..SECP256K1_N;

#[test]
fn test_secp_range() {
    assert!(SECP256K1_RANGE.contains(&U256::ONE));
    assert!(!SECP256K1_RANGE.contains(&U256::ZERO));
    assert!(!SECP256K1_RANGE.contains(&SECP256K1_N));
}

fn ec_recover_internal<RT: Runtime>(system: &mut System<RT>, input: &[u8]) -> PrecompileResult {
    let mut input_params = ValueReader::new(input);
    let hash: [u8; SECP_SIG_MESSAGE_HASH_SIZE] = input_params.read_fixed();
    let recovery_byte: u8 = input_params.read_value()?;
    let r: U256 = input_params.read_value()?;
    let s: U256 = input_params.read_value()?;

    // Must be either 27 or 28
    let v = recovery_byte
        .checked_sub(27)
        .ok_or(PrecompileError::InvalidInput)?;

    // SECP256K1_HALF_N check in evm was disabled after homestead, both r and s can be in full range of N
    if v > 1 || !SECP256K1_RANGE.contains(&r) || !SECP256K1_RANGE.contains(&s) {
        return Err(PrecompileError::InvalidInput);
    }

    let mut sig: [u8; SECP_SIG_LEN] = [0u8; 65];
    r.to_big_endian(&mut sig[..32]);
    s.to_big_endian(&mut sig[32..64]);
    sig[64] = v;

    let pubkey = system
        .rt
        .recover_secp_public_key(&hash, &sig)
        .map_err(|_| PrecompileError::InvalidInput)?;

    let mut address = system.rt.hash(SupportedHashes::Keccak256, &pubkey[1..]);
    address[..12].copy_from_slice(&[0u8; 12]);

    Ok(address)
}

/// recover a secp256k1 pubkey from a hash, recovery byte, and a signature
pub(super) fn ec_recover<RT: Runtime>(
    system: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    // This precompile is weird and never fails. So we just turn errors into empty results.
    Ok(ec_recover_internal(system, input).unwrap_or_default())
}

/// hash with sha2-256
pub(super) fn sha256<RT: Runtime>(
    system: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    Ok(system.rt.hash(SupportedHashes::Sha2_256, input))
}

/// hash with ripemd160
pub(super) fn ripemd160<RT: Runtime>(
    system: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    let mut out = vec![0; 12];
    let hash = system.rt.hash(SupportedHashes::Ripemd160, input);
    out.extend_from_slice(&hash);
    debug_assert_eq!(out.len(), EVM_WORD_SIZE);
    Ok(out)
}

/// data copy
pub(super) fn identity<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    Ok(Vec::from(input))
}

// https://eips.ethereum.org/EIPS/eip-198
/// modulus exponent a number
pub(super) fn modexp<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    let mut reader = ValueReader::new(input);

    // This will error out if the user passes values greater than u32, but that's fine. The user
    // would run out of gas anyways.
    let base_len = reader.read_value::<u32>()? as usize;
    let exponent_len = reader.read_value::<u32>()? as usize;
    let mod_len = reader.read_value::<u32>()? as usize;

    if base_len == 0 && mod_len == 0 {
        return Ok(Vec::new());
    }

    let base = reader.read_biguint(base_len);
    let exponent = reader.read_biguint(exponent_len);
    let modulus = reader.read_biguint(mod_len);

    if modulus.is_zero() || modulus.is_one() {
        // mod 0 is undefined: 0, base mod 1 is always 0
        return Ok(vec![0; mod_len]);
    }

    let mut output = base.modpow(&exponent, &modulus).to_bytes_be();

    if output.len() < mod_len {
        let mut ret = Vec::with_capacity(mod_len);
        ret.resize(mod_len - output.len(), 0); // left padding
        ret.extend_from_slice(&output);
        output = ret;
    }

    Ok(output)
}

pub(super) fn curve_to_vec(curve: G1) -> Vec<u8> {
    let mut output = vec![0; 64];
    if let Some(product) = AffineG1::from_jacobian(curve) {
        product.x().to_big_endian(&mut output[0..32]).unwrap();
        product.y().to_big_endian(&mut output[32..64]).unwrap();
    }
    output
}

/// add 2 points together on an elliptic curve
pub(super) fn ec_add<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    let mut input_params = ValueReader::new(input);
    let point1: G1 = input_params.read_value()?;
    let point2: G1 = input_params.read_value()?;

    Ok(curve_to_vec(point1 + point2))
}

/// multiply a point on an elliptic curve by a scalar value
pub(super) fn ec_mul<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    let mut input_params = ValueReader::new(input);
    let point: G1 = input_params.read_value()?;
    let scalar: Fr = input_params.read_value()?;

    Ok(curve_to_vec(point * scalar))
}

/// pairs multple groups of twisted bn curves
pub(super) fn ec_pairing<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    fn read_group(input: &[u8]) -> Result<(G1, G2), PrecompileError> {
        let mut reader = ValueReader::new(input);

        let x: Fq = reader.read_value()?;
        let y: Fq = reader.read_value()?;

        let twisted_x = {
            let b: Fq = reader.read_value()?;
            let a: Fq = reader.read_value()?;
            Fq2::new(a, b)
        };
        let twisted_y = {
            let b: Fq = reader.read_value()?;
            let a: Fq = reader.read_value()?;
            Fq2::new(a, b)
        };

        let twisted = {
            if twisted_x.is_zero() && twisted_y.is_zero() {
                G2::zero()
            } else {
                AffineG2::new(twisted_x, twisted_y)?.into()
            }
        };

        let a = {
            if x.is_zero() && y.is_zero() {
                substrate_bn::G1::zero()
            } else {
                AffineG1::new(x, y)?.into()
            }
        };

        Ok((a, twisted))
    }

    const GROUP_BYTE_LEN: usize = 192;

    // This precompile is strange in that it doesn't automatically "pad" the input.
    // So we have to check the sizes.
    if input.len() % GROUP_BYTE_LEN != 0 {
        return Err(PrecompileError::IncorrectInputSize);
    }

    let mut groups = Vec::new();
    for i in 0..input.len() / GROUP_BYTE_LEN {
        let offset = i * GROUP_BYTE_LEN;
        groups.push(read_group(&input[offset..offset + GROUP_BYTE_LEN])?);
    }

    let accumulated = pairing_batch(&groups);

    let paring_success = if accumulated == Gt::one() {
        U256::one()
    } else {
        U256::zero()
    };
    let mut ret = [0u8; EVM_WORD_SIZE];
    paring_success.to_big_endian(&mut ret);
    Ok(ret.to_vec())
}

/// https://eips.ethereum.org/EIPS/eip-152
pub(super) fn blake2f<RT: Runtime>(
    _: &mut System<RT>,
    input: &[u8],
    _: PrecompileContext,
) -> PrecompileResult {
    if input.len() != 213 {
        return Err(PrecompileError::IncorrectInputSize);
    }
    let mut rounds = [0u8; 4];

    let mut start = 0;

    // 4 bytes
    rounds.copy_from_slice(&input[..4]);
    start += 4;
    // 64 bytes
    let h = &input[start..start + 64];
    start += 64;
    // 128 bytes
    let m = &input[start..start + 128];
    start += 128;
    // 16 bytes
    let t = &input[start..start + 16];
    start += 16;

    debug_assert_eq!(start, 212, "expected start to be at the last byte");
    let f = match input[start] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PrecompileError::IncorrectInputSize),
    }?;

    let rounds = u32::from_be_bytes(rounds);
    let mut h = {
        let mut ret = [0u64; 8];
        LE::read_u64_into(h, &mut ret);
        ret
    };
    let m = {
        let mut ret = [0u64; 16];
        LE::read_u64_into(m, &mut ret);
        ret
    };
    let t = {
        let mut ret = [0u64; 2];
        LE::read_u64_into(t, &mut ret);
        ret
    };

    super::blake2f_impl::compress(&mut h, m, t, f, rounds as usize);

    let mut output = vec![0; 64];
    LE::write_u64_into(&h, &mut output);
    Ok(output)
}
