use crate::curves::Curve;
use crate::curves::Point;
use crate::ecdsa::{PrivateKey, Signature};
use crate::fields::FiniteFieldU256;
use crate::hashing::{base58, base58_with_checksum, hash160};
use crate::u256::U256;

pub enum Comp {
    Compressed,
    Uncompressed,
}

pub enum Net {
    Testnet,
    Mainnet,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    ParsingError(&'static str),
    NotEnoughData,
}

pub type Result<T> = std::result::Result<T, SerializationError>;

pub fn slice_to_array<T: Default + Copy, const LEN: usize>(slice: &[T]) -> [T; LEN]
where
    T: Clone,
{
    let mut array: [T; LEN] = [T::default(); LEN];
    array.clone_from_slice(&slice);
    array
}

impl U256 {
    pub fn to_base58(&self) -> String {
        base58(self.to_big_endian())
    }
}

impl Point<U256> {
    // Serialize into Uncompressed SEC format
    pub fn to_sec(&self) -> [u8; 65] {
        match self {
            Point::Inf => panic!("Cannot serialize point at infinity."),
            Point::Coords { x, y } => {
                let mut output: [u8; 65] = [0; 65];
                output[0] = 0x04;
                output[1..33].copy_from_slice(x.to_big_endian().as_slice());
                output[33..65].copy_from_slice(y.to_big_endian().as_slice());
                return output;
            }
        }
    }

    // Serialize into Compressed SEC format
    pub fn to_csec(&self) -> [u8; 33] {
        match self {
            Point::Inf => panic!("Cannot serialize point at infinity."),
            Point::Coords { x, y } => {
                let mut output: [u8; 33] = [0; 33];
                output[0] = if y.is_even() { 0x02 } else { 0x03 };
                output[1..33].copy_from_slice(x.to_big_endian().as_slice());
                return output;
            }
        }
    }

    pub fn parse(curve: &Curve<U256, FiniteFieldU256>, data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        match data[0] {
            0x04 => Self::parse_sec(data),
            0x02 | 0x03 => Self::parse_csec(curve, data),
            _ => None,
        }
    }

    fn parse_sec(data: &[u8]) -> Option<Self> {
        if data.len() != 65 {
            return None;
        }
        Some(Self::coords(
            U256::from_big_endian(&data[1..33]),
            U256::from_big_endian(&data[33..65]),
        ))
    }

    fn parse_csec(curve: &Curve<U256, FiniteFieldU256>, data: &[u8]) -> Option<Self> {
        if data.len() != 33 {
            return None;
        }
        let x = U256::from_big_endian(&data[1..33]);
        let y_is_even = match data[0] {
            0x02 => true,
            0x03 => false,
            _ => panic!("Unrecognized compressed SEC marker"),
        };
        Some(Self::coords(x, curve.compute_y(&x, y_is_even)))
    }

    pub fn to_address(&self, compression: Comp, network: Net) -> String {
        let data = match compression {
            Comp::Compressed => self.to_csec().to_vec(),
            Comp::Uncompressed => self.to_sec().to_vec(),
        };
        let hash = hash160(data);
        let prefix = match network {
            Net::Testnet => 0x6f,
            Net::Mainnet => 0x00,
        };
        let addr = [vec![prefix], hash].concat();
        base58_with_checksum(addr)
    }
}

impl Signature {
    fn to_der(&self) -> Vec<u8> {
        let comp_r = Self::compress(&self.r);
        let comp_s = Self::compress(&self.s);

        let mut enc_r = vec![0x02, comp_r.len() as u8];
        enc_r.extend_from_slice(comp_r.as_slice());
        let mut enc_s = vec![0x02, comp_s.len() as u8];
        enc_s.extend_from_slice(comp_s.as_slice());

        let mut enc_sig = vec![0x30, (enc_r.len() + enc_s.len()) as u8];
        enc_sig.extend_from_slice(enc_r.as_slice());
        enc_sig.extend_from_slice(enc_s.as_slice());

        enc_sig
    }

    fn compress(num: &U256) -> Vec<u8> {
        let mut cur_i = 0;
        let bytes = num.to_big_endian().clone();
        for (i, byte) in bytes.iter().enumerate() {
            if *byte == 0 && i < bytes.len() - 1 {
                cur_i += 1;
            }
        }
        if bytes[cur_i] >= 0x80 {
            [&[0_u8], &bytes[cur_i..bytes.len()]].concat()
        } else {
            bytes[cur_i..bytes.len()].to_vec()
        }
    }

    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(SerializationError::ParsingError("Data too short"));
        }

