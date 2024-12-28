use crate::hashing::{hash256, to_hex_str, Hash};
use crate::tx::TxId;
use crate::u256::U256;
use std::ops::Range;

pub struct Bitfield {
    bytes: Vec<u8>,
}

impl Bitfield {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn from_little_endian(bytes: &[u8]) -> Self {
        let mut bit_field = Vec::<u8>::with_capacity(bytes.len());
        for byte in bytes {
            let mut byte = *byte;
            let mut rev_byte: u8 = 0;
            for i in 0..8 {
                let val = byte & 1;
                byte = byte >> 1;
                rev_byte = rev_byte | val << (7 - i);
            }
            bit_field.push(rev_byte);
        }
        Self { bytes: bit_field }
    }

    fn get(&self, i: usize) -> Option<bool> {
        if i >= self.bytes.len() * 8 {
            return None;
        }

        Some((self.bytes[i / 8] << (i % 8)) & 0b10000000 != 0)
    }
}

fn range_split(range: Range<usize>) -> (Range<usize>, Range<usize>) {
    let split = range.start + range.len() / 2;
    ((range.start..split), (split..range.end))
}

pub struct MerkleTree {
    leaf_count: usize,
    max_leaf_count: usize,
}

impl MerkleTree {
    pub fn new(tx_count: usize) -> Self {
        let depth = MerkleTree::compute_depth(tx_count);
        let leaf_count = tx_count;
        let max_leaf_count = 2_u32.pow(depth as u32) as usize;
        log::debug!(
            "Merkle tree init. Depth: {}; Leaf count: {}; Max leaves: {}",
            depth,
            leaf_count,
            max_leaf_count,
        );
        Self {
            leaf_count,
            max_leaf_count,
        }
    }

    fn depth(&self) -> usize {
        Self::compute_depth(self.leaf_count)
    }

    fn compute_depth(leaf_count: usize) -> usize {
        (leaf_count as f32).log2().ceil() as usize
    }

    pub fn hash(&self, bits: &Bitfield, txs: &[TxId]) -> Hash {
        let hashes: Vec<[u8; 32]> = txs.iter().map(|id| id.to_big_endian()).collect();
        let (hash, _, _) = self.compute_hash(bits, hashes.as_slice(), 0..self.max_leaf_count, 0, 0);
        U256::from_big_endian(&hash.unwrap()).into()
    }

    fn compute_hash(
        &self,
        bits: &Bitfield,
        txs: &[[u8; 32]],
        leaf_range: Range<usize>,
        bit_index: usize,
        tx_index: usize,
    ) -> (Option<Vec<u8>>, usize, usize) {
        if leaf_range.start > self.leaf_count - 1 {
            return (None, bit_index, tx_index);
        }

        let flag = match bits.get(bit_index) {
            None => {
                log::error!("Not enough merkle tree flag bits");
                panic!("Not enough merkle tree flag bits");
            }
            Some(flag) => flag,
        };

        log::debug!(
            "Merkle hash. Leaf range: {:?}; bit_index: {} (flag: {}); tx_index: {}",
            leaf_range,
            bit_index,
            flag,
            tx_index
        );

        if self.is_leaf(&leaf_range) && flag == true {
            log::debug!(
                "Leaf range: {:?}; Hash value: {} (from tx list, leaf node)",
                leaf_range,
                to_hex_str(&txs[tx_index])
            );
            return (Some(txs[tx_index].to_vec()), bit_index + 1, tx_index + 1);
        } else if flag == false {
            log::debug!(
                "Leaf range: {:?}; Hash value: {} (from tx list, internal node)",
                leaf_range,
                to_hex_str(&txs[tx_index])
            );
            return (Some(txs[tx_index].to_vec()), bit_index + 1, tx_index + 1);
        }

        let (left_range, right_range) = range_split(leaf_range.clone());
        let (left, bit_index, tx_index) =
            self.compute_hash(bits, txs, left_range, bit_index + 1, tx_index);
        let (right, bit_index, tx_index) =
            self.compute_hash(bits, txs, right_range, bit_index, tx_index);

        let left = left.unwrap();
        let right = if right.is_none() {
            left.clone()
        } else {
            right.unwrap()
        };

        let hash = hash256([left, right].concat());
        log::debug!(
            "Leaf range: {:?}; Hash value: {} (computed, internal node)",
            leaf_range,
            to_hex_str(&hash)
        );
        return (Some(hash), bit_index, tx_index);
    }

