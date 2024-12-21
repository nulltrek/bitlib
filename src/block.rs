use crate::hashing::{hash256, to_hex_str, Hash};
use crate::http::HttpError;
use crate::serialization::{slice_to_array, SerializationError};
use crate::u256::U256;
use core::fmt;

#[derive(Debug)]
pub enum BlockError {
    SerializationError(SerializationError),
    HttpError(HttpError),
}

impl From<HttpError> for BlockError {
    fn from(error: HttpError) -> Self {
        BlockError::HttpError(error)
    }
}

pub type Result<T> = std::result::Result<T, BlockError>;

pub type BlockId = Hash;

#[derive(PartialEq, Debug)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block: BlockId,
    pub merkle_root: Hash,
    pub timestamp: u32,
    pub bits: [u8; 4],
    pub nonce: [u8; 4],
}

impl BlockHeader {
    pub fn id(&self) -> String {
        let hash = hash256(self.serialize())
            .into_iter()
            .rev()
            .collect::<Vec<u8>>();
        to_hex_str(hash.as_slice())
    }

    pub fn hash(&self) -> BlockId {
        BlockId::from(U256::from_hex(&self.id()))
    }

    pub fn target(&self) -> U256 {
        let exponent = U256::from_little_endian(&[self.bits[3] - 3]);
        let coefficient = U256::from_little_endian(&self.bits[0..3]);
        coefficient * U256::from_big_endian(&[0x01, 0x00]).pow(exponent)
    }

    pub fn parse(data: &[u8]) -> Result<BlockHeader> {
        log::debug!("Parsing block header...");
        let header = BlockHeader {
            version: u32::from_le_bytes(slice_to_array(&data[0..4])),
            prev_block: Hash::from(U256::from_little_endian(&data[4..36])),
            merkle_root: Hash::from(U256::from_little_endian(&data[36..68])),
            timestamp: u32::from_le_bytes(slice_to_array(&data[68..72])),
            bits: slice_to_array(&data[72..76]),
            nonce: slice_to_array(&data[76..80]),
        };
        log::debug!("...done.");
        Ok(header)
    }

    pub fn check_pow(&self) -> bool {
        *self.hash() < self.target()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let version = self.version.to_le_bytes();
        let prev_block = self.prev_block.to_little_endian();
        let merkle_root = self.merkle_root.to_little_endian();
        let timestamp = self.timestamp.to_le_bytes();

        [
            version.as_slice(),
            prev_block.as_slice(),
            merkle_root.as_slice(),
            timestamp.as_slice(),
            self.bits.as_slice(),
            self.nonce.as_slice(),
        ]
        .concat()
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "version: {:032b}\nprevious block: {}\nmerkle root: {}\ntimestamp: {}\nbits:  {:032b}\nnonce: {:032b}",
            self.version, self.prev_block, self.merkle_root, self.timestamp,
            u32::from_le_bytes(self.bits), u32::from_le_bytes(self.nonce),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn parsing() {
        let bytes = hex!("020000208ec39428b17323fa0ddec8e887b4a7c53b8c0a0a220cfd0000000000000000005b0750fce0a889502d40508d39576821155e9c9e3f5c3157f961db38fd8b25be1e77a759e93c0118a4ffd71d");
        let header = BlockHeader::parse(&bytes).unwrap();
        assert_eq!(header.version, 0x20000002);
        assert_eq!(
            header.prev_block,
            BlockId::from(U256::from_hex(
                "000000000000000000fd0c220a0a8c3bc5a7b487e8c8de0dfa2373b12894c38e"
            ))
        );
        assert_eq!(
            header.merkle_root,
            Hash::from(U256::from_hex(
                "be258bfd38db61f957315c3f9e9c5e15216857398d50402d5089a8e0fc50075b"
            ))
        );
        assert_eq!(header.timestamp, 0x59a7771e);
        assert_eq!(header.bits, hex!("e93c0118"));
        assert_eq!(header.nonce, hex!("a4ffd71d"));

        assert_eq!(
            header.id(),
            "0000000000000000007e9e4c586439b0cdbe13b1370bdd9435d76a644d047523"
        );

        assert_eq!(
            header.target(),
            U256::from_hex("0000000000000000013ce9000000000000000000000000000000000000000000")
        );

        assert!(header.check_pow());
    }

    #[test]
    fn serializing() {
        let bytes = hex!("020000208ec39428b17323fa0ddec8e887b4a7c53b8c0a0a220cfd0000000000000000005b0750fce0a889502d40508d39576821155e9c9e3f5c3157f961db38fd8b25be1e77a759e93c0118a4ffd71d");
        let header = BlockHeader::parse(&bytes).unwrap();
        assert_eq!(header.serialize(), bytes);
    }
}
