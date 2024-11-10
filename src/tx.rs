use crate::definitions::Network;
use crate::ecdsa::Hash;
use crate::hashing::{hash256, to_hex_str};
use crate::network::{fetch_tx, NetworkError};
use crate::script::Script;
use crate::serialization::{slice_to_array, varint, SerializationError};
use crate::u256::U256;
use core::fmt;

#[derive(Debug)]
pub enum TxError {
    NetworkError(NetworkError),
    SerializationError(SerializationError),
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

    pub fn value(&self, network: Network) -> Result<u64> {
        let tx_data = fetch_tx(network, &self.prev_tx)?;
        let tx = Tx::parse(&tx_data)?;
        Ok(tx.outputs[self.prev_tx_index as usize].amount)
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

struct TxOutput {
    amount: u64,
    script_pubkey: Script,
}

impl TxOutput {
    pub fn parse(data: &[u8]) -> Result<(TxOutput, usize)> {
        let amount = u64::from_le_bytes(slice_to_array(&data[0..8]));
        let (script_pubkey, script_pubkey_length) = Script::parse(&data[8..])?;
        let end_offset = 8 + script_pubkey_length;
        Ok((
            TxOutput {
                amount,
                script_pubkey,
            },
            end_offset,
        ))
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

    pub fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    pub fn parse(data: &[u8]) -> Result<Tx> {
        let version = u32::from_le_bytes(slice_to_array(&data[0..4]));

        let mut offset = if data[4] == 0 { 6 } else { 4 };

        let (input_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        let mut inputs = vec![];
        for _ in 0..input_count {
            let (input, input_length) = TxInput::parse(&data[offset..])?;
            offset += input_length;
            inputs.push(input)
        }

        let (output_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        let mut outputs = vec![];
        for _ in 0..output_count {
            let (output, output_length) = TxOutput::parse(&data[offset..])?;
            offset += output_length;
            outputs.push(output)
        }

        let locktime = u32::from_le_bytes(slice_to_array(&data[offset..offset + 4]));

        Ok(Tx {
            version,
            inputs,
            outputs,
            locktime,
        })
    }

    pub fn fee(&self, network: Network) -> Result<u64> {
        let mut input_value = 0;
        for input in &self.inputs {
            input_value += input.value(network)?;
        }

        Ok(input_value - self.outputs.iter().map(|tx_out| tx_out.amount).sum::<u64>())
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
    fn tx_version() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.version, 1)
    }

    #[test]
    fn tx_inputs() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
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
    }

    #[test]
    fn tx_outputs() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
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
    }

    #[test]
    fn tx_locktime() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.locktime, 410393);
    }

    #[test]
    fn parse_genesis() {
        let bytes = fetch_tx(
            Network::Main,
            &TxId::from(U256::from_hex(
                "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
            )),
        )
        .unwrap();
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

    #[test]
    fn tx_fee() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.fee(Network::Main).unwrap(), 40000);

        let bytes = hex!("010000000456919960ac691763688d3d3bcea9ad6ecaf875df5339e148a1fc61c6ed7a069e010000006a47304402204585bcdef85e6b1c6af5c2669d4830ff86e42dd205c0e089bc2a821657e951c002201024a10366077f87d6bce1f7100ad8cfa8a064b39d4e8fe4ea13a7b71aa8180f012102f0da57e85eec2934a82a585ea337ce2f4998b50ae699dd79f5880e253dafafb7feffffffeb8f51f4038dc17e6313cf831d4f02281c2a468bde0fafd37f1bf882729e7fd3000000006a47304402207899531a52d59a6de200179928ca900254a36b8dff8bb75f5f5d71b1cdc26125022008b422690b8461cb52c3cc30330b23d574351872b7c361e9aae3649071c1a7160121035d5c93d9ac96881f19ba1f686f15f009ded7c62efe85a872e6a19b43c15a2937feffffff567bf40595119d1bb8a3037c356efd56170b64cbcc160fb028fa10704b45d775000000006a47304402204c7c7818424c7f7911da6cddc59655a70af1cb5eaf17c69dadbfc74ffa0b662f02207599e08bc8023693ad4e9527dc42c34210f7a7d1d1ddfc8492b654a11e7620a0012102158b46fbdff65d0172b7989aec8850aa0dae49abfb84c81ae6e5b251a58ace5cfeffffffd63a5e6c16e620f86f375925b21cabaf736c779f88fd04dcad51d26690f7f345010000006a47304402200633ea0d3314bea0d95b3cd8dadb2ef79ea8331ffe1e61f762c0f6daea0fabde022029f23b3e9c30f080446150b23852028751635dcee2be669c2a1686a4b5edf304012103ffd6f4a67e94aba353a00882e563ff2722eb4cff0ad6006e86ee20dfe7520d55feffffff0251430f00000000001976a914ab0c0b2e98b1ab6dbf67d4750b0a56244948a87988ac005a6202000000001976a9143c82d7df364eb6c75be8c80df2b3eda8db57397088ac46430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.fee(Network::Main).unwrap(), 140500);
    }
}
