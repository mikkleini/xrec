use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};

use crate::Result;

pub(crate) fn crc(
    data: &[u8],
    width: u32,
    poly: u64,
    init: u64,
    xor_out: u64,
    refin: bool,
    refout: bool,
) -> u64 {
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    let top = 1u64 << (width - 1);
    let mut value = init & mask;
    for byte in data {
        // Reflect each input byte for CRC variants that process bits least-significant first.
        let byte = if refin {
            reflect(*byte as u64, 8)
        } else {
            *byte as u64
        };
        value ^= byte << (width - 8);
        for _ in 0..8 {
            value = if value & top != 0 {
                (value << 1) ^ poly
            } else {
                value << 1
            };
            value &= mask;
        }
    }
    if refout {
        // Output reflection is independent of input reflection in the CRC specification.
        value = reflect(value, width);
    }
    (value ^ xor_out) & mask
}

fn reflect(mut value: u64, width: u32) -> u64 {
    let mut result = 0;
    for _ in 0..width {
        result = (result << 1) | (value & 1);
        value >>= 1;
    }
    result
}

pub(crate) fn hash(data: &[u8], bits: u64, sha3: bool) -> Result<Vec<u8>> {
    match (sha3, bits) {
        (false, 224) => Ok(Sha224::digest(data).to_vec()),
        (false, 256) => Ok(Sha256::digest(data).to_vec()),
        (false, 384) => Ok(Sha384::digest(data).to_vec()),
        (false, 512) => Ok(Sha512::digest(data).to_vec()),
        (true, 224) => Ok(Sha3_224::digest(data).to_vec()),
        (true, 256) => Ok(Sha3_256::digest(data).to_vec()),
        (true, 384) => Ok(Sha3_384::digest(data).to_vec()),
        (true, 512) => Ok(Sha3_512::digest(data).to_vec()),
        _ => Err("SHA bits must be 224, 256, 384, or 512".into()),
    }
}