    fn is_leaf(&self, range: &Range<usize>) -> bool {
        range.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    fn init_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn range_ops() {
        let (l, r) = range_split(0..8);
        assert_eq!(l, (0..4));
        assert_eq!(r, (4..8));

        let (l, r) = range_split(0..2);
        assert_eq!(l, (0..1));
        assert_eq!(r, (1..2));
        assert_eq!(l.len(), 1);
        assert_eq!(r.len(), 1);

        let (l, r) = range_split(0..7);
        assert_eq!(l, (0..3));
        assert_eq!(r, (3..7));
    }

    #[test]
    fn bitfield() {
        let bf = Bitfield::new(&[0b0100_1100, 0b0000_1111]);

        assert_eq!(bf.get(0), Some(false));
        assert_eq!(bf.get(1), Some(true));
        assert_eq!(bf.get(2), Some(false));
        assert_eq!(bf.get(3), Some(false));

        assert_eq!(bf.get(4), Some(true));
        assert_eq!(bf.get(5), Some(true));
        assert_eq!(bf.get(6), Some(false));
        assert_eq!(bf.get(7), Some(false));

        assert_eq!(bf.get(8), Some(false));
        assert_eq!(bf.get(9), Some(false));
        assert_eq!(bf.get(10), Some(false));
        assert_eq!(bf.get(11), Some(false));

        assert_eq!(bf.get(12), Some(true));
        assert_eq!(bf.get(13), Some(true));
        assert_eq!(bf.get(14), Some(true));
        assert_eq!(bf.get(15), Some(true));

        assert_eq!(bf.get(16), None);
        assert_eq!(bf.get(20), None);
    }

    #[test]
    fn bitfield_little_endian() {
        let bf = Bitfield::from_little_endian(&hex!("b55635"));
        assert_eq!(bf.bytes, &[0b10101101, 0b01101010, 0b10101100]);
    }

    #[test]
    fn merkle_tree_16() {
        init_logging();
        let hashes: [TxId; 16] = [
            U256::from_hex("9745f7173ef14ee4155722d1cbf13304339fd00d900b759c6f9d58579b5765fb")
                .into(),
            U256::from_hex("5573c8ede34936c29cdfdfe743f7f5fdfbd4f54ba0705259e62f39917065cb9b")
                .into(),
            U256::from_hex("82a02ecbb6623b4274dfcab82b336dc017a27136e08521091e443e62582e8f05")
                .into(),
            U256::from_hex("507ccae5ed9b340363a0e6d765af148be9cb1c8766ccc922f83e4ae681658308")
                .into(),
            U256::from_hex("a7a4aec28e7162e1e9ef33dfa30f0bc0526e6cf4b11a576f6c5de58593898330")
                .into(),
            U256::from_hex("bb6267664bd833fd9fc82582853ab144fece26b7a8a5bf328f8a059445b59add")
                .into(),
            U256::from_hex("ea6d7ac1ee77fbacee58fc717b990c4fcccf1b19af43103c090f601677fd8836")
                .into(),
            U256::from_hex("457743861de496c429912558a106b810b0507975a49773228aa788df40730d41")
                .into(),
            U256::from_hex("7688029288efc9e9a0011c960a6ed9e5466581abf3e3a6c26ee317461add619a")
                .into(),
            U256::from_hex("b1ae7f15836cb2286cdd4e2c37bf9bb7da0a2846d06867a429f654b2e7f383c9")
                .into(),
            U256::from_hex("9b74f89fa3f93e71ff2c241f32945d877281a6a50a6bf94adac002980aafe5ab")
                .into(),
            U256::from_hex("b3a92b5b255019bdaf754875633c2de9fec2ab03e6b8ce669d07cb5b18804638")
                .into(),
            U256::from_hex("b5c0b915312b9bdaedd2b86aa2d0f8feffc73a2d37668fd9010179261e25e263")
                .into(),
            U256::from_hex("c9d52c5cb1e557b92c84c52e7c4bfbce859408bedffc8a5560fd6e35e10b8800")
                .into(),
            U256::from_hex("c555bc5fc3bc096df0a0c9532f07640bfb76bfe4fc1ace214b8b228a1297a4c2")
                .into(),
            U256::from_hex("f9dbfafc3af3400954975da24eb325e326960a25b87fffe23eef3e7ed2fb610e")
                .into(),
        ];

        let tree = MerkleTree::new(hashes.len());
        assert_eq!(tree.depth(), 4);

        let bits = Bitfield::new(&[0xff, 0xff, 0xff, 0xff]);
        assert_eq!(
            tree.hash(&bits, &hashes),
            U256::from_hex("597c4bafe3832b17cbbabe56f878f4fc2ad0f6a402cee7fa851a9cb205f87ed1")
                .into()
        );
    }

    #[test]
    fn merkle_tree_5() {
        let hashes: [TxId; 5] = [
            U256::from_hex("42f6f52f17620653dcc909e58bb352e0bd4bd1381e2955d19c00959a22122b2e")
                .into(),
            U256::from_hex("94c3af34b9667bf787e1c6a0a009201589755d01d02fe2877cc69b929d2418d4")
                .into(),
            U256::from_hex("959428d7c48113cb9149d0566bde3d46e98cf028053c522b8fa8f735241aa953")
                .into(),
            U256::from_hex("a9f27b99d5d108dede755710d4a1ffa2c74af70b4ca71726fa57d68454e609a2")
                .into(),
            U256::from_hex("62af110031e29de1efcad103b3ad4bec7bdcf6cb9c9f4afdd586981795516577")
                .into(),
        ];

        let tree = MerkleTree::new(hashes.len());
        assert_eq!(tree.depth(), 3);

        let bits = Bitfield::new(&[0b11111111, 0b11100000]);
        assert_eq!(
            tree.hash(&bits, &hashes),
            U256::from_hex("a8e8bd023169b81bc56854137a135b97ef47a6a7237f4c6e037baed16285a5ab")
                .into()
        );
    }

    #[test]
    fn merkle_tree_3519() {
        let hashes: [TxId; 10] = [
            U256::from_hex("ba412a0d1480e370173072c9562becffe87aa661c1e4a6dbc305d38ec5dc088a")
                .into(),
            U256::from_hex("7cf92e6458aca7b32edae818f9c2c98c37e06bf72ae0ce80649a38655ee1e27d")
                .into(),
            U256::from_hex("34d9421d940b16732f24b94023e9d572a7f9ab8023434a4feb532d2adfc8c2c2")
                .into(),
            U256::from_hex("158785d1bd04eb99df2e86c54bc13e139862897217400def5d72c280222c4cba")
                .into(),
            U256::from_hex("ee7261831e1550dbb8fa82853e9fe506fc5fda3f7b919d8fe74b6282f92763ce")
                .into(),
            U256::from_hex("f8e625f977af7c8619c32a369b832bc2d051ecd9c73c51e76370ceabd4f25097")
                .into(),
            U256::from_hex("c256597fa898d404ed53425de608ac6bfe426f6e2bb457f1c554866eb69dcb8d")
                .into(),
            U256::from_hex("6bf6f880e9a59b3cd053e6c7060eeacaacf4dac6697dac20e4bd3f38a2ea2543")
                .into(),
            U256::from_hex("d1ab7953e3430790a9f81e1c67f5b58c825acf46bd02848384eebe9af917274c")
                .into(),
            U256::from_hex("dfbb1a28a5d58a23a17977def0de10d644258d9c54f886d47d293a411cb62261")
                .into(),
        ];

        init_logging();

        let tree = MerkleTree::new(3519);
        assert_eq!(tree.depth(), 12);

        let bits = Bitfield::new(&[0b10101101, 0b01101010, 0b10101100]);
        assert_eq!(
            tree.hash(&bits, &hashes),
            U256::from_hex("ef445fef2ed495c275892206ca533e7411907971013ab83e3b47bd0d692d14d4")
                .into()
        );
    }
}
