use crate::hashing::{hash256, to_hex_str, Hash};
use crate::network::{NetworkError, TxFetcher};
use crate::script::{self, Script};
use crate::serialization::{slice_to_array, varint, SerializationError};
use crate::u256::U256;
use core::fmt;

#[derive(Debug)]
pub enum TxError {
    NetworkError(NetworkError),
    SerializationError(SerializationError),
    InputNotFound(usize),
    GenericError,
}

impl From<SerializationError> for TxError {
    fn from(error: SerializationError) -> Self {
        TxError::SerializationError(error)
    }
}

impl From<NetworkError> for TxError {
    fn from(error: NetworkError) -> Self {
        TxError::NetworkError(error)
    }
}

pub type Result<T> = std::result::Result<T, TxError>;

pub type TxId = Hash;

pub enum SigHash {
    All,
    None,
    Single,
}

impl SigHash {
    fn to_little_endian(&self) -> Vec<u8> {
        let value: u32 = match self {
            Self::All => 1,
            Self::None => 2,
            Self::Single => 3,
        };
        value.to_le_bytes().to_vec()
    }
}

#[derive(Clone)]
struct TxInput {
    prev_tx: TxId,
    prev_tx_index: u32,
    script_sig: Script,
    sequence: u32,
}

impl TxInput {
    pub fn parse(data: &[u8]) -> Result<(TxInput, usize)> {
        let prev_tx = Hash::from(U256::from_little_endian(&data[0..32]));
        let prev_tx_index = u32::from_le_bytes(slice_to_array(&data[32..36]));
        let (script_sig, script_sig_len) = Script::parse(&data[36..])?;
        log::debug!("Script: {}", script_sig);
        let seq_offset = 36 + script_sig_len;
        let end_offset = seq_offset + 4;
        let sequence = u32::from_le_bytes(slice_to_array(&data[seq_offset..end_offset]));

        Ok((
            TxInput {
                prev_tx,
                prev_tx_index,
                script_sig,
                sequence,
            },
            end_offset,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let prev_tx = (*self.prev_tx).to_little_endian();
        let prev_tx_index = self.prev_tx_index.to_le_bytes();
        let script_sig = self.script_sig.serialize();
        let sequence = self.sequence.to_le_bytes();

        [
            prev_tx.as_slice(),
            prev_tx_index.as_slice(),
            script_sig.as_slice(),
            sequence.as_slice(),
        ]
        .concat()
    }

    pub fn value<F: TxFetcher>(&self, fetcher: &mut F) -> Result<u64> {
        let tx_data = fetcher.fetch_tx(&self.prev_tx)?;
        let tx = Tx::parse(&tx_data)?;
        Ok(tx.outputs[self.prev_tx_index as usize].amount)
    }

    pub fn get_script_pubkey<F: TxFetcher>(&self, fetcher: &mut F) -> Result<Script> {
        let tx_data = fetcher.fetch_tx(&self.prev_tx)?;
        let tx = Tx::parse(&tx_data)?;
        Ok(tx.outputs[self.prev_tx_index as usize]
            .script_pubkey
            .clone())
    }
}

impl fmt::Display for TxInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "prev_tx: {}\nprev_tx_index: {}\nscript_sig: {}\nsequence: {}\n",
            self.prev_tx, self.prev_tx_index, self.script_sig, self.sequence
        )
    }
}

#[derive(Clone)]
struct TxOutput {
    amount: u64,
    script_pubkey: Script,
}