        if data[0] != 0x30 {
            return Err(SerializationError::ParsingError("Bad initial marker"));
        }

        let tot_len = data[1] as usize;
        if data.len() != 2 + tot_len {
            return Err(SerializationError::ParsingError("Bad signature length"));
        }

        if data[2] != 0x02 {
            return Err(SerializationError::ParsingError("Bad marker for r"));
        }
        let len = data[3] as usize;
        let (r_start, r_len, pad_len) = match len {
            33 => (5, 32, 0),
            _ => (4, len, 32 - len),
        };
        let r = vec![
            &vec![0; pad_len].as_slice(),
            &data[r_start..r_start + r_len],
        ]
        .concat();

        if data[r_start + r_len] != 0x02 {
            return Err(SerializationError::ParsingError("Bad marker for s"));
        }
        let len = data[r_start + r_len + 1] as usize;
        let (s_start, s_len, pad_len) = match len {
            33 => (r_start + r_len + 3, 32, 0),
            _ => (r_start + r_len + 2, len, 32 - len),
        };
        let s = vec![
            &vec![0; pad_len].as_slice(),
            &data[s_start..s_start + s_len],
        ]
        .concat();

        Ok(Signature {
            r: U256::from_big_endian(r.as_slice()),
            s: U256::from_big_endian(s.as_slice()),
        })
    }
}

impl PrivateKey {
    fn to_wif(&self, compression: Comp, network: Net) -> String {
        let prefix = match network {
            Net::Testnet => vec![0xef_u8],
            Net::Mainnet => vec![0x80],
        };
        let suffix = match compression {
            Comp::Compressed => vec![0x01_u8],
            Comp::Uncompressed => vec![],
        };
        base58_with_checksum(
            [
                prefix.as_slice(),
                &(*self).to_big_endian(),
                suffix.as_slice(),
            ]
            .concat(),
        )
    }
}

pub mod varint {
    use super::*;

    pub fn parse(data: &[u8]) -> Result<(u64, usize)> {
        if data.len() == 0 {
            return Err(SerializationError::NotEnoughData);
        }
        let mut result = [0_u8; 8];
        let bytes_read = match data[0] {
            0xfd => {
                if data.len() < 3 {
                    return Err(SerializationError::NotEnoughData);
                }
                result[0..2].copy_from_slice(&data[1..3]);
                3
            }
            0xfe => {
                if data.len() < 5 {
                    return Err(SerializationError::NotEnoughData);
                }
                result[0..4].copy_from_slice(&data[1..5]);
                5
            }
            0xff => {
                if data.len() < 9 {
                    return Err(SerializationError::NotEnoughData);
                }
                result[0..8].copy_from_slice(&data[1..9]);
                9
            }
            _ => {
                result[0] = data[0];
                1
            }
        };

        Ok((u64::from_le_bytes(result), bytes_read))
    }

