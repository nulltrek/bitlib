use hmac::{Hmac, Mac};
use num_bigint::BigUint;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::num::ParseIntError;

pub fn to_hex_str(data: impl AsRef<[u8]>) -> String {
    data.as_ref()
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<Vec<_>>()
        .join("")
}

pub fn from_hex_str(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn hash256(data: impl AsRef<[u8]>) -> Vec<u8> {
    let digest = Sha256::digest(Sha256::digest(data));
    digest.to_vec()
}

pub fn hash160(data: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(Sha256::digest(data));
    hasher.finalize().to_vec()
}

pub fn hmac(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().into()
}

pub fn base58(data: impl AsRef<[u8]>) -> String {
    const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    let mut prefix = String::new();
    for byte in data.as_ref() {
        if *byte == 0 {
            prefix.push('1');
        } else {
            break;
        }
    }

    let mut result = String::new();
    let mut num = BigUint::from_bytes_be(data.as_ref());
    while num > BigUint::ZERO {
        let rem = &num % 58_u32;
        num = num / 58_u32;
        let index: usize = rem.try_into().unwrap();
        result.push_str(&ALPHABET[index..index + 1]);
    }

    let rev: String = result.chars().rev().collect();
    prefix.push_str(&rev);
    return prefix;
}

pub fn base58_with_checksum(data: impl AsRef<[u8]>) -> String {
    let hash = hash256(&data);
    base58([data.as_ref(), &hash[..4]].concat())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::u256::U256;

    #[test]
    fn test_to_hex_string() {
        assert_eq!(to_hex_str(&[0xec_u8, 0x20, 0x8b, 0xaa, 0x0f]), "ec208baa0f");
        let reversed = [0xec_u8, 0x20, 0x8b, 0xaa, 0x0f]
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        assert_eq!(to_hex_str(&reversed), "0faa8b20ec");
    }

    #[test]
    fn test_from_hex_string() {
        assert_eq!(
            from_hex_str("ec208baa0f").unwrap(),
            &[0xec_u8, 0x20, 0x8b, 0xaa, 0x0f]
        );
        assert_eq!(
            from_hex_str("0faa8b20ec").unwrap(),
            [0xec_u8, 0x20, 0x8b, 0xaa, 0x0f]
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_hash256() {
        let data = [
            0xec_u8, 0x20, 0x8b, 0xaa, 0x0f, 0xc1, 0xc1, 0x9f, 0x70, 0x8a, 0x9c, 0xa9, 0x6f, 0xde,
            0xff, 0x3a, 0xc3, 0xf2, 0x30, 0xbb, 0x4a, 0x7b, 0xa4, 0xae, 0xde, 0x49, 0x42, 0xad,
            0x00, 0x3c, 0x0f, 0x60,
        ];
        assert_eq!(
            hash256(data).as_slice(),
            &[
                0xca_u8, 0xea, 0x07, 0x4a, 0x08, 0x89, 0x74, 0xef, 0xfb, 0x28, 0x20, 0x3f, 0x65,
                0x4b, 0x07, 0x65, 0x79, 0xa9, 0x2e, 0xcf, 0x34, 0x7e, 0xa0, 0x5e, 0x12, 0x32, 0x35,
                0x6e, 0xab, 0x16, 0xed, 0xb1
            ]
        );
    }

    #[test]
    fn test_hash160() {
        let data = [
            0x02, 0xe3, 0xaf, 0x28, 0x96, 0x56, 0x93, 0xb9, 0xce, 0x12, 0x28, 0xf9, 0xd4, 0x68,
            0x14, 0x9b, 0x83, 0x1d, 0x6a, 0x05, 0x40, 0xb2, 0x5e, 0x8a, 0x99, 0x00, 0xf7, 0x13,
            0x72, 0xc1, 0x1f, 0xb2, 0x77,
        ];
        assert_eq!(
            hash160(data).as_slice(),
            &[
                0x1e, 0x51, 0xfc, 0xdc, 0x14, 0xbe, 0x9a, 0x14, 0x8b, 0xb0, 0xaa, 0xec, 0x91, 0x97,
                0xeb, 0x47, 0xc8, 0x37, 0x76, 0xfb
            ],
        );
    }

    #[test]
    fn test_hmac() {
        assert_eq!(
            U256::from_big_endian(&hmac(b"my secret and secure key", b"input message")),
            U256::from_hex("97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9")
        );
    }

    #[test]
    fn base58_serialization() {
        let data = [
            0x7c, 0x07, 0x6f, 0xf3, 0x16, 0x69, 0x2a, 0x3d, 0x7e, 0xb3, 0xc3, 0xbb, 0x0f, 0x8b,
            0x14, 0x88, 0xcf, 0x72, 0xe1, 0xaf, 0xcd, 0x92, 0x9e, 0x29, 0x30, 0x70, 0x32, 0x99,
            0x7a, 0x83, 0x8a, 0x3d,
        ];
        assert_eq!(
            base58(&data),
            "9MA8fRQrT4u8Zj8ZRd6MAiiyaxb2Y1CMpvVkHQu5hVM6",
        );

        let data = [
            0x00, 0xef, 0xf6, 0x9e, 0xf2, 0xb1, 0xbd, 0x93, 0xa6, 0x6e, 0xd5, 0x21, 0x9a, 0xdd,
            0x4f, 0xb5, 0x1e, 0x11, 0xa8, 0x40, 0xf4, 0x04, 0x87, 0x63, 0x25, 0xa1, 0xe8, 0xff,
            0xe0, 0x52, 0x9a, 0x2c,
        ];
        assert_eq!(
            base58(&data),
            "14fE3H2E6XMp4SsxtwinF7w9a34ooUrwWe4WsW1458Pd",
        );

        let data = [
            0xc7, 0x20, 0x7f, 0xee, 0x19, 0x7d, 0x27, 0xc6, 0x18, 0xae, 0xa6, 0x21, 0x40, 0x6f,
            0x6b, 0xf5, 0xef, 0x6f, 0xca, 0x38, 0x68, 0x1d, 0x82, 0xb2, 0xf0, 0x6f, 0xdd, 0xbd,
            0xce, 0x6f, 0xea, 0xb6,
        ];
        assert_eq!(
            base58(&data),
            "EQJsjkd6JaGwxrjEhfeqPenqHwrBmPQZjJGNSCHBkcF7",
        );
    }
}