impl TxOutput {
    pub fn parse(data: &[u8]) -> Result<(TxOutput, usize)> {
        let amount = u64::from_le_bytes(slice_to_array(&data[0..8]));
        let (script_pubkey, script_pubkey_length) = Script::parse(&data[8..])?;
        let end_offset = 8 + script_pubkey_length;
        log::debug!("Script: {}", script_pubkey);
        Ok((
            TxOutput {
                amount,
                script_pubkey,
            },
            end_offset,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let amount = self.amount.to_le_bytes();
        let script_pubkey = self.script_pubkey.serialize();

        [amount.as_slice(), script_pubkey.as_slice()].concat()
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "amount: {}\nscript_pubkey: {}\n",
            self.amount, self.script_pubkey
        )
    }
}

#[derive(Clone)]
pub struct Tx {
    version: u32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
}

impl Tx {
    pub fn id(&self) -> String {
        let hash = hash256(self.serialize())
            .into_iter()
            .rev()
            .collect::<Vec<u8>>();
        to_hex_str(hash.as_slice())
    }

    pub fn parse(data: &[u8]) -> Result<Tx> {
        log::info!("Parsing transaction...");
        let version = u32::from_le_bytes(slice_to_array(&data[0..4]));

        let mut offset = if data[4] == 0 { 6 } else { 4 };

        let (input_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        log::debug!("Inputs: {}", input_count);
        let mut inputs = vec![];
        for _ in 0..input_count {
            let (input, input_length) = TxInput::parse(&data[offset..])?;
            offset += input_length;
            inputs.push(input)
        }

        let (output_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        log::debug!("Outputs: {}", output_count);
        let mut outputs = vec![];
        for _ in 0..output_count {
            let (output, output_length) = TxOutput::parse(&data[offset..])?;
            offset += output_length;
            outputs.push(output)
        }

        let locktime = u32::from_le_bytes(slice_to_array(&data[offset..offset + 4]));

        let tx = Tx {
            version,
            inputs,
            outputs,
            locktime,
        };
        log::info!("...done. Transaction id: {}", tx.id());
        Ok(tx)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let version = self.version.to_le_bytes();
        let input_count = varint::encode(self.inputs.len() as u64);
        let inputs = self
            .inputs
            .iter()
            .map(|e| e.serialize())
            .collect::<Vec<_>>()
            .concat();
        let output_count = varint::encode(self.outputs.len() as u64);
        let outputs = self
            .outputs
            .iter()
            .map(|e| e.serialize())
            .collect::<Vec<_>>()
            .concat();
        let locktime = self.locktime.to_le_bytes();

        [
            version.as_slice(),
            input_count.as_slice(),
            inputs.as_slice(),
            output_count.as_slice(),
            outputs.as_slice(),
            locktime.as_slice(),
        ]
        .concat()
    }

    pub fn fee<F: TxFetcher>(&self, fetcher: &mut F) -> Result<u64> {
        let mut input_value = 0;
        for input in &self.inputs {
            input_value += input.value(fetcher)?;
        }

        Ok(input_value - self.outputs.iter().map(|tx_out| tx_out.amount).sum::<u64>())
    }

    pub fn sig_hash<F: TxFetcher>(
        &self,
        fetcher: &mut F,
        input_idx: usize,
        flag: SigHash,
    ) -> Result<Hash> {
        if input_idx >= self.inputs.len() {
            return Err(TxError::InputNotFound(input_idx));
        }

        let mut mod_tx = self.clone();
        // Clean the inputs except the one we want to sign
        for i in 0..mod_tx.inputs.len() {
            let mut input = &mut mod_tx.inputs[i];

            if i == input_idx {
                input.script_sig = input.get_script_pubkey(fetcher)?;
            } else {
                input.script_sig = Script::default();
            }
        }
        let bytes = [mod_tx.serialize(), flag.to_little_endian()].concat();
        Ok(Hash::hash256(&bytes))
    }

    pub fn verify_input<F: TxFetcher>(&self, fetcher: &mut F, input_idx: usize) -> bool {
        if input_idx >= self.inputs.len() {
            return false;
        }

        let input = &self.inputs[input_idx];
        let script_sig = &input.script_sig;
        let script_pubkey = match input.get_script_pubkey(fetcher) {
            Err(_) => return false,
            Ok(script) => script,
        };
        let sig_hash = match self.sig_hash(fetcher, input_idx, SigHash::All) {
            Err(_) => return false,
            Ok(hash) => hash,
        };

        script::evaluate(&sig_hash, &script_pubkey, &script_sig)
    }
}

impl fmt::Display for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tx: {}\nversion: {}\ntx_inputs:\n",
            self.id(),
            self.version,
        )?;