    pub fn encode(num: u64) -> Vec<u8> {
        if num < 0xfd {
            vec![num.to_le_bytes()[0]]
        } else if num < 0x10000 {
            [vec![0xfd], num.to_le_bytes()[0..2].to_vec()].concat()
        } else if num < 0x100000000 {
            [vec![0xfe], num.to_le_bytes()[0..4].to_vec()].concat()
        } else {
            [vec![0xff], num.to_le_bytes().to_vec()].concat()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecdsa::{PrivateKey, Secp256k1};

    #[test]
    fn slice_array_conversion() {
        assert_eq!(slice_to_array(&[1, 2, 3]), [1, 2, 3]);
    }

    #[test]
    fn point_to_sec() {
        let point = Point::coords(
            U256::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            U256::from_hex("0101010101010101010101010101010101010101010101010101010101010101"),
        );
        assert_eq!(
            point.to_sec(),
            [
                0x04, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            ]
        );

        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("5000")));
        assert_eq!(
            pubkey.to_sec(),
            [
                0x04, 0xff, 0xe5, 0x58, 0xe3, 0x88, 0x85, 0x2f, 0x01, 0x20, 0xe4, 0x6a, 0xf2, 0xd1,
                0xb3, 0x70, 0xf8, 0x58, 0x54, 0xa8, 0xeb, 0x08, 0x41, 0x81, 0x1e, 0xce, 0x0e, 0x3e,
                0x03, 0xd2, 0x82, 0xd5, 0x7c, 0x31, 0x5d, 0xc7, 0x28, 0x90, 0xa4, 0xf1, 0x0a, 0x14,
                0x81, 0xc0, 0x31, 0xb0, 0x3b, 0x35, 0x1b, 0x0d, 0xc7, 0x99, 0x01, 0xca, 0x18, 0xa0,
                0x0c, 0xf0, 0x09, 0xdb, 0xdb, 0x15, 0x7a, 0x1d, 0x10
            ]
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568")));
        assert_eq!(
            pubkey.to_sec(),
            [
                0x04, 0x02, 0x7f, 0x3d, 0xa1, 0x91, 0x84, 0x55, 0xe0, 0x3c, 0x46, 0xf6, 0x59, 0x26,
                0x6a, 0x1b, 0xb5, 0x20, 0x4e, 0x95, 0x9d, 0xb7, 0x36, 0x4d, 0x2f, 0x47, 0x3b, 0xdf,
                0x8f, 0x0a, 0x13, 0xcc, 0x9d, 0xff, 0x87, 0x64, 0x7f, 0xd0, 0x23, 0xc1, 0x3b, 0x4a,
                0x49, 0x94, 0xf1, 0x76, 0x91, 0x89, 0x58, 0x06, 0xe1, 0xb4, 0x0b, 0x57, 0xf4, 0xfd,
                0x22, 0x58, 0x1a, 0x4f, 0x46, 0x85, 0x1f, 0x3b, 0x06
            ]
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_hex("deadbeef12345")));
        assert_eq!(
            pubkey.to_sec(),
            [
                0x04, 0xd9, 0x0c, 0xd6, 0x25, 0xee, 0x87, 0xdd, 0x38, 0x65, 0x6d, 0xd9, 0x5c, 0xf7,
                0x9f, 0x65, 0xf6, 0x0f, 0x72, 0x73, 0xb6, 0x7d, 0x30, 0x96, 0xe6, 0x8b, 0xd8, 0x1e,
                0x4f, 0x53, 0x42, 0x69, 0x1f, 0x84, 0x2e, 0xfa, 0x76, 0x2f, 0xd5, 0x99, 0x61, 0xd0,
                0xe9, 0x98, 0x03, 0xc6, 0x1e, 0xdb, 0xa8, 0xb3, 0xe3, 0xf7, 0xdc, 0x3a, 0x34, 0x18,
                0x36, 0xf9, 0x77, 0x33, 0xae, 0xbf, 0x98, 0x71, 0x21
            ]
        );
    }