        for (i, tx_in) in self.inputs.iter().enumerate() {
            write!(f, "{}:\n{}\n", i, tx_in)?;
        }
        write!(f, "tx_outputs:\n",)?;
        for (i, tx_out) in self.outputs.iter().enumerate() {
            write!(f, "{}: {}\n", i, tx_out)?;
        }

        write!(f, "locktime: {}\n", self.locktime,)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn sighash_value() {
        assert_eq!(
            SigHash::All.to_little_endian(),
            vec![0x01, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            SigHash::None.to_little_endian(),
            vec![0x02, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            SigHash::Single.to_little_endian(),
            vec![0x03, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn tx_parse() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.version, 1);

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(
            tx.inputs[0].prev_tx,
            Hash::from(U256::from_hex(
                "d1c789a9c60383bf715f3f6ad9d14b91fe55f3deb369fe5d9280cb1a01793f81"
            ))
        );
        assert_eq!(tx.inputs[0].prev_tx_index, 0);
        assert_eq!(tx.inputs[0].script_sig.serialize(), hex!("6b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278a"));
        assert_eq!(tx.inputs[0].sequence, 0xfffffffe);

        assert_eq!(tx.outputs.len(), 2);
        assert_eq!(tx.outputs[0].amount, 32454049);
        assert_eq!(
            tx.outputs[0].script_pubkey.serialize(),
            hex!("1976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac")
        );

        assert_eq!(tx.outputs[1].amount, 10011545);
        assert_eq!(
            tx.outputs[1].script_pubkey.serialize(),
            hex!("1976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac")
        );

        assert_eq!(tx.locktime, 410393);
    }

    #[test]
    fn parse_genesis() {
        let bytes = hex!("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.version, 1);
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(
            tx.inputs[0].prev_tx,
            Hash::from(U256::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
        );
        assert_eq!(tx.inputs[0].prev_tx_index, 4294967295);
        assert_eq!(tx.inputs[0].sequence, 0xffffffff);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].amount, 5000000000);
        assert_eq!(tx.locktime, 0);
    }

    use crate::hashing::from_hex_str;
    use std::collections::HashMap;

    struct MockFetcher {
        cache: HashMap<String, &'static str>,
    }

    impl MockFetcher {
        fn new() -> Self {
            Self{
                cache: HashMap::from([
                    ("d1c789a9c60383bf715f3f6ad9d14b91fe55f3deb369fe5d9280cb1a01793f81".to_string(), "0100000002137c53f0fb48f83666fcfd2fe9f12d13e94ee109c5aeabbfa32bb9e02538f4cb000000006a47304402207e6009ad86367fc4b166bc80bf10cf1e78832a01e9bb491c6d126ee8aa436cb502200e29e6dd7708ed419cd5ba798981c960f0cc811b24e894bff072fea8074a7c4c012103bc9e7397f739c70f424aa7dcce9d2e521eb228b0ccba619cd6a0b9691da796a1ffffffff517472e77bc29ae59a914f55211f05024556812a2dd7d8df293265acd8330159010000006b483045022100f4bfdb0b3185c778cf28acbaf115376352f091ad9e27225e6f3f350b847579c702200d69177773cd2bb993a816a5ae08e77a6270cf46b33f8f79d45b0cd1244d9c4c0121031c0b0b95b522805ea9d0225b1946ecaeb1727c0b36c7e34165769fd8ed860bf5ffffffff027a958802000000001976a914a802fc56c704ce87c42d7c92eb75e7896bdc41ae88aca5515e00000000001976a914e82bd75c9c662c3f5700b33fec8a676b6e9391d588ac00000000"),
                    ("9e067aedc661fca148e13953df75f8ca6eada9ce3b3d8d68631769ac60999156".to_string(), "0100000001c228021e1fee6f158cc506edea6bad7ffa421dd14fb7fd7e01c50cc9693e8dbe02000000fdfe0000483045022100c679944ff8f20373685e1122b581f64752c1d22c67f6f3ae26333aa9c3f43d730220793233401f87f640f9c39207349ffef42d0e27046755263c0a69c436ab07febc01483045022100eadc1c6e72f241c3e076a7109b8053db53987f3fcc99e3f88fc4e52dbfd5f3a202201f02cbff194c41e6f8da762e024a7ab85c1b1616b74720f13283043e9e99dab8014c69522102b0c7be446b92624112f3c7d4ffc214921c74c1cb891bf945c49fbe5981ee026b21039021c9391e328e0cb3b61ba05dcc5e122ab234e55d1502e59b10d8f588aea4632102f3bd8f64363066f35968bd82ed9c6e8afecbd6136311bb51e91204f614144e9b53aeffffffff05a08601000000000017a914081fbb6ec9d83104367eb1a6a59e2a92417d79298700350c00000000001976a914677345c7376dfda2c52ad9b6a153b643b6409a3788acc7f341160000000017a914234c15756b9599314c9299340eaabab7f1810d8287c02709000000000017a91469be3ca6195efcab5194e1530164ec47637d44308740420f00000000001976a91487fadba66b9e48c0c8082f33107fdb01970eb80388ac00000000"),
                    ("d37f9e7282f81b7fd3af0fde8b462a1c28024f1d83cf13637ec18d03f4518feb".to_string(), "0100000001b74780c0b9903472f84f8697a7449faebbfb1af659ecb8148ce8104347f3f72d010000006b483045022100bb8792c98141bcf4dab4fd4030743b4eff9edde59cec62380c60ffb90121ab7802204b439e3572b51382540c3b652b01327ee8b14cededc992fbc69b1e077a2c3f9f0121027c975c8bdc9717de310998494a2ae63f01b7a390bd34ef5b4c346fa717cba012ffffffff01a627c901000000001976a914af24b3f3e987c23528b366122a7ed2af199b36bc88ac00000000"),
                    ("75d7454b7010fa28b00f16cccb640b1756fd6e357c03a3b81b9d119505f47b56".to_string(), "010000000367d54ded4c43569acbc213073fc63bfc49bf420391f0ab304758b16600a8ea88010000006a4730440220404b3bb28af45437c989328122aa6f4462021a0a2d4f20141ebe84e80edd72e202204184dd9d833d57246eaeed39021e9ab8c0546f3270bd9d2fc138a4bf161ea2310121039550662b907f788cc96708dc017aee0d407b74427f11e656b87f84146337f183feffffff5edf7dbc586b5fddace63a6614f5a731787c104d3c1c9225c4542db067d4296d010000006b483045022100b2335adb91e1ac3bb4e0479b54a9e7d4b765d9b646ca71e2547776c4e7e6bdfb02201fa8aaa4d2557768329befd61d4abda95668f88065df6eac6076e3e123c121eb012103b80229ec7a62793132ff432be0ecf21bca774ade18af7eaf2215febad0c4321ffeffffffdfa74eb50768daeb4beca2ca83d1732128d2439f9df9508efc8f7820718b4ae1000000006a47304402204818b29bed4a8ea4eb383f996389866a732b44d98f6342ecc25007ca472526fb0220496ed1213d63b7686f6936940e8f566f291bab211e6600c0f71e3659787b91fc0121036a30f9e6f645191c6216f84c21ae3b4f0aca0c4be987889276089cf9ef7a89d6feffffff028deb0f00000000001976a914cd0b3a22cd16e182291aa2708c41cb38de5a330788acc0e1e400000000001976a91424505f6d2f0fe7c4a3f4af32f50506034d89095d88ac43430600"),
                    ("45f3f79066d251addc04fd889f776c73afab1cb22559376ff820e6166c5e3ad6".to_string(), "01000000012aa311f7789d362ceb2d802a98a703e0ac44815c021293633b80d08e67232e36010000006a4730440220142d8810ab29cac9199e6b570d47bd5ee402accf9d754cfa7de9b2e84e3997b402207a7d8c77c6a721bc64dba39eabe23e915c979683e621921c243bb35b3f538dfb01210371cb7d04e95471c4ea5c200e8c4729608754c74bee4e289bd66f431482407ec8feffffff02a08601000000000017a914fc7d096f19063ece361e2b309ec4da41fe4d789487f2798e00000000001976a914311b232c3400080eb2636edb8548b47f6835be7688ac31430600"),
                    ("9e067aedc661fca148e13953df75f8ca6eada9ce3b3d8d68631769ac60999156".to_string(), "0100000001c228021e1fee6f158cc506edea6bad7ffa421dd14fb7fd7e01c50cc9693e8dbe02000000fdfe0000483045022100c679944ff8f20373685e1122b581f64752c1d22c67f6f3ae26333aa9c3f43d730220793233401f87f640f9c39207349ffef42d0e27046755263c0a69c436ab07febc01483045022100eadc1c6e72f241c3e076a7109b8053db53987f3fcc99e3f88fc4e52dbfd5f3a202201f02cbff194c41e6f8da762e024a7ab85c1b1616b74720f13283043e9e99dab8014c69522102b0c7be446b92624112f3c7d4ffc214921c74c1cb891bf945c49fbe5981ee026b21039021c9391e328e0cb3b61ba05dcc5e122ab234e55d1502e59b10d8f588aea4632102f3bd8f64363066f35968bd82ed9c6e8afecbd6136311bb51e91204f614144e9b53aeffffffff05a08601000000000017a914081fbb6ec9d83104367eb1a6a59e2a92417d79298700350c00000000001976a914677345c7376dfda2c52ad9b6a153b643b6409a3788acc7f341160000000017a914234c15756b9599314c9299340eaabab7f1810d8287c02709000000000017a91469be3ca6195efcab5194e1530164ec47637d44308740420f00000000001976a91487fadba66b9e48c0c8082f33107fdb01970eb80388ac00000000"),
                    ("d37f9e7282f81b7fd3af0fde8b462a1c28024f1d83cf13637ec18d03f4518feb".to_string(), "0100000001b74780c0b9903472f84f8697a7449faebbfb1af659ecb8148ce8104347f3f72d010000006b483045022100bb8792c98141bcf4dab4fd4030743b4eff9edde59cec62380c60ffb90121ab7802204b439e3572b51382540c3b652b01327ee8b14cededc992fbc69b1e077a2c3f9f0121027c975c8bdc9717de310998494a2ae63f01b7a390bd34ef5b4c346fa717cba012ffffffff01a627c901000000001976a914af24b3f3e987c23528b366122a7ed2af199b36bc88ac00000000"),
                    ("75d7454b7010fa28b00f16cccb640b1756fd6e357c03a3b81b9d119505f47b56".to_string(), "010000000367d54ded4c43569acbc213073fc63bfc49bf420391f0ab304758b16600a8ea88010000006a4730440220404b3bb28af45437c989328122aa6f4462021a0a2d4f20141ebe84e80edd72e202204184dd9d833d57246eaeed39021e9ab8c0546f3270bd9d2fc138a4bf161ea2310121039550662b907f788cc96708dc017aee0d407b74427f11e656b87f84146337f183feffffff5edf7dbc586b5fddace63a6614f5a731787c104d3c1c9225c4542db067d4296d010000006b483045022100b2335adb91e1ac3bb4e0479b54a9e7d4b765d9b646ca71e2547776c4e7e6bdfb02201fa8aaa4d2557768329befd61d4abda95668f88065df6eac6076e3e123c121eb012103b80229ec7a62793132ff432be0ecf21bca774ade18af7eaf2215febad0c4321ffeffffffdfa74eb50768daeb4beca2ca83d1732128d2439f9df9508efc8f7820718b4ae1000000006a47304402204818b29bed4a8ea4eb383f996389866a732b44d98f6342ecc25007ca472526fb0220496ed1213d63b7686f6936940e8f566f291bab211e6600c0f71e3659787b91fc0121036a30f9e6f645191c6216f84c21ae3b4f0aca0c4be987889276089cf9ef7a89d6feffffff028deb0f00000000001976a914cd0b3a22cd16e182291aa2708c41cb38de5a330788acc0e1e400000000001976a91424505f6d2f0fe7c4a3f4af32f50506034d89095d88ac43430600"),
                    ("45f3f79066d251addc04fd889f776c73afab1cb22559376ff820e6166c5e3ad6".to_string(), "01000000012aa311f7789d362ceb2d802a98a703e0ac44815c021293633b80d08e67232e36010000006a4730440220142d8810ab29cac9199e6b570d47bd5ee402accf9d754cfa7de9b2e84e3997b402207a7d8c77c6a721bc64dba39eabe23e915c979683e621921c243bb35b3f538dfb01210371cb7d04e95471c4ea5c200e8c4729608754c74bee4e289bd66f431482407ec8feffffff02a08601000000000017a914fc7d096f19063ece361e2b309ec4da41fe4d789487f2798e00000000001976a914311b232c3400080eb2636edb8548b47f6835be7688ac31430600"),
                ]),
            }
        }
    }

    impl TxFetcher for MockFetcher {
        fn fetch_tx(&mut self, id: &TxId) -> std::result::Result<Vec<u8>, NetworkError> {
            let id = format!("{}", id);
            let result = match self.cache.get(&id) {
                None => return Err(NetworkError::ParsingError),
                Some(tx) => tx,
            };
            match from_hex_str(&result) {
                Err(_) => Err(NetworkError::ParsingError),
                Ok(bytes) => Ok(bytes),
            }
        }
    }

    #[test]
    fn tx_fee() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.fee(&mut MockFetcher::new()).unwrap(), 40000);

        let bytes = hex!("010000000456919960ac691763688d3d3bcea9ad6ecaf875df5339e148a1fc61c6ed7a069e010000006a47304402204585bcdef85e6b1c6af5c2669d4830ff86e42dd205c0e089bc2a821657e951c002201024a10366077f87d6bce1f7100ad8cfa8a064b39d4e8fe4ea13a7b71aa8180f012102f0da57e85eec2934a82a585ea337ce2f4998b50ae699dd79f5880e253dafafb7feffffffeb8f51f4038dc17e6313cf831d4f02281c2a468bde0fafd37f1bf882729e7fd3000000006a47304402207899531a52d59a6de200179928ca900254a36b8dff8bb75f5f5d71b1cdc26125022008b422690b8461cb52c3cc30330b23d574351872b7c361e9aae3649071c1a7160121035d5c93d9ac96881f19ba1f686f15f009ded7c62efe85a872e6a19b43c15a2937feffffff567bf40595119d1bb8a3037c356efd56170b64cbcc160fb028fa10704b45d775000000006a47304402204c7c7818424c7f7911da6cddc59655a70af1cb5eaf17c69dadbfc74ffa0b662f02207599e08bc8023693ad4e9527dc42c34210f7a7d1d1ddfc8492b654a11e7620a0012102158b46fbdff65d0172b7989aec8850aa0dae49abfb84c81ae6e5b251a58ace5cfeffffffd63a5e6c16e620f86f375925b21cabaf736c779f88fd04dcad51d26690f7f345010000006a47304402200633ea0d3314bea0d95b3cd8dadb2ef79ea8331ffe1e61f762c0f6daea0fabde022029f23b3e9c30f080446150b23852028751635dcee2be669c2a1686a4b5edf304012103ffd6f4a67e94aba353a00882e563ff2722eb4cff0ad6006e86ee20dfe7520d55feffffff0251430f00000000001976a914ab0c0b2e98b1ab6dbf67d4750b0a56244948a87988ac005a6202000000001976a9143c82d7df364eb6c75be8c80df2b3eda8db57397088ac46430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.fee(&mut MockFetcher::new()).unwrap(), 140500);
    }

    #[test]
    fn tx_serialization() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.serialize(), bytes);

        let bytes = hex!("010000000456919960ac691763688d3d3bcea9ad6ecaf875df5339e148a1fc61c6ed7a069e010000006a47304402204585bcdef85e6b1c6af5c2669d4830ff86e42dd205c0e089bc2a821657e951c002201024a10366077f87d6bce1f7100ad8cfa8a064b39d4e8fe4ea13a7b71aa8180f012102f0da57e85eec2934a82a585ea337ce2f4998b50ae699dd79f5880e253dafafb7feffffffeb8f51f4038dc17e6313cf831d4f02281c2a468bde0fafd37f1bf882729e7fd3000000006a47304402207899531a52d59a6de200179928ca900254a36b8dff8bb75f5f5d71b1cdc26125022008b422690b8461cb52c3cc30330b23d574351872b7c361e9aae3649071c1a7160121035d5c93d9ac96881f19ba1f686f15f009ded7c62efe85a872e6a19b43c15a2937feffffff567bf40595119d1bb8a3037c356efd56170b64cbcc160fb028fa10704b45d775000000006a47304402204c7c7818424c7f7911da6cddc59655a70af1cb5eaf17c69dadbfc74ffa0b662f02207599e08bc8023693ad4e9527dc42c34210f7a7d1d1ddfc8492b654a11e7620a0012102158b46fbdff65d0172b7989aec8850aa0dae49abfb84c81ae6e5b251a58ace5cfeffffffd63a5e6c16e620f86f375925b21cabaf736c779f88fd04dcad51d26690f7f345010000006a47304402200633ea0d3314bea0d95b3cd8dadb2ef79ea8331ffe1e61f762c0f6daea0fabde022029f23b3e9c30f080446150b23852028751635dcee2be669c2a1686a4b5edf304012103ffd6f4a67e94aba353a00882e563ff2722eb4cff0ad6006e86ee20dfe7520d55feffffff0251430f00000000001976a914ab0c0b2e98b1ab6dbf67d4750b0a56244948a87988ac005a6202000000001976a9143c82d7df364eb6c75be8c80df2b3eda8db57397088ac46430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.serialize(), bytes);

        let bytes = hex!("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.serialize(), bytes);
    }

    #[test]
    fn sig_hash() {
        let bytes = hex!(
            // version
            "01000000"
            // input 0
            "01 813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1 00000000"
            // input 0 script
            "6b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278a"
            // input 0 sequence
            "feffffff"
            // rest of tx
            "02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");

        let modified_tx = hex!(
            // version
            "01000000"
            // input 0
            "01 813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1 00000000"
            // imported script from referenced output
            "1976a914a802fc56c704ce87c42d7c92eb75e7896bdc41ae88ac"
            // sequence
            "feffffff"
            // rest of tx
            "02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600"
            // sighash flag
            "01000000");

        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(
            tx.sig_hash(&mut MockFetcher::new(), 0, SigHash::All)
                .unwrap(),
            Hash::hash256(modified_tx)
        );
    }

    // fn init_logging() {
    //     let _ = env_logger::builder().is_test(true).try_init();
    // }

    #[test]
    fn tx_verification() {
        // init_logging();
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert!(tx.verify_input(&mut MockFetcher::new(), 0))
    }
}