    #[test]
    fn point_to_compressed_sec() {
        let point = Point::coords(
            U256::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            U256::from_hex("0101010101010101010101010101010101010101010101010101010101010101"),
        );
        assert_eq!(
            point.to_csec(),
            [
                0x03, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );

        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("5000")));
        assert_eq!(
            pubkey.to_csec(),
            [
                0x02, 0xff, 0xe5, 0x58, 0xe3, 0x88, 0x85, 0x2f, 0x01, 0x20, 0xe4, 0x6a, 0xf2, 0xd1,
                0xb3, 0x70, 0xf8, 0x58, 0x54, 0xa8, 0xeb, 0x08, 0x41, 0x81, 0x1e, 0xce, 0x0e, 0x3e,
                0x03, 0xd2, 0x82, 0xd5, 0x7c
            ]
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568")));
        assert_eq!(
            pubkey.to_csec(),
            [
                0x02, 0x02, 0x7f, 0x3d, 0xa1, 0x91, 0x84, 0x55, 0xe0, 0x3c, 0x46, 0xf6, 0x59, 0x26,
                0x6a, 0x1b, 0xb5, 0x20, 0x4e, 0x95, 0x9d, 0xb7, 0x36, 0x4d, 0x2f, 0x47, 0x3b, 0xdf,
                0x8f, 0x0a, 0x13, 0xcc, 0x9d
            ]
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_hex("deadbeef12345")));
        assert_eq!(
            pubkey.to_csec(),
            [
                0x03, 0xd9, 0x0c, 0xd6, 0x25, 0xee, 0x87, 0xdd, 0x38, 0x65, 0x6d, 0xd9, 0x5c, 0xf7,
                0x9f, 0x65, 0xf6, 0x0f, 0x72, 0x73, 0xb6, 0x7d, 0x30, 0x96, 0xe6, 0x8b, 0xd8, 0x1e,
                0x4f, 0x53, 0x42, 0x69, 0x1f
            ]
        );
    }

    #[test]
    fn parse_sec() {
        let point = Point::<U256>::parse_sec(&[
            0x04, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        ]);

        assert!(point.is_some());
        assert_eq!(
            point.unwrap(),
            Point::coords(
                U256::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
                U256::from_hex("0101010101010101010101010101010101010101010101010101010101010101"),
            )
        );

        let point = Point::<U256>::parse_sec(&[
            0x04, 0xff, 0xe5, 0x58, 0xe3, 0x88, 0x85, 0x2f, 0x01, 0x20, 0xe4, 0x6a, 0xf2, 0xd1,
            0xb3, 0x70, 0xf8, 0x58, 0x54, 0xa8, 0xeb, 0x08, 0x41, 0x81, 0x1e, 0xce, 0x0e, 0x3e,
            0x03, 0xd2, 0x82, 0xd5, 0x7c, 0x31, 0x5d, 0xc7, 0x28, 0x90, 0xa4, 0xf1, 0x0a, 0x14,
            0x81, 0xc0, 0x31, 0xb0, 0x3b, 0x35, 0x1b, 0x0d, 0xc7, 0x99, 0x01, 0xca, 0x18, 0xa0,
            0x0c, 0xf0, 0x09, 0xdb, 0xdb, 0x15, 0x7a, 0x1d, 0x10,
        ]);

        assert!(point.is_some());
        let secp = Secp256k1::new();
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_dec("5000"))),
        );

        let point = Point::<U256>::parse_sec(&[
            0x04, 0x02, 0x7f, 0x3d, 0xa1, 0x91, 0x84, 0x55, 0xe0, 0x3c, 0x46, 0xf6, 0x59, 0x26,
            0x6a, 0x1b, 0xb5, 0x20, 0x4e, 0x95, 0x9d, 0xb7, 0x36, 0x4d, 0x2f, 0x47, 0x3b, 0xdf,
            0x8f, 0x0a, 0x13, 0xcc, 0x9d, 0xff, 0x87, 0x64, 0x7f, 0xd0, 0x23, 0xc1, 0x3b, 0x4a,
            0x49, 0x94, 0xf1, 0x76, 0x91, 0x89, 0x58, 0x06, 0xe1, 0xb4, 0x0b, 0x57, 0xf4, 0xfd,
            0x22, 0x58, 0x1a, 0x4f, 0x46, 0x85, 0x1f, 0x3b, 0x06,
        ]);

        assert!(point.is_some());
        let secp = Secp256k1::new();
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568"))),
        );

        let point = Point::<U256>::parse_sec(&[
            0x04, 0xd9, 0x0c, 0xd6, 0x25, 0xee, 0x87, 0xdd, 0x38, 0x65, 0x6d, 0xd9, 0x5c, 0xf7,
            0x9f, 0x65, 0xf6, 0x0f, 0x72, 0x73, 0xb6, 0x7d, 0x30, 0x96, 0xe6, 0x8b, 0xd8, 0x1e,
            0x4f, 0x53, 0x42, 0x69, 0x1f, 0x84, 0x2e, 0xfa, 0x76, 0x2f, 0xd5, 0x99, 0x61, 0xd0,
            0xe9, 0x98, 0x03, 0xc6, 0x1e, 0xdb, 0xa8, 0xb3, 0xe3, 0xf7, 0xdc, 0x3a, 0x34, 0x18,
            0x36, 0xf9, 0x77, 0x33, 0xae, 0xbf, 0x98, 0x71, 0x21,
        ]);

        assert!(point.is_some());
        let secp = Secp256k1::new();
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_hex("deadbeef12345"))),
        );
    }

    #[test]
    fn parse_compressed_sec() {
        let secp = Secp256k1::new();
        let point = Point::<U256>::parse_csec(
            &secp.curve,
            &[
                0x02, 0xff, 0xe5, 0x58, 0xe3, 0x88, 0x85, 0x2f, 0x01, 0x20, 0xe4, 0x6a, 0xf2, 0xd1,
                0xb3, 0x70, 0xf8, 0x58, 0x54, 0xa8, 0xeb, 0x08, 0x41, 0x81, 0x1e, 0xce, 0x0e, 0x3e,
                0x03, 0xd2, 0x82, 0xd5, 0x7c,
            ],
        );

        assert!(point.is_some());
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_dec("5000"))),
        );

        let point = Point::<U256>::parse_csec(
            &secp.curve,
            &[
                0x02, 0x02, 0x7f, 0x3d, 0xa1, 0x91, 0x84, 0x55, 0xe0, 0x3c, 0x46, 0xf6, 0x59, 0x26,
                0x6a, 0x1b, 0xb5, 0x20, 0x4e, 0x95, 0x9d, 0xb7, 0x36, 0x4d, 0x2f, 0x47, 0x3b, 0xdf,
                0x8f, 0x0a, 0x13, 0xcc, 0x9d,
            ],
        );

        assert!(point.is_some());
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568"))),
        );

        let point = Point::<U256>::parse_csec(
            &secp.curve,
            &[
                0x03, 0xd9, 0x0c, 0xd6, 0x25, 0xee, 0x87, 0xdd, 0x38, 0x65, 0x6d, 0xd9, 0x5c, 0xf7,
                0x9f, 0x65, 0xf6, 0x0f, 0x72, 0x73, 0xb6, 0x7d, 0x30, 0x96, 0xe6, 0x8b, 0xd8, 0x1e,
                0x4f, 0x53, 0x42, 0x69, 0x1f,
            ],
        );

        assert!(point.is_some());
        assert_eq!(
            point.unwrap(),
            *secp.get_pubkey(&PrivateKey::new(U256::from_hex("deadbeef12345"))),
        );
    }

    #[test]
    fn serialize_and_parse() {
        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("5000")));
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_csec()),
            Some(*pubkey),
        );
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_sec()),
            Some(*pubkey),
        );

        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568")));
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_csec()),
            Some(*pubkey),
        );
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_sec()),
            Some(*pubkey),
        );

        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_hex("deadbeef12345")));
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_csec()),
            Some(*pubkey),
        );
        assert_eq!(
            Point::<U256>::parse(&secp.curve, &pubkey.to_sec()),
            Some(*pubkey),
        );
    }

    #[test]
    fn signature_serialization() {
        let sig = Signature {
            r: U256::from_hex("37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6"),
            s: U256::from_hex("8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec"),
        };
        let data = [
            0x30, 0x45, 0x02, 0x20, 0x37, 0x20, 0x6a, 0x06, 0x10, 0x99, 0x5c, 0x58, 0x07, 0x49,
            0x99, 0xcb, 0x97, 0x67, 0xb8, 0x7a, 0xf4, 0xc4, 0x97, 0x8d, 0xb6, 0x8c, 0x06, 0xe8,
            0xe6, 0xe8, 0x1d, 0x28, 0x20, 0x47, 0xa7, 0xc6, 0x02, 0x21, 0x00, 0x8c, 0xa6, 0x37,
            0x59, 0xc1, 0x15, 0x7e, 0xbe, 0xae, 0xc0, 0xd0, 0x3c, 0xec, 0xca, 0x11, 0x9f, 0xc9,
            0xa7, 0x5b, 0xf8, 0xe6, 0xd0, 0xfa, 0x65, 0xc8, 0x41, 0xc8, 0xe2, 0x73, 0x8c, 0xda,
            0xec,
        ];
        assert_eq!(sig.to_der(), data);

        assert_eq!(Signature::parse(&data), Ok(sig));

        let sig = Signature {
            r: U256::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            s: U256::from_hex("0101010101010101010101010101010101010101010101010101010101010101"),
        };

        assert_eq!(Signature::parse(&sig.to_der()), Ok(sig));
    }

    #[test]
    fn base58_point_serialization() {
        assert_eq!(
            U256::from_hex("7c076ff316692a3d7eb3c3bb0f8b1488cf72e1afcd929e29307032997a838a3d")
                .to_base58(),
            "9MA8fRQrT4u8Zj8ZRd6MAiiyaxb2Y1CMpvVkHQu5hVM6",
        );
        assert_eq!(
            U256::from_hex("00eff69ef2b1bd93a66ed5219add4fb51e11a840f404876325a1e8ffe0529a2c")
                .to_base58(),
            "14fE3H2E6XMp4SsxtwinF7w9a34ooUrwWe4WsW1458Pd",
        );
        assert_eq!(
            U256::from_hex("c7207fee197d27c618aea621406f6bf5ef6fca38681d82b2f06fddbdce6feab6")
                .to_base58(),
            "EQJsjkd6JaGwxrjEhfeqPenqHwrBmPQZjJGNSCHBkcF7",
        );
    }

    #[test]
    fn point_to_address() {
        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("5002")));
        assert_eq!(
            pubkey.to_address(Comp::Uncompressed, Net::Testnet),
            "mmTPbXQFxboEtNRkwfh6K51jvdtHLxGeMA",
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_dec("33632321603200000")));
        assert_eq!(
            pubkey.to_address(Comp::Compressed, Net::Testnet),
            "mopVkxp8UhXqRYbCYJsbeE1h1fiF64jcoH",
        );

        let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_hex("12345deadbeef")));
        assert_eq!(
            pubkey.to_address(Comp::Compressed, Net::Mainnet),
            "1F1Pn2y6pDb68E5nYJJeba4TLg2U7B6KF1",
        );
    }

    #[test]
    fn privkey_to_wif() {
        let privkey = PrivateKey::new(U256::from_dec("5003"));
        assert_eq!(
            privkey.to_wif(Comp::Compressed, Net::Testnet),
            "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN8rFTv2sfUK",
        );

        let privkey = PrivateKey::new(U256::from_dec("33715652388894101"));
        assert_eq!(
            privkey.to_wif(Comp::Uncompressed, Net::Testnet),
            "91avARGdfge8E4tZfYLoxeJ5sGBdNJQH4kvjpWAxgzczjbCwxic",
        );

        let privkey = PrivateKey::new(U256::from_hex("0x54321deadbeef"));
        assert_eq!(
            privkey.to_wif(Comp::Compressed, Net::Mainnet),
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgiuQJv1h8Ytr2S53a",
        );
    }

    #[test]
    fn parse_varint() {
        // Parse correct data
        for (data, result, bytes_read) in [
            (vec![0x50], 0x50, 1),
            (vec![0xfd, 0x1a, 0xe3], 0xe31a, 3),
            (vec![0xfe, 0x1a, 0xe3, 0x46, 0xb4], 0xb446e31a, 5),
            (
                vec![0xff, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98],
                0x98cc4a67b446e31a,
                9,
            ),
        ] {
            assert_eq!(varint::parse(data.as_slice()), Ok((result, bytes_read)));
        }

        // Throw error if data is not enough
        for data in [
            vec![],
            vec![0xfd, 0x1a],
            vec![0xfe, 0x1a, 0xe3, 0x46],
            vec![0xff, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc],
        ] {
            assert_eq!(
                varint::parse(data.as_slice()),
                Err(SerializationError::NotEnoughData)
            );
        }

        // Ignore exceeding data
        for (data, result, bytes_read) in [
            (
                vec![0x50, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98],
                0x50,
                1,
            ),
            (
                vec![0xfd, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98],
                0xe31a,
                3,
            ),
            (
                vec![0xfe, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98],
                0xb446e31a,
                5,
            ),
            (
                vec![0xff, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98, 0x12],
                0x98cc4a67b446e31a,
                9,
            ),
        ] {
            assert_eq!(varint::parse(data.as_slice()), Ok((result, bytes_read)));
        }
    }

    #[test]
    fn encode_varint() {
        for (data, result) in [
            (0x50, vec![0x50]),
            (0xe31a, vec![0xfd, 0x1a, 0xe3]),
            (0xb446e31a, vec![0xfe, 0x1a, 0xe3, 0x46, 0xb4]),
            (
                0x98cc4a67b446e31a,
                vec![0xff, 0x1a, 0xe3, 0x46, 0xb4, 0x67, 0x4a, 0xcc, 0x98],
            ),
        ] {
            assert_eq!(varint::encode(data), result);
        }
    }

    #[test]
    fn encode_parse_varint() {
        for num in [0x50, 0xe31a, 0xb446e31a, 0x98cc4a67b446e31a] {
            let result = varint::parse(varint::encode(num).as_slice());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().0, num);
        }
    }
}
